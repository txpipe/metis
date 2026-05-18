use crate::policy::ApprovalClass;
use crate::policy::Scope;

use super::ToolDefinition;

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
