use serde::Serialize;
use serde_json::Value;

pub type ExtensionId = String;
pub type ExtensionConfiguration = Value;
pub type ExtensionMetrics = Value;

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
    pub secrets: Vec<String>,
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
        secrets: Vec<&str>,
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
            secrets: secrets.into_iter().map(str::to_string).collect(),
            dependencies: dependencies.into_iter().map(str::to_string).collect(),
            metrics,
            outputs,
            chart,
        }
    }
}
