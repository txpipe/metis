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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SessionStoreConfig {
    Memory,
    Sqlite {
        path: PathBuf,
        ttl_seconds: Option<i64>,
    },
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

        Ok(Self {
            bind_addr,
            auth_mode,
            log_level,
            session_store,
        })
    }
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
}
