use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;

use crate::auth::AuthContext;
use crate::auth::AuthMode;
use crate::policy::PolicyDecision;
use crate::policy::PolicyOutcome;
use crate::policy::Scope;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditTarget {
    None,
    VaultRuntime {
        path: String,
        written_keys: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub auth_mode: AuthMode,
    pub enforced: bool,
    pub subject: String,
    pub client_id: Option<String>,
    pub scopes: Vec<Scope>,
    pub required_scopes: Vec<Scope>,
    pub tool: String,
    pub approval_id: Option<String>,
    pub target: AuditTarget,
    pub decision: PolicyDecision,
    pub reason: Option<String>,
    pub secret_values_included: bool,
}

impl AuditEvent {
    pub fn from_policy_outcome(
        auth: &AuthContext,
        tool: impl Into<String>,
        approval_id: Option<String>,
        target: AuditTarget,
        outcome: &PolicyOutcome,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            auth_mode: auth.auth_mode,
            enforced: auth.enforced,
            subject: auth.subject.clone(),
            client_id: auth.client_id.clone(),
            scopes: auth.scopes.iter().cloned().collect(),
            required_scopes: outcome.required_scopes.iter().cloned().collect(),
            tool: tool.into(),
            approval_id,
            target,
            decision: outcome.decision,
            reason: outcome.reason.clone(),
            secret_values_included: false,
        }
    }
}
