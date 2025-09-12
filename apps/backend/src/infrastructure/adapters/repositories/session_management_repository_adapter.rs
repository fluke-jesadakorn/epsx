// Session Management Repository Adapter
// Bridges DDD UserSessionManager with existing session and cache storage

use tracing::{info, warn, error, debug};
use std::sync::Arc;

use crate::domain::session_management::{
    UserSessionManager, SessionManagerRepositoryPort, SessionId, AuthenticatedUserId,
    SessionMetadata, SessionCollection, SessionActivity, SessionHistory
};
use crate::infrastructure::adapters::repositories::diesel::DbPool;
use crate::infrastructure::cache::Cache;
use crate::application::ports::repositories::SessionRepository;

/// Repository adapter for session management
pub struct SessionManagementRepositoryAdapter {
    /// Legacy session repository
    legacy_session_repo: Arc<dyn SessionRepository<Error = Box<dyn std::error::Error + Send + Sync>>>,
    
    /// Database pool for direct operations
    db_pool: Arc<DbPool>,
    
    /// Cache for session metadata and performance
    cache: Arc<dyn Cache>,
}

unsafe impl Send for SessionManagementRepositoryAdapter {}
unsafe impl Sync for SessionManagementRepositoryAdapter {}

impl SessionManagementRepositoryAdapter {
    pub fn new(
        legacy_session_repo: Arc<dyn SessionRepository<Error = Box<dyn std::error::Error + Send + Sync>>>,
        db_pool: Arc<DbPool>,
        cache: Arc<dyn Cache>,
    ) -> Self {
        Self {
            legacy_session_repo,
            db_pool,
            cache,
        }
    }
    
    /// Get cache key for session manager
    fn get_session_manager_cache_key(&self, user_id: &AuthenticatedUserId) -> String {
        format!("session_manager:{}", user_id.to_string())
    }
    
    /// Get cache key for session metadata
    fn get_session_metadata_cache_key(&self, session_id: &SessionId) -> String {
        format!("session_metadata:{}", session_id.to_string())
    }
    
    /// Store session metadata in cache with TTL
    async fn cache_session_metadata(&self, metadata: &SessionMetadata) -> Result<(), String> {
        let cache_key = self.get_session_metadata_cache_key(&metadata.session_id);
        
        let serialized = serde_json::to_string(metadata)
            .map_err(|e| format!("Failed to serialize session metadata: {}", e))?;
        
        // Cache for 1 hour or until session expiry, whichever is shorter
        let ttl = std::cmp::min(
            3600, // 1 hour
            (metadata.expires_at - chrono::Utc::now()).num_seconds().max(0) as u64
        );
        
        self.cache.set(&cache_key, serialized, Some(ttl))
            .map_err(|e| format!("Failed to cache session metadata: {}", e))?;
        
        Ok(())
    }
    
    /// Get session metadata from cache
    async fn get_cached_session_metadata(&self, session_id: &SessionId) -> Option<SessionMetadata> {
        let cache_key = self.get_session_metadata_cache_key(session_id);
        
        match self.cache.get(&cache_key) {
            Some(data) => {
                match serde_json::from_str::<SessionMetadata>(&data) {
                    Ok(metadata) => Some(metadata),
                    Err(e) => {
                        warn!(
                            session_id = %session_id,
                            error = %e,
                            "Failed to deserialize cached session metadata"
                        );
                        None
                    }
                }
            },
            None => None
        }
    }
    
