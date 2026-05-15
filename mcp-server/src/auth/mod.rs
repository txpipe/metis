use std::collections::BTreeSet;
use std::str::FromStr;

use serde::Serialize;

use crate::errors::ConfigError;
use crate::policy::Role;
use crate::policy::Scope;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Trusted,
    OAuth,
}

impl FromStr for AuthMode {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "trusted" => Ok(Self::Trusted),
            "oauth" => Ok(Self::OAuth),
            other => Err(ConfigError::InvalidAuthMode(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthContext {
    pub auth_mode: AuthMode,
    pub subject: String,
    pub client_id: Option<String>,
    pub roles: BTreeSet<Role>,
    pub scopes: BTreeSet<Scope>,
    pub issuer: String,
    pub audience: Vec<String>,
    pub enforced: bool,
}

impl AuthContext {
    pub fn trusted() -> Self {
        Self {
            auth_mode: AuthMode::Trusted,
            subject: "trusted-local-operator".to_string(),
            client_id: Some("trusted-mcp-client".to_string()),
            roles: BTreeSet::from([Role::Admin]),
            scopes: Scope::all(),
            issuer: "trusted".to_string(),
            audience: vec!["metis-supernode-mcp".to_string()],
            enforced: false,
        }
    }
}
