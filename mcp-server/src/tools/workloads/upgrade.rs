use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::catalog::ExtensionCatalog;
use crate::helm::{self, HelmChartRef, HelmUpgradePlan};
use crate::k8s::{HelmReleaseDiscovery, KubernetesClient};
use crate::tools::common::{kube_error, success, tool_error};

use super::{install, registry};

const TOOL_NAME: &str = "workloads.upgrade";

pub(crate) async fn upgrade(
    arguments: Option<&JsonObject>,
    catalog: &ExtensionCatalog,
) -> CallToolResult {
    let namespace = match required_string(arguments, "namespace") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let release_name = match required_string(arguments, "releaseName") {
        Ok(value) => value,
        Err(error) => return error,
    };
    let dry_run = optional_bool(arguments, "dryRun").unwrap_or(true);
    let configuration = match required_object(arguments, "configuration") {
        Ok(value) => value,
        Err(error) => return error,
    };

    let client = match KubernetesClient::try_default().await {
        Ok(client) => client,
        Err(error) => return kube_error(TOOL_NAME, error),
    };
    let release = match HelmReleaseDiscovery::new(client)
        .get_latest(&namespace, &release_name)
        .await
    {
        Ok(Some(release)) => release,
        Ok(None) => {
            return tool_error(
                "not_found",
                format!("workload release not found: {namespace}/{release_name}"),
                json!({ "namespace": namespace, "releaseName": release_name }),
            );
        }
        Err(error) => {
            return tool_error(
                "helm_release_discovery_error",
                error.to_string(),
                json!({ "tool": TOOL_NAME, "namespace": namespace, "releaseName": release_name }),
            );
        }
    };
    let extension = match registry::extension_for_release(&release, catalog) {
        Some(extension) => extension,
        None => {
            return tool_error(
                "unsupported_workload",
                "workloads.upgrade only upgrades catalog-managed extension releases",
                json!({
                    "namespace": namespace,
                    "releaseName": release_name,
                    "chart": release.chart,
                }),
            );
        }
    };

    if let Err(error) = validate_configuration_schema(&configuration, &extension.configuration) {
        return error;
    }
    if !matches!(
        extension.id.as_str(),
        registry::DOLOS_EXTENSION_ID
            | registry::CARDANO_RELAY_EXTENSION_ID
            | registry::CARDANO_BLOCK_PRODUCER_EXTENSION_ID
            | registry::APEX_FUSION_RELAY_EXTENSION_ID
            | registry::APEX_FUSION_BLOCK_PRODUCER_EXTENSION_ID
            | registry::HYDRA_NODE_EXTENSION_ID
    ) && configuration.get("namespace").and_then(Value::as_str) != Some(namespace.as_str())
    {
        return tool_error(
            "invalid_arguments",
            "configuration.namespace must match namespace",
            json!({ "namespace": namespace, "configurationNamespace": configuration.get("namespace") }),
        );
    }

    let resolved_configuration = install::apply_defaults(extension, Value::Object(configuration));
    let resolution = match install::resolve_configuration(
        extension,
        &namespace,
        resolved_configuration,
        dry_run,
    )
    .await
    {
        Ok(resolution) => resolution,
        Err(error) => return error,
    };
    let helm_values =
        install::planned_helm_values(extension, &release_name, &resolution.configuration);
    let chart = HelmChartRef {
        chart: extension.chart.clone(),
        version: extension.default_version.clone(),
    };

    if dry_run {
        return success(json!({
            "action": "upgrade",
            "dryRun": true,
            "wouldMutate": false,
            "release": {
                "name": release_name,
                "namespace": namespace,
                "current": release,
            },
            "extension": {
                "id": extension.id,
                "name": extension.name,
                "version": extension.default_version,
            },
            "chart": chart,
            "resolvedConfiguration": resolution.configuration,
            "helmValues": helm_values,
            "availableStorageClasses": resolution.available_storage_classes,
            "recommendedStorageClasses": resolution.recommended_storage_classes,
            "notes": [
                "dry-run planning only; no Kubernetes or Helm mutation was performed",
                "raw Helm values are rejected by the extension configuration schema",
                "upgrade requires an existing release and will not install missing releases"
            ],
        }));
    }

    let helm_result = match helm::upgrade(&HelmUpgradePlan {
        release_name: release_name.clone(),
        namespace: namespace.clone(),
        chart: chart.clone(),
        values: helm_values.clone(),
    })
    .await
    {
        Ok(result) => result,
        Err(error) => {
            let helm_details = match &error {
                helm::HelmUpgradeError::Failed {
                    status,
                    stdout,
                    stderr,
                } => json!({
                    "tool": TOOL_NAME,
                    "extensionId": extension.id,
                    "releaseName": release_name,
                    "namespace": namespace,
                    "status": status,
                    "stdout": stdout,
                    "stderr": stderr,
                }),
                _ => json!({
                    "tool": TOOL_NAME,
                    "extensionId": extension.id,
                    "releaseName": release_name,
                    "namespace": namespace,
                }),
            };
            return tool_error("helm_upgrade_failed", error.to_string(), helm_details);
        }
    };

    success(json!({
        "action": "upgrade",
        "dryRun": false,
        "wouldMutate": true,
        "release": {
            "name": release_name,
            "namespace": namespace,
            "previous": release,
        },
        "extension": {
            "id": extension.id,
            "name": extension.name,
            "version": extension.default_version,
        },
        "chart": chart,
        "resolvedConfiguration": resolution.configuration,
        "helmValues": helm_values,
        "availableStorageClasses": resolution.available_storage_classes,
        "recommendedStorageClasses": resolution.recommended_storage_classes,
        "helm": helm_result,
        "notes": [
            "Helm upgrade completed successfully",
            "raw Helm values are rejected by the extension configuration schema"
        ],
    }))
}