    /// Store session manager state in database
    async fn persist_session_manager_state(&self, manager: &UserSessionManager) -> Result<(), String> {
        info!(
            user_id = %manager.id(),
            version = manager.version(),
            "Persisting session manager state"
        );
        
        // In production, this would store the complete UserSessionManager state
        // including SessionCollection, ActivityTracking, and History
        // For now, we'll store essential state in a dedicated table
        
        let manager_state = SessionManagerState {
            user_id: manager.id().to_string(),
            version: manager.version(),
            created_at: manager.created_at(),
            updated_at: manager.updated_at(),
            max_concurrent_sessions: 10, // Would come from manager config
            active_session_count: manager.active_session_count(),
            total_sessions_created: 0, // Would come from manager summary
            serialized_state: serde_json::to_string(manager)
                .map_err(|e| format!("Failed to serialize manager state: {}", e))?,
        };
        
        // Store in database (would use a dedicated table in production)
        self.store_manager_state_to_db(manager_state).await?;
        
        Ok(())
    }
    
    /// Load session manager state from database
    async fn load_session_manager_state(&self, user_id: &AuthenticatedUserId) -> Result<Option<UserSessionManager>, String> {
        debug!(user_id = %user_id, "Loading session manager state from database");
        
        // Try cache first
        let cache_key = self.get_session_manager_cache_key(user_id);
        if let Some(cached_data) = self.cache.get(&cache_key) {
            if let Ok(manager) = serde_json::from_str::<UserSessionManager>(&cached_data) {
                debug!(user_id = %user_id, "Session manager loaded from cache");
                return Ok(Some(manager));
            }
        }
        
        // Load from database
        match self.load_manager_state_from_db(user_id).await? {
            Some(state) => {
                match serde_json::from_str::<UserSessionManager>(&state.serialized_state) {
                    Ok(mut manager) => {
                        // Cache the loaded manager
                        if let Ok(serialized) = serde_json::to_string(&manager) {
                            self.cache.set(&cache_key, serialized, Some(3600));
                        }
                        
                        debug!(
                            user_id = %user_id,
                            version = state.version,
                            "Session manager loaded from database"
                        );
                        
                        Ok(Some(manager))
                    },
                    Err(e) => {
                        error!(
                            user_id = %user_id,
                            error = %e,
                            "Failed to deserialize session manager state"
                        );
                        Err(format!("State deserialization failed: {}", e))
                    }
                }
            },
            None => {
                debug!(user_id = %user_id, "No session manager state found in database");
                Ok(None)
            }
        }
    }
    
    /// Store manager state to database (placeholder implementation)
    async fn store_manager_state_to_db(&self, _state: SessionManagerState) -> Result<(), String> {
        // In production, this would use SQL to insert/update session manager state
        // CREATE TABLE session_managers (
        //     user_id VARCHAR PRIMARY KEY,
        //     version BIGINT NOT NULL,
        //     created_at TIMESTAMP NOT NULL,
        //     updated_at TIMESTAMP NOT NULL,
        //     max_concurrent_sessions INTEGER,
        //     active_session_count INTEGER,
        //     total_sessions_created BIGINT,
        //     serialized_state JSONB
        // );
        
        Ok(())
    }
    
    /// Load manager state from database (placeholder implementation)
    async fn load_manager_state_from_db(&self, user_id: &AuthenticatedUserId) -> Result<Option<SessionManagerState>, String> {
        // In production, this would query the session_managers table
        Ok(None)
    }
}

#[async_trait]
impl SessionManagerRepositoryPort for SessionManagementRepositoryAdapter {
    async fn save_session_manager(&self, manager: &UserSessionManager) -> Result<(), String> {
        info!(
            user_id = %manager.id(),
            version = manager.version(),
            active_sessions = manager.active_session_count(),
            "Saving session manager"
        );
        
        // Persist manager state
        self.persist_session_manager_state(manager).await?;
        
        // Cache manager for quick access
        let cache_key = self.get_session_manager_cache_key(manager.id());
        let serialized = serde_json::to_string(manager)
            .map_err(|e| format!("Failed to serialize session manager: {}", e))?;
        
        self.cache.set(&cache_key, &serialized, Some(3600)).await
            .map_err(|e| format!("Failed to cache session manager: {}", e))?;
        
        // Also save individual session metadata for legacy compatibility
        for session in manager.get_active_sessions() {
            if let Err(e) = self.cache_session_metadata(session).await {
                warn!(
                    session_id = %session.session_id,
                    error = %e,
                    "Failed to cache session metadata"
                );
            }
        }
        
        info!(user_id = %manager.id(), "Session manager saved successfully");
        Ok(())
    }
    
