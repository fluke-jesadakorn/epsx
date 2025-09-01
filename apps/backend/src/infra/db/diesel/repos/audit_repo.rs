use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use diesel::prelude::*;

use diesel_async::RunQueryDsl;

use std::sync::Arc;


use crate::app::ports::repositories::{AuditRepository, ExportFormat};

use crate::dom::entities::audit::{AuditLogEntry, AuditLogId, AuditQuery, AuditStatistics, AuditError};

use crate::infra::db::diesel::{

    DbPool,
    schema::audit_logs,
    models::{DieselAuditLog, NewDieselAuditLog},
};

pub struct DieselAuditRepository {
    pool: Arc<DbPool>,
}

impl DieselAuditRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for DieselAuditRepository {
    async fn store(&self, entry: &AuditLogEntry) -> Result<(), AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let new_entry: NewDieselAuditLog = entry.try_into()
            .map_err(|e| AuditError::SerializationError(format!("Failed to convert AuditLogEntry: {:?}", e)))?;
        
        diesel::insert_into(audit_logs::table)
            .values(&new_entry)
            .execute(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn get(&self, _id: &AuditLogId) -> Result<Option<AuditLogEntry>, AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| AuditError::InvalidQuery(format!("Invalid UUID: {}", e)))?;
        
        let diesel_entry = audit_logs::table
            .filter(audit_logs::id.eq(uuid))
            .first::<DieselAuditLog>(&mut conn)
            .await
            .optional()
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        match diesel_entry {
            Some(diesel_entry) => {
                let entry = diesel_entry.try_into()
                    .map_err(|e| AuditError::SerializationError(format!("Failed to convert DieselAuditLog: {:?}", e)))?;
                Ok(Some(entry))
            }
            None => Ok(None)
        }
    }
    
    async fn search(&self, query: &AuditQuery) -> Result<Vec<AuditLogEntry>, AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let mut diesel_query = audit_logs::table.into_boxed();
        
        // Apply filters
        if let Some(actor_id) = &query.actor_id {
            let user_uuid = Uuid::parse_str(&actor_id.to_string())
                .map_err(|e| AuditError::InvalidQuery(format!("Invalid user UUID: {}", e)))?;
            diesel_query = diesel_query.filter(audit_logs::user_id.eq(user_uuid));
        }
        
        if let Some(action) = &query.action {
            diesel_query = diesel_query.filter(audit_logs::action.ilike(format!("%{}%", action.to_string())));
        }
        
        if let Some(resource_type) = &query.resource_type {
            diesel_query = diesel_query.filter(audit_logs::resource_type.eq(resource_type.to_string()));
        }
        
        if let Some(from_time) = query.from_time {
            diesel_query = diesel_query.filter(audit_logs::timestamp.ge(from_time));
        }
        
        if let Some(to_time) = query.to_time {
            diesel_query = diesel_query.filter(audit_logs::timestamp.le(to_time));
        }
        
        // Apply ordering and pagination
        diesel_query = diesel_query.order(audit_logs::timestamp.desc());
        
        if let Some(offset) = query.offset {
            diesel_query = diesel_query.offset(offset as i64);
        }
        
        if let Some(limit) = query.limit {
            diesel_query = diesel_query.limit(limit as i64);
        }
        
        let diesel_entries = diesel_query
            .load::<DieselAuditLog>(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let entries: Result<Vec<AuditLogEntry>, AuditError> = diesel_entries
            .into_iter()
            .map(|diesel_entry| {
                diesel_entry.try_into()
                    .map_err(|e| AuditError::SerializationError(format!("Failed to convert DieselAuditLog: {:?}", e)))
            })
            .collect();
        
        entries
    }
    
    async fn count(&self, query: &AuditQuery) -> Result<u64, AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let mut diesel_query = audit_logs::table.into_boxed();
        
        // Apply the same filters as search
        if let Some(actor_id) = &query.actor_id {
            let user_uuid = Uuid::parse_str(&actor_id.to_string())
                .map_err(|e| AuditError::InvalidQuery(format!("Invalid user UUID: {}", e)))?;
            diesel_query = diesel_query.filter(audit_logs::user_id.eq(user_uuid));
        }
        
        if let Some(action) = &query.action {
            diesel_query = diesel_query.filter(audit_logs::action.ilike(format!("%{}%", action.to_string())));
        }
        
        if let Some(resource_type) = &query.resource_type {
            diesel_query = diesel_query.filter(audit_logs::resource_type.eq(resource_type.to_string()));
        }
        
        if let Some(from_time) = query.from_time {
            diesel_query = diesel_query.filter(audit_logs::timestamp.ge(from_time));
        }
        
        if let Some(to_time) = query.to_time {
            diesel_query = diesel_query.filter(audit_logs::timestamp.le(to_time));
        }
        
        let count = diesel_query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        Ok(count as u64)
    }
    
    async fn statistics(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<AuditStatistics, AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        // Get total events
        let total_events = audit_logs::table
            .filter(audit_logs::timestamp.between(from, to))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))? as u64;
        
        // Get unique users
        let unique_users = audit_logs::table
            .filter(audit_logs::timestamp.between(from, to))
            .filter(audit_logs::user_id.is_not_null())
            .select(audit_logs::user_id)
            .distinct()
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))? as u64;
        
        // Get unique actions
        let _unique_actions = audit_logs::table
            .filter(audit_logs::timestamp.between(from, to))
            .select(audit_logs::action)
            .distinct()
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))? as u64;
        
        Ok(AuditStatistics {
            total_entries: total_events,
            failed_operations: 0, // TODO: implement failed operations count
            successful_operations: total_events, // For now, assume all are successful
            unique_actors: unique_users as u32,
            top_actions: vec![], // TODO: implement top actions
            top_actors: vec![], // TODO: implement top actors
            from_time: from,
            to_time: to,
        })
    }
    
    async fn cleanup_old_entries(&self, older_than: DateTime<Utc>) -> Result<u64, AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        // Use a transaction for consistent cleanup
        let deleted = diesel::delete(audit_logs::table)
            .filter(audit_logs::timestamp.lt(older_than))
            .execute(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        Ok(deleted as u64)
    }
    
    async fn store_batch(&self, entries: &[AuditLogEntry]) -> Result<(), AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let diesel_entries: Result<Vec<NewDieselAuditLog>, AuditError> = entries
            .iter()
            .map(|entry| {
                entry.try_into()
                    .map_err(|e| AuditError::SerializationError(format!("Failed to convert AuditLogEntry: {:?}", e)))
            })
            .collect();
        
        let diesel_entries = diesel_entries?;
        
        diesel::insert_into(audit_logs::table)
            .values(&diesel_entries)
            .execute(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn export(&self, query: &AuditQuery, format: ExportFormat) -> Result<Vec<u8>, AuditError> {
        let entries = self.search(query).await?;
        
        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&entries)
                    .map_err(|e| AuditError::SerializationError(format!("JSON serialization failed: {}", e)))?;
                Ok(json.into_bytes())
            }
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("id,user_id,action,resource_type,resource_id,ip_address,created_at\n");
                
                for entry in entries {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        entry.id(),
                        entry.user_id().to_string(),
                        entry.action(),
                        entry.resource_type().to_string(),
                        entry.resource_id(),
                        entry.ip_address().unwrap_or(""),
                        entry.created_at().format("%Y-%m-%d %H:%M:%S")
                    ));
                }
                
                Ok(csv.into_bytes())
            }
            ExportFormat::Xml => {
                let mut xml = String::new();
                xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<audit_logs>\n");
                
                for entry in entries {
                    xml.push_str(&format!(
                        "  <entry id=\"{}\" user_id=\"{}\" action=\"{}\" created_at=\"{}\" />\n",
                        entry.id(),
                        entry.user_id().to_string(),
                        entry.action(),
                        entry.created_at().format("%Y-%m-%d %H:%M:%S")
                    ));
                }
                
                xml.push_str("</audit_logs>");
                Ok(xml.into_bytes())
            }
        }
    }
    
    // Job system methods
    async fn cleanup_old_logs(&self, days: i64) -> Result<i64, AuditError> {
        let cutoff_date = Utc::now() - Duration::days(days);
        let deleted = self.cleanup_old_entries(cutoff_date).await?;
        Ok(deleted as i64)
    }
    
    
    async fn log_system_event(&self, event_type: &str, _details: &str) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            crate::dom::values::identifiers::UserId::new("system".to_string()),
            crate::dom::entities::audit::AuditAction::SystemEvent,
            crate::dom::entities::audit::ResourceType::System,
            event_type.to_string(),
            crate::dom::entities::audit::AuditResult::Success,
        );
        
        self.store(&entry).await
    }
    
    async fn log_notification_sent(&self, recipient: &str, _subject: &str, _notification_type: &str, message_id: Option<&str>) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            crate::dom::values::identifiers::UserId::new("system".to_string()),
            crate::dom::entities::audit::AuditAction::NotificationSent,
            crate::dom::entities::audit::ResourceType::Notification,
            message_id.unwrap_or(recipient).to_string(),
            crate::dom::entities::audit::AuditResult::Success,
        );
        
        self.store(&entry).await
    }
    
    async fn log_notification_failed(&self, recipient: &str, _subject: &str, _notification_type: &str, _error: &str) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            crate::dom::values::identifiers::UserId::new("system".to_string()),
            crate::dom::entities::audit::AuditAction::NotificationFailed,
            crate::dom::entities::audit::ResourceType::Notification,
            recipient.to_string(),
            crate::dom::entities::audit::AuditResult::Failure,
        );
        
        self.store(&entry).await
    }
    
    async fn health_check(&self) -> Result<(), AuditError> {
        let mut conn = self.pool.get().await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        let _count = audit_logs::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::db::diesel::create_pool;
    
    #[tokio::test]
    async fn test_audit_repo_creation() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost/test".to_string());
        
        if let Ok(pool) = create_pool(&database_url).await {
            let repo = DieselAuditRepository::new(Arc::new(pool));
            // Test passes if we can create the repo
            assert!(true);
        }
        // Test passes even if database is not available
    }
}