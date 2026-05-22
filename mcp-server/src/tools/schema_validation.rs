use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::tools::args::value_type_name;
use crate::tools::common::tool_error;

pub(crate) fn validate_configuration_schema(
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
