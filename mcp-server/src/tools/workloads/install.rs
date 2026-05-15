use k8s_openapi::api::storage::v1::StorageClass;
use rmcp::model::CallToolResult;
use rmcp::model::JsonObject;
use serde_json::Value;
use serde_json::json;

use crate::catalog::ExtensionDefinition;
use crate::k8s::HelmReleaseDiscovery;
use crate::k8s::HelmReleaseSummary;
use crate::k8s::KubernetesClient;
use crate::k8s::ResourceListParams;

use super::registry;
use crate::tools::common::kube_error;
use crate::tools::common::tool_error;
use crate::tools::k8s_summaries::storage_class_summary;

#[derive(Debug, Clone)]
pub(crate) struct InstallResolution {
    pub configuration: Value,
    pub available_storage_classes: Vec<Value>,
    pub recommended_storage_classes: Vec<String>,
}

pub(crate) async fn resolve_configuration(
    extension: &ExtensionDefinition,
    namespace: &str,
    resolved_configuration: Value,
    dry_run: bool,
) -> Result<InstallResolution, CallToolResult> {
    if extension.id != registry::DOLOS_EXTENSION_ID {
        return Ok(InstallResolution {
            configuration: resolved_configuration,
            available_storage_classes: vec![],
            recommended_storage_classes: vec![],
        });
    }

    let client = match KubernetesClient::try_default().await {
        Ok(client) => Some(client),
        Err(_) if dry_run => None,
        Err(error) => return Err(kube_error("workloads.install", error)),
    };

    let mut configuration = resolved_configuration;
    let mut available_storage_classes = vec![];
    let mut recommended_storage_classes = vec![];

    if let Some(client) = &client {
        let storage_classes = client
            .list_storage_classes(&ResourceListParams::default())
            .await
            .map_err(|error| kube_error("workloads.install", error))?;
        available_storage_classes = storage_classes
            .items
            .iter()
            .map(storage_class_summary)
            .collect();
        recommended_storage_classes = recommended_storage_class_names(&storage_classes.items);

        if let Some(storage_class) = input_string(&configuration, "storageClass")
            && !storage_classes
                .items
                .iter()
                .any(|candidate| candidate.metadata.name.as_deref() == Some(storage_class))
        {
            return Err(tool_error(
                "invalid_extension_configuration",
                "unsupported storage class for extension configuration field: storageClass",
                json!({
                    "field": "storageClass",
                    "actualValue": storage_class,
                    "availableStorageClasses": available_storage_classes,
                    "recommendedStorageClasses": recommended_storage_classes,
                }),
            ));
        }

        configuration =
            resolve_dolos_upstream_configuration(namespace, configuration, client).await?;
    }

    if input_string(&configuration, "upstreamAddress").is_none() {
        return Err(tool_error(
            "invalid_extension_configuration",
            "missing required extension configuration value: upstreamAddress",
            json!({
                "field": "upstreamAddress",
                "reason": "no same-network Cardano relay could be resolved from installed workloads",
            }),
        ));
    }

    Ok(InstallResolution {
        configuration,
        available_storage_classes,
        recommended_storage_classes,
    })
}

pub(crate) fn apply_defaults(extension: &ExtensionDefinition, inputs: Value) -> Value {
    let defaults = match extension.id.as_str() {
        "cardano-node-relay" => cardano_node_relay_defaults(input_string(&inputs, "network")),
        "dolos" => dolos_defaults(input_string(&inputs, "network")),
        _ => json!({}),
    };

    merge_defaults(&defaults, inputs)
}

pub(crate) fn planned_helm_values(
    extension: &ExtensionDefinition,
    release_name: &str,
    inputs: &Value,
) -> Value {
    let mut helm_values = JsonObject::new();
    helm_values.insert(
        "displayName".to_string(),
        Value::String(release_name.to_string()),
    );

    if let Some(storage_class) = input_string(inputs, "storageClass") {
        insert_path(
            &mut helm_values,
            &["persistence", "storageClass"],
            Value::String(storage_class.to_string()),
        );
    }

    if extension.id == "cardano-node-relay" {
        plan_cardano_node_values(inputs, &mut helm_values);
    } else if extension.id == registry::DOLOS_EXTENSION_ID {
        plan_dolos_values(inputs, &mut helm_values);
    }

    Value::Object(helm_values)
}

pub(crate) fn dolos_defaults(network: Option<&str>) -> Value {
    let pvc_size = match network {
        Some("cardano-mainnet") => "300Gi",
        _ => "50Gi",
    };

    json!({
        "imageTag": "v1.1.1",
        "exposeLoadBalancer": false,
        "pvcSize": pvc_size,
    })
}

