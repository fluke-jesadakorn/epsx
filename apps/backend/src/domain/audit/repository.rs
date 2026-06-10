use async_trait::async_trait;
use crate::domain::shared_kernel::entities::audit::{AuditLogEntry, AuditQuery};
use anyhow::Result;

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn save(&self, entry: AuditLogEntry) -> Result<AuditLogEntry>;
    async fn find_all(&self, query: AuditQuery) -> Result<(Vec<AuditLogEntry>, i64)>;
}
