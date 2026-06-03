use crate::policy::ApprovalClass;
use crate::policy::Scope;

use super::ToolDefinition;
use std::collections::BTreeSet;

pub(crate) mod delete;
pub(crate) mod dolos;
pub(crate) mod get;
pub(crate) mod install;
pub(crate) mod list;
pub(crate) mod logs;
pub(crate) mod metrics;
pub(crate) mod outputs;
pub(crate) mod registry;
pub(crate) mod upgrade;

pub fn definitions() -> &'static [ToolDefinition] {
    &[
        ToolDefinition {
            name: "workloads.list",
            title: "List Workloads",
            description: "List installed workload summaries with catalog extension outputs when available.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","properties":{"includeControlPlane":{"type":"boolean"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.get",
            title: "Get Workload",
            description: "Inspect one workload release and related Kubernetes objects in detail.",
            required_scope: Scope::Discover,
            approval_class: ApprovalClass::Discovery,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","name"],"properties":{"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"name":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.logs.get",
            title: "Get Workload Logs",
            description: "Read bounded pod logs for one selected workload pod and container, including selected-target diagnostics.",
            required_scope: Scope::Debug,
            approval_class: ApprovalClass::ReadOnlyDebug,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","workload"],"properties":{"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"workload":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"pod":{"type":"string","description":"Exact pod name to read. Required when the workload has multiple active pods."},"container":{"type":"string","description":"Exact container or init-container name to read. Required when the selected pod has multiple loggable containers."},"tailLines":{"type":"integer","minimum":1,"maximum":1000},"sinceSeconds":{"type":"integer","minimum":1},"previous":{"type":"boolean","description":"When true, read logs from the previous terminated instance of the selected container."},"timestamps":{"type":"boolean"}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.metrics.get",
            title: "Get Workload Metrics",
            description: "Read raw or derived metrics for a workload.",
            required_scope: Scope::Debug,
            approval_class: ApprovalClass::ReadOnlyDebug,
            read_only: true,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","workload"],"properties":{"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"workload":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.install",
            title: "Install Workload",
            description: "Install a supported catalog extension with validated inputs.",
            required_scope: Scope::WorkloadsInstall,
            approval_class: ApprovalClass::Mutation,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["extensionId","releaseName","namespace","configuration"],"properties":{"extensionId":{"type":"string","description":"Catalog extension ID to install, for example dolos, cardano-relay, cardano-block-producer, apex-fusion-relay, apex-fusion-block-producer, or hydra-node."},"releaseName":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63,"description":"Helm release name to create or update."},"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63,"description":"Kubernetes namespace where the workload will be installed."},"configuration":{"type":"object","description":"Required extension-specific configuration object. Use extensions.catalog.get for the selected extensionId and pass values matching that extension configuration schema.","additionalProperties":true},"dryRun":{"type":"boolean","description":"When true, validate and return the install plan without mutating Kubernetes. Defaults to true."}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.upgrade",
            title: "Upgrade Workload",
            description: "Upgrade an installed workload using catalog schema validation.",
            required_scope: Scope::WorkloadsUpgrade,
            approval_class: ApprovalClass::Mutation,
            read_only: false,
            destructive: false,
            input_schema: r#"{"type":"object","required":["namespace","releaseName","configuration"],"properties":{"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"releaseName":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"configuration":{"type":"object","description":"Required extension-specific configuration object. Use extensions.catalog.get for the installed extension and pass values matching that extension configuration schema.","additionalProperties":true},"dryRun":{"type":"boolean","description":"When true, validate and return the upgrade plan without mutating Kubernetes."}},"additionalProperties":false}"#,
        },
        ToolDefinition {
            name: "workloads.delete",
            title: "Delete Workload",
            description: "Delete a workload, preserving PVCs by default.",
            required_scope: Scope::WorkloadsDelete,
            approval_class: ApprovalClass::Destructive,
            read_only: false,
            destructive: true,
            input_schema: r#"{"type":"object","required":["namespace","releaseName"],"properties":{"namespace":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"releaseName":{"type":"string","pattern":"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$","maxLength":63},"dryRun":{"type":"boolean","description":"When true, validate and return the delete plan without mutating Kubernetes. Defaults to true."},"deletePvcs":{"type":"boolean","description":"Delete candidate PVCs after Helm uninstall. Defaults to false."},"approvalId":{"type":"string"}},"additionalProperties":false}"#,
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
