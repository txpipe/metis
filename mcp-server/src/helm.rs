use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Serialize;
use serde_json::Value;
use tokio::process::Command;

const DEFAULT_HELM_BIN: &str = "helm";

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmChartRef {
    pub chart: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HelmInstallPlan {
    pub release_name: String,
    pub namespace: String,
    pub chart: HelmChartRef,
    pub values: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HelmUpgradePlan {
    pub release_name: String,
    pub namespace: String,
    pub chart: HelmChartRef,
    pub values: Value,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HelmUninstallPlan {
    pub release_name: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmInstallResult {
    pub command: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmUpgradeResult {
    pub command: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmUninstallResult {
    pub command: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
pub enum HelmInstallError {
    #[error("failed to create Helm values directory: {0}")]
    CreateValuesDir(std::io::Error),
    #[error("failed to serialize Helm values: {0}")]
    SerializeValues(serde_json::Error),
    #[error("failed to write Helm values file: {0}")]
    WriteValues(std::io::Error),
    #[error("failed to execute Helm: {0}")]
    Execute(std::io::Error),
    #[error("Helm install failed with status {status}")]
    Failed {
        status: i32,
        stdout: String,
        stderr: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum HelmUpgradeError {
    #[error("failed to create Helm values directory: {0}")]
    CreateValuesDir(std::io::Error),
    #[error("failed to serialize Helm values: {0}")]
    SerializeValues(serde_json::Error),
    #[error("failed to write Helm values file: {0}")]
    WriteValues(std::io::Error),
    #[error("failed to execute Helm: {0}")]
    Execute(std::io::Error),
    #[error("Helm upgrade failed with status {status}")]
    Failed {
        status: i32,
        stdout: String,
        stderr: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum HelmUninstallError {
    #[error("failed to execute Helm: {0}")]
    Execute(std::io::Error),
    #[error("Helm uninstall failed with status {status}")]
    Failed {
        status: i32,
        stdout: String,
        stderr: String,
    },
}

pub async fn install(plan: &HelmInstallPlan) -> Result<HelmInstallResult, HelmInstallError> {
    let values_path = write_values_file(&plan.values).map_err(HelmInstallError::from)?;
    let helm_bin = std::env::var("MCP_HELM_BIN").unwrap_or_else(|_| DEFAULT_HELM_BIN.to_string());
    let args = install_args(plan, &values_path);

    let output = Command::new(&helm_bin).args(&args).output().await;
    let _ = std::fs::remove_file(&values_path);
    let output = output.map_err(HelmInstallError::Execute)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(HelmInstallError::Failed {
            status: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
        });
    }

    let mut command = vec![helm_bin];
    command.extend(args);

    Ok(HelmInstallResult {
        command,
        stdout,
        stderr,
    })
}

pub async fn upgrade(plan: &HelmUpgradePlan) -> Result<HelmUpgradeResult, HelmUpgradeError> {
    let values_path = write_values_file(&plan.values).map_err(HelmUpgradeError::from)?;
    let helm_bin = std::env::var("MCP_HELM_BIN").unwrap_or_else(|_| DEFAULT_HELM_BIN.to_string());
    let args = upgrade_args(plan, &values_path);

    let output = Command::new(&helm_bin).args(&args).output().await;
    let _ = std::fs::remove_file(&values_path);
    let output = output.map_err(HelmUpgradeError::Execute)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(HelmUpgradeError::Failed {
            status: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
        });
    }

    let mut command = vec![helm_bin];
    command.extend(args);

    Ok(HelmUpgradeResult {
        command,
        stdout,
        stderr,
    })
}

pub async fn uninstall(
    plan: &HelmUninstallPlan,
) -> Result<HelmUninstallResult, HelmUninstallError> {
    let helm_bin = std::env::var("MCP_HELM_BIN").unwrap_or_else(|_| DEFAULT_HELM_BIN.to_string());
    let args = uninstall_args(plan);
    let output = Command::new(&helm_bin)
        .args(&args)
        .output()
        .await
        .map_err(HelmUninstallError::Execute)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(HelmUninstallError::Failed {
            status: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
        });
    }

    let mut command = vec![helm_bin];
    command.extend(args);

    Ok(HelmUninstallResult {
        command,
        stdout,
        stderr,
    })
}

fn write_values_file(values: &Value) -> Result<PathBuf, HelmValuesFileError> {
    let dir = std::env::var("MCP_HELM_VALUES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("supernode-mcp-helm"));
    std::fs::create_dir_all(&dir).map_err(HelmValuesFileError::CreateValuesDir)?;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = dir.join(format!("values-{unique}.json"));
    let payload =
        serde_json::to_vec_pretty(values).map_err(HelmValuesFileError::SerializeValues)?;
    std::fs::write(&path, payload).map_err(HelmValuesFileError::WriteValues)?;
    Ok(path)
}

#[derive(Debug, thiserror::Error)]
enum HelmValuesFileError {
    #[error("failed to create Helm values directory: {0}")]
    CreateValuesDir(std::io::Error),
    #[error("failed to serialize Helm values: {0}")]
    SerializeValues(serde_json::Error),
    #[error("failed to write Helm values file: {0}")]
    WriteValues(std::io::Error),
}

impl From<HelmValuesFileError> for HelmInstallError {
    fn from(error: HelmValuesFileError) -> Self {
        match error {
            HelmValuesFileError::CreateValuesDir(error) => Self::CreateValuesDir(error),
            HelmValuesFileError::SerializeValues(error) => Self::SerializeValues(error),
            HelmValuesFileError::WriteValues(error) => Self::WriteValues(error),
        }
    }
}

impl From<HelmValuesFileError> for HelmUpgradeError {
    fn from(error: HelmValuesFileError) -> Self {
        match error {
            HelmValuesFileError::CreateValuesDir(error) => Self::CreateValuesDir(error),
            HelmValuesFileError::SerializeValues(error) => Self::SerializeValues(error),
            HelmValuesFileError::WriteValues(error) => Self::WriteValues(error),
        }
    }
}

fn install_args(plan: &HelmInstallPlan, values_path: &Path) -> Vec<String> {
    vec![
        "upgrade".to_string(),
        "--install".to_string(),
        plan.release_name.clone(),
        plan.chart.chart.clone(),
        "--namespace".to_string(),
        plan.namespace.clone(),
        "--create-namespace".to_string(),
        "--version".to_string(),
        plan.chart.version.clone(),
        "--values".to_string(),
        values_path.display().to_string(),
    ]
}

fn upgrade_args(plan: &HelmUpgradePlan, values_path: &Path) -> Vec<String> {
    vec![
        "upgrade".to_string(),
        plan.release_name.clone(),
        plan.chart.chart.clone(),
        "--namespace".to_string(),
        plan.namespace.clone(),
        "--version".to_string(),
        plan.chart.version.clone(),
        "--values".to_string(),
        values_path.display().to_string(),
    ]
}

fn uninstall_args(plan: &HelmUninstallPlan) -> Vec<String> {
    vec![
        "uninstall".to_string(),
        plan.release_name.clone(),
        "--namespace".to_string(),
        plan.namespace.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn install_args_are_bounded_to_upgrade_install() {
        let plan = HelmInstallPlan {
            release_name: "cardano-preview".to_string(),
            namespace: "cardano-preview".to_string(),
            chart: HelmChartRef {
                chart: "oci://oci.supernode.store/extensions/cardano-node".to_string(),
                version: "0.1.0-rc1".to_string(),
            },
            values: json!({ "node": { "network": "preview" } }),
        };
        let args = install_args(&plan, &PathBuf::from("/tmp/values.json"));

        assert_eq!(args[0], "upgrade");
        assert_eq!(args[1], "--install");
        assert!(args.contains(&"--create-namespace".to_string()));
        assert!(args.contains(&"/tmp/values.json".to_string()));
        assert!(!args.contains(&"--wait".to_string()));
        assert!(!args.contains(&"--timeout".to_string()));
        assert!(!args.contains(&"--atomic".to_string()));
        assert!(!args.iter().any(|arg| arg.contains("rawValues")));
    }

    #[test]
    fn uninstall_args_are_bounded_to_release_and_namespace() {
        let plan = HelmUninstallPlan {
            release_name: "hydra-offline".to_string(),
            namespace: "hydra".to_string(),
        };

        let args = uninstall_args(&plan);

        assert_eq!(args[0], "uninstall");
        assert_eq!(args[1], "hydra-offline");
        assert!(args.contains(&"--namespace".to_string()));
        assert!(args.contains(&"hydra".to_string()));
        assert!(!args.contains(&"--wait".to_string()));
        assert!(!args.contains(&"--timeout".to_string()));
    }

    #[test]
    fn upgrade_args_do_not_install_missing_releases() {
        let plan = HelmUpgradePlan {
            release_name: "hydra-offline".to_string(),
            namespace: "hydra".to_string(),
            chart: HelmChartRef {
                chart: "oci://oci.supernode.store/extensions/hydra-node".to_string(),
                version: "0.2.0".to_string(),
            },
            values: json!({ "node": { "offlineMode": true } }),
        };

        let args = upgrade_args(&plan, &PathBuf::from("/tmp/values.json"));

        assert_eq!(args[0], "upgrade");
        assert_eq!(args[1], "hydra-offline");
        assert!(args.contains(&"/tmp/values.json".to_string()));
        assert!(!args.contains(&"--wait".to_string()));
        assert!(!args.contains(&"--timeout".to_string()));
        assert!(!args.contains(&"--atomic".to_string()));
        assert!(!args.contains(&"--install".to_string()));
        assert!(!args.contains(&"--create-namespace".to_string()));
    }
}
