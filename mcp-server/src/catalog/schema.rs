use serde_json::Value;
use serde_json::json;

pub(super) fn nullable_number(description: &str) -> Value {
    json!({ "type": ["number", "null"], "description": description })
}

pub(super) fn nullable_string(description: &str) -> Value {
    json!({ "type": ["string", "null"], "description": description })
}

pub(super) fn nullable_boolean(description: &str) -> Value {
    json!({ "type": ["boolean", "null"], "description": description })
}
