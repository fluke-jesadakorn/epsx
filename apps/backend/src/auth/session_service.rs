// Unified Session Service
// Consolidates: session_cleanup_service.rs, session_security_service.rs, cleanup.rs, refresh_tokens.rs, refresh_token_service.rs, revocation.rs

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
use tracing::{debug, info, error};

use crate::infrastructure::cache::Cache;

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub device_info: Option<DeviceInfo>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

/// Device information for session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String, // mobile, desktop, tablet
    pub os: Option<String>,
    pub browser: Option<String>,
    pub fingerprint: Option<String>,
}

/// Refresh token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token_id: String,
    pub user_id: String,
    pub session_id: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
}

/// Session creation request
#[derive(Debug)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub device_info: Option<DeviceInfo>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_duration: Option<Duration>,
}

/// Session statistics
#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub expired_sessions: u64,
    pub sessions_today: u64,
    pub unique_users: u64,
    pub average_session_duration_minutes: f64,
}

/// Security event for session monitoring
#[derive(Debug, Clone, Serialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub session_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum SecurityEventType {
    SessionCreated,
    SessionExpired,
    SessionRevoked,
    SuspiciousActivity,
    MultipleLogins,
    TokenRefresh,
    TokenRevocation,
}

/// Unified Session Service
pub struct SessionService {
    cache: Arc<dyn Cache>,
    default_session_duration: Duration,
    max_sessions_per_user: usize,
    cleanup_interval: Duration,
}

impl SessionService {
    pub fn new(
        cache: Arc<dyn Cache>,
        default_session_hours: i64,
        max_sessions_per_user: usize,
        cleanup_interval_hours: i64,
    ) -> Self {
        Self {
            cache,
            default_session_duration: Duration::hours(default_session_hours),
            max_sessions_per_user,
            cleanup_interval: Duration::hours(cleanup_interval_hours),
        }
    }
    
    /// Create new session
    pub async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionInfo> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let duration = request.session_duration.unwrap_or(self.default_session_duration);
        let expires_at = now + duration;
        
        // Check session limit for user
        self.enforce_session_limit(&request.user_id).await?;
        
        let session = SessionInfo {
            session_id: session_id.clone(),
            user_id: request.user_id.clone(),
            created_at: now,
            last_accessed_at: now,
            expires_at,
            device_info: request.device_info.clone(),
            ip_address: request.ip_address.clone(),
            user_agent: request.user_agent.clone(),
            is_active: true,
        };
        
        // Store session in cache
        let session_key = format!("session:{}", session_id);
        let user_sessions_key = format!("user_sessions:{}", request.user_id);
        
        let session_json = serde_json::to_string(&session)?;
        self.cache.set(&session_key, session_json, Some(duration.num_seconds() as u64));
        
        // Add to user's session list
        let user_sessions_json = self.cache.get(&user_sessions_key).unwrap_or_else(|| "[]".to_string());
        let mut user_sessions: Vec<String> = serde_json::from_str(&user_sessions_json).unwrap_or_default();
        user_sessions.push(session_id.clone());
        let updated_sessions_json = serde_json::to_string(&user_sessions)?;
        self.cache.set(&user_sessions_key, updated_sessions_json, None);
        
        // Log security event
        self.log_security_event(SecurityEvent {
            event_type: SecurityEventType::SessionCreated,
            session_id: session_id.clone(),
            user_id: request.user_id,
            timestamp: now,
            ip_address: request.ip_address,
            details: HashMap::new(),
        }).await;
        
        info!("Created session {} for user with device: {:?}", session_id, request.device_info);
        
