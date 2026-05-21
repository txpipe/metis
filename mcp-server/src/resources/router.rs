use std::sync::Arc;

use rmcp::model::Annotated;
use rmcp::model::ListResourcesResult;
use rmcp::model::RawResource;
use rmcp::model::ReadResourceResult;
use rmcp::model::Resource;
use rmcp::model::ResourceContents;
use serde::Serialize;
use serde_json::json;

use crate::auth::AuthContext;
use crate::catalog::{ExtensionCatalog, extension_summary};
use crate::skills::{SkillCatalog, skill_summary};

use super::uri::CONTROL_PLANE_STATUS_URI;
use super::uri::EXTENSION_CATALOG_URI;
use super::uri::SKILLS_URI;
use super::uri::STATUS_URI;
use super::uri::SupernodeResourceUri;
use super::uri::extension_catalog_entry_uri;
use super::uri::skill_entry_uri;

const JSON_MIME_TYPE: &str = "application/json";

#[derive(Debug, Clone)]
pub struct ResourceRouter {
    catalog: Arc<ExtensionCatalog>,
    skill_catalog: Arc<SkillCatalog>,
}

impl ResourceRouter {
    pub fn new(catalog: Arc<ExtensionCatalog>, skill_catalog: Arc<SkillCatalog>) -> Self {
        Self {
            catalog,
            skill_catalog,
        }
    }

    pub fn list(&self) -> ListResourcesResult {
        let mut resources = vec![
            resource(
                STATUS_URI,
                "supernode-status",
                "Supernode Status",
                "Static MCP server status and runtime mode.",
            ),
            resource(
                CONTROL_PLANE_STATUS_URI,
                "control-plane-status",
                "Control Plane Status",
                "Static control-plane status placeholder until Kubernetes discovery is available.",
            ),
            resource(
                EXTENSION_CATALOG_URI,
                "extensions-catalog",
                "Extension Catalog",
                "Summary catalog of extensions supported by this MCP server.",
            ),
            resource(
                SKILLS_URI,
                "skills-catalog",
                "Skill Catalog",
                "Summary catalog of operational skill guides available through this MCP server.",
            ),
        ];

        resources.extend(self.catalog.list().map(|extension| {
            resource(
                extension_catalog_entry_uri(&extension.id),
                format!("extension-catalog-{}", extension.id),
                extension.name.clone(),
                format!("Full catalog entry for {}.", extension.name),
            )
        }));

        resources.extend(self.skill_catalog.list().map(|skill| {
            resource(
                skill_entry_uri(&skill.id),
                format!("skill-{}", skill.id),
                skill.title.clone(),
                format!("Full operational skill guide for {}.", skill.title),
            )
        }));

        ListResourcesResult::with_all_items(resources)
    }

