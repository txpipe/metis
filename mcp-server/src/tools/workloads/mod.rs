use crate::policy::ApprovalClass;
use crate::policy::Scope;

use super::ToolDefinition;
use std::collections::BTreeSet;

pub(crate) mod dolos;
pub(crate) mod install;
pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod outputs;
pub(crate) mod registry;

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
            description: "Read bounded pod logs for one selected workload pod and container.",
            required_scope: Scope::Debug,
            approval_class: ApprovalClass::ReadOnlyDebug,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","workload"],"properties":{"namespace":{"type":"string"},"workload":{"type":"string"},"pod":{"type":"string","description":"Exact pod name to read. Required when the workload has multiple active pods."},"container":{"type":"string","description":"Exact container or init-container name to read. Required when the selected pod has multiple loggable containers."},"tailLines":{"type":"integer","minimum":1,"maximum":1000},"sinceSeconds":{"type":"integer","minimum":1},"previous":{"type":"boolean"},"timestamps":{"type":"boolean"}},"additionalProperties":false}"#,
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
            input_schema: r#"{"type":"object","required":["extensionId","releaseName","namespace","configuration"],"properties":{"extensionId":{"type":"string","description":"Catalog extension ID to install, for example dolos or cardano-node-relay."},"releaseName":{"type":"string","description":"Helm release name to create or update."},"namespace":{"type":"string","description":"Kubernetes namespace where the workload will be installed."},"configuration":{"type":"object","description":"Required extension-specific configuration object. Use extensions.catalog.get for the selected extensionId and pass values matching that extension configuration schema.","additionalProperties":true},"dryRun":{"type":"boolean","description":"When true, validate and return the install plan without mutating Kubernetes. Defaults to true."}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.upgrade",
            title: "Upgrade Workload",
            description: "Upgrade an installed workload using catalog schema validation.",
            required_scope: Scope::WorkloadsUpgrade,
            approval_class: ApprovalClass::Mutation,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","releaseName","configuration"],"properties":{"namespace":{"type":"string"},"releaseName":{"type":"string"},"configuration":{"type":"object","description":"Required extension-specific configuration object. Use extensions.catalog.get for the installed extension and pass values matching that extension configuration schema.","additionalProperties":true},"dryRun":{"type":"boolean","description":"When true, validate and return the upgrade plan without mutating Kubernetes."}},"additionalProperties":false}"#,
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

pub(crate) fn dynamic_definitions(
    installed_extension_ids: &BTreeSet<String>,
) -> Vec<ToolDefinition> {
    let mut definitions = Vec::new();

    if installed_extension_ids.contains(registry::DOLOS_EXTENSION_ID) {
        definitions.extend(dolos::definitions().iter().copied());
    }

    definitions
}
