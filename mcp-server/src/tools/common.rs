use rmcp::model::CallToolResult;
use serde_json::Value;
use serde_json::json;

use crate::k8s::PodExecError;
use crate::vault::VaultError;

pub(crate) fn success(value: Value) -> CallToolResult {
    CallToolResult::structured(value)
}

pub(crate) fn tool_error(code: &str, message: impl Into<String>, details: Value) -> CallToolResult {
    CallToolResult::structured_error(json!({
        "error": code,
        "message": message.into(),
        "details": details,
    }))
}

pub(crate) fn kube_error(tool: &str, error: kube::Error) -> CallToolResult {
    tool_error(
        "kubernetes_error",
        error.to_string(),
        json!({ "tool": tool }),
    )
}

pub(crate) fn pod_exec_error(tool: &str, error: PodExecError) -> CallToolResult {
    let code = match &error {
        PodExecError::Timeout { .. } => "pod_exec_timeout",
        PodExecError::OutputTooLarge { .. } => "pod_exec_output_too_large",
        PodExecError::CommandFailed { .. } => "pod_exec_command_failed",
        PodExecError::Kubernetes(_) => "kubernetes_error",
        PodExecError::MissingStream(_)
        | PodExecError::Read { .. }
        | PodExecError::RemoteCommand(_) => "pod_exec_error",
    };

    tool_error(code, error.to_string(), json!({ "tool": tool }))
}

pub(crate) fn vault_error(tool: &str, error: VaultError) -> CallToolResult {
    let code = match &error {
        VaultError::Path(_) => "vault_path_not_allowed",
        VaultError::MissingConfig(_) | VaultError::InvalidConfig(_) => "vault_not_configured",
        VaultError::RootTokenRejected => "vault_root_token_rejected",
        VaultError::InvalidWriteMode => "invalid_arguments",
        VaultError::SecretValue(_) => "invalid_secret_values",
        VaultError::Status(_) | VaultError::Http(_) | VaultError::TokenFile(_) => "vault_error",
    };

    tool_error(code, error.to_string(), json!({ "tool": tool }))
}
