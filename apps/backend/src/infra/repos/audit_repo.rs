// Audit Repository implementation for storing and querying audit logs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::app::ports::repositories::{AuditRepo, ExportFormat};
use crate::dom::entities::audit::{
    AuditLogEntry, AuditLogId, AuditQuery, AuditStatistics, 
    AuditError, AuditAction, AuditResult
};
use crate::dom::values::UserId;

/// In-memory audit repository implementation (for development/testing)
/// In production, this would be replaced with a database implementation
pub struct AuditRepoImpl {
    entries: Mutex<HashMap<String, AuditLogEntry>>,
    // For production: replaced with PostgreSQL implementation
}

impl AuditRepoImpl {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl AuditRepo for AuditRepoImpl {
    async fn store(&self, entry: &AuditLogEntry) -> Result<(), AuditError> {
        let mut entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        let entry_id = entry.id().value().to_string();
        entries.insert(entry_id, entry.clone());
        
        // In production: also write to persistent storage
        tracing::info!(
            "Audit log stored: {} performed {} on {} {} - {}",
            entry.actor_id(),
            entry.action(),
            entry.resource_type(),
            entry.resource_id(),
            entry.result()
        );
        
        Ok(())
    }
    
    async fn get(&self, id: &AuditLogId) -> Result<Option<AuditLogEntry>, AuditError> {
        let entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        Ok(entries.get(id.value()).cloned())
    }
    
    async fn search(&self, query: &AuditQuery) -> Result<Vec<AuditLogEntry>, AuditError> {
        let entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        let mut results: Vec<AuditLogEntry> = entries.values()
            .filter(|entry| self.matches_query(entry, query))
            .cloned()
            .collect();
        
        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp().cmp(a.timestamp()));
        
        // Apply pagination
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(100) as usize;
        
        let end = std::cmp::min(offset + limit, results.len());
        if offset >= results.len() {
            return Ok(Vec::new());
        }
        
        Ok(results[offset..end].to_vec())
    }
    
    async fn count(&self, query: &AuditQuery) -> Result<u64, AuditError> {
        let entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        let count = entries.values()
            .filter(|entry| self.matches_query(entry, query))
            .count();
        
        Ok(count as u64)
    }
    
