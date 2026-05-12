use std::env;
use std::fs;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::vault::paths::VaultPath;
use crate::vault::redaction::SecretObject;
use crate::vault::redaction::SecretString;
use crate::vault::redaction::sorted_keys;

const DEFAULT_KV_MOUNT: &str = "kv";

#[derive(Clone)]
pub struct VaultClient {
    http: reqwest::Client,
    addr: String,
    token: SecretString,
    kv_mount: String,
}

impl VaultClient {
    pub fn from_env() -> Result<Self, VaultError> {
        let addr = env::var("VAULT_ADDR").map_err(|_| VaultError::MissingConfig("VAULT_ADDR"))?;
        let token = vault_token_from_env()?;
        let kv_mount = env::var("VAULT_KV_MOUNT").unwrap_or_else(|_| DEFAULT_KV_MOUNT.to_string());

        Self::new(addr, token, kv_mount)
    }

    pub fn new(
        addr: impl Into<String>,
        token: SecretString,
        kv_mount: impl Into<String>,
    ) -> Result<Self, VaultError> {
        let addr = addr.into().trim_end_matches('/').to_string();
        let kv_mount = kv_mount.into().trim_matches('/').to_string();

        if addr.is_empty() {
            return Err(VaultError::MissingConfig("VAULT_ADDR"));
        }

        if kv_mount.is_empty() || kv_mount.contains('/') {
            return Err(VaultError::InvalidConfig("VAULT_KV_MOUNT"));
        }

        if token.expose_secret().trim().is_empty() {
            return Err(VaultError::MissingConfig("VAULT_TOKEN"));
        }

        if token.expose_secret() == "root" {
            return Err(VaultError::RootTokenRejected);
        }

        Ok(Self {
            http: reqwest::Client::new(),
            addr,
            token,
            kv_mount,
        })
    }

    pub async fn runtime_metadata(
        &self,
        path: &VaultPath,
    ) -> Result<VaultSecretMetadata, VaultError> {
        let metadata_response = self.get_metadata(path).await?;
        let secret_response = self.get_secret(path).await?;

        Ok(VaultSecretMetadata {
            path: path.as_str().to_string(),
            exists: metadata_response.is_some() || secret_response.is_some(),
            key_names: secret_response
                .as_ref()
                .map(|secret| sorted_keys(secret.data.data.as_object()))
                .unwrap_or_default(),
            key_names_available: secret_response.is_some(),
            current_version: metadata_response
                .as_ref()
                .and_then(|metadata| metadata.data.current_version),
        })
    }

    pub async fn write_runtime_secret(
        &self,
        path: &VaultPath,
        secret: &SecretObject,
        mode: WriteMode,
    ) -> Result<VaultWriteReceipt, VaultError> {
        let data = match mode {
            WriteMode::Replace => secret.expose_secret().clone(),
            WriteMode::Patch => {
                let mut existing = self
                    .get_secret(path)
                    .await?
                    .map(|secret| secret.data.data)
                    .unwrap_or_else(|| json!({}));
                merge_secret_objects(&mut existing, secret.expose_secret())?;
                existing
            }
        };
        let written_keys = sorted_keys(data.as_object());
        let response = self.put_secret(path, &data).await?;

        Ok(VaultWriteReceipt {
            path: path.as_str().to_string(),
            written_keys,
            version: response.and_then(|response| response.data.version),
        })
    }

    async fn get_metadata(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultMetadataResponse>, VaultError> {
        self.get_optional_json(self.metadata_url(path)).await
    }

    async fn get_secret(
        &self,
        path: &VaultPath,
    ) -> Result<Option<VaultSecretResponse>, VaultError> {
        self.get_optional_json(self.data_url(path)).await
    }

    async fn put_secret(
        &self,
        path: &VaultPath,
        data: &Value,
    ) -> Result<Option<VaultWriteResponse>, VaultError> {
        let response = self
            .http
            .post(self.data_url(path))
            .header("X-Vault-Token", self.token.expose_secret())
            .json(&json!({ "data": data }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(VaultError::Status(response.status()));
        }

        if response.status() == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        Ok(Some(response.json().await?))
    }

    async fn get_optional_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: String,
    ) -> Result<Option<T>, VaultError> {
        let response = self
            .http
            .get(url)
            .header("X-Vault-Token", self.token.expose_secret())
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(VaultError::Status(response.status()));
        }

        Ok(Some(response.json().await?))
    }

    fn metadata_url(&self, path: &VaultPath) -> String {
        format!(
            "{}/v1/{}/metadata/{}",
            self.addr,
            self.kv_mount,
            path.as_str()
        )
    }

