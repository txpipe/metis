use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use crate::auth::AuthMode;
use crate::errors::ConfigError;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub auth_mode: AuthMode,
    pub log_level: String,
    pub session_store: SessionStoreConfig,
    pub extension_catalog: ExtensionCatalogConfig,
    pub skill_catalog: SkillCatalogConfig,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SessionStoreConfig {
    Memory,
    Sqlite {
        path: PathBuf,
        ttl_seconds: Option<i64>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExtensionCatalogConfig {
    pub source: ExtensionCatalogSource,
    pub oci_ref: Option<String>,
    pub max_bytes: usize,
    pub allow_untrusted: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExtensionCatalogSource {
    Bundled,
    Oci,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SkillCatalogConfig {
    pub source: SkillCatalogSource,
    pub oci_ref: Option<String>,
    pub max_bytes: usize,
    pub allow_untrusted: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SkillCatalogSource {
    Bundled,
    Oci,
}

impl FromStr for ExtensionCatalogSource {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "bundled" => Ok(Self::Bundled),
            "oci" => Ok(Self::Oci),
            other => Err(ConfigError::InvalidExtensionCatalogSource(
                other.to_string(),
            )),
        }
    }
}

impl FromStr for SkillCatalogSource {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "bundled" => Ok(Self::Bundled),
            "oci" => Ok(Self::Oci),
            other => Err(ConfigError::InvalidSkillCatalogSource(other.to_string())),
        }
    }
}

impl FromStr for SessionStoreConfig {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "memory" => Ok(Self::Memory),
            "sqlite" => Ok(Self::Sqlite {
                path: session_sqlite_path(),
                ttl_seconds: session_ttl_seconds()?,
            }),
            other => Err(ConfigError::InvalidSessionStore(other.to_string())),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = env::var("MCP_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8443".to_string())
            .parse()
            .map_err(ConfigError::InvalidBindAddr)?;

        let auth_mode = env::var("MCP_AUTH_MODE")
            .unwrap_or_else(|_| "trusted".to_string())
            .parse()?;

        let log_level = env::var("MCP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let session_store = env::var("MCP_SESSION_STORE")
            .unwrap_or_else(|_| "memory".to_string())
            .parse()?;
        let extension_catalog = extension_catalog_config()?;
        let skill_catalog = skill_catalog_config()?;

        Ok(Self {
            bind_addr,
            auth_mode,
            log_level,
            session_store,
            extension_catalog,
            skill_catalog,
        })
    }
}

fn extension_catalog_config() -> Result<ExtensionCatalogConfig, ConfigError> {
    let source = env::var("MCP_EXTENSION_CATALOG_SOURCE")
        .unwrap_or_else(|_| "bundled".to_string())
        .parse()?;
    let oci_ref = env::var("MCP_EXTENSION_CATALOG_OCI_REF")
        .ok()
        .filter(|value| !value.trim().is_empty());
    if source == ExtensionCatalogSource::Oci && oci_ref.is_none() {
        return Err(ConfigError::MissingExtensionCatalogOciRef);
    }

    Ok(ExtensionCatalogConfig {
        source,
        oci_ref,
        max_bytes: extension_catalog_max_bytes()?,
        allow_untrusted: extension_catalog_allow_untrusted()?,
    })
}

fn skill_catalog_config() -> Result<SkillCatalogConfig, ConfigError> {
    let source = env::var("MCP_SKILL_CATALOG_SOURCE")
        .unwrap_or_else(|_| "bundled".to_string())
        .parse()?;
    let oci_ref = env::var("MCP_SKILL_CATALOG_OCI_REF")
        .ok()
        .filter(|value| !value.trim().is_empty());
    if source == SkillCatalogSource::Oci && oci_ref.is_none() {
        return Err(ConfigError::MissingSkillCatalogOciRef);
    }

    Ok(SkillCatalogConfig {
        source,
        oci_ref,
        max_bytes: skill_catalog_max_bytes()?,
        allow_untrusted: skill_catalog_allow_untrusted()?,
    })
}

fn session_sqlite_path() -> PathBuf {
    env::var("MCP_SESSION_SQLITE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/lib/supernode-mcp/sessions.sqlite3"))
}

fn session_ttl_seconds() -> Result<Option<i64>, ConfigError> {
    env::var("MCP_SESSION_TTL_SECONDS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.parse().map_err(ConfigError::InvalidSessionTtl))
        .transpose()
}

fn extension_catalog_max_bytes() -> Result<usize, ConfigError> {
    env::var("MCP_EXTENSION_CATALOG_MAX_BYTES")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse()
                .map_err(ConfigError::InvalidExtensionCatalogMaxBytes)
        })
        .transpose()
        .map(|value| value.unwrap_or(1_048_576))
}

fn extension_catalog_allow_untrusted() -> Result<bool, ConfigError> {
    match env::var("MCP_EXTENSION_CATALOG_ALLOW_UNTRUSTED") {
        Ok(value) if value.trim().is_empty() => Ok(false),
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(ConfigError::InvalidExtensionCatalogAllowUntrusted(value)),
        },
        Err(_) => Ok(false),
    }
}

fn skill_catalog_max_bytes() -> Result<usize, ConfigError> {
    env::var("MCP_SKILL_CATALOG_MAX_BYTES")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse()
                .map_err(ConfigError::InvalidSkillCatalogMaxBytes)
        })
        .transpose()
        .map(|value| value.unwrap_or(1_048_576))
}

fn skill_catalog_allow_untrusted() -> Result<bool, ConfigError> {
    match env::var("MCP_SKILL_CATALOG_ALLOW_UNTRUSTED") {
        Ok(value) if value.trim().is_empty() => Ok(false),
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(ConfigError::InvalidSkillCatalogAllowUntrusted(value)),
        },
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_memory_session_store() {
        assert_eq!(
            "memory".parse::<SessionStoreConfig>().unwrap(),
            SessionStoreConfig::Memory
        );
    }

    #[test]
    fn rejects_unknown_session_store() {
        let error = "redis".parse::<SessionStoreConfig>().unwrap_err();

        assert!(matches!(error, ConfigError::InvalidSessionStore(_)));
    }

    #[test]
    fn parses_catalog_sources() {
        assert_eq!(
            "bundled".parse::<ExtensionCatalogSource>().unwrap(),
            ExtensionCatalogSource::Bundled
        );
        assert_eq!(
            "oci".parse::<ExtensionCatalogSource>().unwrap(),
            ExtensionCatalogSource::Oci
        );
    }

    #[test]
    fn parses_skill_catalog_sources() {
        assert_eq!(
            "bundled".parse::<SkillCatalogSource>().unwrap(),
            SkillCatalogSource::Bundled
        );
        assert_eq!(
            "oci".parse::<SkillCatalogSource>().unwrap(),
            SkillCatalogSource::Oci
        );
    }

    #[test]
    fn rejects_unknown_catalog_source() {
        let error = "file".parse::<ExtensionCatalogSource>().unwrap_err();

        assert!(matches!(
            error,
            ConfigError::InvalidExtensionCatalogSource(_)
        ));
    }

    #[test]
    fn rejects_unknown_skill_catalog_source() {
        let error = "file".parse::<SkillCatalogSource>().unwrap_err();

        assert!(matches!(error, ConfigError::InvalidSkillCatalogSource(_)));
    }
}
