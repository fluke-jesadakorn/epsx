use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

use crate::infra::firebase_admin::{FirebaseAdmin, FirebaseUser};
use crate::dom::services::firebase_user_service::{FirebaseUserService, FirebaseUserServiceTrait};

/// Minimal session management service for Firebase-authenticated users
/// Only stores session tokens and references - all user data comes from Firebase
#[derive(Clone)]
pub struct FirebaseSessionService {
    db_pool: PgPool,
    firebase_admin: FirebaseAdmin,
    firebase_user_service: FirebaseUserService,
}

/// Session information
#[derive(Debug, Clone)]
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
pub trait FirebaseSessionServiceTrait: Send + Sync {
    async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionInfo, SessionServiceError>;
    async fn validate_session(&self, session_token: &str) -> Result<SessionValidationResult, SessionServiceError>;
    async fn refresh_session(&self, session_token: &str) -> Result<SessionInfo, SessionServiceError>;
    async fn revoke_session(&self, session_token: &str) -> Result<(), SessionServiceError>;
    async fn revoke_all_user_sessions(&self, firebase_uid: &str) -> Result<u32, SessionServiceError>;
    async fn cleanup_expired_sessions(&self) -> Result<u32, SessionServiceError>;
    async fn get_user_sessions(&self, firebase_uid: &str) -> Result<Vec<SessionInfo>, SessionServiceError>;
}

/// Session service errors
#[derive(Debug, thiserror::Error)]
pub enum SessionServiceError {
    #[error("Invalid session token")]
    InvalidSessionToken,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Firebase token validation failed: {0}")]
    FirebaseTokenValidationFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl FirebaseSessionService {
    /// Create new Firebase session service
    pub fn new(
        db_pool: PgPool,
        firebase_admin: FirebaseAdmin,
        firebase_user_service: FirebaseUserService,
    ) -> Self {
        Self {
            db_pool,
            firebase_admin,
            firebase_user_service,
        }
    }
}

#[async_trait]
impl FirebaseSessionServiceTrait for FirebaseSessionService {
    /// Create new session from Firebase ID token
    async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionInfo, SessionServiceError> {
        tracing::info!("Creating new Firebase session");
        
        // Validate Firebase ID token and get user data
        let firebase_user = self.firebase_admin
            .verify_id_token(&request.firebase_id_token)
            .await
            .map_err(|e| {
                tracing::error!("Failed to verify Firebase ID token: {}", e);
                SessionServiceError::FirebaseTokenValidationFailed(e.to_string())
            })?;
            
        // Extract Firebase token ID (jti) for tracking
        let firebase_token_id = self.extract_token_jti(&request.firebase_id_token)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
            
        // Generate internal session token
        let session_token = self.generate_session_token();
        let session_duration = request.session_duration_hours.unwrap_or(8); // 8 hours default
        let expires_at = Utc::now() + Duration::hours(session_duration);
        
        // Insert session into database (minimal storage)
        let session_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO firebase_sessions (
                id, firebase_uid, session_token, firebase_token_id, 
                expires_at, user_agent, ip_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::inet)
            RETURNING id, created_at, last_accessed_at
        "#;
        
        let row = sqlx::query(query)
            .bind(&session_id)
            .bind(&firebase_user.uid)
            .bind(&session_token)
            .bind(&firebase_token_id)
            .bind(&expires_at)
            .bind(&request.user_agent)
            .bind(&request.ip_address)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create session in database: {}", e);
                SessionServiceError::DatabaseError(e.to_string())
            })?;
            
        let created_at: DateTime<Utc> = row.get("created_at");
        let last_accessed_at: DateTime<Utc> = row.get("last_accessed_at");
        
        let session_info = SessionInfo {
            session_id,
            firebase_uid: firebase_user.uid.clone(),
            session_token,
            firebase_token_id,
            expires_at,
            created_at,
            last_accessed_at,
            user_agent: request.user_agent,
            ip_address: request.ip_address,
            is_active: true,
        };
        
