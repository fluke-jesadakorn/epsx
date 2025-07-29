// In-memory Level History Repository Implementation

use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use crate::app::ports::repositories::{LevelHistoryRepo, RepoError};
use crate::app::dtos::LevelChangeRecord;
use crate::dom::values::UserId;

/// In-memory implementation of LevelHistoryRepo for development/testing
#[derive(Debug, Default)]
pub struct InMemoryLevelHistoryRepo {
    records: Arc<RwLock<HashMap<String, LevelChangeRecord>>>,
    user_index: Arc<RwLock<HashMap<String, Vec<String>>>>, // UserId -> Vec<RecordId>
}

impl InMemoryLevelHistoryRepo {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            user_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LevelHistoryRepo for InMemoryLevelHistoryRepo {
    async fn save_level_change(&self, record: &LevelChangeRecord) -> Result<(), RepoError> {
        let mut records = self.records.write()
            .map_err(|e| RepoError::Internal(format!("Failed to acquire write lock: {}", e)))?;
        
        let mut user_index = self.user_index.write()
            .map_err(|e| RepoError::Internal(format!("Failed to acquire write lock: {}", e)))?;
        
        // Store the record
        records.insert(record.id.clone(), record.clone());
        
        // Update user index
        user_index
            .entry(record.usr_id.clone())
            .or_insert_with(Vec::new)
            .push(record.id.clone());
        
        tracing::info!("Saved level change record: {} for user {}", record.id, record.usr_id);
        Ok(())
    }
    
    async fn get_user_level_history(
        &self, 
        user_id: &UserId, 
        limit: u32, 
        offset: u32
    ) -> Result<Vec<LevelChangeRecord>, RepoError> {
        let records = self.records.read()
            .map_err(|e| RepoError::Internal(format!("Failed to acquire read lock: {}", e)))?;
        
        let user_index = self.user_index.read()
            .map_err(|e| RepoError::Internal(format!("Failed to acquire read lock: {}", e)))?;
        
        let user_id_str = user_id.to_string();
        let empty_vec = vec![];
        let record_ids = user_index.get(&user_id_str).unwrap_or(&empty_vec);
        
        let mut user_records: Vec<_> = record_ids
            .iter()
            .filter_map(|id| records.get(id))
            .cloned()
            .collect();
        
        // Sort by changed_at timestamp, most recent first
        user_records.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));
        
        // Apply pagination
        let start = offset as usize;
        let end = start + limit as usize;
        
        let page = if start < user_records.len() {
            user_records[start..end.min(user_records.len())].to_vec()
        } else {
            vec![]
        };
        
        Ok(page)
    }
    
    async fn count_user_level_changes(&self, user_id: &UserId) -> Result<u64, RepoError> {
        let user_index = self.user_index.read()
            .map_err(|e| RepoError::Internal(format!("Failed to acquire read lock: {}", e)))?;
        
        let user_id_str = user_id.to_string();
        let count = user_index.get(&user_id_str).map(|v| v.len()).unwrap_or(0);
        
        Ok(count as u64)
    }
    
    async fn get_recent_level_changes(&self, limit: u32) -> Result<Vec<LevelChangeRecord>, RepoError> {
        let records = self.records.read()
            .map_err(|e| RepoError::Internal(format!("Failed to acquire read lock: {}", e)))?;
        
        let mut all_records: Vec<_> = records.values().cloned().collect();
        
        // Sort by changed_at timestamp, most recent first
        all_records.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));
        
        // Take only the requested number of records
        all_records.truncate(limit as usize);
        
        Ok(all_records)
    }
}

/// Create a level change record
pub fn create_level_change_record(
    user_id: &UserId,
    old_role: &str,
    new_role: &str,
    changed_by: &UserId,
    reason: Option<String>,
) -> LevelChangeRecord {
    LevelChangeRecord {
        id: uuid::Uuid::new_v4().to_string(),
        usr_id: user_id.to_string(),
        old_role: old_role.to_string(),
        new_role: new_role.to_string(),
        changed_by: changed_by.to_string(),
        reason,
        changed_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn should_save_and_retrieve_level_change() {
        let repo = InMemoryLevelHistoryRepo::new();
        let user_id = UserId::generate();
        let admin_id = UserId::generate();
        
        let record = create_level_change_record(
            &user_id,
            "user",
            "premium",
            &admin_id,
            Some("Upgrade test".to_string()),
        );
        
        // Save record
        repo.save_level_change(&record).await.unwrap();
        
        // Retrieve record
        let history = repo.get_user_level_history(&user_id, 10, 0).await.unwrap();
        
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].old_role, "user");
        assert_eq!(history[0].new_role, "premium");
    }
    
    #[tokio::test]
    async fn should_count_user_level_changes() {
        let repo = InMemoryLevelHistoryRepo::new();
        let user_id = UserId::generate();
        let admin_id = UserId::generate();
        
        // Save multiple records
        for i in 0..3 {
            let record = create_level_change_record(
                &user_id,
                "user",
                &format!("role_{}", i),
                &admin_id,
                None,
            );
            repo.save_level_change(&record).await.unwrap();
        }
        
        let count = repo.count_user_level_changes(&user_id).await.unwrap();
        assert_eq!(count, 3);
    }
}