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
    #[error("invalid MCP_EXTENSION_CATALOG_SOURCE '{0}', expected 'bundled' or 'oci'")]
    InvalidExtensionCatalogSource(String),
    #[error("MCP_EXTENSION_CATALOG_OCI_REF is required when MCP_EXTENSION_CATALOG_SOURCE=oci")]
    MissingExtensionCatalogOciRef,
    #[error("invalid MCP_EXTENSION_CATALOG_MAX_BYTES: {0}")]
    InvalidExtensionCatalogMaxBytes(ParseIntError),
    #[error("invalid MCP_EXTENSION_CATALOG_ALLOW_UNTRUSTED '{0}', expected 'true' or 'false'")]
    InvalidExtensionCatalogAllowUntrusted(String),
    #[error("invalid MCP_SKILL_CATALOG_SOURCE '{0}', expected 'bundled' or 'oci'")]
    InvalidSkillCatalogSource(String),
    #[error("MCP_SKILL_CATALOG_OCI_REF is required when MCP_SKILL_CATALOG_SOURCE=oci")]
    MissingSkillCatalogOciRef,
    #[error("invalid MCP_SKILL_CATALOG_MAX_BYTES: {0}")]
    InvalidSkillCatalogMaxBytes(ParseIntError),
    #[error("invalid MCP_SKILL_CATALOG_ALLOW_UNTRUSTED '{0}', expected 'true' or 'false'")]
    InvalidSkillCatalogAllowUntrusted(String),
}
