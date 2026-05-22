use reqwest::header::ACCEPT;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;

const JSON_MEDIA_TYPE: &str = "application/json";
const MANIFEST_ACCEPT: &str = "application/vnd.oci.image.manifest.v1+json, application/vnd.oci.artifact.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json";
const TRUSTED_CATALOG_REGISTRY: &str = "oci.supernode.store";
const MAX_MANIFEST_SIZE: usize = 1024 * 1024;

pub(crate) fn is_trusted_reference(reference: &str) -> Result<bool, OciArtifactError> {
    Ok(OciReference::parse(reference)?.registry == TRUSTED_CATALOG_REGISTRY)
}

// TODO(security): Implement OCI artifact signature verification (cosign/sigstore).
// Current trust model relies on TLS hostname verification and self-consistent digest checks.
// A registry serving valid TLS for the hostname can still ship arbitrary content.
pub(crate) async fn fetch_artifact_json(
    reference: &str,
    max_bytes: usize,
    layer_media_type: &str,
) -> Result<Vec<u8>, OciArtifactError> {
    let reference = OciReference::parse(reference)?;
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;
    let manifest = fetch_manifest(&client, &reference).await?;
    let descriptor = select_artifact_descriptor(&manifest, layer_media_type)
        .ok_or_else(|| OciArtifactError::MissingCatalogLayer(reference.original.clone()))?;

    if descriptor.size.is_some_and(|size| size > max_bytes) {
        return Err(OciArtifactError::CatalogTooLarge {
            actual: descriptor.size.unwrap_or_default(),
            max: max_bytes,
        });
    }

    // Use a separate client for blob fetch with redirects disabled to prevent
    // a compromised registry from redirecting to an untrusted third-party host.
    let blob_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let payload = fetch_blob(&blob_client, &reference, &descriptor.digest, max_bytes).await?;
    verify_sha256_digest(&descriptor.digest, &payload)?;

    Ok(payload)
}

async fn fetch_manifest(
    client: &reqwest::Client,
    reference: &OciReference,
) -> Result<OciManifest, OciArtifactError> {
    let response = client
        .get(reference.manifest_url())
        .header(ACCEPT, MANIFEST_ACCEPT)
        .send()
        .await?
        .error_for_status()
        .map_err(OciArtifactError::HttpStatus)?;

    let body = read_bounded_response(response, MAX_MANIFEST_SIZE).await?;
    serde_json::from_slice(&body).map_err(OciArtifactError::InvalidManifest)
}