        tracing::info!("Successfully created session {} for user {}", session_id, firebase_user.uid);
        Ok(session_info)
    }
    
    /// Validate session token and get user data from Firebase
    async fn validate_session(&self, session_token: &str) -> Result<SessionValidationResult, SessionServiceError> {
        tracing::debug!("Validating session token");
        
        // Get session from database
        let query = r#"
            SELECT id, firebase_uid, session_token, firebase_token_id, expires_at, 
                   created_at, last_accessed_at, user_agent, ip_address, is_active
            FROM firebase_sessions 
            WHERE session_token = $1 AND is_active = true
        "#;
        
        let session_row = match sqlx::query(query)
            .bind(session_token)
            .fetch_optional(&self.db_pool)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                return Ok(SessionValidationResult {
                    is_valid: false,
                    session_info: None,
                    firebase_user: None,
                    error_message: Some("Session not found".to_string()),
                });
            },
            Err(e) => {
                tracing::error!("Database error validating session: {}", e);
                return Err(SessionServiceError::DatabaseError(e.to_string()));
            }
        };
        
        // Parse session data
        let session_id: Uuid = session_row.get("id");
        let firebase_uid: String = session_row.get("firebase_uid");
        let expires_at: DateTime<Utc> = session_row.get("expires_at");
        
        // Check if session is expired
        if Utc::now() > expires_at {
            tracing::warn!("Session {} expired at {}", session_id, expires_at);
            
            // Mark session as inactive
            let _ = self.mark_session_inactive(&session_id).await;
            
            return Ok(SessionValidationResult {
                is_valid: false,
                session_info: None,
                firebase_user: None,
                error_message: Some("Session expired".to_string()),
            });
        }
        
        // Get current user data from Firebase (always fresh)
        let firebase_user = match self.firebase_user_service.get_user_by_uid(&firebase_uid).await {
            Ok(user) => user,
            Err(e) => {
                tracing::error!("Failed to get Firebase user {}: {}", firebase_uid, e);
                
                return Ok(SessionValidationResult {
                    is_valid: false,
                    session_info: None,
                    firebase_user: None,
                    error_message: Some(format!("User not found in Firebase: {}", e)),
                });
            }
        };
        
        // Build session info
        let session_info = SessionInfo {
            session_id,
            firebase_uid: firebase_uid.clone(),
            session_token: session_token.to_string(),
            firebase_token_id: session_row.get("firebase_token_id"),
            expires_at,
            created_at: session_row.get("created_at"),
            last_accessed_at: session_row.get("last_accessed_at"),
            user_agent: session_row.get("user_agent"),
            ip_address: session_row.get("ip_address"),
            is_active: session_row.get("is_active"),
        };
        
        // Update last accessed time
        let _ = self.update_session_last_accessed(&session_id).await;
        
        tracing::debug!("Session {} validated successfully for user {}", session_id, firebase_uid);
        
        Ok(SessionValidationResult {
            is_valid: true,
            session_info: Some(session_info),
            firebase_user: Some(firebase_user),
            error_message: None,
        })
    }
    
    /// Refresh session expiration time
    async fn refresh_session(&self, session_token: &str) -> Result<SessionInfo, SessionServiceError> {
        tracing::info!("Refreshing session");
        
        // Validate current session
        let validation_result = self.validate_session(session_token).await?;
        
        if !validation_result.is_valid {
            return Err(SessionServiceError::InvalidSessionToken);
        }
        
        let session_info = validation_result.session_info
            .ok_or(SessionServiceError::SessionNotFound)?;
            
        // Extend session expiration by 8 hours
        let new_expires_at = Utc::now() + Duration::hours(8);
        
        let query = r#"
            UPDATE firebase_sessions 
            SET expires_at = $1, last_accessed_at = NOW()
            WHERE id = $2 AND is_active = true
            RETURNING expires_at, last_accessed_at
        "#;
        
        let row = sqlx::query(query)
            .bind(&new_expires_at)
            .bind(&session_info.session_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to refresh session: {}", e);
                SessionServiceError::DatabaseError(e.to_string())
            })?;
            
        let updated_session = SessionInfo {
            expires_at: row.get("expires_at"),
            last_accessed_at: row.get("last_accessed_at"),
            ..session_info
        };
        
        tracing::info!("Successfully refreshed session {}", updated_session.session_id);
        Ok(updated_session)
    }
    
    /// Revoke single session
    async fn revoke_session(&self, session_token: &str) -> Result<(), SessionServiceError> {
        tracing::info!("Revoking session");
        
        let query = r#"
            UPDATE firebase_sessions 
            SET is_active = false 
            WHERE session_token = $1
        "#;
        
        let result = sqlx::query(query)
            .bind(session_token)
            .execute(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke session: {}", e);
                SessionServiceError::DatabaseError(e.to_string())
            })?;
            
        if result.rows_affected() == 0 {
            return Err(SessionServiceError::SessionNotFound);
        }
        
        tracing::info!("Successfully revoked session");
        Ok(())
    }
    
    /// Revoke all sessions for a Firebase user
    async fn revoke_all_user_sessions(&self, firebase_uid: &str) -> Result<u32, SessionServiceError> {
        tracing::info!("Revoking all sessions for user: {}", firebase_uid);
        
        let query = r#"
            UPDATE firebase_sessions 
            SET is_active = false 
            WHERE firebase_uid = $1 AND is_active = true
        "#;
        
        let result = sqlx::query(query)
            .bind(firebase_uid)
            .execute(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke user sessions: {}", e);
                SessionServiceError::DatabaseError(e.to_string())
            })?;
            
        let revoked_count = result.rows_affected() as u32;
        tracing::info!("Successfully revoked {} sessions for user {}", revoked_count, firebase_uid);
        Ok(revoked_count)
    }
    
    /// Clean up expired sessions
    async fn cleanup_expired_sessions(&self) -> Result<u32, SessionServiceError> {
        tracing::info!("Cleaning up expired sessions");
        
        let query = r#"
            DELETE FROM firebase_sessions 
            WHERE expires_at < NOW() - INTERVAL '1 day'
        "#;
        
        let result = sqlx::query(query)
            .execute(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to cleanup expired sessions: {}", e);
                SessionServiceError::DatabaseError(e.to_string())
            })?;
            
        let cleaned_count = result.rows_affected() as u32;
        tracing::info!("Successfully cleaned up {} expired sessions", cleaned_count);
        Ok(cleaned_count)
    }
    
    /// Get all active sessions for a Firebase user
    async fn get_user_sessions(&self, firebase_uid: &str) -> Result<Vec<SessionInfo>, SessionServiceError> {
        tracing::info!("Getting sessions for user: {}", firebase_uid);
        
        let query = r#"
            SELECT id, firebase_uid, session_token, firebase_token_id, expires_at,
                   created_at, last_accessed_at, user_agent, ip_address, is_active
            FROM firebase_sessions 
            WHERE firebase_uid = $1 AND is_active = true
            ORDER BY last_accessed_at DESC
        "#;
        
        let rows = sqlx::query(query)
            .bind(firebase_uid)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get user sessions: {}", e);
                SessionServiceError::DatabaseError(e.to_string())
            })?;
            
        let sessions: Vec<SessionInfo> = rows.into_iter().map(|row| SessionInfo {
            session_id: row.get("id"),
            firebase_uid: row.get("firebase_uid"),
            session_token: row.get("session_token"),
            firebase_token_id: row.get("firebase_token_id"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            last_accessed_at: row.get("last_accessed_at"),
            user_agent: row.get("user_agent"),
            ip_address: row.get("ip_address"),
            is_active: row.get("is_active"),
        }).collect();
        
        tracing::info!("Found {} active sessions for user {}", sessions.len(), firebase_uid);
        Ok(sessions)
    }
}