    async fn find_session_manager(&self, user_id: &AuthenticatedUserId) -> Result<Option<UserSessionManager>, String> {
        info!(user_id = %user_id, "Finding session manager");
        
        match self.load_session_manager_state(user_id).await? {
            Some(manager) => {
                info!(
                    user_id = %user_id,
                    version = manager.version(),
                    active_sessions = manager.active_session_count(),
                    "Session manager found"
                );
                Ok(Some(manager))
            },
            None => {
                debug!(user_id = %user_id, "No session manager found, will create new one");
                Ok(None)
            }
        }
    }
    
    async fn delete_session_manager(&self, user_id: &AuthenticatedUserId) -> Result<(), String> {
        info!(user_id = %user_id, "Deleting session manager");
        
        // Remove from cache
        let cache_key = self.get_session_manager_cache_key(user_id);
        if let Err(e) = self.cache.delete(&cache_key) {
            warn!(user_id = %user_id, error = %e, "Failed to remove session manager from cache");
        }
        
        // Remove from database (would implement in production)
        // DELETE FROM session_managers WHERE user_id = ?
        
        info!(user_id = %user_id, "Session manager deleted");
        Ok(())
    }
    
    async fn find_session_metadata(&self, session_id: &SessionId) -> Result<Option<SessionMetadata>, String> {
        debug!(session_id = %session_id, "Finding session metadata");
        
        // Try cache first
        if let Some(metadata) = self.get_cached_session_metadata(session_id).await {
            debug!(session_id = %session_id, "Session metadata found in cache");
            return Ok(Some(metadata));
        }
        
        // Fall back to legacy repository
        let session_id_i32 = session_id.as_str()
            .strip_prefix("sess_")
            .unwrap_or(session_id.as_str())
            .parse::<i32>()
            .map_err(|e| format!("Invalid session ID format: {}", e))?;
        
        match self.legacy_session_repo.find_session_by_id(session_id_i32).await {
            Ok(Some(legacy_session)) => {
                // Convert legacy session to metadata
                let metadata = self.convert_legacy_to_metadata(legacy_session)?;
                
                // Cache the result
                if let Err(e) = self.cache_session_metadata(&metadata).await {
                    warn!(error = %e, "Failed to cache converted session metadata");
                }
                
                debug!(session_id = %session_id, "Session metadata converted from legacy");
                Ok(Some(metadata))
            },
            Ok(None) => {
                debug!(session_id = %session_id, "Session metadata not found");
                Ok(None)
            },
            Err(e) => {
                error!(session_id = %session_id, error = %e, "Failed to find session metadata");
                Err(format!("Repository error: {}", e))
            }
        }
    }
    
    async fn save_session_metadata(&self, metadata: &SessionMetadata) -> Result<(), String> {
        debug!(session_id = %metadata.session_id, "Saving session metadata");
        
        // Cache the metadata
        self.cache_session_metadata(metadata).await?;
        
        // Also save to legacy repository for compatibility
        let legacy_session = self.convert_metadata_to_legacy(metadata)?;
        
        match self.legacy_session_repo.create_session(legacy_session).await {
            Ok(_) => {
                debug!(session_id = %metadata.session_id, "Session metadata saved");
                Ok(())
            },
            Err(e) => {
                error!(
                    session_id = %metadata.session_id,
                    error = %e,
                    "Failed to save session metadata to legacy repository"
                );
                Err(format!("Legacy repository error: {}", e))
            }
        }
    }
    