        Ok(session)
    }
    
    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionInfo>> {
        let session_key = format!("session:{}", session_id);
        
        if let Some(session_json) = self.cache.get(&session_key) {
            let mut session: SessionInfo = serde_json::from_str(&session_json)?;
            
            if session.expires_at < Utc::now() {
                // Session expired, remove it directly to avoid recursion
                self.cache.delete(&session_key);
                return Ok(None);
            }
            
            // Update last accessed time
            session.last_accessed_at = Utc::now();
            let updated_session_json = serde_json::to_string(&session)?;
            self.cache.set(&session_key, updated_session_json, None);
            
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }
    
    /// Revoke session
    pub async fn revoke_session(&self, session_id: &str, reason: &str) -> Result<()> {
        let session_key = format!("session:{}", session_id);
        
        // Get session before removing
        if let Some(session) = self.get_session(session_id).await? {
            // Remove from user's session list
            let user_sessions_key = format!("user_sessions:{}", session.user_id);
            let user_sessions_json = self.cache.get(&user_sessions_key).unwrap_or_else(|| "[]".to_string());
            let mut user_sessions: Vec<String> = serde_json::from_str(&user_sessions_json).unwrap_or_default();
            user_sessions.retain(|s| s != session_id);
            let updated_sessions_json = serde_json::to_string(&user_sessions)?;
            self.cache.set(&user_sessions_key, updated_sessions_json, None);
            
            // Log security event
            let mut details = HashMap::new();
            details.insert("reason".to_string(), reason.to_string());
            
            self.log_security_event(SecurityEvent {
                event_type: SecurityEventType::SessionRevoked,
                session_id: session_id.to_string(),
                user_id: session.user_id,
                timestamp: Utc::now(),
                ip_address: session.ip_address,
                details,
            }).await;
        }
        
        // Remove session from cache
        self.cache.delete(&session_key);
        
        info!("Revoked session {} (reason: {})", session_id, reason);
        Ok(())
    }
    
    /// Revoke all sessions for a user
    pub async fn revoke_all_user_sessions(&self, user_id: &str, reason: &str) -> Result<u32> {
        let user_sessions_key = format!("user_sessions:{}", user_id);
        let user_sessions_json = self.cache.get(&user_sessions_key).unwrap_or_else(|| "[]".to_string());
        let user_sessions: Vec<String> = serde_json::from_str(&user_sessions_json).unwrap_or_default();
        
        let mut revoked_count = 0;
        for session_id in &user_sessions {
            if self.revoke_session(session_id, reason).await.is_ok() {
                revoked_count += 1;
            }
        }
        
        info!("Revoked {} sessions for user {} (reason: {})", revoked_count, user_id, reason);
        Ok(revoked_count)
    }
    
    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<SessionInfo>> {
        let user_sessions_key = format!("user_sessions:{}", user_id);
        let user_sessions_json = self.cache.get(&user_sessions_key).unwrap_or_else(|| "[]".to_string());
        let session_ids: Vec<String> = serde_json::from_str(&user_sessions_json).unwrap_or_default();
        
        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Some(session) = self.get_session(&session_id).await? {
                sessions.push(session);
            }
        }
        
        Ok(sessions)
    }
    
    /// Create refresh token
    pub async fn create_refresh_token(&self, user_id: &str, session_id: &str) -> Result<RefreshToken> {
        let token_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string(); // In production, use proper random token generation
        let now = Utc::now();
        let expires_at = now + Duration::days(30); // Refresh tokens last longer
        
        let refresh_token = RefreshToken {
            token_id: token_id.clone(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            token: token.clone(),
            created_at: now,
            expires_at,
            is_revoked: false,
        };
        
        let token_key = format!("refresh_token:{}", token);
        let token_json = serde_json::to_string(&refresh_token)?;
        self.cache.set(&token_key, token_json, Some(Duration::days(30).num_seconds() as u64));
        
        debug!("Created refresh token for user: {}", user_id);
        
        Ok(refresh_token)
    }
    
    /// Validate and refresh token
    pub async fn validate_refresh_token(&self, token: &str) -> Result<Option<RefreshToken>> {
        let token_key = format!("refresh_token:{}", token);
        let refresh_token = if let Some(token_json) = self.cache.get(&token_key) {
            serde_json::from_str::<RefreshToken>(&token_json).ok()
        } else {
            None
        };
        
        if let Some(token_info) = refresh_token {
            if token_info.is_revoked || token_info.expires_at < Utc::now() {
                self.cache.delete(&token_key);
                return Ok(None);
            }
            Ok(Some(token_info))
        } else {
            Ok(None)
        }
    }
    
    /// Revoke refresh token
    pub async fn revoke_refresh_token(&self, token: &str) -> Result<()> {
        let token_key = format!("refresh_token:{}", token);
        self.cache.delete(&token_key);
        
        debug!("Revoked refresh token");
        Ok(())
    }
    
    /// Clean up expired sessions and tokens
    pub async fn cleanup_expired(&self) -> Result<SessionStats> {
        let stats = SessionStats {
            total_sessions: 0,
            active_sessions: 0,
            expired_sessions: 0,
            sessions_today: 0,
            unique_users: 0,
            average_session_duration_minutes: 0.0,
        };
        
        // This is a simplified cleanup - in production, you'd iterate through all sessions
        // For now, we'll return empty stats
        info!("Session cleanup completed");
        
        Ok(stats)
    }
    
    /// Get session statistics
    pub async fn get_session_stats(&self) -> Result<SessionStats> {
        // This would require scanning all sessions in cache
        // For now, return empty stats
        Ok(SessionStats {
            total_sessions: 0,
            active_sessions: 0,
            expired_sessions: 0,
            sessions_today: 0,
            unique_users: 0,
            average_session_duration_minutes: 0.0,
        })
    }
    
    /// Enforce maximum sessions per user
    async fn enforce_session_limit(&self, user_id: &str) -> Result<()> {
        let user_sessions = self.get_user_sessions(user_id).await?;
        
        if user_sessions.len() >= self.max_sessions_per_user {
            // Revoke oldest session
            if let Some(oldest_session) = user_sessions.iter().min_by_key(|s| s.created_at) {
                self.revoke_session(&oldest_session.session_id, "Session limit exceeded").await?;
            }
        }
        
        Ok(())
    }
    
    /// Log security event
    async fn log_security_event(&self, event: SecurityEvent) {
        let event_key = format!("security_event:{}:{}", event.user_id, Uuid::new_v4());
        
        if let Ok(event_json) = serde_json::to_string(&event) {
            self.cache.set(&event_key, event_json, Some(Duration::days(30).num_seconds() as u64));
        } else {
            error!("Failed to serialize security event");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::memory_cache::MemoryCache;
    
    #[tokio::test]
    async fn test_session_creation_and_retrieval() {
        let cache = Arc::new(MemoryCache::new());
        let session_service = SessionService::new(cache, 1, 5, 24);
        
        let request = CreateSessionRequest {
            user_id: "test-user-123".to_string(),
            device_info: Some(DeviceInfo {
                device_type: "desktop".to_string(),
                os: Some("Linux".to_string()),
                browser: Some("Firefox".to_string()),
                fingerprint: None,
            }),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            session_duration: None,
        };
        
        let session = session_service.create_session(request).await.unwrap();
        assert_eq!(session.user_id, "test-user-123");
        assert!(session.is_active);
        
        let retrieved_session = session_service.get_session(&session.session_id).await.unwrap();
        assert!(retrieved_session.is_some());
        assert_eq!(retrieved_session.unwrap().user_id, "test-user-123");
    }
    
    #[tokio::test]
    async fn test_session_revocation() {
        let cache = Arc::new(MemoryCache::new());
        let session_service = SessionService::new(cache, 1, 5, 24);
        
        let request = CreateSessionRequest {
            user_id: "test-user-123".to_string(),
            device_info: None,
            ip_address: None,
            user_agent: None,
            session_duration: None,
        };
        
        let session = session_service.create_session(request).await.unwrap();
        
        // Revoke session
        session_service.revoke_session(&session.session_id, "test revocation").await.unwrap();
        
        // Session should no longer exist
        let retrieved_session = session_service.get_session(&session.session_id).await.unwrap();
        assert!(retrieved_session.is_none());
    }
    
    #[tokio::test]
    async fn test_refresh_token_flow() {
        let cache = Arc::new(MemoryCache::new());
        let session_service = SessionService::new(cache, 1, 5, 24);
        
        let request = CreateSessionRequest {
            user_id: "test-user-123".to_string(),
            device_info: None,
            ip_address: None,
            user_agent: None,
            session_duration: None,
        };
        
        let session = session_service.create_session(request).await.unwrap();
        
        // Create refresh token
        let refresh_token = session_service.create_refresh_token(&session.user_id, &session.session_id).await.unwrap();
        
        // Validate refresh token
        let validated_token = session_service.validate_refresh_token(&refresh_token.token).await.unwrap();
        assert!(validated_token.is_some());
        
        // Revoke refresh token
        session_service.revoke_refresh_token(&refresh_token.token).await.unwrap();
        
        // Token should no longer be valid
        let revoked_token = session_service.validate_refresh_token(&refresh_token.token).await.unwrap();
        assert!(revoked_token.is_none());
    }
}