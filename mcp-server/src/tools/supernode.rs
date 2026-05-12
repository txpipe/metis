use crate::policy::ApprovalClass;
use crate::policy::Scope;

use super::ToolDefinition;

pub fn definitions() -> &'static [ToolDefinition] {
    &[
        ToolDefinition {
            name: "supernode.status.get",
            title: "Get Supernode Status",
            description: "Return overall Supernode control-plane and workload status.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "cluster.storage_classes.list",
            title: "List Storage Classes",
            description: "List available cluster storage classes and defaults.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "cluster.events.list",
            title: "List Cluster Events",
            description: "Read bounded cluster or namespace events for debugging.",
            required_scope: Scope::Debug,
            approval_class: ApprovalClass::ReadOnlyDebug,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{"namespace":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":200}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "extensions.catalog.list",
            title: "List Extension Catalog",
            description: "List supported embedded extension catalog entries.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "extensions.catalog.get",
            title: "Get Extension Catalog Entry",
            description: "Get one extension's configuration and metrics schemas.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["extensionId"],"properties":{"extensionId":{"type":"string"}},"additionalProperties":false}"#,
        },
    ]
}