fn validate_configuration_schema(
    values: &JsonObject,
    schema: &Value,
) -> Result<(), CallToolResult> {
    validate_object_value("", values, schema, schema)
}

fn validate_object_value(
    path: &str,
    values: &JsonObject,
    schema: &Value,
    root_schema: &Value,
) -> Result<(), CallToolResult> {
    let schema = dereference_schema(schema, root_schema);
    let schema = schema.as_object().ok_or_else(|| {
        tool_error(
            "catalog_schema_error",
            "extension configuration schema must be an object schema",
            json!({}),
        )
    })?;
    let properties = schema.get("properties").and_then(Value::as_object);

    let additional_properties = schema.get("additionalProperties");
    if additional_properties.and_then(Value::as_bool) == Some(false) {
        for key in values.keys() {
            if !properties.is_some_and(|properties| properties.contains_key(key)) {
                let field = field_path(path, key);
                return Err(tool_error(
                    "invalid_extension_configuration",
                    format!("unknown extension configuration value: {field}"),
                    json!({ "field": field }),
                ));
            }
        }
    }

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for field in required.iter().filter_map(Value::as_str) {
            if !values.contains_key(field) {
                let field = field_path(path, field);
                return Err(tool_error(
                    "invalid_extension_configuration",
                    format!("missing required extension configuration value: {field}"),
                    json!({ "field": field }),
                ));
            }
        }
    }

    for (key, value) in values {
        let field = field_path(path, key);
        if let Some(property_schema) = properties.and_then(|properties| properties.get(key)) {
            validate_property_value(&field, value, property_schema, root_schema)?;
        } else if let Some(additional_schema) = additional_properties.and_then(Value::as_object) {
            validate_property_value(
                &field,
                value,
                &Value::Object(additional_schema.clone()),
                root_schema,
            )?;
        }
    }

    Ok(())
}

fn validate_property_value(
    name: &str,
    value: &Value,
    schema: &Value,
    root_schema: &Value,
) -> Result<(), CallToolResult> {
    let schema = dereference_schema(schema, root_schema);
    if let Some(expected_type) = schema.get("type") {
        let matches = expected_type_matches(value, expected_type);

        if !matches {
            return Err(tool_error(
                "invalid_extension_configuration",
                format!("invalid type for extension configuration value: {name}"),
                json!({
                    "field": name,
                    "expectedType": expected_type,
                    "actualType": value_type_name(value),
                }),
            ));
        }
    }

    if let Some(allowed_values) = schema.get("enum").and_then(Value::as_array)
        && !allowed_values.iter().any(|allowed| allowed == value)
    {
        return Err(tool_error(
            "invalid_extension_configuration",
            format!("unsupported value for extension configuration field: {name}"),
            json!({
                "field": name,
                "allowedValues": allowed_values,
                "actualValue": value,
            }),
        ));
    }

    if let (Some(value), Some(min_length)) = (
        value.as_str(),
        schema.get("minLength").and_then(Value::as_u64),
    ) && value.chars().count() < min_length as usize
    {
        return Err(tool_error(
            "invalid_extension_configuration",
            format!("extension configuration value is too short: {name}"),
            json!({
                "field": name,
                "minLength": min_length,
            }),
        ));
    }

    if let Some(values) = value.as_object() {
        validate_object_value(name, values, schema, root_schema)?;
    } else if let (Some(items), Some(item_schema)) = (value.as_array(), schema.get("items")) {
        for (index, item) in items.iter().enumerate() {
            validate_property_value(&format!("{name}[{index}]"), item, item_schema, root_schema)?;
        }
    }

    Ok(())
}

