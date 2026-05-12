pub mod router;
pub mod supernode;
pub mod vault;
pub mod workloads;

pub use router::ToolRouter;

use crate::policy::ApprovalClass;
use crate::policy::Scope;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub required_scope: Scope,
    pub approval_class: ApprovalClass,
    pub read_only: bool,
    pub destructive: bool,
    pub input_schema: &'static str,
}
