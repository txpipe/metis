use std::collections::BTreeMap;
use std::io::Read;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use flate2::read::GzDecoder;
use k8s_openapi::ByteString;
use k8s_openapi::api::core::v1::Secret;
use kube::Api;
use kube::api::ListParams;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::k8s::KubernetesClient;

const HELM_SECRET_TYPE: &str = "helm.sh/release.v1";
const HELM_RELEASE_DATA_KEY: &str = "release";
const HELM_OWNER_LABEL: &str = "owner=helm";
const CONTROL_PLANE_RELEASE_NAME: &str = "control-plane";

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmReleaseSummary {
    pub name: String,
    pub namespace: String,
    pub revision: i32,
    pub status: Option<String>,
    pub chart: HelmChartSummary,
    pub app_version: Option<String>,
    pub description: Option<String>,
    pub updated: Option<Value>,
    pub secret_name: Option<String>,
    #[serde(skip_serializing)]
    pub config: Option<Value>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmChartSummary {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone)]
pub struct HelmReleaseDiscovery {
    client: KubernetesClient,
}

impl HelmReleaseDiscovery {
    pub fn new(client: KubernetesClient) -> Self {
        Self { client }
    }

    pub async fn list_latest(
        &self,
        namespace: Option<&str>,
        include_control_plane: bool,
    ) -> Result<Vec<HelmReleaseSummary>, HelmReleaseError> {
        let secrets = self.list_helm_secrets(namespace).await?;
        Ok(latest_releases(
            secrets
                .items
                .iter()
                .map(decode_helm_release_secret)
                .collect::<Result<Vec<_>, _>>()?,
            include_control_plane,
        ))
    }

