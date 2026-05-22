use rmcp::model::{CallToolResult, JsonObject};
use serde_json::{Value, json};

use crate::tools::common::tool_error;

pub(crate) fn required_object(
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

pub(crate) fn required_string(
    arguments: Option<&JsonObject>,
    name: &str,
) -> Result<String, CallToolResult> {
    optional_string(arguments, name).ok_or_else(|| {
        tool_error(
            "invalid_arguments",
            format!("missing required string argument: {name}"),
            json!({ "argument": name }),
        )
    })
}

pub(crate) fn optional_string(arguments: Option<&JsonObject>, name: &str) -> Option<String> {
    arguments?
        .get(name)?
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
}

pub(crate) fn optional_bool(arguments: Option<&JsonObject>, name: &str) -> Option<bool> {
    arguments?.get(name)?.as_bool()
}

pub(crate) fn optional_i64(arguments: Option<&JsonObject>, name: &str) -> Option<i64> {
    arguments?.get(name)?.as_i64()
}

pub(crate) fn optional_u32(arguments: Option<&JsonObject>, name: &str) -> Option<u32> {
    arguments?
        .get(name)?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
}

pub(crate) fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
