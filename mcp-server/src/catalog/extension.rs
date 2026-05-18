use serde::Serialize;
use serde_json::Value;

pub type ExtensionId = String;
pub type ExtensionConfiguration = Value;
pub type ExtensionMetrics = Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSecretDefinition {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub required_when: Option<String>,
    pub scope: String,
    pub material: String,
    pub write_only: bool,
    pub accepted_sources: Vec<String>,
}

impl ExtensionSecretDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        description: &str,
        required: bool,
        required_when: Option<&str>,
        scope: &str,
        material: &str,
        write_only: bool,
        accepted_sources: Vec<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required,
            required_when: required_when.map(str::to_string),
            scope: scope.to_string(),
            material: material.to_string(),
            write_only,
            accepted_sources: accepted_sources.into_iter().map(str::to_string).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionOutputDefinition {
    pub name: String,
    pub description: String,
    pub port_name: String,
    pub protocol: String,
}

impl ExtensionOutputDefinition {
    pub fn new(name: &str, description: &str, port_name: &str, protocol: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            port_name: port_name.to_string(),
            protocol: protocol.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDefinition {
    pub id: ExtensionId,
    pub name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub default_version: String,
    pub configuration: ExtensionConfiguration,
    pub secrets: Vec<ExtensionSecretDefinition>,
    pub dependencies: Vec<ExtensionId>,
    pub metrics: ExtensionMetrics,
    pub outputs: Vec<ExtensionOutputDefinition>,
    pub chart: String,
}

impl ExtensionDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        versions: Vec<&str>,
        default_version: &str,
        configuration: ExtensionConfiguration,
        secrets: Vec<ExtensionSecretDefinition>,
        dependencies: Vec<&str>,
        metrics: ExtensionMetrics,
        outputs: Vec<ExtensionOutputDefinition>,
        chart: String,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            versions: versions.into_iter().map(str::to_string).collect(),
            default_version: default_version.to_string(),
            configuration,
            secrets,
            dependencies: dependencies.into_iter().map(str::to_string).collect(),
            metrics,
            outputs,
            chart,
        }
    }
}
