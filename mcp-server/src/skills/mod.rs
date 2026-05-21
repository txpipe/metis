mod oci;
pub mod source;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

const SKILL_CATALOG_SCHEMA_VERSION: &str = "supernode.skillCatalog/v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillCatalogDocument {
    pub schema_version: String,
    pub skills: Vec<SkillDefinition>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub extensions: Vec<String>,
    pub tools: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillCatalog {
    skills: BTreeMap<String, SkillDefinition>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSummary<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub tags: &'a [String],
    pub extensions: &'a [String],
    pub tools: &'a [String],
}

pub fn skill_summary(skill: &SkillDefinition) -> SkillSummary<'_> {
    SkillSummary {
        id: &skill.id,
        title: &skill.title,
        description: &skill.description,
        tags: &skill.tags,
        extensions: &skill.extensions,
        tools: &skill.tools,
    }
}

impl SkillCatalog {
    pub fn bundled() -> Self {
        Self::from_json_str(include_str!("../../../catalog/skill-catalog.json"))
            .expect("bundled skill catalog must be valid")
    }

    pub fn from_skills(skills: impl IntoIterator<Item = SkillDefinition>) -> Self {
        let skills = skills
            .into_iter()
            .map(|skill| (skill.id.clone(), skill))
            .collect();
        Self { skills }
    }

    pub fn from_json_str(payload: &str) -> Result<Self, SkillCatalogLoadError> {
        let document = serde_json::from_str::<SkillCatalogDocument>(payload)?;
        Self::from_document(document)
    }

    pub fn from_document(document: SkillCatalogDocument) -> Result<Self, SkillCatalogLoadError> {
        if document.schema_version != SKILL_CATALOG_SCHEMA_VERSION {
            return Err(SkillCatalogLoadError::UnsupportedSchemaVersion(
                document.schema_version,
            ));
        }

        validate_skills(&document.skills)?;
        Ok(Self::from_skills(document.skills))
    }

    #[cfg(test)]
    pub fn testing() -> Self {
        Self::bundled()
    }

    pub fn list(&self) -> impl Iterator<Item = &SkillDefinition> {
        self.skills.values()
    }

    pub fn get(&self, skill_id: &str) -> Option<&SkillDefinition> {
        self.skills.get(skill_id)
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkillCatalogLoadError {
    #[error("skill catalog JSON is invalid: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("skill catalog JSON is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("unsupported skill catalog schema version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("invalid skill catalog: {0}")]
    InvalidCatalog(String),
    #[error("missing skill catalog OCI reference")]
    MissingOciReference,
    #[error("untrusted skill catalog OCI reference: {0}")]
    UntrustedCatalogReference(String),
    #[error("failed to load skill catalog from OCI: {0}")]
    Oci(#[from] oci::OciSkillCatalogError),
}

fn validate_skills(skills: &[SkillDefinition]) -> Result<(), SkillCatalogLoadError> {
    let mut ids = BTreeSet::new();
    for skill in skills {
        if skill.id.trim().is_empty() {
            return invalid_catalog("skill id must not be empty");
        }
        if !is_valid_skill_id(&skill.id) {
            return invalid_catalog(format!("skill id is not URI-safe: {}", skill.id));
        }
        if !ids.insert(skill.id.as_str()) {
            return invalid_catalog(format!("duplicate skill id: {}", skill.id));
        }
        if skill.title.trim().is_empty() {
            return invalid_catalog(format!("skill title must not be empty: {}", skill.id));
        }
        if skill.description.trim().is_empty() {
            return invalid_catalog(format!("skill description must not be empty: {}", skill.id));
        }
        if skill.content.trim().is_empty() {
            return invalid_catalog(format!("skill content must not be empty: {}", skill.id));
        }
    }

    Ok(())
}

fn is_valid_skill_id(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn invalid_catalog<T>(message: impl Into<String>) -> Result<T, SkillCatalogLoadError> {
    Err(SkillCatalogLoadError::InvalidCatalog(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_loads_skills() {
        let catalog = SkillCatalog::bundled();

        assert!(catalog.len() >= 20);
        assert!(catalog.get("cardano-relay-setup").is_some());
        assert!(catalog.get("hydra-node-troubleshooting").is_some());
    }

    #[test]
    fn rejects_duplicate_skill_ids() {
        let skill = test_skill("duplicate");

        let error = SkillCatalog::from_document(SkillCatalogDocument {
            schema_version: SKILL_CATALOG_SCHEMA_VERSION.to_string(),
            skills: vec![skill.clone(), skill],
        })
        .unwrap_err();

        assert!(matches!(error, SkillCatalogLoadError::InvalidCatalog(_)));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let error = SkillCatalog::from_document(SkillCatalogDocument {
            schema_version: "not-real".to_string(),
            skills: vec![],
        })
        .unwrap_err();

        assert!(matches!(
            error,
            SkillCatalogLoadError::UnsupportedSchemaVersion(_)
        ));
    }

    #[test]
    fn summaries_omit_content() {
        let catalog = SkillCatalog::bundled();
        let skill = catalog.get("cardano-relay-setup").unwrap();

        let value = serde_json::to_value(skill_summary(skill)).unwrap();

        assert_eq!(value.pointer("/id").unwrap(), "cardano-relay-setup");
        assert!(value.get("content").is_none());
    }

    fn test_skill(id: &str) -> SkillDefinition {
        SkillDefinition {
            id: id.to_string(),
            title: "Test Skill".to_string(),
            description: "Test skill.".to_string(),
            tags: vec![],
            extensions: vec![],
            tools: vec![],
            content: "content".to_string(),
        }
    }
}