    async fn statistics(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<AuditStatistics, AuditError> {
        let entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        let filtered_entries: Vec<&AuditLogEntry> = entries.values()
            .filter(|entry| {
                entry.timestamp() >= &from && entry.timestamp() <= &to
            })
            .collect();
        
        let total_entries = filtered_entries.len() as u64;
        
        let failed_operations = filtered_entries.iter()
            .filter(|entry| matches!(entry.result(), AuditResult::Failure | AuditResult::Error | AuditResult::Denied))
            .count() as u64;
        
        let successful_operations = filtered_entries.iter()
            .filter(|entry| matches!(entry.result(), AuditResult::Success | AuditResult::PartialSuccess))
            .count() as u64;
        
        // Count unique actors
        let mut unique_actors = std::collections::HashSet::new();
        for entry in &filtered_entries {
            unique_actors.insert(entry.actor_id().clone());
        }
        
        // Count actions
        let mut action_counts = HashMap::new();
        for entry in &filtered_entries {
            *action_counts.entry(entry.action().clone()).or_insert(0u32) += 1;
        }
        
        // Get top 5 actions
        let mut top_actions: Vec<(AuditAction, u32)> = action_counts.into_iter().collect();
        top_actions.sort_by(|a, b| b.1.cmp(&a.1));
        top_actions.truncate(5);
        
        // Count actor activity
        let mut actor_counts = HashMap::new();
        for entry in &filtered_entries {
            *actor_counts.entry(entry.actor_id().clone()).or_insert(0u32) += 1;
        }
        
        // Get top 5 actors
        let mut top_actors: Vec<(UserId, u32)> = actor_counts.into_iter().collect();
        top_actors.sort_by(|a, b| b.1.cmp(&a.1));
        top_actors.truncate(5);
        
        Ok(AuditStatistics {
            total_entries,
            failed_operations,
            successful_operations,
            unique_actors: unique_actors.len() as u32,
            top_actions,
            top_actors,
            from_time: from,
            to_time: to,
        })
    }
    
    async fn cleanup_old_entries(&self, older_than: DateTime<Utc>) -> Result<u64, AuditError> {
        let mut entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        let initial_count = entries.len();
        
        entries.retain(|_, entry| entry.timestamp() > &older_than);
        
        let removed_count = initial_count - entries.len();
        
        tracing::info!("Cleaned up {} old audit entries", removed_count);
        
        Ok(removed_count as u64)
    }
    
    async fn store_batch(&self, audit_entries: &[AuditLogEntry]) -> Result<(), AuditError> {
        let mut entries = self.entries.lock()
            .map_err(|e| AuditError::StorageError(format!("Lock error: {}", e)))?;
        
        for entry in audit_entries {
            let entry_id = entry.id().value().to_string();
            entries.insert(entry_id, entry.clone());
        }
        
        tracing::info!("Stored {} audit entries in batch", audit_entries.len());
        Ok(())
    }
    
    async fn export(&self, query: &AuditQuery, format: ExportFormat) -> Result<Vec<u8>, AuditError> {
        let audit_entries = self.search(query).await?;
        
        match format {
            ExportFormat::Json => {
                let json_data = serde_json::to_vec_pretty(&audit_entries)
                    .map_err(|e| AuditError::StorageError(format!("JSON serialization error: {}", e)))?;
                Ok(json_data)
            },
            
            ExportFormat::Csv => {
                let mut csv_data = String::new();
                
                // CSV Header
                csv_data.push_str("ID,Actor,Action,ResourceType,ResourceID,Result,Timestamp,ClientIP,UserAgent,SessionID\n");
                
                // CSV Rows
                for entry in &audit_entries {
                    csv_data.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{},{}\n",
                        entry.id().value(),
                        entry.actor_id(),
                        entry.action(),
                        entry.resource_type(),
                        entry.resource_id(),
                        entry.result(),
                        entry.timestamp().format("%Y-%m-%d %H:%M:%S UTC"),
                        entry.client_ip().unwrap_or(""),
                        entry.user_agent().unwrap_or("").replace(",", ";"), // Escape commas
                        entry.session_id().unwrap_or(""),
                    ));
                }
                
                Ok(csv_data.into_bytes())
            },
            
            ExportFormat::Xml => {
                let mut xml_data = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<audit_logs>\n");
                
                for entry in &audit_entries {
                    xml_data.push_str("  <entry>\n");
                    xml_data.push_str(&format!("    <id>{}</id>\n", entry.id().value()));
                    xml_data.push_str(&format!("    <actor>{}</actor>\n", entry.actor_id()));
                    xml_data.push_str(&format!("    <action>{}</action>\n", entry.action()));
                    xml_data.push_str(&format!("    <resource_type>{}</resource_type>\n", entry.resource_type()));
                    xml_data.push_str(&format!("    <resource_id>{}</resource_id>\n", entry.resource_id()));
                    xml_data.push_str(&format!("    <result>{}</result>\n", entry.result()));
                    xml_data.push_str(&format!("    <timestamp>{}</timestamp>\n", entry.timestamp().format("%Y-%m-%d %H:%M:%S UTC")));
                    
                    if let Some(client_ip) = entry.client_ip() {
                        xml_data.push_str(&format!("    <client_ip>{}</client_ip>\n", client_ip));
                    }
                    if let Some(user_agent) = entry.user_agent() {
                        xml_data.push_str(&format!("    <user_agent>{}</user_agent>\n", 
                            user_agent.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")));
                    }
                    if let Some(session_id) = entry.session_id() {
                        xml_data.push_str(&format!("    <session_id>{}</session_id>\n", session_id));
                    }
                    
                    xml_data.push_str("  </entry>\n");
                }
                
                xml_data.push_str("</audit_logs>\n");
                Ok(xml_data.into_bytes())
            },
        }
    }
}

impl AuditRepoImpl {
    fn matches_query(&self, entry: &AuditLogEntry, query: &AuditQuery) -> bool {
        // Filter by actor
        if let Some(ref actor_id) = query.actor_id {
            if entry.actor_id() != actor_id {
                return false;
            }
        }
        
        // Filter by action
        if let Some(ref action) = query.action {
            if entry.action() != action {
                return false;
            }
        }
        
        // Filter by resource type
        if let Some(ref resource_type) = query.resource_type {
            if entry.resource_type() != resource_type {
                return false;
            }
        }
        
        // Filter by resource ID
        if let Some(ref resource_id) = query.resource_id {
            if entry.resource_id() != resource_id {
                return false;
            }
        }
        
        // Filter by result
        if let Some(ref result) = query.result {
            if entry.result() != result {
                return false;
            }
        }
        
        // Filter by time range
        if let Some(ref from_time) = query.from_time {
            if entry.timestamp() < from_time {
                return false;
            }
        }
        
        if let Some(ref to_time) = query.to_time {
            if entry.timestamp() > to_time {
                return false;
            }
        }
        
        // Filter by client IP
        if let Some(ref client_ip) = query.client_ip {
            if entry.client_ip() != Some(client_ip.as_str()) {
                return false;
            }
        }
        
        // Filter by session ID
        if let Some(ref session_id) = query.session_id {
            if entry.session_id() != Some(session_id.as_str()) {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::audit::{AuditAction, ResourceType, AuditResult};
    
    #[tokio::test]
    async fn should_store_and_retrieve_audit_entry() {
        let repo = AuditRepoImpl::new();
        let actor_id = UserId::new("user123".to_string());
        
        let entry = AuditLogEntry::new(
            actor_id.clone(),
            AuditAction::UserCreated,
            ResourceType::User,
            "user456".to_string(),
            AuditResult::Success,
        );
        
        let entry_id = entry.id().clone();
        
        // Store entry
        repo.store(&entry).await.unwrap();
        
        // Retrieve entry
        let retrieved = repo.get(&entry_id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.actor_id(), &actor_id);
        assert_eq!(retrieved_entry.action(), &AuditAction::UserCreated);
        assert_eq!(retrieved_entry.resource_id(), "user456");
    }
    
    #[tokio::test]
    async fn should_search_audit_entries() {
        let repo = AuditRepoImpl::new();
        let actor_id = UserId::new("user123".to_string());
        
        // Create multiple entries
        let entries = vec![
            AuditLogEntry::new(
                actor_id.clone(),
                AuditAction::UserCreated,
                ResourceType::User,
                "user1".to_string(),
                AuditResult::Success,
            ),
            AuditLogEntry::new(
                actor_id.clone(),
                AuditAction::RoleCreated,
                ResourceType::Role,
                "role1".to_string(),
                AuditResult::Success,
            ),
        ];
        
        // Store entries
        for entry in &entries {
            repo.store(entry).await.unwrap();
        }
        
        // Search by action
        let query = AuditQuery::new().by_action(AuditAction::UserCreated);
        let results = repo.search(&query).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action(), &AuditAction::UserCreated);
    }
    
    #[tokio::test]
    async fn should_generate_statistics() {
        let repo = AuditRepoImpl::new();
        let actor_id = UserId::new("user123".to_string());
        
        // Create entries with different results
        let entries = vec![
            AuditLogEntry::new(
                actor_id.clone(),
                AuditAction::UserCreated,
                ResourceType::User,
                "user1".to_string(),
                AuditResult::Success,
            ),
            AuditLogEntry::new(
                actor_id.clone(),
                AuditAction::UserCreated,
                ResourceType::User,
                "user2".to_string(),
                AuditResult::Failure,
            ),
        ];
        
        for entry in &entries {
            repo.store(entry).await.unwrap();
        }
        
        let from = Utc::now() - chrono::Duration::hours(1);
        let to = Utc::now() + chrono::Duration::hours(1);
        
        let stats = repo.statistics(from, to).await.unwrap();
        
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.unique_actors, 1);
    }
    
    #[tokio::test]
    async fn should_export_to_json() {
        let repo = AuditRepoImpl::new();
        let actor_id = UserId::new("user123".to_string());
        
        let entry = AuditLogEntry::new(
            actor_id,
            AuditAction::UserCreated,
            ResourceType::User,
            "user456".to_string(),
            AuditResult::Success,
        );
        
        repo.store(&entry).await.unwrap();
        
        let query = AuditQuery::new();
        let exported = repo.export(&query, ExportFormat::Json).await.unwrap();
        
        assert!(!exported.is_empty());
        
        // Verify it's valid JSON
        let json_str = String::from_utf8(exported).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    }
    
    #[tokio::test]
    async fn should_cleanup_old_entries() {
        let repo = AuditRepoImpl::new();
        let actor_id = UserId::new("user123".to_string());
        
        let entry = AuditLogEntry::new(
            actor_id,
            AuditAction::UserCreated,
            ResourceType::User,
            "user456".to_string(),
            AuditResult::Success,
        );
        
        repo.store(&entry).await.unwrap();
        
        // Clean up entries older than "now" (should remove all entries)
        let cleaned = repo.cleanup_old_entries(Utc::now() + chrono::Duration::seconds(1)).await.unwrap();
        
        assert_eq!(cleaned, 1);
        
        // Verify entry was removed
        let query = AuditQuery::new();
        let results = repo.search(&query).await.unwrap();
        assert_eq!(results.len(), 0);
    }
}