pub(crate) fn resolve_dolos_upstream_from_releases(
    namespace: &str,
    dolos_network: &str,
    releases: &[HelmReleaseSummary],
) -> Option<String> {
    let mut matches = releases
        .iter()
        .filter(|release| release.chart.name.as_deref() == Some(registry::CARDANO_NODE_CHART_NAME))
        .filter(|release| {
            release
                .status
                .as_deref()
                .is_none_or(|status| status == "deployed")
        })
        .filter(|release| {
            cardano_release_network(release).as_deref()
                == Some(dolos_to_cardano_network(dolos_network))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        let left_same_namespace = left.namespace == namespace;
        let right_same_namespace = right.namespace == namespace;
        right_same_namespace
            .cmp(&left_same_namespace)
            .then_with(|| left.namespace.cmp(&right.namespace))
            .then_with(|| left.name.cmp(&right.name))
    });

    let selected = matches.into_iter().next()?;
    let port = selected
        .config
        .as_ref()
        .and_then(|config| config.pointer("/service/n2nPort"))
        .and_then(Value::as_i64)
        .unwrap_or(3000);
    let service_name = release_fullname(&selected.name, registry::CARDANO_NODE_CHART_NAME);

    Some(format!(
        "{service_name}.{}.svc.cluster.local:{port}",
        selected.namespace
    ))
}

async fn resolve_dolos_upstream_configuration(
    namespace: &str,
    configuration: Value,
    client: &KubernetesClient,
) -> Result<Value, CallToolResult> {
    if input_string(&configuration, "upstreamAddress").is_some() {
        return Ok(configuration);
    }

    let network = input_string(&configuration, "network").ok_or_else(|| {
        tool_error(
            "invalid_extension_configuration",
            "missing required extension configuration value: network",
            json!({ "field": "network" }),
        )
    })?;
    let releases = HelmReleaseDiscovery::new(client.clone())
        .list_latest(None, true)
        .await
        .map_err(|error| {
            tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": "workloads.install" }),
            )
        })?;

    let Some(upstream_address) =
        resolve_dolos_upstream_from_releases(namespace, network, &releases)
    else {
        return Ok(configuration);
    };

    Ok(merge_defaults(
        &json!({ "upstreamAddress": upstream_address }),
        configuration,
    ))
}

fn cardano_node_relay_defaults(network: Option<&str>) -> Value {
    let (pvc_size, cpu_request, memory_request, cpu_limit, memory_limit) = match network {
        Some("mainnet") => ("250Gi", "2", "8Gi", "4", "16Gi"),
        Some("preprod") => ("120Gi", "1", "4Gi", "2", "8Gi"),
        _ => ("80Gi", "500m", "2Gi", "1", "4Gi"),
    };

    json!({
        "topology": { "mode": "image-default" },
        "exposeLoadBalancer": false,
        "imageTag": "11.0.1",
        "resources": {
            "requests": {
                "cpu": cpu_request,
                "memory": memory_request,
            },
            "limits": {
                "cpu": cpu_limit,
                "memory": memory_limit,
            },
        },
        "pvcSize": pvc_size,
    })
}

fn plan_cardano_node_values(inputs: &Value, helm_values: &mut JsonObject) {
    if let Some(network) = input_string(inputs, "network") {
        insert_path(
            helm_values,
            &["node", "network"],
            Value::String(network.to_string()),
        );
    }
    if let Some(pvc_size) = input_string(inputs, "pvcSize") {
        insert_path(
            helm_values,
            &["persistence", "size"],
            Value::String(pvc_size.to_string()),
        );
    }
    if let Some(image_tag) = input_string(inputs, "imageTag") {
        insert_path(
            helm_values,
            &["image", "tag"],
            Value::String(image_tag.to_string()),
        );
    }
    if inputs
        .get("exposeLoadBalancer")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        insert_path(
            helm_values,
            &["service", "type"],
            Value::String("LoadBalancer".to_string()),
        );
    }
    if let Some(resources) = inputs.get("resources") {
        helm_values.insert("resources".to_string(), resources.clone());
    }
    if let Some(topology) = inputs.get("topology") {
        insert_path(helm_values, &["node", "topology"], topology.clone());
    }
    insert_path(
        helm_values,
        &["node", "blockProducer", "enabled"],
        Value::Bool(false),
    );
}

fn plan_dolos_values(inputs: &Value, helm_values: &mut JsonObject) {
    if let Some(network) = input_string(inputs, "network") {
        insert_path(
            helm_values,
            &["dolos", "network"],
            Value::String(network.to_string()),
        );
    }
    if let Some(upstream_address) = input_string(inputs, "upstreamAddress") {
        insert_path(
            helm_values,
            &["config", "upstreamAddress"],
            Value::String(upstream_address.to_string()),
        );
    }
    if let Some(pvc_size) = input_string(inputs, "pvcSize") {
        insert_path(
            helm_values,
            &["persistence", "size"],
            Value::String(pvc_size.to_string()),
        );
    }
    if let Some(image_tag) = input_string(inputs, "imageTag") {
        insert_path(
            helm_values,
            &["image", "tag"],
            Value::String(image_tag.to_string()),
        );
    }
    if inputs
        .get("exposeLoadBalancer")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        insert_path(
            helm_values,
            &["service", "type"],
            Value::String("LoadBalancer".to_string()),
        );
    }
    if let Some(resources) = inputs.get("resources") {
        helm_values.insert("resources".to_string(), resources.clone());
    }
}

