// In-memory cache implementation

use super::{
  Cache,
  CacheExt,
  CacheConfig,
  CacheStats,
  CacheError,
  CachedEntry,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory cache implementation
pub struct InMemoryCache {
  storage: Arc<RwLock<HashMap<String, String>>>,
  config: CacheConfig,
  stats: Arc<RwLock<CacheStatsInternal>>,
}

#[derive(Debug, Default)]
struct CacheStatsInternal {
  hit_count: u64,
  miss_count: u64,
  set_count: u64,
  delete_count: u64,
}

impl InMemoryCache {
  pub fn new(config: CacheConfig) -> Self {
    Self {
      storage: Arc::new(RwLock::new(HashMap::new())),
      config,
      stats: Arc::new(RwLock::new(CacheStatsInternal::default())),
    }
  }

  async fn serialize_value<T: Serialize>(
    &self,
    value: &T
  ) -> Result<String, CacheError> {
    serde_json
      ::to_string(value)
      .map_err(|e| CacheError::SerializationError(e.to_string()))
  }

  async fn cleanup_expired(&self) {
    let mut storage = self.storage.write().await;
    let _now = Utc::now();

    storage.retain(|_, value| {
      if
        let Ok(entry) =
          serde_json::from_str::<CachedEntry<serde_json::Value>>(value)
      {
        !entry.is_expired()
      } else {
        // Keep entries that can't be parsed as they might be direct values
        true
      }
    });
  }

  async fn check_capacity(&self) -> Result<(), CacheError> {
    if let Some(max_entries) = self.config.max_entries {
      let storage = self.storage.read().await;
      if storage.len() >= max_entries {
        drop(storage);
        self.cleanup_expired().await;

        let storage = self.storage.read().await;
        if storage.len() >= max_entries {
          return Err(CacheError::CacheFull);
        }
      }
    }
    Ok(())
  }
}

#[async_trait]
impl Cache for InMemoryCache {
  async fn get_raw(&self, key: &str) -> Result<Option<String>, CacheError> {
    let storage = self.storage.read().await;
    let mut stats = self.stats.write().await;

    if let Some(data) = storage.get(key) {
      // Check if it's an expired cached entry
      if
        let Ok(entry) =
          serde_json::from_str::<CachedEntry<serde_json::Value>>(data)
      {
        if !entry.is_expired() {
          stats.hit_count += 1;
          Ok(Some(data.clone()))
        } else {
          stats.miss_count += 1;
          Ok(None)
        }
      } else {
        // Direct value
        stats.hit_count += 1;
        Ok(Some(data.clone()))
      }
    } else {
      stats.miss_count += 1;
      Ok(None)
    }
  }

  async fn set_raw(
    &self,
    key: &str,
    value: &str,
    _ttl_seconds: Option<i64>
  ) -> Result<(), CacheError> {
    self.check_capacity().await?;

    let mut storage = self.storage.write().await;
    let mut stats = self.stats.write().await;

    storage.insert(key.to_string(), value.to_string());
    stats.set_count += 1;

    Ok(())
  }

  async fn delete(&self, key: &str) -> Result<bool, CacheError> {
    let mut storage = self.storage.write().await;
    let mut stats = self.stats.write().await;

    let existed = storage.remove(key).is_some();
    if existed {
      stats.delete_count += 1;
    }

    Ok(existed)
  }

  async fn exists(&self, key: &str) -> Result<bool, CacheError> {
    let storage = self.storage.read().await;

    if let Some(data) = storage.get(key) {
      // Check if it's an expired cached entry
      if
        let Ok(entry) =
          serde_json::from_str::<CachedEntry<serde_json::Value>>(data)
      {
        Ok(!entry.is_expired())
      } else {
        // Direct value, assume it exists
        Ok(true)
      }
    } else {
      Ok(false)
    }
  }

  async fn clear(&self) -> Result<(), CacheError> {
    let mut storage = self.storage.write().await;
    storage.clear();
    Ok(())
  }

  async fn stats(&self) -> Result<CacheStats, CacheError> {
    self.cleanup_expired().await;

    let storage = self.storage.read().await;
    let stats = self.stats.read().await;

    let total_entries = storage.len() as u64;
    let hit_rate = if stats.hit_count + stats.miss_count > 0 {
      Some(
        (stats.hit_count as f64) / ((stats.hit_count + stats.miss_count) as f64)
      )
    } else {
      None
    };

    // Estimate memory usage
    let memory_usage = storage
      .iter()
      .map(|(k, v)| k.len() + v.len())
      .sum::<usize>() as u64;

    Ok(CacheStats {
      total_entries,
      expired_entries: 0, // Cleaned up
      active_entries: total_entries,
      memory_usage_bytes: Some(memory_usage),
      hit_count: Some(stats.hit_count),
      miss_count: Some(stats.miss_count),
      hit_rate,
    })
  }

  async fn delete_many(&self, keys: &[String]) -> Result<u64, CacheError> {
    let mut deleted_count = 0;

    for key in keys {
      if self.delete(key).await? {
        deleted_count += 1;
      }
    }

    Ok(deleted_count)
  }

  async fn increment(
    &self,
    key: &str,
    delta: i64,
    ttl_seconds: Option<i64>
  ) -> Result<i64, CacheError> {
    let storage = self.storage.read().await;

    let current_value = if let Some(data) = storage.get(key) {
      // Try to get as cached entry of i64
      if let Ok(entry) = serde_json::from_str::<CachedEntry<i64>>(data) {
        if !entry.is_expired() { entry.value } else { 0 }
      } else {
        // Try direct deserialization
        serde_json::from_str::<i64>(data).unwrap_or(0)
      }
    } else {
      0
    };

    drop(storage);

    let new_value = current_value + delta;
    self.set(key, &new_value, ttl_seconds).await?;

    Ok(new_value)
  }

  async fn expire(
    &self,
    key: &str,
    ttl_seconds: i64
  ) -> Result<bool, CacheError> {
    let storage = self.storage.read().await;

    if let Some(data) = storage.get(key) {
      // Try to parse as cached entry and update expiration
      if
        let Ok(mut entry) =
          serde_json::from_str::<CachedEntry<serde_json::Value>>(data)
      {
        drop(storage);

        entry.expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds);
        let serialized = self.serialize_value(&entry).await?;

        let mut storage = self.storage.write().await;
        storage.insert(key.to_string(), serialized);
        Ok(true)
      } else {
        // Can't update expiration for direct values
        Ok(false)
      }
    } else {
      Ok(false)
    }
  }

  async fn set_add(
    &self,
    key: &str,
    value: &str,
    _ttl_seconds: Option<i64>
  ) -> Result<bool, CacheError> {
    // Simplified set implementation using JSON arrays
    let mut storage = self.storage.write().await;

    let mut set_values: Vec<String> = if let Some(existing) = storage.get(key) {
      serde_json::from_str(existing).unwrap_or_default()
    } else {
      Vec::new()
    };

    if !set_values.contains(&value.to_string()) {
      set_values.push(value.to_string());
      let serialized = serde_json
        ::to_string(&set_values)
        .map_err(|e| CacheError::SerializationError(e.to_string()))?;
      storage.insert(key.to_string(), serialized);
      Ok(true)
    } else {
      Ok(false)
    }
  }

  async fn set_card(&self, key: &str) -> Result<u64, CacheError> {
    let storage = self.storage.read().await;

    if let Some(existing) = storage.get(key) {
      let set_values: Vec<String> = serde_json
        ::from_str(existing)
        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
      Ok(set_values.len() as u64)
    } else {
      Ok(0)
    }
  }

  async fn list_push(
    &self,
    key: &str,
    value: &str,
    _ttl_seconds: Option<i64>
  ) -> Result<u64, CacheError> {
    let mut storage = self.storage.write().await;

    let mut list_values: Vec<String> = if let Some(existing) = storage.get(key) {
      serde_json::from_str(existing).unwrap_or_default()
    } else {
      Vec::new()
    };

    list_values.push(value.to_string());

    // Keep only last 100 items to prevent unlimited growth
    if list_values.len() > 100 {
      let skip_count = list_values.len() - 100;
      list_values = list_values.into_iter().skip(skip_count).collect();
    }

    let serialized = serde_json
      ::to_string(&list_values)
      .map_err(|e| CacheError::SerializationError(e.to_string()))?;
    storage.insert(key.to_string(), serialized);

    Ok(list_values.len() as u64)
  }

  async fn list_range(
    &self,
    key: &str,
    start: i64,
    stop: i64
  ) -> Result<Vec<String>, CacheError> {
    let storage = self.storage.read().await;

    if let Some(existing) = storage.get(key) {
      let list_values: Vec<String> = serde_json
        ::from_str(existing)
        .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

      let len = list_values.len() as i64;
      let actual_start = (if start < 0 {
        (len + start).max(0)
      } else {
        start.min(len)
      }) as usize;
      let actual_stop = (if stop < 0 {
        (len + stop + 1).max(0)
      } else {
        (stop + 1).min(len)
      }) as usize;

      let range = if actual_start < actual_stop {
        &list_values[actual_start..actual_stop]
      } else {
        &[]
      };

      Ok(range.to_vec())
    } else {
      Ok(Vec::new())
    }
  }
}
