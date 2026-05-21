use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ExtensionId = String;
pub type ExtensionConfiguration = Value;
pub type ExtensionMetrics = Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionMetricsCollection {
    pub container: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionOutputDefinition {
    pub name: String,
    pub description: String,
    pub port_name: String,
    pub protocol: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics_collection: Option<ExtensionMetricsCollection>,
    pub outputs: Vec<ExtensionOutputDefinition>,
    pub chart: String,
}
