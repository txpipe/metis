use ed25519_dalek::SigningKey;
use rmcp::model::{CallToolResult, JsonObject};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::policy::{ApprovalClass, Scope};
use crate::vault::{SecretObject, VaultClient, VaultPath, WriteMode};

use super::{
    ToolDefinition,
    args::{optional_bool, optional_string, required_string},
    common::{success, tool_error, vault_error},
};

const TOOL_NAME: &str = "hydra.keys.generate";
const DEFAULT_SIGNING_KEY_NAME: &str = "hydra.sk";
const DEFAULT_VERIFICATION_KEY_NAME: &str = "hydra.vk";
const HYDRA_SIGNING_KEY_TYPE: &str = "HydraSigningKey_ed25519";
const HYDRA_VERIFICATION_KEY_TYPE: &str = "HydraVerificationKey_ed25519";

pub fn definitions() -> &'static [ToolDefinition] {
    &[ToolDefinition {
        name: TOOL_NAME,
        title: "Generate Hydra Keys",
        description: "Generate Hydra off-chain signing and verification keys and save both text envelopes to runtime Vault. The private signing key is never returned.",
        required_scope: Scope::VaultRuntimeWrite,
        approval_class: ApprovalClass::RuntimeSecretWrite,
        read_only: false,
        destructive: true,
        input_schema: r#"{"type":"object","required":["path","approvalId"],"properties":{"path":{"type":"string","pattern":"^runtime/","description":"Runtime Vault path without kv/data prefix."},"approvalId":{"type":"string"},"signingKeyName":{"type":"string","minLength":1,"default":"hydra.sk","description":"Vault field name for the Hydra signing key text envelope."},"verificationKeyName":{"type":"string","minLength":1,"default":"hydra.vk","description":"Vault field name for the Hydra verification key text envelope."},"overwrite":{"type":"boolean","default":false,"description":"Allow replacing existing signing or verification key fields at the target path."}},"additionalProperties":false}"#,
    }]
}

pub async fn generate_keys(arguments: Option<&JsonObject>) -> CallToolResult {
    let path = match runtime_vault_path(arguments) {
        Ok(path) => path,
        Err(error) => return error,
    };
    let signing_key_name = optional_string(arguments, "signingKeyName")
        .unwrap_or_else(|| DEFAULT_SIGNING_KEY_NAME.to_string());
    let verification_key_name = optional_string(arguments, "verificationKeyName")
        .unwrap_or_else(|| DEFAULT_VERIFICATION_KEY_NAME.to_string());
    let overwrite = optional_bool(arguments, "overwrite").unwrap_or(false);

    if signing_key_name == verification_key_name {
        return tool_error(
            "invalid_arguments",
            "signingKeyName and verificationKeyName must be different",
            json!({ "signingKeyName": signing_key_name, "verificationKeyName": verification_key_name }),
        );
    }

    let client = match VaultClient::from_env() {
        Ok(client) => client,
        Err(error) => return vault_error(TOOL_NAME, error),
    };

    if !overwrite {
        match client.runtime_metadata(&path).await {
            Ok(metadata) => {
                let conflicting_keys = metadata
                    .key_names
                    .iter()
                    .filter(|key| *key == &signing_key_name || *key == &verification_key_name)
                    .cloned()
                    .collect::<Vec<_>>();
                if !conflicting_keys.is_empty() {
                    return tool_error(
                        "vault_secret_key_exists",
                        "target Vault path already contains one or more Hydra key fields; set overwrite=true to replace them",
                        json!({
                            "path": path.as_str(),
                            "conflictingKeys": conflicting_keys,
                        }),
                    );
                }
            }
            Err(error) => return vault_error(TOOL_NAME, error),
        }
    }

    let key_pair = match generate_hydra_key_pair() {
        Ok(key_pair) => key_pair,
        Err(error) => {
            return tool_error(
                "hydra_key_generation_failed",
                error.to_string(),
                json!({ "tool": TOOL_NAME }),
            );
        }
    };
    let secret = match SecretObject::new(json!({
        signing_key_name.clone(): key_pair.signing_key,
        verification_key_name.clone(): key_pair.verification_key,
    })) {
        Ok(secret) => secret,
        Err(error) => {
            return tool_error(
                "invalid_secret_values",
                error.to_string(),
                json!({ "tool": TOOL_NAME }),
            );
        }
    };

    match client
        .write_runtime_secret(&path, &secret, WriteMode::Patch)
        .await
    {
        Ok(receipt) => success(json!({
            "path": receipt.path,
            "writtenKeys": receipt.written_keys,
            "version": receipt.version,
            "verificationKey": {
                "filename": verification_key_name,
                "value": key_pair.verification_key,
            },
            "signingKey": {
                "filename": signing_key_name,
                "returned": false,
            },
            "secretValuesReturned": false,
            "signingKeyReturned": false,
        })),
        Err(error) => vault_error(TOOL_NAME, error),
    }
}

