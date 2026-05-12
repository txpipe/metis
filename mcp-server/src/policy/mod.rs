pub mod approvals;
pub mod roles;
pub mod scopes;

use std::collections::BTreeSet;

use serde::Serialize;

use crate::auth::AuthContext;

pub use approvals::ApprovalClass;
pub use roles::Role;
pub use scopes::Scope;

#[derive(Debug, Clone, Default)]
pub struct Policy;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allowed,
    AdvisoryAllowed,
    Denied,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyOutcome {
    pub decision: PolicyDecision,
    pub required_scopes: BTreeSet<Scope>,
    pub approval_required: bool,
    pub approval_present: bool,
    pub reason: Option<String>,
}

impl Policy {
    pub fn require_scope(&self, auth: &AuthContext, scope: Scope) -> PolicyOutcome {
        let required_scopes = BTreeSet::from([scope]);

        if auth.scopes.contains(&scope) {
            return PolicyOutcome {
                decision: PolicyDecision::Allowed,
                required_scopes,
                approval_required: false,
                approval_present: false,
                reason: None,
            };
        }

        let decision = if auth.enforced {
            PolicyDecision::Denied
        } else {
            PolicyDecision::AdvisoryAllowed
        };

        PolicyOutcome {
            decision,
            required_scopes,
            approval_required: false,
            approval_present: false,
            reason: Some(format!("missing required scope {scope:?}")),
        }
    }

    pub fn require_approval(
        &self,
        auth: &AuthContext,
        approval_id: Option<&str>,
        approval_class: ApprovalClass,
    ) -> PolicyOutcome {
        let approval_required = approval_class.requires_approval();
        let approval_present = approval_id.is_some_and(|value| !value.trim().is_empty());
        let missing_required_approval = approval_required && !approval_present;

        if !missing_required_approval {
            return PolicyOutcome {
                decision: PolicyDecision::Allowed,
                required_scopes: BTreeSet::new(),
                approval_required,
                approval_present,
                reason: None,
            };
        }

        let decision = if auth.enforced {
            PolicyDecision::Denied
        } else {
            PolicyDecision::AdvisoryAllowed
        };

        PolicyOutcome {
            decision,
            required_scopes: BTreeSet::new(),
            approval_required,
            approval_present,
            reason: Some(format!("missing approval for {approval_class:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;

    #[test]
    fn trusted_context_has_all_scopes() {
        let auth = AuthContext::trusted();

        assert_eq!(auth.scopes, Scope::all());
        assert!(!auth.enforced);
    }

    #[test]
    fn trusted_policy_allows_missing_scope_as_advisory() {
        let mut auth = AuthContext::trusted();
        auth.scopes.remove(&Scope::Discover);

        let outcome = Policy.require_scope(&auth, Scope::Discover);

        assert_eq!(outcome.decision, PolicyDecision::AdvisoryAllowed);
        assert!(outcome.reason.is_some());
    }

    #[test]
    fn enforced_policy_denies_missing_scope() {
        let mut auth = AuthContext::trusted();
        auth.enforced = true;
        auth.scopes.remove(&Scope::Discover);

        let outcome = Policy.require_scope(&auth, Scope::Discover);

        assert_eq!(outcome.decision, PolicyDecision::Denied);
    }

    #[test]
    fn trusted_policy_allows_missing_approval_as_advisory() {
        let auth = AuthContext::trusted();

        let outcome = Policy.require_approval(&auth, None, ApprovalClass::Mutation);

        assert_eq!(outcome.decision, PolicyDecision::AdvisoryAllowed);
        assert!(outcome.approval_required);
        assert!(!outcome.approval_present);
    }
}
