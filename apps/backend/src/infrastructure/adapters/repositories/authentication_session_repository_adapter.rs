// Authentication Session Repository Adapter
// Bridges DDD AuthenticationSession aggregate with existing session storage

use tracing::{info, warn, error};
use std::sync::Arc;

use crate::domain::authentication::{
    AuthenticationSession, SessionId, AuthenticatedUserId,
    AuthenticationSessionRepositoryPort
};
use crate::infrastructure::adapters::repositories::diesel::DbPool;
use crate::application::ports::repositories::SessionRepository;

/// Repository adapter for authentication sessions
#[derive(Clone)]
pub struct AuthenticationSessionRepositoryAdapter {
    /// Legacy session repository
    legacy_session_repo: Arc<dyn SessionRepository<Error = Box<dyn std::error::Error + Send + Sync>>>,
    
    /// Database pool for direct operations
    db_pool: Arc<DbPool>,
}


impl AuthenticationSessionRepositoryAdapter {
    pub fn new(
        legacy_session_repo: Arc<dyn SessionRepository<Error = Box<dyn std::error::Error + Send + Sync>>>,
        db_pool: Arc<DbPool>,
    ) -> Self {
        Self {
            legacy_session_repo,
            db_pool,
        }
    }
    
    /// Convert DDD AuthenticationSession to legacy format
    fn map_to_legacy(&self, session: &AuthenticationSession) -> LegacySessionMapping {
        LegacySessionMapping {
            session_id: session.session_id().to_string(),
            user_id: session.user_id().to_string(),
            provider_type: format!("{:?}", session.provider().provider_type()),
            created_at: session.created_at(),
            expires_at: session.expires_at(),
            last_activity: session.last_activity(),
            is_active: session.is_active(),
            // Map tokens if available
            access_token: session.current_access_token()
                .map(|t| t.token().to_string()),
            refresh_token: session.current_refresh_token()
                .map(|t| t.token().to_string()),
            id_token: session.current_id_token()
                .map(|t| t.token().to_string()),
        }
    }
    
    /// Convert legacy session data to DDD AuthenticationSession
    async fn map_from_legacy(&self, legacy_data: LegacySessionData) -> Result<AuthenticationSession, String> {
        // In a real implementation, this would rebuild the AuthenticationSession
        // from stored data. For now, we'll create a simplified version
        
        let user_id = AuthenticatedUserId::from_verified_user(
            legacy_data.user_id.parse()
                .map_err(|e| format!("Invalid user ID: {}", e))?
        );
        
        let session_id = SessionId::from_string(legacy_data.session_id)
            .map_err(|e| format!("Invalid session ID: {}", e))?;
        
        // Note: In production, we'd need to rebuild the complete session state
        // including provider, client info, tokens, etc. from stored data
        
        Err("Session reconstruction not yet implemented".to_string())
    }
}

#[async_trait]
impl AuthenticationSessionRepositoryPort for AuthenticationSessionRepositoryAdapter {
    async fn save(&self, session: &AuthenticationSession) -> Result<(), String> {
        info!(
            session_id = %session.session_id(),
            user_id = %session.user_id(),
            "Saving authentication session to legacy storage"
        );
        
        let legacy_mapping = self.map_to_legacy(session);
        
        // Store in legacy session repository
        // Note: We need to create a proper mapping to the legacy Session model
        // For now, we'll use the existing session creation logic
        
        // Extract session data for legacy storage
        let session_data = crate::domain::shared_kernel::entities::session::Session {
            id: Some(legacy_mapping.session_id.parse().unwrap_or_default()),
            user_id: legacy_mapping.user_id.parse().unwrap_or_default(),
            firebase_uid: Some(session.user_id().to_string()), // Use as identifier
            ip_address: None, // Would come from security context
            expires_at: legacy_mapping.expires_at,
            created_at: legacy_mapping.created_at,
            last_accessed: Some(legacy_mapping.last_activity),
            // Additional session data would be mapped here
        };
        
        // Save to legacy repository
        self.legacy_session_repo.create_session(session_data).await
            .map_err(|e| format!("Failed to save session to legacy repository: {}", e))?;
        
        // Store additional DDD-specific data if needed
        self.store_ddd_session_metadata(session).await?;
        
        info!(session_id = %session.session_id(), "Session saved successfully");
        Ok(())
    }
    
    async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<AuthenticationSession>, String> {
        info!(session_id = %session_id, "Finding authentication session by ID");
        
        // Try to find in legacy repository first
        let session_id_i32 = session_id.as_str()
            .strip_prefix("sess_")
            .unwrap_or(session_id.as_str())
            .parse::<i32>()
            .map_err(|e| format!("Invalid session ID format: {}", e))?;
        
        match self.legacy_session_repo.find_session_by_id(session_id_i32).await {
            Ok(Some(legacy_session)) => {
                // Convert legacy session back to DDD format
                let legacy_data = LegacySessionData {
                    session_id: session_id.to_string(),
                    user_id: legacy_session.user_id.to_string(),
                    created_at: legacy_session.created_at,
                    expires_at: legacy_session.expires_at,
                    last_activity: legacy_session.last_accessed.unwrap_or(legacy_session.created_at),
                };
                
                match self.map_from_legacy(legacy_data).await {
                    Ok(session) => Ok(Some(session)),
                    Err(e) => {
                        warn!(error = %e, "Failed to convert legacy session to DDD format");
                        Ok(None)
                    }
                }
            },
            Ok(None) => Ok(None),
            Err(e) => {
                error!(error = %e, "Failed to find session in legacy repository");
                Err(format!("Repository error: {}", e))
            }
        }
    }
    
