// In-memory level history repository (stub implementation)
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::app::ports::{LevelHistoryRepo, RepoError};
use crate::dom::values::UserId;
use crate::app::dtos::user::LevelChangeRecord;

/// Simple in-memory implementation for level history tracking
pub struct InMemoryLevelHistoryRepo {
    storage: Arc<Mutex<HashMap<String, Vec<LevelChangeRecord>>>>,
}

impl InMemoryLevelHistoryRepo {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl LevelHistoryRepo for InMemoryLevelHistoryRepo {
    async fn save_level_change(&self, record: &LevelChangeRecord) -> Result<(), RepoError> {
        let mut storage = self.storage.lock().unwrap();
        let key = record.usr_id.clone();
        storage.entry(key).or_insert_with(Vec::new).push(record.clone());
        Ok(())
    }

    async fn get_user_level_history(
        &self,
        user_id: &UserId,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<LevelChangeRecord>, RepoError> {
        let storage = self.storage.lock().unwrap();
        let key = user_id.to_string();
        
        if let Some(records) = storage.get(&key) {
            let start = offset as usize;
            let end = std::cmp::min(start + limit as usize, records.len());
            
            if start < records.len() {
                Ok(records[start..end].to_vec())
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    async fn count_user_level_changes(&self, user_id: &UserId) -> Result<u64, RepoError> {
        let storage = self.storage.lock().unwrap();
        let key = user_id.to_string();
        
        Ok(storage.get(&key).map(|records| records.len() as u64).unwrap_or(0))
    }

    async fn get_recent_level_changes(&self, limit: u32) -> Result<Vec<LevelChangeRecord>, RepoError> {
        let storage = self.storage.lock().unwrap();
        let mut all_records: Vec<LevelChangeRecord> = storage
            .values()
            .flat_map(|records| records.iter().cloned())
            .collect();
        
        // Sort by timestamp (most recent first)
        all_records.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));
        
        // Take only the requested limit
        all_records.truncate(limit as usize);
        
        Ok(all_records)
    }
}