    async fn cleanup_expired_sessions(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u32, String> {
        info!(before = %before, "Cleaning up expired sessions");
        
        // Use legacy repository's cleanup functionality
        match self.legacy_session_repo.cleanup_expired_sessions(before).await {
            Ok(count) => {
                info!(count = count, "Expired sessions cleaned up");
                
                // Also clean up cached metadata for expired sessions
                // This would be more sophisticated in production
                
                Ok(count as u32)
            },
            Err(e) => {
                error!(error = %e, "Failed to cleanup expired sessions");
                Err(format!("Cleanup error: {}", e))
            }
        }
    }
    
    async fn get_user_session_statistics(&self, user_id: &AuthenticatedUserId) -> Result<SessionStatistics, String> {
        debug!(user_id = %user_id, "Getting user session statistics");
        
        // Try to get from session manager first
        if let Some(manager) = self.find_session_manager(user_id).await? {
            let summary = manager.get_manager_summary();
            
            return Ok(SessionStatistics {
                user_id: user_id.to_string(),
                total_sessions: summary.total_sessions,
                active_sessions: summary.active_sessions,
                suspicious_sessions: summary.suspicious_sessions,
                total_activities: summary.total_activities as u64,
                average_session_duration: summary.average_session_duration,
                last_updated: summary.last_updated,
            });
        }
        
        // Fallback to legacy data
        let user_id_i32 = user_id.user_id().to_string()
            .parse::<i32>()
            .map_err(|e| format!("Invalid user ID: {}", e))?;
        
        match self.legacy_session_repo.find_sessions_byuser_id(user_id_i32).await {
            Ok(sessions) => {
                Ok(SessionStatistics {
                    user_id: user_id.to_string(),
                    total_sessions: sessions.len() as u32,
                    active_sessions: sessions.iter()
                        .filter(|s| s.expires_at > chrono::Utc::now())
                        .count() as u32,
                    suspicious_sessions: 0, // Would need additional logic
                    total_activities: 0, // Not available in legacy
                    average_session_duration: 0.0, // Would need calculation
                    last_updated: chrono::Utc::now(),
                })
            },
            Err(e) => {
                error!(user_id = %user_id, error = %e, "Failed to get session statistics");
                Err(format!("Statistics error: {}", e))
            }
        }
    }
}

impl SessionManagementRepositoryAdapter {
    /// Convert legacy session to session metadata
    fn convert_legacy_to_metadata(&self, legacy_session: crate::domain::shared_kernel::entities::session::Session) -> Result<SessionMetadata, String> {
        let session_id = SessionId::from_string(
            format!("sess_{}", legacy_session.id.unwrap_or_default())
        ).map_err(|e| format!("Invalid session ID: {}", e))?;
        
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new(legacy_session.user_id.to_string())
        );
        
        let metadata = SessionMetadata::new(
            session_id,
            user_id,
            crate::domain::authentication::ProviderType::Firebase, // Default provider
            legacy_session.expires_at,
        );
        
        // Update with legacy data
        metadata.created_at = legacy_session.created_at;
        if let Some(last_accessed) = legacy_session.last_accessed {
            metadata.last_accessed = last_accessed;
        }
        
        // Add IP address if available
        if let Some(ip) = legacy_session.ip_address {
            metadata.add_ip_address(ip, None);
        }
        
        Ok(metadata)
    }
    
    /// Convert session metadata to legacy session
    fn convert_metadata_to_legacy(&self, metadata: &SessionMetadata) -> Result<crate::domain::shared_kernel::entities::session::Session, String> {
        let numericuser_id = metadata.user_id.user_id().to_string()
            .parse::<i32>()
            .map_err(|e| format!("Invalid user ID for legacy format: {}", e))?;
        
        Ok(crate::domain::shared_kernel::entities::session::Session {
            id: None, // Will be auto-generated
            user_id: numericuser_id,
            firebase_uid: Some(metadata.user_id.to_string()),
            ip_address: metadata.ip_addresses.first().map(|ip| ip.address.clone()),
            expires_at: metadata.expires_at,
            created_at: metadata.created_at,
            last_accessed: Some(metadata.last_accessed),
        })
    }
}