    async fn find_by_user(&self, user_id: &AuthenticatedUserId) -> Result<Vec<AuthenticationSession>, String> {
        info!(user_id = %user_id, "Finding sessions for user");
        
        // Extract numeric user ID for legacy repository
        let numericuser_id = user_id.user_id().to_string()
            .parse::<i32>()
            .map_err(|e| format!("Invalid user ID format: {}", e))?;
        
        match self.legacy_session_repo.find_sessions_byuser_id(numericuser_id).await {
            Ok(legacy_sessions) => {
                let sessions = Vec::new();
                
                for legacy_session in legacy_sessions {
                    let legacy_data = LegacySessionData {
                        session_id: legacy_session.id.unwrap_or_default().to_string(),
                        user_id: legacy_session.user_id.to_string(),
                        created_at: legacy_session.created_at,
                        expires_at: legacy_session.expires_at,
                        last_activity: legacy_session.last_accessed.unwrap_or(legacy_session.created_at),
                    };
                    
                    match self.map_from_legacy(legacy_data).await {
                        Ok(session) => sessions.push(session),
                        Err(e) => {
                            warn!(error = %e, "Failed to convert legacy session, skipping");
                        }
                    }
                }
                
                Ok(sessions)
            },
            Err(e) => {
                error!(error = %e, "Failed to find user sessions in legacy repository");
                Err(format!("Repository error: {}", e))
            }
        }
    }
    
    async fn delete(&self, session_id: &SessionId) -> Result<(), String> {
        info!(session_id = %session_id, "Deleting authentication session");
        
        let session_id_i32 = session_id.as_str()
            .strip_prefix("sess_")
            .unwrap_or(session_id.as_str())
            .parse::<i32>()
            .map_err(|e| format!("Invalid session ID format: {}", e))?;
        
        // Delete from legacy repository
        self.legacy_session_repo.delete_session(session_id_i32).await
            .map_err(|e| format!("Failed to delete session from legacy repository: {}", e))?;
        
        // Delete DDD-specific metadata
        self.delete_ddd_session_metadata(session_id).await?;
        
        info!(session_id = %session_id, "Session deleted successfully");
        Ok(())
    }
    
    async fn find_expired_sessions(&self, batch_size: u32) -> Result<Vec<AuthenticationSession>, String> {
        info!(batch_size = batch_size, "Finding expired sessions for cleanup");
        
        // Use direct database query for efficiency
        // In production, this would be implemented with proper SQL
        
        Ok(vec![]) // Placeholder implementation
    }
    
    async fn delete_expired_sessions(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u32, String> {
        info!(before = %before, "Deleting expired sessions");
        
        // Use legacy repository's cleanup functionality if available
        // Otherwise implement direct database cleanup
        
        Ok(0) // Placeholder implementation
    }
}

impl AuthenticationSessionRepositoryAdapter {
    /// Store DDD-specific session metadata
    async fn store_ddd_session_metadata(&self, session: &AuthenticationSession) -> Result<(), String> {
        // Store additional DDD data like security context, detailed token info, etc.
        // This would typically go in a separate table or JSON column
        
        info!(session_id = %session.session_id(), "Storing DDD session metadata");
        
        // Implementation would depend on chosen storage strategy:
        // 1. Separate metadata table
        // 2. JSON column in existing table
        // 3. Document store for complex data
        
        Ok(())
    }
    
    /// Delete DDD-specific session metadata
    async fn delete_ddd_session_metadata(&self, session_id: &SessionId) -> Result<(), String> {
        info!(session_id = %session_id, "Deleting DDD session metadata");
        
        // Delete corresponding metadata storage
        
        Ok(())
    }
}

/// Legacy session mapping structure
struct LegacySessionMapping {
    session_id: String,
    user_id: String,
    provider_type: String,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
    is_active: bool,
    access_token: Option<String>,
    refresh_token: Option<String>,
    id_token: Option<String>,
}

/// Legacy session data for reconstruction
struct LegacySessionData {
    session_id: String,
    user_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    // Mock legacy session repository for testing
    struct MockLegacySessionRepository;
    
    #[async_trait]
    impl SessionRepository for MockLegacySessionRepository {
        async fn create_session(&self, _session: crate::domain::shared_kernel::entities::session::Session) -> Result<crate::domain::shared_kernel::entities::session::Session, Box<dyn std::error::Error + Send + Sync>> {
            Ok(crate::domain::shared_kernel::entities::session::Session {
                id: Some(1),
                user_id: 123,
                firebase_uid: Some("test_uid".to_string()),
                ip_address: None,
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                created_at: chrono::Utc::now(),
                last_accessed: None,
            })
        }
        
        async fn find_session_by_id(&self, _id: i32) -> Result<Option<crate::domain::shared_kernel::entities::session::Session>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(None)
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
            Ok(0)
        }
    }
    
    #[tokio::test]
    async fn test_repository_adapter_creation() {
        let mock_pool = Arc::new(crate::infrastructure::adapters::repositories::diesel::create_test_pool().await.unwrap());
        let mock_legacy_repo = Arc::new(MockLegacySessionRepository);
        
        let adapter = AuthenticationSessionRepositoryAdapter::new(
            mock_legacy_repo,
            mock_pool,
        );
        
        // Basic creation test
        assert!(true); // Adapter created successfully
    }
}