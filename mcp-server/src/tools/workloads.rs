use crate::policy::ApprovalClass;
use crate::policy::Scope;

use super::ToolDefinition;

pub fn definitions() -> &'static [ToolDefinition] {
    &[
        ToolDefinition {
            name: "workloads.list",
            title: "List Workloads",
            description: "List installed Helm workloads, namespaces, charts, versions, and status.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{"includeControlPlane":{"type":"boolean"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.get",
            title: "Get Workload",
            description: "Inspect one workload release and related Kubernetes objects.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","name"],"properties":{"namespace":{"type":"string"},"name":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.logs.get",
            title: "Get Workload Logs",
            description: "Read bounded pod logs for a workload.",
            required_scope: Scope::Debug,
            approval_class: ApprovalClass::ReadOnlyDebug,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","workload"],"properties":{"namespace":{"type":"string"},"workload":{"type":"string"},"tailLines":{"type":"integer","minimum":1,"maximum":1000}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.metrics.get",
            title: "Get Workload Metrics",
            description: "Read raw or derived metrics for a workload.",
            required_scope: Scope::Debug,
            approval_class: ApprovalClass::ReadOnlyDebug,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","workload"],"properties":{"namespace":{"type":"string"},"workload":{"type":"string"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.install",
            title: "Install Workload",
            description: "Install a supported catalog extension with validated inputs.",
            required_scope: Scope::WorkloadsInstall,
            approval_class: ApprovalClass::Mutation,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["extensionId","releaseName","namespace","configuration"],"properties":{"extensionId":{"type":"string"},"releaseName":{"type":"string"},"namespace":{"type":"string"},"configuration":{"type":"object"},"dryRun":{"type":"boolean"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.upgrade",
            title: "Upgrade Workload",
            description: "Upgrade an installed workload using catalog schema validation.",
            required_scope: Scope::WorkloadsUpgrade,
            approval_class: ApprovalClass::Mutation,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","releaseName","configuration"],"properties":{"namespace":{"type":"string"},"releaseName":{"type":"string"},"configuration":{"type":"object"},"dryRun":{"type":"boolean"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.delete",
            title: "Delete Workload",
            description: "Delete a workload, preserving PVCs by default.",
            required_scope: Scope::WorkloadsDelete,
            approval_class: ApprovalClass::Destructive,
            read_only: false,
            destructive: true,
            input_schema: r#"{"type":"object","required":["namespace","releaseName"],"properties":{"namespace":{"type":"string"},"releaseName":{"type":"string"},"deletePvcs":{"type":"boolean"},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
        },
    ]
}
