use std::fmt;

use serde_json::Map;
use serde_json::Value;

#[derive(Clone, Eq, PartialEq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

#[derive(Clone)]
pub struct SecretObject {
    value: Value,
}

impl SecretObject {
    pub fn new(value: Value) -> Result<Self, SecretValueError> {
        if !value.is_object() {
            return Err(SecretValueError::NotObject);
        }

        Ok(Self { value })
    }

    pub fn expose_secret(&self) -> &Value {
        &self.value
    }

    pub fn key_names(&self) -> Vec<String> {
        sorted_keys(self.value.as_object())
    }
}

impl fmt::Debug for SecretObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretObject")
            .field("keys", &self.key_names())
            .field("values", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecretValueError {
    #[error("secret value must be a JSON object")]
    NotObject,
}

pub fn sorted_keys(object: Option<&Map<String, Value>>) -> Vec<String> {
    let mut keys = object
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();
    keys
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn secret_string_debug_and_display_are_redacted() {
        let secret = SecretString::new("plaintext");

        assert_eq!(format!("{secret:?}"), "[REDACTED]");
        assert_eq!(secret.to_string(), "[REDACTED]");
        assert_eq!(secret.expose_secret(), "plaintext");
    }

    #[test]
    fn secret_object_debug_redacts_values_but_keeps_keys() {
        let secret = SecretObject::new(json!({ "b": "two", "a": "one" })).unwrap();
        let debug = format!("{secret:?}");

        assert!(debug.contains("a"));
        assert!(debug.contains("b"));
        assert!(!debug.contains("one"));
        assert!(!debug.contains("two"));
        assert_eq!(secret.key_names(), vec!["a".to_string(), "b".to_string()]);
    }
}
