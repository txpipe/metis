use std::sync::Arc;

use rmcp::RoleServer;
use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;
use rmcp::model::CallToolResult;
use rmcp::model::ErrorData as McpError;
use rmcp::model::GetPromptRequestParams;
use rmcp::model::GetPromptResult;
use rmcp::model::Implementation;
use rmcp::model::InitializeRequestParams;
use rmcp::model::InitializeResult;
use rmcp::model::ListPromptsResult;
use rmcp::model::ListResourcesResult;
use rmcp::model::ListToolsResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ProtocolVersion;
use rmcp::model::ReadResourceRequestParams;
use rmcp::model::ReadResourceResult;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerInfo;
use rmcp::service::RequestContext;

use crate::audit::AuditEvent;
use crate::audit::AuditSink;
use crate::audit::AuditTarget;
use crate::auth::AuthContext;
use crate::catalog::ExtensionCatalog;
use crate::policy::ApprovalClass;
use crate::policy::Policy;
use crate::policy::PolicyDecision;
use crate::policy::Scope;
use crate::prompts::PromptCatalog;
use crate::resources::ResourceRouter;
use crate::resources::router::ResourceReadError;
use crate::tools::ToolRouter;

#[derive(Clone)]
pub struct SupernodeMcpServer {
    auth: AuthContext,
    policy: Policy,
    audit: Arc<dyn AuditSink>,
    catalog: Arc<ExtensionCatalog>,
    resources: ResourceRouter,
    prompts: PromptCatalog,
    tools: ToolRouter,
}

impl SupernodeMcpServer {
    pub fn new(
        auth: AuthContext,
        policy: Policy,
        audit: Arc<dyn AuditSink>,
        catalog: Arc<ExtensionCatalog>,
    ) -> Self {
        let resources = ResourceRouter::new(catalog.clone());

        Self {
            auth,
            policy,
            audit,
            catalog,
            resources,
            prompts: PromptCatalog,
            tools: ToolRouter::new(),
        }
    }

    fn audit_discovery(&self, action: &str) {
        let scope_outcome = self.policy.require_scope(&self.auth, Scope::Discover);
        let audit_event = AuditEvent::from_policy_outcome(
            &self.auth,
            action,
            None,
            AuditTarget::None,
            &scope_outcome,
        );
        self.audit.record(&audit_event);
    }

    fn server_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(
            Implementation::new("metis-supernode-mcp", env!("CARGO_PKG_VERSION"))
                .with_title("Metis Supernode MCP"),
        )
        .with_protocol_version(ProtocolVersion::V_2025_11_25)
        .with_instructions("Operate an existing Metis Supernode cluster using typed tools only. Trusted MVP mode uses advisory policy and audit, not OAuth enforcement.".to_string())
    }
}

impl ServerHandler for SupernodeMcpServer {
    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        self.audit_discovery("initialize");

        let approval_outcome =
            self.policy
                .require_approval(&self.auth, None, ApprovalClass::Discovery);
        let audit_event = AuditEvent::from_policy_outcome(
            &self.auth,
            "initialize.approval",
            None,
            AuditTarget::None,
            &approval_outcome,
        );
        self.audit.record(&audit_event);

        tracing::debug!(
            supported_approval_classes = ApprovalClass::all().len(),
            extension_count = self.catalog.len(),
            listed_extension_count = self.catalog.list().count(),
            "initialized MCP session"
        );

        Ok(self.server_info())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        self.audit_discovery("resources/list");

        Ok(self.resources.list())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        self.audit_discovery("resources/read");

        self.resources
            .read(&request.uri, &self.auth)
            .map_err(|error| match error {
                ResourceReadError::NotFound => McpError::resource_not_found(
                    format!("resource not found: {}", request.uri),
                    None,
                ),
                ResourceReadError::Serialize(error) => McpError::internal_error(
                    "failed to serialize resource".to_string(),
                    Some(serde_json::json!({ "error": error.to_string() })),
                ),
            })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.audit_discovery("prompts/list");

        Ok(self.prompts.list())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.audit_discovery("prompts/get");