    fn data_url(&self, path: &VaultPath) -> String {
        format!("{}/v1/{}/data/{}", self.addr, self.kv_mount, path.as_str())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WriteMode {
    Replace,
    Patch,
}

impl WriteMode {
    pub fn parse(value: Option<&str>) -> Result<Self, VaultError> {
        match value.unwrap_or("patch") {
            "replace" => Ok(Self::Replace),
            "patch" => Ok(Self::Patch),
            _ => Err(VaultError::InvalidWriteMode),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultSecretMetadata {
    pub path: String,
    pub exists: bool,
    pub key_names: Vec<String>,
    pub key_names_available: bool,
    pub current_version: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultWriteReceipt {
    pub path: String,
    pub written_keys: Vec<String>,
    pub version: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("missing Vault configuration: {0}")]
    MissingConfig(&'static str),
    #[error("invalid Vault configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("Vault root token is not allowed")]
    RootTokenRejected,
    #[error("invalid Vault write mode")]
    InvalidWriteMode,
    #[error("Vault path is invalid: {0}")]
    Path(#[from] crate::vault::paths::VaultPathError),
    #[error("Vault secret value is invalid: {0}")]
    SecretValue(#[from] crate::vault::redaction::SecretValueError),
    #[error("Vault request failed with status {0}")]
    Status(StatusCode),
    #[error("Vault HTTP request failed")]
    Http(#[from] reqwest::Error),
    #[error("failed to read Vault token file")]
    TokenFile(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct VaultSecretResponse {
    data: VaultSecretEnvelope,
}

#[derive(Debug, Deserialize)]
struct VaultSecretEnvelope {
    data: Value,
}

#[derive(Debug, Deserialize)]
struct VaultMetadataResponse {
    data: VaultMetadataEnvelope,
}

#[derive(Debug, Deserialize)]
struct VaultMetadataEnvelope {
    current_version: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct VaultWriteResponse {
    data: VaultWriteEnvelope,
}

#[derive(Debug, Deserialize)]
struct VaultWriteEnvelope {
    version: Option<u64>,
}

fn vault_token_from_env() -> Result<SecretString, VaultError> {
    if let Ok(token) = env::var("VAULT_TOKEN") {
        return Ok(SecretString::new(token));
    }

    if let Ok(path) = env::var("VAULT_TOKEN_FILE") {
        return Ok(SecretString::new(
            fs::read_to_string(path)?.trim().to_string(),
        ));
    }

    Err(VaultError::MissingConfig("VAULT_TOKEN or VAULT_TOKEN_FILE"))
}

fn merge_secret_objects(existing: &mut Value, patch: &Value) -> Result<(), VaultError> {
    let existing = existing
        .as_object_mut()
        .ok_or(crate::vault::redaction::SecretValueError::NotObject)?;
    let patch = patch
        .as_object()
        .ok_or(crate::vault::redaction::SecretValueError::NotObject)?;

    for (key, value) in patch {
        existing.insert(key.clone(), value.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::vault::paths::VaultPath;

    #[test]
    fn rejects_root_token() {
        let error = match VaultClient::new("http://127.0.0.1:8200", SecretString::new("root"), "kv")
        {
            Ok(_) => panic!("root token should be rejected"),
            Err(error) => error,
        };

        assert!(matches!(error, VaultError::RootTokenRejected));
    }

    #[test]
    fn write_mode_defaults_to_patch() {
        assert_eq!(WriteMode::parse(None).unwrap(), WriteMode::Patch);
        assert_eq!(
            WriteMode::parse(Some("replace")).unwrap(),
            WriteMode::Replace
        );
        assert!(matches!(
            WriteMode::parse(Some("delete")),
            Err(VaultError::InvalidWriteMode)
        ));
    }

    #[test]
    fn patch_merge_preserves_existing_keys() {
        let mut existing = json!({ "a": "one", "b": "two" });
        let patch = json!({ "b": "changed", "c": "three" });

        merge_secret_objects(&mut existing, &patch).unwrap();

        assert_eq!(
            existing,
            json!({ "a": "one", "b": "changed", "c": "three" })
        );
    }

    #[test]
    fn client_builds_runtime_kv_v2_urls() {
        let client =
            VaultClient::new("http://vault:8200/", SecretString::new("hvs.token"), "kv").unwrap();
        let path = VaultPath::runtime("runtime/cardano-node/mainnet").unwrap();

        assert_eq!(
            client.data_url(&path),
            "http://vault:8200/v1/kv/data/runtime/cardano-node/mainnet"
        );
        assert_eq!(
            client.metadata_url(&path),
            "http://vault:8200/v1/kv/metadata/runtime/cardano-node/mainnet"
        );
    }
}
