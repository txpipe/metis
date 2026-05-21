use rmcp::model::GetPromptResult;
use rmcp::model::ListPromptsResult;
use rmcp::model::Prompt;
use rmcp::model::PromptMessage;
use rmcp::model::PromptMessageRole;

#[derive(Debug, Clone, Copy)]
struct PromptSpec {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    text: &'static str,
}

#[derive(Debug, Clone)]
pub struct PromptCatalog;

impl PromptCatalog {
    pub fn list(&self) -> ListPromptsResult {
        ListPromptsResult::with_all_items(
            PROMPTS
                .iter()
                .map(|prompt| {
                    Prompt::new(prompt.name, Some(prompt.description), None)
                        .with_title(prompt.title)
                })
                .collect(),
        )
    }

    pub fn get(&self, name: &str) -> Option<GetPromptResult> {
        let prompt = PROMPTS.iter().find(|prompt| prompt.name == name)?;
        Some(
            GetPromptResult::new(vec![PromptMessage::new_text(
                PromptMessageRole::User,
                prompt.text,
            )])
            .with_description(prompt.description),
        )
    }
}

const PROMPTS: &[PromptSpec] = &[
    PromptSpec {
        name: "bootstrap-and-discovery",
        title: "Bootstrap And Discovery",
        description: "Discover an existing Supernode cluster before making changes.",
        text: "Inspect the Supernode through read-only MCP resources and discovery tools first. Read supernode://status, supernode://extensions/catalog, and supernode://skills. Select and read a specific supernode://skills/{skillId} guide only when it matches the operator task. Do not bootstrap infrastructure from MCP, do not shell out, and do not request raw Kubernetes, Vault, or Helm proxy access.",
    },
    PromptSpec {
        name: "install-catalog-extension",
        title: "Install Catalog Extension",
        description: "Plan a catalog-backed workload install with skill guidance.",
        text: "Use MCP tools only. Read supernode://skills and select the most relevant skill guide for the requested workload. Read supernode://extensions/catalog/{extensionId} as the source of truth for configuration shape. Start workloads.install with dryRun=true and run live only after operator approval.",
    },
    PromptSpec {
        name: "troubleshoot-workload",
        title: "Troubleshoot Workload",
        description: "Troubleshoot catalog-managed workloads with scoped MCP reads.",
        text: "Use MCP tools only. Read supernode://skills and select the relevant troubleshooting or verification skill guide. Inspect workload state, logs, metrics, and cluster events through MCP tools. If the required operation is outside MCP capabilities, state that MCP does not currently expose it and stop for operator direction.",
    },
    PromptSpec {
        name: "dashboard-access",
        title: "Dashboard Access",
        description: "Inspect dashboard/control-plane access without broad privileges.",
        text: "Inspect control-plane status through read-only MCP resources and typed discovery tools. Read supernode://skills/supernode-dashboard-port-forward for the current access boundary. Keep access scoped to the MCP server permissions; do not reuse the dashboard superadmin cluster role and do not expose bearer tokens.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_expected_static_prompts() {
        let catalog = PromptCatalog;

        let prompts = catalog.list().prompts;

        assert_eq!(prompts.len(), 4);
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "bootstrap-and-discovery")
        );
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "install-catalog-extension")
        );
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "troubleshoot-workload")
        );
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "dashboard-access")
        );
    }

    #[test]
    fn prompts_reference_skill_driven_workflows() {
        let catalog = PromptCatalog;

        let prompt = catalog.get("install-catalog-extension").unwrap();
        let message = &prompt.messages[0];

        assert_eq!(message.role, PromptMessageRole::User);
        let rmcp::model::PromptMessageContent::Text { text } = &message.content else {
            panic!("expected text prompt");
        };
        assert!(text.contains("workloads.install"));
        assert!(text.contains("supernode://skills"));
        assert!(text.contains("supernode://extensions/catalog/{extensionId}"));
        assert!(!text.contains("cardano.relay.install"));
    }

    #[test]
    fn unknown_prompt_returns_none() {
        let catalog = PromptCatalog;

        assert!(catalog.get("not-real").is_none());
    }
}