impl FirebaseSessionService {
    /// Create a new session
    pub async fn create_session(&self, request: CreateSessionRequest) -> Result<SessionInfo, SessionServiceError> {
        // For now, implement a simple session creation
        let firebase_user = self.firebase_admin.verify_id_token(&request.firebase_id_token)
            .await
            .map_err(|e| SessionServiceError::FirebaseTokenValidationFailed(e.to_string()))?;
            
        let session_id = uuid::Uuid::new_v4();
        let session_token = self.generate_session_token();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(
            request.session_duration_hours.unwrap_or(24)
        );
        
        // Store session in database
        let _ = sqlx::query(
            r#"
            INSERT INTO firebase_sessions (id, firebase_uid, session_token, firebase_token_id, expires_at, user_agent, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(session_id)
        .bind(&firebase_user.uid)
        .bind(&session_token)
        .bind(&request.firebase_id_token)
        .bind(expires_at)
        .bind(&request.user_agent)
        .bind(&request.ip_address)
        .execute(&self.db_pool)
        .await
        .map_err(|e| SessionServiceError::DatabaseError(e.to_string()))?;
        
        Ok(SessionInfo {
            session_id,
            firebase_uid: firebase_user.uid,
            session_token,
            firebase_token_id: request.firebase_id_token,
            expires_at,
            created_at: chrono::Utc::now(),
            last_accessed_at: chrono::Utc::now(),
            user_agent: request.user_agent,
            ip_address: request.ip_address,
            is_active: true,
        })
    }

    /// Validate an existing session
    pub async fn validate_session(&self, _session_token: &str) -> Result<SessionValidationResult, SessionServiceError> {
        // For now, implement basic session validation
        // TODO: Implement proper token validation against database
        Ok(SessionValidationResult {
            is_valid: true,
            session_info: None,
            firebase_user: None,
            error_message: None,
        })
    }

    /// Refresh a session
    pub async fn refresh_session(&self, session_token: &str) -> Result<SessionInfo, SessionServiceError> {
        // For now, just validate and return the same session
        // In production, you'd want to generate a new token
        let validation = self.validate_session(session_token).await?;
        validation.session_info.ok_or_else(|| SessionServiceError::SessionNotFound)
    }

    /// Generate secure session token
    fn generate_session_token(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
    
    /// Extract JWT ID (jti) from Firebase ID token
    fn extract_token_jti(&self, id_token: &str) -> Option<String> {
        // Simple JWT payload extraction without verification
        // We already verified the token in create_session
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        
        let payload = parts[1];
        let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let payload_json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
        
        payload_json.get("jti").and_then(|v| v.as_str()).map(|s| s.to_string())
    }
    
    /// Mark session as inactive
    async fn mark_session_inactive(&self, session_id: &Uuid) -> Result<(), SessionServiceError> {
        let query = "UPDATE firebase_sessions SET is_active = false WHERE id = $1";
        
        sqlx::query(query)
            .bind(session_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| SessionServiceError::DatabaseError(e.to_string()))?;
            
        Ok(())
    }
    
    /// Update session last accessed time
    async fn update_session_last_accessed(&self, session_id: &Uuid) -> Result<(), SessionServiceError> {
        let query = "UPDATE firebase_sessions SET last_accessed_at = NOW() WHERE id = $1";
        
        sqlx::query(query)
            .bind(session_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| SessionServiceError::DatabaseError(e.to_string()))?;
            
        Ok(())
    }
}