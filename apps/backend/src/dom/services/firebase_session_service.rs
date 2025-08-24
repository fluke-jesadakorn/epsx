use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use crate::infra::db::diesel::DbPool;
use crate::infra::cache::{Cache, CacheExt};
use uuid::Uuid;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use tracing::{info, debug, warn};

use crate::infra::firebase_admin::{FirebaseAdmin, FirebaseUser};
use crate::dom::services::firebase_user_service::FirebaseUserService;

/// Minimal session management service for Firebase-authenticated users
/// Only stores session tokens and references - all user data comes from Firebase
#[derive(Clone)]
pub struct FirebaseSessionService {
    db_pool: Arc<DbPool>,
    firebase_admin: FirebaseAdmin,
    firebase_user_service: FirebaseUserService,
    cache: Arc<dyn Cache>,
}

/// Session information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub firebase_uid: String,
    pub session_token: String,
    pub firebase_token_id: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub is_active: bool,
}

/// Session creation request
#[derive(Debug, Clone)]
pub struct CreateSessionRequest {
    pub firebase_id_token: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub session_duration_hours: Option<i64>,
}

/// Session validation result
#[derive(Debug, Clone)]
pub struct SessionValidationResult {
    pub is_valid: bool,
    pub session_info: Option<SessionInfo>,
    pub firebase_user: Option<FirebaseUser>,
    pub error_message: Option<String>,
}

/// Firebase session service trait
#[async_trait]
pub trait FirebaseSessionServiceTrait {
    async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionInfo, Box<dyn std::error::Error + Send + Sync>>;
    async fn validate_session(&self, session_token: &str) -> Result<SessionValidationResult, Box<dyn std::error::Error + Send + Sync>>;
    async fn refresh_session(&self, session_token: &str) -> Result<SessionInfo, Box<dyn std::error::Error + Send + Sync>>;
    async fn invalidate_session(&self, session_token: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_active_sessions(&self, firebase_uid: &str) -> Result<Vec<SessionInfo>, Box<dyn std::error::Error + Send + Sync>>;
    async fn cleanup_expired_sessions(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;
}

impl FirebaseSessionService {
    pub fn new(
        db_pool: Arc<DbPool>,
        firebase_admin: FirebaseAdmin,
        firebase_user_service: FirebaseUserService,
        cache: Arc<dyn Cache>,
    ) -> Self {
        Self {
            db_pool,
            firebase_admin,
            firebase_user_service,
            cache,
        }
    }

    /// Generate secure session token
    fn generate_session_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    /// Create cache key for session token
    fn session_key(&self, session_token: &str) -> String {
        format!("session:token:{}", session_token)
    }

    /// Create cache key for user sessions list
    fn user_sessions_key(&self, firebase_uid: &str) -> String {
        format!("session:user:{}", firebase_uid)
    }

    /// Cache session information
    async fn cache_session(&self, session: &SessionInfo) -> bool {
        let session_key = self.session_key(&session.session_token);
        let ttl_seconds = (session.expires_at - Utc::now()).num_seconds().max(0);
        
        match self.cache.set(&session_key, session, Some(ttl_seconds)).await {
            Ok(_) => {
                debug!("Successfully cached session: {}", session.session_token);
                true
            }
            Err(e) => {
                warn!("Failed to cache session {}: {}", session.session_token, e);
                false
            }
        }
    }

    /// Get cached session information
    async fn get_cached_session(&self, session_token: &str) -> Option<SessionInfo> {
        let session_key = self.session_key(session_token);
        
        match self.cache.get::<SessionInfo>(&session_key).await {
            Ok(Some(session)) => {
                if session.expires_at > Utc::now() {
                    debug!("Cache hit for session: {}", session_token);
                    Some(session)
                } else {
                    debug!("Cached session expired: {}", session_token);
                    let _ = self.cache.delete(&session_key).await;
                    None
                }
            }
            Ok(None) => {
                debug!("Cache miss for session: {}", session_token);
                None
            }
            Err(e) => {
                warn!("Cache error for session {}: {}", session_token, e);
                None
            }
        }
    }

    /// Invalidate cached session
    async fn invalidate_cached_session(&self, session_token: &str) -> bool {
        let session_key = self.session_key(session_token);
        
        match self.cache.delete(&session_key).await {
            Ok(deleted) => {
                if deleted {
                    debug!("Invalidated cached session: {}", session_token);
                }
                deleted
            }
            Err(e) => {
                warn!("Failed to invalidate cached session {}: {}", session_token, e);
                false
            }
        }
    }

    /// Cache user's active sessions list
    async fn cache_user_sessions(&self, firebase_uid: &str, sessions: &[SessionInfo]) -> bool {
        let user_sessions_key = self.user_sessions_key(firebase_uid);
        let ttl_seconds = 1800; // 30 minutes for session lists
        
        match self.cache.set(&user_sessions_key, &sessions, Some(ttl_seconds)).await {
            Ok(_) => {
                debug!("Successfully cached {} sessions for user: {}", sessions.len(), firebase_uid);
                true
            }
            Err(e) => {
                warn!("Failed to cache sessions for user {}: {}", firebase_uid, e);
                false
            }
        }
    }

    /// Get cached user sessions list
    async fn get_cached_user_sessions(&self, firebase_uid: &str) -> Option<Vec<SessionInfo>> {
        let user_sessions_key = self.user_sessions_key(firebase_uid);
        
        match self.cache.get::<Vec<SessionInfo>>(&user_sessions_key).await {
            Ok(Some(sessions)) => {
                debug!("Cache hit for user sessions: {}", firebase_uid);
                Some(sessions)
            }
            Ok(None) => {
                debug!("Cache miss for user sessions: {}", firebase_uid);
                None
            }
            Err(e) => {
                warn!("Cache error for user sessions {}: {}", firebase_uid, e);
                None
            }
        }
    }
}

#[async_trait]
impl FirebaseSessionServiceTrait for FirebaseSessionService {
    /// Create new session with caching support
    async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionInfo, Box<dyn std::error::Error + Send + Sync>> {
        info!("Creating session with cache support");
        // TODO: Implement Firebase ID token validation and database persistence with Diesel
        
        let session = SessionInfo {
            session_id: Uuid::new_v4(),
            firebase_uid: "stub-uid".to_string(), // TODO: Extract from validated Firebase token
            session_token: Self::generate_session_token(),
            firebase_token_id: "stub-token-id".to_string(), // TODO: Extract from Firebase token
            expires_at: Utc::now() + Duration::hours(request.session_duration_hours.unwrap_or(24)),
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
            user_agent: request.user_agent,
            ip_address: request.ip_address,
            is_active: true,
        };

        // Cache the session for fast access
        self.cache_session(&session).await;
        
        // TODO: Persist to database using Diesel
        debug!("Session created and cached: {}", session.session_token);
        
        Ok(session)
    }

    /// Validate session with cache support and fallback
    async fn validate_session(&self, session_token: &str) -> Result<SessionValidationResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Validating session token with cache support: {}", session_token);
        
        // Try to get session from cache first
        if let Some(session) = self.get_cached_session(session_token).await {
            if session.is_active {
                // TODO: Validate Firebase token is still valid
                // For now, assume cached sessions are valid
                return Ok(SessionValidationResult {
                    is_valid: true,
                    session_info: Some(session),
                    firebase_user: None, // TODO: Get from Firebase or cache
                    error_message: None,
                });
            }
        }

        // Cache miss or expired - check database
        // TODO: Implement database lookup with Diesel
        debug!("Session not found in cache, would check database: {}", session_token);
        
        Ok(SessionValidationResult {
            is_valid: false,
            session_info: None,
            firebase_user: None,
            error_message: Some("Session not found or expired".to_string()),
        })
    }

    /// Refresh session with cache support
    async fn refresh_session(&self, session_token: &str) -> Result<SessionInfo, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Refreshing session token with cache support: {}", session_token);
        
        // Get current session from cache
        if let Some(mut session) = self.get_cached_session(session_token).await {
            if session.is_active {
                // Update session with new expiration and last accessed time
                session.expires_at = Utc::now() + Duration::hours(24);
                session.last_accessed_at = Utc::now();
                
                // Cache the refreshed session
                self.cache_session(&session).await;
                
                // TODO: Update in database with Diesel
                debug!("Session refreshed and cached: {}", session_token);
                
                return Ok(session);
            }
        }
        
        // Session not found or not active
        Err("Session not found or expired".into())
    }