        self.prompts.get(&request.name).ok_or_else(|| {
            McpError::invalid_params(format!("prompt not found: {}", request.name), None)
        })
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        self.audit_discovery("tools/list");

        Ok(self.tools.list())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        let definition = self.tools.get(&tool_name).ok_or_else(|| {
            McpError::invalid_params(format!("tool not found: {tool_name}"), None)
        })?;
        let approval_id = request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("approvalId"))
            .and_then(|value| value.as_str())
            .map(str::to_string);

        let audit_target = audit_target_for_tool(&tool_name, request.arguments.as_ref());
        let scope_outcome = self
            .policy
            .require_scope(&self.auth, definition.required_scope);
        let audit_event = AuditEvent::from_policy_outcome(
            &self.auth,
            tool_name.clone(),
            approval_id.clone(),
            audit_target.clone(),
            &scope_outcome,
        );
        self.audit.record(&audit_event);
        if scope_outcome.decision == PolicyDecision::Denied {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": "policy_denied",
                "message": "missing required scope",
                "policy": scope_outcome,
            })));
        }

        let approval_outcome = self.policy.require_approval(
            &self.auth,
            approval_id.as_deref(),
            definition.approval_class,
        );
        let audit_event = AuditEvent::from_policy_outcome(
            &self.auth,
            format!("{tool_name}.approval"),
            approval_id,
            audit_target,
            &approval_outcome,
        );
        self.audit.record(&audit_event);
        if approval_outcome.decision == PolicyDecision::Denied {
            return Ok(CallToolResult::structured_error(serde_json::json!({
                "error": "approval_denied",
                "message": "required approval is missing",
                "policy": approval_outcome,
            })));
        }

        Ok(self
            .tools
            .call(definition, request.arguments.as_ref(), &self.catalog)
            .await)
    }

    fn get_info(&self) -> ServerInfo {
        self.server_info()
    }
}

fn audit_target_for_tool(
    tool_name: &str,
    arguments: Option<&rmcp::model::JsonObject>,
) -> AuditTarget {
    match tool_name {
        "vault.runtime.metadata.get" | "vault.runtime.write" | "vault.runtime.patch" => {
            let path = arguments
                .and_then(|arguments| arguments.get("path"))
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let written_keys = arguments
                .and_then(|arguments| arguments.get("values").or_else(|| arguments.get("data")))
                .and_then(|value| value.as_object())
                .map(|object| {
                    let mut keys = object.keys().cloned().collect::<Vec<_>>();
                    keys.sort();
                    keys
                })
                .unwrap_or_default();

            AuditTarget::VaultRuntime { path, written_keys }
        }
        _ => AuditTarget::None,
    }
}

#[cfg(test)]
mod tests {
    use crate::audit::TracingAuditSink;

    use super::*;

    #[test]
    fn server_info_uses_target_protocol_version() {
        let server = SupernodeMcpServer::new(
            AuthContext::trusted(),
            Policy,
            Arc::new(TracingAuditSink),
            Arc::new(ExtensionCatalog::embedded()),
        );

        let info = server.server_info();

        assert_eq!(info.protocol_version, ProtocolVersion::V_2025_11_25);
        assert_eq!(info.server_info.name, "metis-supernode-mcp");
    }

    #[test]
    fn vault_audit_target_records_path_and_keys_only() {
        let mut arguments = rmcp::model::JsonObject::new();
        arguments.insert(
            "path".to_string(),
            serde_json::Value::String("runtime/cardano-node/mainnet".to_string()),
        );
        arguments.insert(
            "values".to_string(),
            serde_json::json!({ "kes.skey": "secret-value", "op.cert": "also-secret" }),
        );

        let target = audit_target_for_tool("vault.runtime.write", Some(&arguments));

        assert_eq!(
            target,
            AuditTarget::VaultRuntime {
                path: "runtime/cardano-node/mainnet".to_string(),
                written_keys: vec!["kes.skey".to_string(), "op.cert".to_string()],
            }
        );
        assert!(
            !serde_json::to_string(&target)
                .unwrap()
                .contains("secret-value")
        );
    }
}