fn merge_defaults(defaults: &Value, values: Value) -> Value {
    match (defaults, values) {
        (Value::Object(defaults), Value::Object(mut values)) => {
            for (key, default_value) in defaults {
                let value = values
                    .remove(key)
                    .map(|value| merge_defaults(default_value, value))
                    .unwrap_or_else(|| default_value.clone());
                values.insert(key.clone(), value);
            }
            Value::Object(values)
        }
        (_, values) => values,
    }
}

fn insert_path(root: &mut JsonObject, path: &[&str], value: Value) {
    let Some((last, parents)) = path.split_last() else {
        return;
    };
    let mut current = root;

    for parent in parents {
        current = current
            .entry((*parent).to_string())
            .or_insert_with(|| Value::Object(JsonObject::new()))
            .as_object_mut()
            .expect("planned Helm value parent must be an object");
    }

    current.insert((*last).to_string(), value);
}

fn input_string<'a>(inputs: &'a Value, name: &str) -> Option<&'a str> {
    inputs.get(name).and_then(Value::as_str)
}

fn cardano_release_network(release: &HelmReleaseSummary) -> Option<String> {
    release
        .config
        .as_ref()
        .and_then(|config| config.pointer("/node/network"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn dolos_to_cardano_network(network: &str) -> &str {
    match network {
        "cardano-mainnet" => "mainnet",
        "cardano-preprod" => "preprod",
        _ => "preview",
    }
}

fn recommended_storage_class_names(storage_classes: &[StorageClass]) -> Vec<String> {
    let mut defaults = storage_classes
        .iter()
        .filter(|storage_class| {
            storage_class
                .metadata
                .annotations
                .as_ref()
                .is_some_and(|annotations| {
                    annotations
                        .get("storageclass.kubernetes.io/is-default-class")
                        .is_some_and(|value| value == "true")
                        || annotations
                            .get("storageclass.beta.kubernetes.io/is-default-class")
                            .is_some_and(|value| value == "true")
                })
        })
        .filter_map(|storage_class| storage_class.metadata.name.clone())
        .collect::<Vec<_>>();

    defaults.sort();
    if !defaults.is_empty() {
        return defaults;
    }

    let mut names = storage_classes
        .iter()
        .filter_map(|storage_class| storage_class.metadata.name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn release_fullname(release_name: &str, chart_name: &str) -> String {
    if release_name.contains(chart_name) {
        release_name.to_string()
    } else {
        format!("{release_name}-{chart_name}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::k8s::HelmChartSummary;

    use super::*;

    #[test]
    fn upstream_resolution_prefers_same_namespace_then_name() {
        let releases = vec![
            helm_release_with_config(
                "relay-z",
                "other",
                Some("cardano-node"),
                json!({ "node": { "network": "preview" } }),
            ),
            helm_release_with_config(
                "relay-b",
                "cardano",
                Some("cardano-node"),
                json!({ "node": { "network": "preview" } }),
            ),
            helm_release_with_config(
                "relay-a",
                "cardano",
                Some("cardano-node"),
                json!({ "node": { "network": "preview" }, "service": { "n2nPort": 3100 } }),
            ),
        ];

        let upstream =
            resolve_dolos_upstream_from_releases("cardano", "cardano-preview", &releases).unwrap();

        assert_eq!(
            upstream,
            "relay-a-cardano-node.cardano.svc.cluster.local:3100"
        );
    }

    #[test]
    fn dolos_defaults_match_network_size_policy() {
        assert_eq!(
            dolos_defaults(Some("cardano-mainnet")).pointer("/pvcSize"),
            Some(&json!("300Gi"))
        );
        assert_eq!(
            dolos_defaults(Some("cardano-preprod")).pointer("/pvcSize"),
            Some(&json!("50Gi"))
        );
        assert_eq!(
            dolos_defaults(Some("cardano-preview")).pointer("/imageTag"),
            Some(&json!("v1.1.1"))
        );
    }

    fn helm_release_with_config(
        name: &str,
        namespace: &str,
        chart_name: Option<&str>,
        config: Value,
    ) -> HelmReleaseSummary {
        HelmReleaseSummary {
            name: name.to_string(),
            namespace: namespace.to_string(),
            revision: 1,
            status: Some("deployed".to_string()),
            chart: HelmChartSummary {
                name: chart_name.map(str::to_string),
                version: Some("0.1.0".to_string()),
            },
            app_version: Some("11.0.1".to_string()),
            description: None,
            updated: None,
            secret_name: Some(format!("sh.helm.release.v1.{name}.v1")),
            config: Some(config),
        }
    }
}