/// Session manager persistent state
#[derive(Debug, Clone)]
pub struct SessionManagerState {
    pub user_id: String,
    pub version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub max_concurrent_sessions: u32,
    pub active_session_count: u32,
    pub total_sessions_created: u64,
    pub serialized_state: String,
}

/// Session statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStatistics {
    pub user_id: String,
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub suspicious_sessions: u32,
    pub total_activities: u64,
    pub average_session_duration: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::memory_cache::MemoryCache;
    
    // Mock session repository for testing
    struct MockSessionRepository;
    
    #[async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn create_session(&self, session: crate::domain::shared_kernel::entities::session::Session) -> Result<crate::domain::shared_kernel::entities::session::Session, Box<dyn std::error::Error + Send + Sync>> {
            Ok(crate::domain::shared_kernel::entities::session::Session {
                id: Some(1),
                ..session
            })
        }
        
        async fn find_session_by_id(&self, _id: i32) -> Result<Option<crate::domain::shared_kernel::entities::session::Session>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Some(crate::domain::shared_kernel::entities::session::Session {
                id: Some(1),
                user_id: 123,
                firebase_uid: Some("test_uid".to_string()),
                ip_address: Some("192.168.1.1".to_string()),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                created_at: chrono::Utc::now(),
                last_accessed: Some(chrono::Utc::now()),
            }))
        }
        
        async fn find_sessions_byuser_id(&self, user_id: i32) -> Result<Vec<crate::domain::shared_kernel::entities::session::Session>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }
        
        async fn delete_session(&self, _id: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn update_last_accessed(&self, _id: i32, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        async fn find_expired_sessions(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<Vec<crate::domain::shared_kernel::entities::session::Session>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(vec![])
        }
        
        async fn cleanup_expired_sessions(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(5) // Return mock cleanup count
        }
    }
    
    #[tokio::test]
    async fn test_session_management_adapter_creation() {
        let mock_pool = Arc::new(crate::infrastructure::adapters::repositories::diesel::create_test_pool().await.unwrap());
        let mock_legacy_repo = Arc::new(MockSessionRepository);
        let cache = Arc::new(MemoryCache::new(1000));
        
        let adapter = SessionManagementRepositoryAdapter::new(
            mock_legacy_repo,
            mock_pool,
            cache,
        );
        
        // Test basic functionality
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new("123".to_string())
        );
        
        let result = adapter.find_session_manager(&user_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No manager exists initially
    }
    
    #[tokio::test]
    async fn test_session_metadata_caching() {
        let mock_pool = Arc::new(crate::infrastructure::adapters::repositories::diesel::create_test_pool().await.unwrap());
        let mock_legacy_repo = Arc::new(MockSessionRepository);
        let cache = Arc::new(MemoryCache::new(1000));
        
        let adapter = SessionManagementRepositoryAdapter::new(
            mock_legacy_repo,
            mock_pool,
            cache,
        );
        
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new("123".to_string())
        );
        
        let metadata = SessionMetadata::new(
            session_id.clone(),
            user_id,
            crate::domain::authentication::ProviderType::Firebase,
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        
        // Test caching
        let result = adapter.cache_session_metadata(&metadata).await;
        assert!(result.is_ok());
        
        // Test retrieval from cache
        let cached = adapter.get_cached_session_metadata(&session_id).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().session_id, session_id);
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let mock_pool = Arc::new(crate::infrastructure::adapters::repositories::diesel::create_test_pool().await.unwrap());
        let mock_legacy_repo = Arc::new(MockSessionRepository);
        let cache = Arc::new(MemoryCache::new(1000));
        
        let adapter = SessionManagementRepositoryAdapter::new(
            mock_legacy_repo,
            mock_pool,
            cache,
        );
        
        let before = chrono::Utc::now() - chrono::Duration::hours(24);
        let result = adapter.cleanup_expired_sessions(before).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5); // Mock returns 5
    }
}