use reqwest::header::ACCEPT;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;

const SKILL_CATALOG_LAYER_MEDIA_TYPE: &str = "application/vnd.supernode.skill-catalog.v1+json";
const JSON_MEDIA_TYPE: &str = "application/json";
const MANIFEST_ACCEPT: &str = "application/vnd.oci.image.manifest.v1+json, application/vnd.oci.artifact.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json";
const TRUSTED_CATALOG_REGISTRY: &str = "oci.supernode.store";

pub(super) fn is_trusted_catalog_reference(reference: &str) -> Result<bool, OciSkillCatalogError> {
    Ok(OciReference::parse(reference)?.registry == TRUSTED_CATALOG_REGISTRY)
}

pub(super) async fn fetch_catalog_json(
    reference: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, OciSkillCatalogError> {
    let reference = OciReference::parse(reference)?;
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    let manifest = fetch_manifest(&client, &reference).await?;
    let descriptor = select_catalog_descriptor(&manifest)
        .ok_or_else(|| OciSkillCatalogError::MissingCatalogLayer(reference.original.clone()))?;

    if descriptor.size.is_some_and(|size| size > max_bytes) {
        return Err(OciSkillCatalogError::CatalogTooLarge {
            actual: descriptor.size.unwrap_or_default(),
            max: max_bytes,
        });
    }

    let payload = fetch_blob(&client, &reference, &descriptor.digest).await?;
    if payload.len() > max_bytes {
        return Err(OciSkillCatalogError::CatalogTooLarge {
            actual: payload.len(),
            max: max_bytes,
        });
    }
    verify_sha256_digest(&descriptor.digest, &payload)?;

    Ok(payload)
}

async fn fetch_manifest(
    client: &reqwest::Client,
    reference: &OciReference,
) -> Result<OciManifest, OciSkillCatalogError> {
    let response = client
        .get(reference.manifest_url())
        .header(ACCEPT, MANIFEST_ACCEPT)
        .send()
        .await?
        .error_for_status()
        .map_err(OciSkillCatalogError::HttpStatus)?;

    response.json::<OciManifest>().await.map_err(Into::into)
}

async fn fetch_blob(
    client: &reqwest::Client,
    reference: &OciReference,
    digest: &str,
) -> Result<Vec<u8>, OciSkillCatalogError> {
    let response = client
        .get(reference.blob_url(digest))
        .send()
        .await?
        .error_for_status()
        .map_err(OciSkillCatalogError::HttpStatus)?;

    Ok(response.bytes().await?.to_vec())
}

fn select_catalog_descriptor(manifest: &OciManifest) -> Option<OciDescriptor> {
    manifest
        .layers
        .iter()
        .chain(manifest.blobs.iter())
        .find(|descriptor| descriptor.media_type == SKILL_CATALOG_LAYER_MEDIA_TYPE)
        .or_else(|| {
            manifest
                .layers
                .iter()
                .chain(manifest.blobs.iter())
                .find(|descriptor| descriptor.media_type == JSON_MEDIA_TYPE)
        })
        .cloned()
}

fn verify_sha256_digest(expected: &str, payload: &[u8]) -> Result<(), OciSkillCatalogError> {
    let Some(expected) = expected.strip_prefix("sha256:") else {
        return Err(OciSkillCatalogError::UnsupportedDigest(
            expected.to_string(),
        ));
    };
    let actual = hex_lower(&Sha256::digest(payload));
    if actual != expected.to_ascii_lowercase() {
        return Err(OciSkillCatalogError::DigestMismatch {
            expected: expected.to_string(),
            actual,
        });
    }

    Ok(())
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct OciReference {
    original: String,
    registry: String,
    repository: String,
    reference: String,
}

impl OciReference {
    fn parse(value: &str) -> Result<Self, OciSkillCatalogError> {
        let value = value.trim();
        let Some(rest) = value.strip_prefix("oci://") else {
            return Err(OciSkillCatalogError::InvalidReference(value.to_string()));
        };
        let Some((registry, path)) = rest.split_once('/') else {
            return Err(OciSkillCatalogError::InvalidReference(value.to_string()));
        };
        if registry.is_empty() || path.is_empty() {
            return Err(OciSkillCatalogError::InvalidReference(value.to_string()));
        }

        let last_slash = path.rfind('/');
        let tag_separator = path.rfind(':').filter(|index| {
            last_slash
                .map(|last_slash| *index > last_slash)
                .unwrap_or(true)
        });

        let (repository, reference) = if let Some((repository, digest)) = path.split_once('@') {
            (repository, digest)
        } else if let Some(index) = tag_separator {
            (&path[..index], &path[index + 1..])
        } else {
            return Err(OciSkillCatalogError::MissingReference(value.to_string()));
        };

        if repository.is_empty() || reference.is_empty() {
            return Err(OciSkillCatalogError::InvalidReference(value.to_string()));
        }

        Ok(Self {
            original: value.to_string(),
            registry: registry.to_string(),
            repository: repository.to_string(),
            reference: reference.to_string(),
        })
    }

    fn manifest_url(&self) -> String {
        format!(
            "https://{}/v2/{}/manifests/{}",
            self.registry, self.repository, self.reference
        )
    }

    fn blob_url(&self, digest: &str) -> String {
        format!(
            "https://{}/v2/{}/blobs/{}",
            self.registry, self.repository, digest
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OciManifest {
    #[serde(default)]
    layers: Vec<OciDescriptor>,
    #[serde(default)]
    blobs: Vec<OciDescriptor>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct OciDescriptor {
    media_type: String,
    digest: String,
    size: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum OciSkillCatalogError {
    #[error("invalid OCI reference: {0}")]
    InvalidReference(String),
    #[error("OCI reference must include a tag or digest: {0}")]
    MissingReference(String),
    #[error("OCI registry request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("OCI registry returned an unsuccessful status: {0}")]
    HttpStatus(reqwest::Error),
    #[error("OCI skill catalog artifact does not contain a skill catalog JSON layer: {0}")]
    MissingCatalogLayer(String),
    #[error("OCI skill catalog blob is too large: {actual} bytes exceeds {max} bytes")]
    CatalogTooLarge { actual: usize, max: usize },
    #[error("unsupported OCI skill catalog digest: {0}")]
    UnsupportedDigest(String),
    #[error("OCI skill catalog digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_tagged_oci_references() {
        let reference =
            OciReference::parse("oci://oci.supernode.store/skill-catalog:0.1.0").unwrap();

        assert_eq!(reference.registry, "oci.supernode.store");
        assert_eq!(reference.repository, "skill-catalog");
        assert_eq!(reference.reference, "0.1.0");
        assert_eq!(
            reference.manifest_url(),
            "https://oci.supernode.store/v2/skill-catalog/manifests/0.1.0"
        );
    }

    #[test]
    fn rejects_references_without_tag_or_digest() {
        let error = OciReference::parse("oci://oci.supernode.store/skill-catalog").unwrap_err();

        assert!(matches!(error, OciSkillCatalogError::MissingReference(_)));
    }

    #[test]
    fn identifies_trusted_catalog_registry() {
        assert!(
            is_trusted_catalog_reference("oci://oci.supernode.store/skill-catalog:0.1.0").unwrap()
        );
        assert!(!is_trusted_catalog_reference("oci://example.com/skill-catalog:0.1.0").unwrap());
    }

    #[test]
    fn selects_skill_catalog_layer_before_generic_json() {
        let manifest = OciManifest {
            layers: vec![
                descriptor(JSON_MEDIA_TYPE, "sha256:generic", 10),
                descriptor(SKILL_CATALOG_LAYER_MEDIA_TYPE, "sha256:specific", 20),
            ],
            blobs: vec![],
        };

        let selected = select_catalog_descriptor(&manifest).unwrap();

        assert_eq!(selected.digest, "sha256:specific");
    }

    #[test]
    fn verifies_sha256_digest() {
        let payload = br#"{"schemaVersion":"supernode.skillCatalog/v1","skills":[]}"#;
        let digest = format!("sha256:{}", hex_lower(&Sha256::digest(payload)));

        verify_sha256_digest(&digest, payload).unwrap();
    }

    #[test]
    fn detects_digest_mismatch() {
        let error = verify_sha256_digest("sha256:deadbeef", b"payload").unwrap_err();

        assert!(matches!(error, OciSkillCatalogError::DigestMismatch { .. }));
    }

    fn descriptor(media_type: &str, digest: &str, size: usize) -> OciDescriptor {
        serde_json::from_value(json!({
            "mediaType": media_type,
            "digest": digest,
            "size": size,
        }))
        .unwrap()
    }
}
