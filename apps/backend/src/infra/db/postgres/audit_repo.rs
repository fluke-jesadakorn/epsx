use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde_json::Value as JsonValue;

use crate::{
    app::ports::repositories::{AuditRepo, ExportFormat},
    dom::entities::audit::{
        AuditLogEntry, AuditLogId, AuditQuery, AuditStatistics, AuditError,
        AuditMetadata
    },
    dom::entities::permission_profile::PermissionProfileId,
    dom::values::identifiers::UserId,
};

pub struct PostgresAuditRepo {
    pool: PgPool,
}

impl PostgresAuditRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_entry(row: &sqlx::postgres::PgRow) -> Result<AuditLogEntry, AuditError> {
        let _id: Uuid = row.try_get("id").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        let user_id: Option<Uuid> = row.try_get("user_id").ok();
        let action_str: String = row.try_get("action").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        let resource_type_str: String = row.try_get("resource_type").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        let resource_id: String = row.try_get("resource_id").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        let result_str: String = row.try_get("result").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        let _timestamp: DateTime<Utc> = row.try_get("timestamp").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        let client_ip: Option<String> = row.try_get("client_ip").ok();
        let user_agent: Option<String> = row.try_get("user_agent").ok();
        let session_id: Option<String> = row.try_get("session_id").ok();
        let metadata: JsonValue = row.try_get("metadata").unwrap_or(serde_json::json!({}));

        // Parse enum values
        let action = serde_json::from_value(serde_json::Value::String(action_str))
            .map_err(|e| AuditError::DatabaseError(format!("Invalid action: {}", e)))?;
        let resource_type = serde_json::from_value(serde_json::Value::String(resource_type_str))
            .map_err(|e| AuditError::DatabaseError(format!("Invalid resource type: {}", e)))?;
        let result = serde_json::from_value(serde_json::Value::String(result_str))
            .map_err(|e| AuditError::DatabaseError(format!("Invalid result: {}", e)))?;
        let audit_metadata: AuditMetadata = serde_json::from_value(metadata)
            .unwrap_or_else(|_| AuditMetadata::empty());

        let entry = AuditLogEntry::new(
            user_id.map(|id| UserId::new(id.to_string())).unwrap_or_else(|| UserId::new("system".to_string())),
            action,
            resource_type,
            resource_id,
            result,
        )
        .with_metadata(audit_metadata)
        .with_client_info(client_ip, user_agent);

        let entry = if let Some(session_id) = session_id {
            entry.with_session_id(session_id)
        } else {
            entry
        };

        Ok(entry)
    }
}

