use rmcp::model::{CallToolResult, JsonObject};
use serde_json::json;

use crate::policy::ApprovalClass;
use crate::policy::Scope;
use crate::vault::{SecretObject, VaultClient, VaultError, VaultPath, WriteMode};

use super::ToolDefinition;
use super::args::{optional_string, required_string};
use super::common::{success, tool_error, vault_error};

pub fn definitions() -> &'static [ToolDefinition] {
    &[
        ToolDefinition {
            name: "vault.runtime.metadata.get",
            title: "Get Runtime Secret Metadata",
            description: "Show runtime path existence and key names only.",
            required_scope: Scope::VaultRuntimeMetadata,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path"],"properties":{"path":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.runtime.read",
            title: "Read Runtime Secret",
            description: "Read runtime secret values after approval; values must be redacted in outputs by default.",
            required_scope: Scope::VaultRuntimeRead,
            approval_class: ApprovalClass::SensitiveRuntimeRead,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path","approvalId"],"properties":{"path":{"type":"string"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.runtime.write",
            title: "Write Runtime Secret",
            description: "Write approved runtime secret material under kv/runtime paths.",
            required_scope: Scope::VaultRuntimeWrite,
            approval_class: ApprovalClass::RuntimeSecretWrite,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path","values","approvalId"],"properties":{"path":{"type":"string"},"values":{"type":"object"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.runtime.patch",
            title: "Patch Runtime Secret",
            description: "Patch selected runtime secret keys while preserving other fields.",
            required_scope: Scope::VaultRuntimeWrite,
            approval_class: ApprovalClass::RuntimeSecretWrite,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path","values","approvalId"],"properties":{"path":{"type":"string"},"values":{"type":"object"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.operator.guide",
            title: "Guide Operator Secret Handling",
            description: "Provide instructions for operator-only secret handling without reading values.",
            required_scope: Scope::Admin,
            approval_class: ApprovalClass::OperatorSecretGuidance,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{"topic":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.operator.metadata.get",
            title: "Get Operator Secret Metadata",
            description: "Optional break-glass operator path and key metadata only.",
            required_scope: Scope::VaultOperatorMetadata,
            approval_class: ApprovalClass::OperatorSecretBreakGlass,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path","approvalId"],"properties":{"path":{"type":"string"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.operator.write",
            title: "Write Operator Secret",
            description: "Optional break-glass write to kv/operator paths.",
            required_scope: Scope::VaultOperatorWrite,
            approval_class: ApprovalClass::OperatorSecretBreakGlass,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path","values","approvalId"],"properties":{"path":{"type":"string"},"values":{"type":"object"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "vault.operator.read",
            title: "Read Operator Secret",
            description: "Optional break-glass read from kv/operator paths; redacted by default.",
            required_scope: Scope::VaultOperatorRead,
            approval_class: ApprovalClass::OperatorSecretBreakGlass,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["path","approvalId"],"properties":{"path":{"type":"string"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
    ]
}

pub(crate) async fn runtime_metadata_get(arguments: Option<&JsonObject>) -> CallToolResult {
    let path = match runtime_vault_path(arguments) {
        Ok(path) => path,
        Err(error) => return error,
    };
    let client = match VaultClient::from_env() {
        Ok(client) => client,
        Err(error) => return vault_error("vault.runtime.metadata.get", error),
    };

    match client.runtime_metadata(&path).await {
        Ok(metadata) => success(json!({
            "path": metadata.path,
            "exists": metadata.exists,
            "keyNames": metadata.key_names,
            "keyNamesAvailable": metadata.key_names_available,
            "currentVersion": metadata.current_version,
        })),
        Err(error) => vault_error("vault.runtime.metadata.get", error),
    }
}

pub(crate) async fn runtime_write(
    arguments: Option<&JsonObject>,
    default_mode: WriteMode,
) -> CallToolResult {
    let path = match runtime_vault_path(arguments) {
        Ok(path) => path,
        Err(error) => return error,
    };
    let secret = match secret_argument(arguments) {
        Ok(secret) => secret,
        Err(error) => return error,
    };
    let mode = match write_mode(arguments, default_mode) {
        Ok(mode) => mode,
        Err(error) => return vault_error("vault.runtime.write", error),
    };
    let client = match VaultClient::from_env() {
        Ok(client) => client,
        Err(error) => return vault_error("vault.runtime.write", error),
    };

    match client.write_runtime_secret(&path, &secret, mode).await {
        Ok(receipt) => success(json!({
            "path": receipt.path,
            "writtenKeys": receipt.written_keys,
            "version": receipt.version,
            "secretValuesReturned": false,
        })),
        Err(error) => vault_error("vault.runtime.write", error),
    }
}

fn runtime_vault_path(arguments: Option<&JsonObject>) -> Result<VaultPath, CallToolResult> {
    let path = required_string(arguments, "path")?;
    VaultPath::runtime(&path).map_err(|error| {
        tool_error(
            "vault_path_not_allowed",
            error.to_string(),
            json!({ "path": path }),
        )
    })
}

fn secret_argument(arguments: Option<&JsonObject>) -> Result<SecretObject, CallToolResult> {
    let value = arguments
        .and_then(|arguments| arguments.get("values").or_else(|| arguments.get("data")))
        .cloned()
        .ok_or_else(|| {
            tool_error(
                "invalid_arguments",
                "missing required secret object argument: values",
                json!({ "argument": "values" }),
            )
        })?;

    SecretObject::new(value).map_err(|error| {
        tool_error(
            "invalid_secret_values",
            error.to_string(),
            json!({ "argument": "values" }),
        )
    })
}

fn write_mode(
    arguments: Option<&JsonObject>,
    default_mode: WriteMode,
) -> Result<WriteMode, VaultError> {
    match optional_string(arguments, "mode") {
        Some(mode) => WriteMode::parse(Some(&mode)),
        None => Ok(default_mode),
    }
}
