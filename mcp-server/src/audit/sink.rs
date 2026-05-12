use tracing::info;

use crate::audit::AuditEvent;

pub trait AuditSink: Send + Sync {
    fn record(&self, event: &AuditEvent);
}

#[derive(Debug, Default)]
pub struct TracingAuditSink;

impl AuditSink for TracingAuditSink {
    fn record(&self, event: &AuditEvent) {
        info!(target: "supernode_mcp::audit", ?event, "audit event");
    }
}