#[async_trait]
impl AuditRepo for PostgresAuditRepo {
    async fn store(&self, entry: &AuditLogEntry) -> Result<(), AuditError> {
        // Determine event category based on action
        let event_category = match entry.action() {
            crate::dom::entities::audit::AuditAction::Login | 
            crate::dom::entities::audit::AuditAction::LoginFailed | 
            crate::dom::entities::audit::AuditAction::Logout |
            crate::dom::entities::audit::AuditAction::PasswordReset => "authentication",
            crate::dom::entities::audit::AuditAction::PermissionGranted | 
            crate::dom::entities::audit::AuditAction::PermissionDenied |
            crate::dom::entities::audit::AuditAction::PermissionEvaluated => "authorization",
            crate::dom::entities::audit::AuditAction::SessionExpired => "session_management",
            crate::dom::entities::audit::AuditAction::UserCreated | 
            crate::dom::entities::audit::AuditAction::UserUpdated |
            crate::dom::entities::audit::AuditAction::UserDeleted => "user_management",
            _ => "system_security"
        };

        // Determine severity based on result and action
        let severity = match (entry.result(), entry.action()) {
            (crate::dom::entities::audit::AuditResult::Failure, _) |
            (crate::dom::entities::audit::AuditResult::Error, _) => "high",
            (_, crate::dom::entities::audit::AuditAction::LoginFailed) => "high",
            (crate::dom::entities::audit::AuditResult::Denied, _) => "medium",
            _ => "medium"
        };

        // Determine success based on result
        let success = matches!(entry.result(), crate::dom::entities::audit::AuditResult::Success | crate::dom::entities::audit::AuditResult::PartialSuccess);

        sqlx::query(
            "INSERT INTO audit_logs (id, actor_id, action, resource_type, resource_id, result, timestamp, client_ip, user_agent, session_id, metadata, user_id, event_category, severity, success)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"
        )
        .bind(Uuid::parse_str(entry.id().value()).map_err(|e| AuditError::DatabaseError(format!("Invalid ID UUID: {}", e)))?)
        .bind(entry.actor_id().value())
        .bind(entry.action().to_string())
        .bind(entry.resource_type().to_string())
        .bind(entry.resource_id())
        .bind(entry.result().to_string())
        .bind(entry.timestamp())
        .bind(entry.client_ip())
        .bind(entry.user_agent())
        .bind(entry.session_id())
        .bind(serde_json::to_value(entry.metadata()).map_err(|e| AuditError::DatabaseError(e.to_string()))?)
        .bind(entry.actor_id().value())
        .bind(event_category)
        .bind(severity)
        .bind(success)
        .execute(&self.pool)
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, id: &AuditLogId) -> Result<Option<AuditLogEntry>, AuditError> {
        let row = sqlx::query(
            "SELECT id, user_id, action, resource_type, resource_id, result, client_ip, user_agent, session_id, metadata, timestamp, event_category, severity, success
             FROM audit_logs WHERE id = $1"
        )
        .bind(Uuid::parse_str(id.value()).map_err(|e| AuditError::DatabaseError(format!("Invalid ID UUID: {}", e)))?)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(Self::map_row_to_entry(&row)?)),
            None => Ok(None),
        }
    }

    async fn search(&self, query: &AuditQuery) -> Result<Vec<AuditLogEntry>, AuditError> {
        let mut sql = "SELECT id, user_id, action, resource_type, resource_id, result, client_ip, user_agent, session_id, metadata, timestamp, event_category, severity, success FROM audit_logs WHERE 1=1".to_string();
        let mut query_builder = sqlx::QueryBuilder::new(&sql);

        // Add filters based on query
        if let Some(actor_id) = &query.actor_id {
            sql.push_str(" AND user_id = ");
            query_builder.push_bind(actor_id.value());
        }

        if let Some(action) = &query.action {
            sql.push_str(" AND action = ");
            query_builder.push_bind(action.to_string());
        }

        if let Some(resource_type) = &query.resource_type {
            sql.push_str(" AND resource_type = ");
            query_builder.push_bind(resource_type.to_string());
        }

        if let Some(resource_id) = &query.resource_id {
            sql.push_str(" AND resource_id = ");
            query_builder.push_bind(resource_id);
        }

        if let Some(result) = &query.result {
            sql.push_str(" AND result = ");
            query_builder.push_bind(result.to_string());
        }

        if let Some(from_time) = &query.from_time {
            sql.push_str(" AND timestamp >= ");
            query_builder.push_bind(from_time);
        }

        if let Some(to_time) = &query.to_time {
            sql.push_str(" AND timestamp <= ");
            query_builder.push_bind(to_time);
        }

        if let Some(client_ip) = &query.client_ip {
            sql.push_str(" AND client_ip = ");
            query_builder.push_bind(client_ip);
        }

        if let Some(session_id) = &query.session_id {
            sql.push_str(" AND session_id = ");
            query_builder.push_bind(session_id);
        }

        // Add ordering and pagination
        sql.push_str(" ORDER BY timestamp DESC");
        
        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ");
            query_builder.push_bind(limit as i64);
        }

        if let Some(offset) = query.offset {
            sql.push_str(" OFFSET ");
            query_builder.push_bind(offset as i64);
        }

        // Execute the query with proper parameter binding
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(Self::map_row_to_entry(&row)?);
        }

        Ok(entries)
    }

    async fn count(&self, _query: &AuditQuery) -> Result<u64, AuditError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM audit_logs")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        let count: i64 = row.try_get("count").map_err(|e| AuditError::DatabaseError(e.to_string()))?;
        Ok(count as u64)
    }

    async fn statistics(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<AuditStatistics, AuditError> {
        let row = sqlx::query(
            "SELECT 
                COUNT(*) as total_entries,
                COUNT(DISTINCT user_id) as unique_actors,
                COUNT(CASE WHEN result = 'Success' THEN 1 END) as successful_operations,
                COUNT(CASE WHEN result != 'Success' THEN 1 END) as failed_operations
             FROM audit_logs 
             WHERE timestamp >= $1 AND timestamp <= $2"
        )
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        let total_entries: i64 = row.try_get("total_entries").unwrap_or(0);
        let unique_actors: i64 = row.try_get("unique_actors").unwrap_or(0);
        let successful_operations: i64 = row.try_get("successful_operations").unwrap_or(0);
        let failed_operations: i64 = row.try_get("failed_operations").unwrap_or(0);

        Ok(AuditStatistics {
            total_entries: total_entries as u64,
            failed_operations: failed_operations as u64,
            successful_operations: successful_operations as u64,
            unique_actors: unique_actors as u32,
            top_actions: vec![], // TODO: Implement top actions query
            top_actors: vec![], // TODO: Implement top actors query  
            from_time: from,
            to_time: to,
        })
    }

    async fn cleanup_old_entries(&self, older_than: DateTime<Utc>) -> Result<u64, AuditError> {
        let result = sqlx::query(
            "DELETE FROM audit_logs WHERE timestamp < $1"
        )
        .bind(older_than)
        .execute(&self.pool)
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn store_batch(&self, entries: &[AuditLogEntry]) -> Result<(), AuditError> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, result, client_ip, user_agent, session_id, metadata, timestamp, event_category, severity, success) "
        );

        query_builder.push_values(entries, |mut b, entry| {
            // Determine event category based on action
            let event_category = match entry.action() {
                crate::dom::entities::audit::AuditAction::Login | 
                crate::dom::entities::audit::AuditAction::LoginFailed | 
                crate::dom::entities::audit::AuditAction::Logout |
                crate::dom::entities::audit::AuditAction::PasswordReset => "authentication",
                crate::dom::entities::audit::AuditAction::PermissionGranted | 
                crate::dom::entities::audit::AuditAction::PermissionDenied |
                crate::dom::entities::audit::AuditAction::PermissionEvaluated => "authorization",
                crate::dom::entities::audit::AuditAction::SessionExpired => "session_management",
                crate::dom::entities::audit::AuditAction::UserCreated | 
                crate::dom::entities::audit::AuditAction::UserUpdated |
                crate::dom::entities::audit::AuditAction::UserDeleted => "user_management",
                _ => "system_security"
            };

            // Determine severity based on result and action
            let severity = match (entry.result(), entry.action()) {
                (crate::dom::entities::audit::AuditResult::Failure, _) |
                (crate::dom::entities::audit::AuditResult::Error, _) => "high",
                (_, crate::dom::entities::audit::AuditAction::LoginFailed) => "high",
                (crate::dom::entities::audit::AuditResult::Denied, _) => "medium",
                _ => "medium"
            };

            // Determine success based on result
            let success = matches!(entry.result(), crate::dom::entities::audit::AuditResult::Success | crate::dom::entities::audit::AuditResult::PartialSuccess);

            b.push_bind(entry.id().value())
             .push_bind(entry.actor_id().value())
             .push_bind(entry.action().to_string())
             .push_bind(entry.resource_type().to_string())
             .push_bind(entry.resource_id())
             .push_bind(entry.result().to_string())
             .push_bind(entry.client_ip())
             .push_bind(entry.user_agent())
             .push_bind(entry.session_id())
             .push_bind(serde_json::to_value(entry.metadata()).unwrap_or(serde_json::Value::Null))
             .push_bind(entry.timestamp())
             .push_bind(event_category)
             .push_bind(severity)
             .push_bind(success);
        });

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn export(&self, query: &AuditQuery, format: ExportFormat) -> Result<Vec<u8>, AuditError> {
        let entries = self.search(query).await?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_vec(&entries)
                    .map_err(|e| AuditError::StorageError(e.to_string()))?;
                Ok(json)
            },
            ExportFormat::Csv => {
                let mut csv_data = String::new();
                csv_data.push_str("id,actor_id,action,resource_type,resource_id,result,client_ip,user_agent,session_id,timestamp\n");
                
                for entry in entries {
                    csv_data.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{},{}\n",
                        entry.id().value(),
                        entry.actor_id().value(),
                        entry.action(),
                        entry.resource_type(),
                        entry.resource_id(),
                        entry.result(),
                        entry.client_ip().unwrap_or_default(),
                        entry.user_agent().unwrap_or_default(),
                        entry.session_id().unwrap_or_default(),
                        entry.timestamp().to_rfc3339()
                    ));
                }
                
                Ok(csv_data.into_bytes())
            },
            ExportFormat::Xml => {
                let mut xml_data = String::new();
                xml_data.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<audit_logs>\n");
                
                for entry in entries {
                    xml_data.push_str(&format!(
                        "  <entry id=\"{}\" timestamp=\"{}\">\n    <actor_id>{}</actor_id>\n    <action>{}</action>\n    <resource_type>{}</resource_type>\n    <result>{}</result>\n  </entry>\n",
                        entry.id().value(),
                        entry.timestamp().to_rfc3339(),
                        entry.actor_id().value(),
                        entry.action(),
                        entry.resource_type(),
                        entry.result()
                    ));
                }
                
                xml_data.push_str("</audit_logs>\n");
                Ok(xml_data.into_bytes())
            },
        }
    }

    async fn cleanup_old_logs(&self, days: i64) -> Result<i64, AuditError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        let result = sqlx::query(
            "DELETE FROM audit_logs WHERE timestamp < $1"
        )
        .bind(cutoff_date)
        .execute(&self.pool)
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    async fn log_permission_assignment(&self, user_id: &UserId, profile_id: &PermissionProfileId, assigned_by: &str, reason: &str) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            UserId::new(assigned_by.to_string()),
            crate::dom::entities::audit::AuditAction::PermissionGranted,
            crate::dom::entities::audit::ResourceType::Permission,
            profile_id.value().to_string(),
            crate::dom::entities::audit::AuditResult::Success,
        ).with_metadata(
            AuditMetadata::empty()
                .with_additional_info("target_user", user_id.to_string())
                .with_additional_info("reason", reason.to_string())
        );

        self.store(&entry).await
    }

    async fn log_permission_revocation(&self, user_id: &UserId, profile_id: &PermissionProfileId, revoked_by: &str, reason: &str) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            UserId::new(revoked_by.to_string()),
            crate::dom::entities::audit::AuditAction::PermissionRevoked,
            crate::dom::entities::audit::ResourceType::Permission,
            profile_id.value().to_string(),
            crate::dom::entities::audit::AuditResult::Success,
        ).with_metadata(
            AuditMetadata::empty()
                .with_additional_info("target_user", user_id.to_string())
                .with_additional_info("reason", reason.to_string())
        );

        self.store(&entry).await
    }

    async fn log_system_event(&self, event_type: &str, details: &str) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            UserId::new("system".to_string()),
            crate::dom::entities::audit::AuditAction::SystemEvent,
            crate::dom::entities::audit::ResourceType::System,
            event_type.to_string(),
            crate::dom::entities::audit::AuditResult::Success,
        ).with_metadata(
            AuditMetadata::empty()
                .with_additional_info("description", details.to_string())
        );

        self.store(&entry).await
    }

    async fn log_notification_sent(&self, recipient: &str, subject: &str, notification_type: &str, message_id: Option<&str>) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            UserId::new("system".to_string()),
            crate::dom::entities::audit::AuditAction::NotificationSent,
            crate::dom::entities::audit::ResourceType::Notification,
            notification_type.to_string(),
            crate::dom::entities::audit::AuditResult::Success,
        ).with_metadata(
            AuditMetadata::empty()
                .with_additional_info("recipient", recipient.to_string())
                .with_additional_info("subject", subject.to_string())
                .with_additional_info("message_id", message_id.unwrap_or("").to_string())
        );

        self.store(&entry).await
    }

    async fn log_notification_failed(&self, recipient: &str, subject: &str, notification_type: &str, error: &str) -> Result<(), AuditError> {
        let entry = AuditLogEntry::new(
            UserId::new("system".to_string()),
            crate::dom::entities::audit::AuditAction::NotificationFailed,
            crate::dom::entities::audit::ResourceType::Notification,
            notification_type.to_string(),
            crate::dom::entities::audit::AuditResult::Failure,
        ).with_metadata(
            AuditMetadata::empty()
                .with_additional_info("recipient", recipient.to_string())
                .with_additional_info("subject", subject.to_string())
                .with_additional_info("error", error.to_string())
        );

        self.store(&entry).await
    }

    async fn health_check(&self) -> Result<(), AuditError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AuditError::DatabaseError(format!("Health check failed: {}", e)))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests would be implemented here with a test database
}