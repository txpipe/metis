use k8s_openapi::NamespaceResourceScope;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::api::core::v1::Endpoints;
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::core::v1::Service;
use k8s_openapi::api::storage::v1::StorageClass;
use kube::Api;
use kube::Client;
use kube::Resource;
use kube::api::AttachParams;
use kube::api::ListParams;
use kube::api::LogParams;
use kube::api::ObjectList;
use serde::de::DeserializeOwned;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::time::Duration;

const DEFAULT_LOG_TAIL_LINES: i64 = 200;
const MAX_LOG_TAIL_LINES: i64 = 1_000;
const DEFAULT_EXEC_TIMEOUT_SECONDS: u64 = 30;
const MAX_EXEC_STDOUT_BYTES: usize = 512 * 1024;
const MAX_EXEC_STDERR_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodExecOutput {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PodExecError {
    #[error("kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),
    #[error("pod exec did not expose {0}")]
    MissingStream(&'static str),
    #[error("failed to read pod exec {stream}: {source}")]
    Read {
        stream: &'static str,
        source: std::io::Error,
    },
    #[error("pod exec {stream} exceeded {max_bytes} bytes")]
    OutputTooLarge {
        stream: &'static str,
        max_bytes: usize,
    },
    #[error("pod exec timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
    #[error("pod exec failed: {0}")]
    RemoteCommand(String),
    #[error("pod exec command failed: {message}")]
    CommandFailed { message: String },
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ResourceListParams {
    pub label_selector: Option<String>,
    pub field_selector: Option<String>,
    pub limit: Option<u32>,
}

impl ResourceListParams {
    pub fn to_kube(&self) -> ListParams {
        let mut params = ListParams::default();

        if let Some(label_selector) = &self.label_selector {
            params = params.labels(label_selector);
        }

        if let Some(field_selector) = &self.field_selector {
            params = params.fields(field_selector);
        }

        if let Some(limit) = self.limit {
            params = params.limit(limit);
        }

        params
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodLogParams {
    pub container: Option<String>,
    pub previous: bool,
    pub tail_lines: Option<i64>,
    pub since_seconds: Option<i64>,
    pub timestamps: bool,
}

impl Default for PodLogParams {
    fn default() -> Self {
        Self {
            container: None,
            previous: false,
            tail_lines: Some(DEFAULT_LOG_TAIL_LINES),
            since_seconds: None,
            timestamps: false,
        }
    }
}

impl PodLogParams {
    pub fn to_kube(&self) -> LogParams {
        LogParams {
            container: self.container.clone(),
            previous: self.previous,
            since_seconds: self.since_seconds,
            tail_lines: Some(self.effective_tail_lines()),
            timestamps: self.timestamps,
            ..Default::default()
        }
    }

    fn effective_tail_lines(&self) -> i64 {
        self.tail_lines
            .unwrap_or(DEFAULT_LOG_TAIL_LINES)
            .clamp(1, MAX_LOG_TAIL_LINES)
    }
}

#[derive(Clone)]
pub struct KubernetesClient {
    client: Client,
}

impl KubernetesClient {
    pub async fn try_default() -> Result<Self, kube::Error> {
        Ok(Self {
            client: Client::try_default().await?,
        })
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }

    pub async fn list_namespaces(
        &self,
        params: &ResourceListParams,
    ) -> Result<ObjectList<Namespace>, kube::Error> {
        Api::<Namespace>::all(self.client.clone())
            .list(&params.to_kube())
            .await
    }

    pub async fn get_namespace(&self, name: &str) -> Result<Namespace, kube::Error> {
        Api::<Namespace>::all(self.client.clone()).get(name).await
    }

    pub async fn list_pods(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<Pod>, kube::Error> {
        self.api::<Pod>(namespace).list(&params.to_kube()).await
    }

    pub async fn get_pod(&self, namespace: &str, name: &str) -> Result<Pod, kube::Error> {
        self.api::<Pod>(Some(namespace)).get(name).await
    }

    pub async fn pod_logs(
        &self,
        namespace: &str,
        name: &str,
        params: &PodLogParams,
    ) -> Result<String, kube::Error> {
        self.api::<Pod>(Some(namespace))
            .logs(name, &params.to_kube())
            .await
    }

    pub async fn pod_exec_capture(
        &self,
        namespace: &str,
        name: &str,
        container: &str,
        command: &[&str],
    ) -> Result<PodExecOutput, PodExecError> {
        let pods = self.api::<Pod>(Some(namespace));
        let params = AttachParams {
            container: Some(container.to_string()),
            stdin: false,
            stdout: true,
            stderr: true,
            tty: false,
            max_stdout_buf_size: Some(8 * 1024),
            max_stderr_buf_size: Some(8 * 1024),
            ..Default::default()
        };
        let command = command
            .iter()
            .map(|argument| (*argument).to_string())
            .collect::<Vec<_>>();
        let mut process = pods.exec(name, command, &params).await?;
        let stdout = process
            .stdout()
            .ok_or(PodExecError::MissingStream("stdout"))?;
        let stderr = process
            .stderr()
            .ok_or(PodExecError::MissingStream("stderr"))?;
        let status = process
            .take_status()
            .ok_or(PodExecError::MissingStream("status"))?;

        let capture = async {
            tokio::try_join!(
                read_limited(stdout, "stdout", MAX_EXEC_STDOUT_BYTES),
                read_limited(stderr, "stderr", MAX_EXEC_STDERR_BYTES),
                async { Ok::<_, PodExecError>(status.await) },
            )
        };

        let (stdout, stderr, status) =
            match tokio::time::timeout(Duration::from_secs(DEFAULT_EXEC_TIMEOUT_SECONDS), capture)
                .await
            {
                Ok(Ok(result)) => result,
                Ok(Err(error)) => {
                    process.abort();
                    return Err(error);
                }
                Err(_) => {
                    process.abort();
                    return Err(PodExecError::Timeout {
                        seconds: DEFAULT_EXEC_TIMEOUT_SECONDS,
                    });
                }
            };

        process
            .join()
            .await
            .map_err(|error| PodExecError::RemoteCommand(error.to_string()))?;

        if let Some(status) = status
            && status.status.as_deref() == Some("Failure")
        {
            return Err(PodExecError::CommandFailed {
                message: status_message(&status),
            });
        }

        Ok(PodExecOutput { stdout, stderr })
    }

    pub async fn list_services(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<Service>, kube::Error> {
        self.api::<Service>(namespace).list(&params.to_kube()).await
    }

    pub async fn get_service(&self, namespace: &str, name: &str) -> Result<Service, kube::Error> {
        self.api::<Service>(Some(namespace)).get(name).await
    }

    pub async fn list_endpoints(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<Endpoints>, kube::Error> {
        self.api::<Endpoints>(namespace)
            .list(&params.to_kube())
            .await
    }

    pub async fn get_endpoints(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<Endpoints, kube::Error> {
        self.api::<Endpoints>(Some(namespace)).get(name).await
    }

    pub async fn list_persistent_volume_claims(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<PersistentVolumeClaim>, kube::Error> {
        self.api::<PersistentVolumeClaim>(namespace)
            .list(&params.to_kube())
            .await
    }

    pub async fn get_persistent_volume_claim(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<PersistentVolumeClaim, kube::Error> {
        self.api::<PersistentVolumeClaim>(Some(namespace))
            .get(name)
            .await
    }

    pub async fn list_config_maps(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<ConfigMap>, kube::Error> {
        self.api::<ConfigMap>(namespace)
            .list(&params.to_kube())
            .await
    }

    pub async fn get_config_map(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<ConfigMap, kube::Error> {
        self.api::<ConfigMap>(Some(namespace)).get(name).await
    }

    pub async fn list_events(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<Event>, kube::Error> {
        self.api::<Event>(namespace).list(&params.to_kube()).await
    }

    pub async fn list_deployments(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<Deployment>, kube::Error> {
        self.api::<Deployment>(namespace)
            .list(&params.to_kube())
            .await
    }

    pub async fn get_deployment(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<Deployment, kube::Error> {
        self.api::<Deployment>(Some(namespace)).get(name).await
    }

    pub async fn list_stateful_sets(
        &self,
        namespace: Option<&str>,
        params: &ResourceListParams,
    ) -> Result<ObjectList<StatefulSet>, kube::Error> {
        self.api::<StatefulSet>(namespace)
            .list(&params.to_kube())
            .await
    }

    pub async fn get_stateful_set(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<StatefulSet, kube::Error> {
        self.api::<StatefulSet>(Some(namespace)).get(name).await
    }

    pub async fn list_storage_classes(
        &self,
        params: &ResourceListParams,
    ) -> Result<ObjectList<StorageClass>, kube::Error> {
        Api::<StorageClass>::all(self.client.clone())
            .list(&params.to_kube())
            .await
    }

    pub async fn get_storage_class(&self, name: &str) -> Result<StorageClass, kube::Error> {
        Api::<StorageClass>::all(self.client.clone())
            .get(name)
            .await
    }

    fn api<K>(&self, namespace: Option<&str>) -> Api<K>
    where
        K: Clone + DeserializeOwned + Resource<DynamicType = (), Scope = NamespaceResourceScope>,
    {
        match namespace {
            Some(namespace) => Api::namespaced(self.client.clone(), namespace),
            None => Api::all(self.client.clone()),
        }
    }
}

async fn read_limited(
    mut reader: impl AsyncRead + Unpin,
    stream: &'static str,
    max_bytes: usize,
) -> Result<String, PodExecError> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let read = reader
            .read(&mut buffer)
            .await
            .map_err(|source| PodExecError::Read { stream, source })?;
        if read == 0 {
            break;
        }
        if output.len() + read > max_bytes {
            return Err(PodExecError::OutputTooLarge { stream, max_bytes });
        }
        output.extend_from_slice(&buffer[..read]);
    }

    Ok(String::from_utf8_lossy(&output).to_string())
}

fn status_message(status: &k8s_openapi::apimachinery::pkg::apis::meta::v1::Status) -> String {
    status
        .message
        .clone()
        .or_else(|| status.reason.clone())
        .or_else(|| status.code.map(|code| format!("status code {code}")))
        .unwrap_or_else(|| "remote command failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_params_preserve_selectors_and_limit() {
        let params = ResourceListParams {
            label_selector: Some("app=cardano-node".to_string()),
            field_selector: Some("metadata.namespace=cardano".to_string()),
            limit: Some(50),
        }
        .to_kube();

        assert_eq!(params.label_selector.as_deref(), Some("app=cardano-node"));
        assert_eq!(
            params.field_selector.as_deref(),
            Some("metadata.namespace=cardano")
        );
        assert_eq!(params.limit, Some(50));
    }

    #[test]
    fn log_params_default_to_bounded_tail() {
        let params = PodLogParams::default().to_kube();

        assert_eq!(params.tail_lines, Some(DEFAULT_LOG_TAIL_LINES));
        assert!(!params.previous);
        assert!(!params.timestamps);
    }

    #[test]
    fn log_params_clamp_tail_lines() {
        let too_high = PodLogParams {
            tail_lines: Some(10_000),
            ..Default::default()
        };
        let too_low = PodLogParams {
            tail_lines: Some(0),
            ..Default::default()
        };

        assert_eq!(too_high.to_kube().tail_lines, Some(MAX_LOG_TAIL_LINES));
        assert_eq!(too_low.to_kube().tail_lines, Some(1));
    }
}
