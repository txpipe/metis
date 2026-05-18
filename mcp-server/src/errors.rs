use std::net::AddrParseError;
use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("invalid MCP_BIND_ADDR: {0}")]
    InvalidBindAddr(AddrParseError),
    #[error("invalid MCP_AUTH_MODE '{0}', expected 'trusted' or 'oauth'")]
    InvalidAuthMode(String),
    #[error("invalid MCP_SESSION_STORE '{0}', expected 'memory' or 'sqlite'")]
    InvalidSessionStore(String),
    #[error("invalid MCP_SESSION_TTL_SECONDS: {0}")]
    InvalidSessionTtl(ParseIntError),
}