async fn fetch_blob(
    client: &reqwest::Client,
    reference: &OciReference,
    digest: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, OciArtifactError> {
    let response = client
        .get(reference.blob_url(digest))
        .send()
        .await?
        .error_for_status()
        .map_err(OciArtifactError::HttpStatus)?;

    read_bounded_response(response, max_bytes).await
}

async fn read_bounded_response(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, OciArtifactError> {
    if let Some(content_length) = response.content_length()
        && content_length as usize > max_bytes
    {
        return Err(OciArtifactError::CatalogTooLarge {
            actual: content_length as usize,
            max: max_bytes,
        });
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > max_bytes {
            return Err(OciArtifactError::CatalogTooLarge {
                actual: body.len() + chunk.len(),
                max: max_bytes,
            });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn select_artifact_descriptor(
    manifest: &OciManifest,
    layer_media_type: &str,
) -> Option<OciDescriptor> {
    manifest
        .layers
        .iter()
        .chain(manifest.blobs.iter())
        .find(|descriptor| descriptor.media_type == layer_media_type)
        .or_else(|| {
            manifest
                .layers
                .iter()
                .chain(manifest.blobs.iter())
                .find(|descriptor| descriptor.media_type == JSON_MEDIA_TYPE)
        })
        .cloned()
}

fn verify_sha256_digest(expected: &str, payload: &[u8]) -> Result<(), OciArtifactError> {
    let Some(expected) = expected.strip_prefix("sha256:") else {
        return Err(OciArtifactError::UnsupportedDigest(expected.to_string()));
    };
    let actual = hex_lower(&Sha256::digest(payload));
    if actual != expected.to_ascii_lowercase() {
        return Err(OciArtifactError::DigestMismatch {
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
    fn parse(value: &str) -> Result<Self, OciArtifactError> {
        let value = value.trim();
        let Some(rest) = value.strip_prefix("oci://") else {
            return Err(OciArtifactError::InvalidReference(value.to_string()));
        };
        let Some((registry, path)) = rest.split_once('/') else {
            return Err(OciArtifactError::InvalidReference(value.to_string()));
        };
        if registry.is_empty() || path.is_empty() {
            return Err(OciArtifactError::InvalidReference(value.to_string()));
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
            return Err(OciArtifactError::MissingReference(value.to_string()));
        };

        if repository.is_empty() || reference.is_empty() {
            return Err(OciArtifactError::InvalidReference(value.to_string()));
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
pub(crate) enum OciArtifactError {
    #[error("invalid OCI reference: {0}")]
    InvalidReference(String),
    #[error("OCI reference must include a tag or digest: {0}")]
    MissingReference(String),
    #[error("OCI registry request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("OCI registry returned an unsuccessful status: {0}")]
    HttpStatus(reqwest::Error),
    #[error("OCI manifest JSON is invalid: {0}")]
    InvalidManifest(serde_json::Error),
    #[error("OCI artifact does not contain a catalog JSON layer: {0}")]
    MissingCatalogLayer(String),
    #[error("OCI catalog blob is too large: {actual} bytes exceeds {max} bytes")]
    CatalogTooLarge { actual: usize, max: usize },
    #[error("unsupported OCI catalog digest: {0}")]
    UnsupportedDigest(String),
    #[error("OCI catalog digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TEST_LAYER_MEDIA_TYPE: &str = "application/vnd.supernode.test-catalog.v1+json";

    #[test]
    fn parses_tagged_oci_references() {
        let reference =
            OciReference::parse("oci://oci.supernode.store/test-catalog:0.1.0").unwrap();

        assert_eq!(reference.registry, "oci.supernode.store");
        assert_eq!(reference.repository, "test-catalog");
        assert_eq!(reference.reference, "0.1.0");
        assert_eq!(
            reference.manifest_url(),
            "https://oci.supernode.store/v2/test-catalog/manifests/0.1.0"
        );
    }

    #[test]
    fn parses_digest_oci_references() {
        let reference =
            OciReference::parse("oci://oci.supernode.store/test-catalog@sha256:abc123").unwrap();

        assert_eq!(reference.repository, "test-catalog");
        assert_eq!(reference.reference, "sha256:abc123");
    }

    #[test]
    fn rejects_references_without_tag_or_digest() {
        let error = OciReference::parse("oci://oci.supernode.store/test-catalog").unwrap_err();

        assert!(matches!(error, OciArtifactError::MissingReference(_)));
    }

    #[test]
    fn identifies_trusted_catalog_registry() {
        assert!(is_trusted_reference("oci://oci.supernode.store/test-catalog:0.1.0").unwrap());
        assert!(!is_trusted_reference("oci://example.com/test-catalog:0.1.0").unwrap());
    }

    #[test]
    fn selects_specific_media_type_before_generic_json() {
        let manifest = serde_json::from_value::<OciManifest>(json!({
            "layers": [
                {
                    "mediaType": "application/json",
                    "digest": "sha256:plain",
                    "size": 1
                },
                {
                    "mediaType": TEST_LAYER_MEDIA_TYPE,
                    "digest": "sha256:catalog",
                    "size": 1
                }
            ]
        }))
        .unwrap();

        let descriptor = select_artifact_descriptor(&manifest, TEST_LAYER_MEDIA_TYPE).unwrap();

        assert_eq!(descriptor.digest, "sha256:catalog");
    }

    #[test]
    fn verifies_sha256_digest() {
        let payload = b"catalog";
        let digest = format!("sha256:{}", hex_lower(&Sha256::digest(payload)));

        verify_sha256_digest(&digest, payload).unwrap();
    }

    #[test]
    fn rejects_sha256_digest_mismatch() {
        let error = verify_sha256_digest("sha256:0000", b"catalog").unwrap_err();

        assert!(matches!(error, OciArtifactError::DigestMismatch { .. }));
    }
}