    pub fn read(
        &self,
        uri: &str,
        auth: &AuthContext,
    ) -> Result<ReadResourceResult, ResourceReadError> {
        let parsed = SupernodeResourceUri::parse(uri).ok_or(ResourceReadError::NotFound)?;
        let value = match parsed {
            SupernodeResourceUri::Status => json!({
                "status": "ok",
                "authMode": auth.auth_mode,
                "policyEnforced": auth.enforced,
                "catalogExtensionCount": self.catalog.len(),
            }),
            SupernodeResourceUri::ControlPlaneStatus => json!({
                "status": "unknown",
                "reason": "kubernetes-discovery-not-implemented",
            }),
            SupernodeResourceUri::ExtensionCatalog => json!({
                "extensions": self.catalog.list().map(extension_summary).collect::<Vec<_>>(),
            }),
            SupernodeResourceUri::ExtensionCatalogEntry { extension_id } => serde_json::to_value(
                self.catalog
                    .get(extension_id)
                    .ok_or(ResourceReadError::NotFound)?,
            )?,
            SupernodeResourceUri::Skills => json!({
                "skills": self.skill_catalog.list().map(skill_summary).collect::<Vec<_>>(),
            }),
            SupernodeResourceUri::SkillEntry { skill_id } => serde_json::to_value(
                self.skill_catalog
                    .get(skill_id)
                    .ok_or(ResourceReadError::NotFound)?,
            )?,
        };

        Ok(text_resource(uri, &value)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceReadError {
    #[error("resource not found")]
    NotFound,
    #[error("failed to serialize resource")]
    Serialize(#[from] serde_json::Error),
}

fn resource(
    uri: impl Into<String>,
    name: impl Into<String>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> Resource {
    Annotated::new(
        RawResource::new(uri, name)
            .with_title(title)
            .with_description(description)
            .with_mime_type(JSON_MIME_TYPE),
        None,
    )
}

fn text_resource(
    uri: &str,
    value: &impl Serialize,
) -> Result<ReadResourceResult, serde_json::Error> {
    let text = serde_json::to_string_pretty(value)?;
    Ok(ReadResourceResult::new(vec![
        ResourceContents::text(text, uri).with_mime_type(JSON_MIME_TYPE),
    ]))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn lists_static_and_extension_catalog_resources() {
        let router = ResourceRouter::new(
            Arc::new(ExtensionCatalog::testing()),
            Arc::new(SkillCatalog::testing()),
        );

        let resources = router.list().resources;

        assert!(resources.iter().any(|resource| resource.uri == STATUS_URI));
        assert!(
            resources
                .iter()
                .any(|resource| resource.uri == EXTENSION_CATALOG_URI)
        );
        assert!(
            resources
                .iter()
                .any(|resource| { resource.uri == extension_catalog_entry_uri("cardano-relay") })
        );
        assert!(resources.iter().any(|resource| resource.uri == SKILLS_URI));
        assert!(
            resources
                .iter()
                .any(|resource| { resource.uri == skill_entry_uri("cardano-relay-setup") })
        );
    }

    #[test]
    fn reads_catalog_resource_as_json() {
        let router = ResourceRouter::new(
            Arc::new(ExtensionCatalog::testing()),
            Arc::new(SkillCatalog::testing()),
        );

        let result = router
            .read(EXTENSION_CATALOG_URI, &AuthContext::trusted())
            .unwrap();

        assert_eq!(result.contents.len(), 1);
        let ResourceContents::TextResourceContents {
            text, mime_type, ..
        } = &result.contents[0]
        else {
            panic!("expected text resource");
        };
        assert_eq!(mime_type.as_deref(), Some(JSON_MIME_TYPE));
        assert!(text.contains("cardano-relay"));
        let value = serde_json::from_str::<Value>(text).unwrap();
        let relay = value
            .pointer("/extensions")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .find(|extension| {
                extension.pointer("/id") == Some(&Value::String("cardano-relay".to_string()))
            })
            .unwrap();
        assert!(relay.get("configuration").is_none());
        assert!(relay.get("metrics").is_none());
        assert!(relay.get("metricsCollection").is_none());
        assert!(relay.get("secrets").is_none());
        assert!(
            relay
                .pointer("/outputs")
                .and_then(Value::as_array)
                .is_some_and(|outputs| !outputs.is_empty())
        );
        assert!(!text.contains("secret-value"));
    }

    #[test]
    fn reads_one_catalog_entry() {
        let router = ResourceRouter::new(
            Arc::new(ExtensionCatalog::testing()),
            Arc::new(SkillCatalog::testing()),
        );

        let result = router
            .read(
                &extension_catalog_entry_uri("cardano-relay"),
                &AuthContext::trusted(),
            )
            .unwrap();

        let ResourceContents::TextResourceContents { text, .. } = &result.contents[0] else {
            panic!("expected text resource");
        };
        assert!(text.contains("Cardano Relay"));
        assert!(text.contains("configuration"));
    }

    #[test]
    fn unknown_resource_returns_not_found() {
        let router = ResourceRouter::new(
            Arc::new(ExtensionCatalog::testing()),
            Arc::new(SkillCatalog::testing()),
        );

        let error = router
            .read(
                "supernode://extensions/catalog/not-real",
                &AuthContext::trusted(),
            )
            .unwrap_err();

        assert!(matches!(error, ResourceReadError::NotFound));
    }

    #[test]
    fn reads_skill_catalog_resource_as_json_summary() {
        let router = ResourceRouter::new(
            Arc::new(ExtensionCatalog::testing()),
            Arc::new(SkillCatalog::testing()),
        );

        let result = router.read(SKILLS_URI, &AuthContext::trusted()).unwrap();

        let ResourceContents::TextResourceContents { text, .. } = &result.contents[0] else {
            panic!("expected text resource");
        };
        let value = serde_json::from_str::<Value>(text).unwrap();
        let relay = value
            .pointer("/skills")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .find(|skill| {
                skill.pointer("/id") == Some(&Value::String("cardano-relay-setup".to_string()))
            })
            .unwrap();
        assert!(relay.get("content").is_none());
        assert!(relay.pointer("/tools").is_some());
    }

    #[test]
    fn reads_one_skill_entry() {
        let router = ResourceRouter::new(
            Arc::new(ExtensionCatalog::testing()),
            Arc::new(SkillCatalog::testing()),
        );

        let result = router
            .read(
                &skill_entry_uri("cardano-relay-setup"),
                &AuthContext::trusted(),
            )
            .unwrap();

        let ResourceContents::TextResourceContents { text, .. } = &result.contents[0] else {
            panic!("expected text resource");
        };
        assert!(text.contains("Cardano Relay Setup"));
        assert!(text.contains("\"content\""));
    }
}