    pub async fn get_latest(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<Option<HelmReleaseSummary>, HelmReleaseError> {
        Ok(self
            .list_latest(Some(namespace), true)
            .await?
            .into_iter()
            .find(|release| release.name == name))
    }

    async fn list_helm_secrets(
        &self,
        namespace: Option<&str>,
    ) -> Result<kube::api::ObjectList<Secret>, kube::Error> {
        let params = ListParams::default().labels(HELM_OWNER_LABEL);
        match namespace {
            Some(namespace) => {
                Api::<Secret>::namespaced(self.client.inner().clone(), namespace)
                    .list(&params)
                    .await
            }
            None => {
                Api::<Secret>::all(self.client.inner().clone())
                    .list(&params)
                    .await
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HelmReleaseError {
    #[error("kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),
    #[error("failed to decode Helm release secret: {0}")]
    Decode(#[from] HelmReleaseDecodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum HelmReleaseDecodeError {
    #[error("secret is not a Helm release secret")]
    NotHelmReleaseSecret,
    #[error("missing Helm release payload")]
    MissingPayload,
    #[error("release payload is not valid UTF-8 or base64/gzip encoded JSON")]
    InvalidPayload,
    #[error("release payload JSON is invalid: {0}")]
    InvalidJson(serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct HelmReleaseData {
    name: String,
    namespace: String,
    version: i32,
    info: Option<HelmReleaseInfo>,
    chart: Option<HelmChart>,
    config: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct HelmReleaseInfo {
    status: Option<String>,
    description: Option<String>,
    #[serde(rename = "last_deployed", alias = "lastDeployed")]
    last_deployed: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct HelmChart {
    metadata: Option<HelmChartMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HelmChartMetadata {
    name: Option<String>,
    version: Option<String>,
    app_version: Option<String>,
}

fn decode_helm_release_secret(
    secret: &Secret,
) -> Result<HelmReleaseSummary, HelmReleaseDecodeError> {
    if secret.type_.as_deref() != Some(HELM_SECRET_TYPE) {
        return Err(HelmReleaseDecodeError::NotHelmReleaseSecret);
    }

    let payload = secret
        .data
        .as_ref()
        .and_then(|data| data.get(HELM_RELEASE_DATA_KEY))
        .ok_or(HelmReleaseDecodeError::MissingPayload)?;
    let release = decode_release_payload(payload)?;
    let chart_metadata = release.chart.and_then(|chart| chart.metadata);
    let info = release.info;

    Ok(HelmReleaseSummary {
        name: release.name,
        namespace: release.namespace,
        revision: revision_from_secret(secret).unwrap_or(release.version),
        status: info.as_ref().and_then(|info| info.status.clone()),
        chart: HelmChartSummary {
            name: chart_metadata
                .as_ref()
                .and_then(|metadata| metadata.name.clone()),
            version: chart_metadata
                .as_ref()
                .and_then(|metadata| metadata.version.clone()),
        },
        app_version: chart_metadata.and_then(|metadata| metadata.app_version),
        description: info.as_ref().and_then(|info| info.description.clone()),
        updated: info.and_then(|info| info.last_deployed),
        secret_name: secret.metadata.name.clone(),
        config: release.config,
    })
}

fn decode_release_payload(payload: &ByteString) -> Result<HelmReleaseData, HelmReleaseDecodeError> {
    let mut candidates = vec![payload.0.clone()];

    if let Ok(text) = std::str::from_utf8(&payload.0)
        && let Ok(decoded) = STANDARD.decode(text.trim())
    {
        candidates.push(decoded);
    }

    for candidate in candidates {
        if let Ok(release) = serde_json::from_slice::<HelmReleaseData>(&candidate) {
            return Ok(release);
        }

        if let Ok(inflated) = gunzip(&candidate) {
            return serde_json::from_slice::<HelmReleaseData>(&inflated)
                .map_err(HelmReleaseDecodeError::InvalidJson);
        }
    }

    Err(HelmReleaseDecodeError::InvalidPayload)
}

fn gunzip(payload: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(payload);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded)?;
    Ok(decoded)
}

fn revision_from_secret(secret: &Secret) -> Option<i32> {
    secret
        .metadata
        .labels
        .as_ref()?
        .get("version")?
        .parse()
        .ok()
}

fn latest_releases(
    releases: Vec<HelmReleaseSummary>,
    include_control_plane: bool,
) -> Vec<HelmReleaseSummary> {
    let mut latest = BTreeMap::<(String, String), HelmReleaseSummary>::new();

    for release in releases {
        if !include_control_plane && is_control_plane_release(&release) {
            continue;
        }

        let key = (release.namespace.clone(), release.name.clone());
        let replace = latest
            .get(&key)
            .is_none_or(|existing| release.revision > existing.revision);

        if replace {
            latest.insert(key, release);
        }
    }

    latest.into_values().collect()
}

fn is_control_plane_release(release: &HelmReleaseSummary) -> bool {
    release.name == CONTROL_PLANE_RELEASE_NAME
        || release.chart.name.as_deref() == Some(CONTROL_PLANE_RELEASE_NAME)
        || release.namespace == CONTROL_PLANE_RELEASE_NAME
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use k8s_openapi::api::core::v1::Secret;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use serde_json::json;

    use super::*;

    #[test]
    fn decodes_base64_gzip_helm_release_payload() {
        let secret = helm_secret("sh.helm.release.v1.cardano.v2", "cardano", "cardano", 2);

        let release = decode_helm_release_secret(&secret).unwrap();

        assert_eq!(release.name, "cardano");
        assert_eq!(release.namespace, "cardano");
        assert_eq!(release.revision, 2);
        assert_eq!(release.status.as_deref(), Some("deployed"));
        assert_eq!(release.chart.name.as_deref(), Some("cardano-relay"));
        assert_eq!(release.chart.version.as_deref(), Some("0.1.0-rc1"));
        assert_eq!(release.app_version.as_deref(), Some("10.7.1"));
        assert_eq!(release.config, Some(json!({ "mustNotBeReturned": true })));
    }

    #[test]
    fn latest_releases_keep_highest_revision_and_exclude_control_plane() {
        let releases = vec![
            decode_helm_release_secret(&helm_secret(
                "sh.helm.release.v1.cardano.v1",
                "cardano",
                "cardano",
                1,
            ))
            .unwrap(),
            decode_helm_release_secret(&helm_secret(
                "sh.helm.release.v1.cardano.v3",
                "cardano",
                "cardano",
                3,
            ))
            .unwrap(),
            decode_helm_release_secret(&helm_secret(
                "sh.helm.release.v1.control-plane.v1",
                "control-plane",
                "control-plane",
                1,
            ))
            .unwrap(),
        ];

        let latest = latest_releases(releases, false);

        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].name, "cardano");
        assert_eq!(latest[0].revision, 3);
    }

    #[test]
    fn malformed_release_payload_is_rejected() {
        let secret = Secret {
            type_: Some(HELM_SECRET_TYPE.to_string()),
            data: Some(BTreeMap::from([(
                HELM_RELEASE_DATA_KEY.to_string(),
                ByteString(b"not-valid".to_vec()),
            )])),
            metadata: ObjectMeta {
                name: Some("sh.helm.release.v1.bad.v1".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let error = decode_helm_release_secret(&secret).unwrap_err();

        assert!(matches!(error, HelmReleaseDecodeError::InvalidPayload));
    }

    fn helm_secret(
        secret_name: &str,
        namespace: &str,
        release_name: &str,
        revision: i32,
    ) -> Secret {
        Secret {
            type_: Some(HELM_SECRET_TYPE.to_string()),
            data: Some(BTreeMap::from([(
                HELM_RELEASE_DATA_KEY.to_string(),
                encoded_release(namespace, release_name, revision),
            )])),
            metadata: ObjectMeta {
                name: Some(secret_name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([
                    ("owner".to_string(), "helm".to_string()),
                    ("name".to_string(), release_name.to_string()),
                    ("version".to_string(), revision.to_string()),
                ])),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn encoded_release(namespace: &str, release_name: &str, revision: i32) -> ByteString {
        let release = json!({
            "name": release_name,
            "namespace": namespace,
            "version": revision,
            "info": {
                "status": "deployed",
                "description": "Install complete",
                "last_deployed": "2026-05-07T12:00:00Z"
            },
            "chart": {
                "metadata": {
                    "name": if release_name == "control-plane" { "control-plane" } else { "cardano-relay" },
                    "version": "0.1.0-rc1",
                    "appVersion": "10.7.1"
                }
            },
            "config": { "mustNotBeReturned": true },
            "manifest": "apiVersion: v1\nkind: Secret\n"
        });
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(release.to_string().as_bytes()).unwrap();
        let gzipped = encoder.finish().unwrap();

        ByteString(STANDARD.encode(gzipped).into_bytes())
    }
}