fn generate_hydra_key_pair() -> Result<HydraKeyPair, getrandom::Error> {
    let mut entropy = [0u8; 16];
    getrandom::getrandom(&mut entropy)?;
    Ok(hydra_key_pair_from_entropy(entropy))
}

fn hydra_key_pair_from_entropy(entropy: [u8; 16]) -> HydraKeyPair {
    let signing_seed = Sha256::digest(entropy);
    let signing_key = SigningKey::from_bytes(&signing_seed.into());
    hydra_key_pair_from_signing_key_bytes(signing_key.to_bytes())
}

fn hydra_key_pair_from_signing_key_bytes(signing_key_bytes: [u8; 32]) -> HydraKeyPair {
    let signing_key = SigningKey::from_bytes(&signing_key_bytes);
    let verification_key_bytes = signing_key.verifying_key().to_bytes();

    HydraKeyPair {
        signing_key: text_envelope(HYDRA_SIGNING_KEY_TYPE, &signing_key_bytes),
        verification_key: text_envelope(HYDRA_VERIFICATION_KEY_TYPE, &verification_key_bytes),
    }
}

fn text_envelope(key_type: &str, raw_key: &[u8; 32]) -> String {
    let envelope = TextEnvelope {
        key_type,
        description: "",
        cbor_hex: format!("5820{}", hex_lower(raw_key)),
    };

    serde_json::to_string_pretty(&envelope).expect("serializing Hydra text envelope must not fail")
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn runtime_vault_path(arguments: Option<&JsonObject>) -> Result<VaultPath, CallToolResult> {
    let path = required_string(arguments, "path")?;
    VaultPath::runtime(&path).map_err(|error| {
        tool_error(
            "vault_path_not_allowed",
            error.to_string(),
            json!({ "path": path }),
        )
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct HydraKeyPair {
    signing_key: String,
    verification_key: String,
}

#[derive(Serialize)]
struct TextEnvelope<'a> {
    #[serde(rename = "type")]
    key_type: &'a str,
    description: &'a str,
    #[serde(rename = "cborHex")]
    cbor_hex: String,
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const ALICE_SIGNING_KEY_CBOR_HEX: &str =
        "5820e4c36e5403e6a02ef4821a34bb71d504916df0ddea476f797a5639110bc1bd52";
    const ALICE_VERIFICATION_KEY_CBOR_HEX: &str =
        "5820b37aabd81024c043f53a069c91e51a5b52e4ea399ae17ee1fe3cb9c44db707eb";

    #[test]
    fn generated_key_pair_uses_hydra_text_envelopes() {
        let pair = hydra_key_pair_from_entropy(*b"1234567890abcdef");
        let signing = parse_json(&pair.signing_key);
        let verification = parse_json(&pair.verification_key);

        assert_eq!(signing.get("type"), Some(&json!(HYDRA_SIGNING_KEY_TYPE)));
        assert_eq!(
            verification.get("type"),
            Some(&json!(HYDRA_VERIFICATION_KEY_TYPE))
        );
        assert_eq!(
            signing
                .get("cborHex")
                .and_then(Value::as_str)
                .unwrap()
                .len(),
            68
        );
        assert!(
            signing
                .get("cborHex")
                .and_then(Value::as_str)
                .unwrap()
                .starts_with("5820")
        );
        assert_eq!(
            verification
                .get("cborHex")
                .and_then(Value::as_str)
                .unwrap()
                .len(),
            68
        );
        assert!(
            verification
                .get("cborHex")
                .and_then(Value::as_str)
                .unwrap()
                .starts_with("5820")
        );
    }

    #[test]
    fn derives_hydra_demo_verification_key_from_signing_key() {
        let signing_key_bytes = raw_key_bytes(ALICE_SIGNING_KEY_CBOR_HEX);

        let pair = hydra_key_pair_from_signing_key_bytes(signing_key_bytes);
        let verification = parse_json(&pair.verification_key);

        assert_eq!(
            verification.get("cborHex"),
            Some(&json!(ALICE_VERIFICATION_KEY_CBOR_HEX))
        );
    }

    #[test]
    fn definitions_expose_runtime_secret_write_policy() {
        let definition = definitions().first().unwrap();

        assert_eq!(definition.name, "hydra.keys.generate");
        assert_eq!(definition.required_scope, Scope::VaultRuntimeWrite);
        assert_eq!(definition.approval_class, ApprovalClass::RuntimeSecretWrite);
        assert!(!definition.read_only);
        assert!(definition.destructive);
        assert!(definition.input_schema.contains("overwrite"));
    }

    fn parse_json(value: &str) -> Value {
        serde_json::from_str(value).unwrap()
    }

    fn raw_key_bytes(cbor_hex: &str) -> [u8; 32] {
        assert_eq!(&cbor_hex[0..4], "5820");
        let hex = &cbor_hex[4..];
        let mut output = [0u8; 32];
        for (index, chunk) in hex.as_bytes().chunks(2).enumerate() {
            output[index] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
        }
        output
    }
}
