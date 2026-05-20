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
        text: "Inspect the Supernode through read-only resources and discovery tools first. Use supernode://status, supernode://control-plane/status, and supernode://extensions/catalog to understand the environment. Do not bootstrap infrastructure from MCP, do not shell out, and do not request raw Kubernetes, Vault, or Helm proxy access.",
    },
    PromptSpec {
        name: "cardano-relay-setup",
        title: "Cardano Relay Setup",
        description: "Plan a Cardano relay install through the catalog workflow.",
        text: "Use the catalog-driven lifecycle workflow for a Cardano relay. Read supernode://extensions/catalog/cardano-relay, validate required extension configuration values, and use workloads.install when tool execution is available. Do not use extension-specific install tools or raw Helm values.",
    },
    PromptSpec {
        name: "dashboard-access",
        title: "Dashboard Access",
        description: "Inspect dashboard/control-plane access without broad privileges.",
        text: "Inspect control-plane status through read-only MCP resources and typed discovery tools. Keep access scoped to the MCP server permissions; do not reuse the dashboard superadmin cluster role, do not expose bearer tokens, and prefer kubectl port-forward access for the trusted MVP.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_expected_static_prompts() {
        let catalog = PromptCatalog;

        let prompts = catalog.list().prompts;

        assert_eq!(prompts.len(), 3);
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "bootstrap-and-discovery")
        );
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "cardano-relay-setup")
        );
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "dashboard-access")
        );
    }

    #[test]
    fn prompts_reference_catalog_driven_workflows() {
        let catalog = PromptCatalog;

        let prompt = catalog.get("cardano-relay-setup").unwrap();
        let message = &prompt.messages[0];

        assert_eq!(message.role, PromptMessageRole::User);
        let rmcp::model::PromptMessageContent::Text { text } = &message.content else {
            panic!("expected text prompt");
        };
        assert!(text.contains("workloads.install"));
        assert!(text.contains("cardano-relay"));
        assert!(!text.contains("cardano.relay.install"));
    }

    #[test]
    fn unknown_prompt_returns_none() {
        let catalog = PromptCatalog;

        assert!(catalog.get("not-real").is_none());
    }
}