    /// Invalidate session with cache support
    async fn invalidate_session(&self, session_token: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Invalidating session token with cache support: {}", session_token);
        
        // Remove from cache
        self.invalidate_cached_session(session_token).await;
        
        // TODO: Mark as inactive in database with Diesel
        // For now, cache invalidation is sufficient for stub implementation
        
        debug!("Session invalidated: {}", session_token);
        Ok(())
    }

    /// Get active sessions for user with cache support
    async fn get_active_sessions(&self, firebase_uid: &str) -> Result<Vec<SessionInfo>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Getting active sessions for user with cache support: {}", firebase_uid);
        
        // Check cache first
        if let Some(sessions) = self.get_cached_user_sessions(firebase_uid).await {
            debug!("Retrieved {} cached sessions for user: {}", sessions.len(), firebase_uid);
            return Ok(sessions);
        }
        
        // Cache miss - query database
        // TODO: Implement Diesel query to firebase_sessions table
        let sessions = vec![]; // Stub: empty list for now
        
        // Cache the result
        self.cache_user_sessions(firebase_uid, &sessions).await;
        
        Ok(sessions)
    }

    /// Clean up expired sessions with cache support
    async fn cleanup_expired_sessions(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Cleaning up expired sessions with cache support");
        
        // TODO: Implement Diesel delete query for expired sessions in database
        // For now, the cache automatically handles expiration via TTL
        
        let cleaned_count = 0; // Stub implementation
        debug!("Cleaned up {} expired sessions", cleaned_count);
        
        Ok(cleaned_count)
    }
}