fn dereference_schema<'a>(schema: &'a Value, root_schema: &'a Value) -> &'a Value {
    let Some(reference) = schema.get("$ref").and_then(Value::as_str) else {
        return schema;
    };
    let Some(name) = reference
        .strip_prefix("#/definitions/")
        .or_else(|| reference.strip_prefix("#/$defs/"))
    else {
        return schema;
    };

    root_schema
        .get("definitions")
        .or_else(|| root_schema.get("$defs"))
        .and_then(|definitions| definitions.get(name))
        .unwrap_or(schema)
}

fn expected_type_matches(value: &Value, expected_type: &Value) -> bool {
    if let Some(expected_type) = expected_type.as_str() {
        return single_type_matches(value, expected_type);
    }

    expected_type.as_array().is_none_or(|types| {
        types
            .iter()
            .filter_map(Value::as_str)
            .any(|ty| single_type_matches(value, ty))
    })
}

fn single_type_matches(value: &Value, expected_type: &str) -> bool {
    match expected_type {
        "array" => value.is_array(),
        "boolean" => value.is_boolean(),
        "integer" => value.as_i64().is_some(),
        "null" => value.is_null(),
        "number" => value.as_f64().is_some(),
        "object" => value.is_object(),
        "string" => value.is_string(),
        _ => true,
    }
}

fn field_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{parent}.{child}")
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn required_object(
    arguments: Option<&JsonObject>,
    name: &str,
) -> Result<JsonObject, CallToolResult> {
    match arguments.and_then(|arguments| arguments.get(name)) {
        Some(Value::Object(value)) => Ok(value.clone()),
        Some(value) => Err(tool_error(
            "invalid_arguments",
            format!("expected object argument: {name}"),
            json!({ "argument": name, "actualType": value_type_name(value) }),
        )),
        None => Err(tool_error(
            "invalid_arguments",
            format!("missing required object argument: {name}"),
            json!({ "argument": name }),
        )),
    }
}

fn required_string(arguments: Option<&JsonObject>, name: &str) -> Result<String, CallToolResult> {
    arguments
        .and_then(|arguments| arguments.get(name))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            tool_error(
                "invalid_arguments",
                format!("missing required string argument: {name}"),
                json!({ "argument": name }),
            )
        })
}

fn optional_bool(arguments: Option<&JsonObject>, name: &str) -> Option<bool> {
    arguments
        .and_then(|arguments| arguments.get(name))
        .and_then(Value::as_bool)
}

#[cfg(test)]
mod tests {
    use crate::policy::{ApprovalClass, Scope};
    use crate::tools::workloads;

    use super::*;

    #[test]
    fn workload_upgrade_definition_is_mutating_but_not_destructive() {
        let definition = workloads::definitions()
            .iter()
            .find(|definition| definition.name == "workloads.upgrade")
            .unwrap();

        assert!(!definition.destructive);
        assert!(!definition.read_only);
        assert_eq!(definition.required_scope, Scope::WorkloadsUpgrade);
        assert_eq!(definition.approval_class, ApprovalClass::Mutation);
    }

    #[test]
    fn missing_configuration_returns_invalid_arguments() {
        let mut arguments = JsonObject::new();
        arguments.insert("namespace".to_string(), Value::String("hydra".to_string()));
        arguments.insert(
            "releaseName".to_string(),
            Value::String("hydra-offline".to_string()),
        );

        let error = required_object(Some(&arguments), "configuration").unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|content| content.get("error")),
            Some(&Value::String("invalid_arguments".to_string()))
        );
    }

    #[test]
    fn schema_validation_rejects_unknown_values() {
        let schema = json!({
            "type": "object",
            "properties": { "namespace": { "type": "string" } },
            "additionalProperties": false
        });
        let mut values = JsonObject::new();
        values.insert("namespace".to_string(), Value::String("hydra".to_string()));
        values.insert("rawValues".to_string(), json!({}));

        let error = validate_configuration_schema(&values, &schema).unwrap_err();

        assert_eq!(
            error
                .structured_content
                .as_ref()
                .and_then(|content| content.get("error")),
            Some(&Value::String(
                "invalid_extension_configuration".to_string()
            ))
        );
    }
}
