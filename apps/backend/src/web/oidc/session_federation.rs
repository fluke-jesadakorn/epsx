// Cross-Application Session Federation System
// Enables seamless SSO across multiple applications within the same tenant

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use tokio::sync::RwLock;
use uuid::Uuid;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};

use crate::core::errors::AppError;

/// Federated session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedSession {
    /// Global session ID across all applications
    pub global_session_id: String,
    /// User ID
    pub user_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Primary provider ID used for authentication
    pub provider_id: String,
    /// User email
    pub email: String,
    /// User role
    pub role: String,
    /// User permissions
    pub permissions: Vec<String>,
    
    /// Application-specific session data
    pub app_sessions: HashMap<String, AppSessionInfo>,
    
    /// Session metadata
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub risk_score: Option<f64>,
    
    /// Security flags
    pub is_active: bool,
    pub requires_mfa: bool,
    pub mfa_completed: bool,
    pub step_up_required: bool,
}

/// Application-specific session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSessionInfo {
    /// Application identifier
    pub app_id: String,
    /// Application-specific session ID
    pub app_session_id: String,
    /// Application-specific access token
    pub access_token: String,
    /// Token expiration
    pub token_expires_at: DateTime<Utc>,
    /// Application-specific scopes
    pub scopes: Vec<String>,
    /// Last access time for this app
    pub last_accessed: DateTime<Utc>,
    /// Application-specific metadata
    pub app_metadata: HashMap<String, String>,
}

/// Session federation token (used for cross-app authentication)
#[derive(Debug, Serialize, Deserialize)]
pub struct FederationToken {
    /// Standard JWT claims
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    
    /// Federation-specific claims
    pub global_session_id: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub app_sessions: Vec<String>, // List of app IDs with active sessions
    
    /// Security claims
    pub auth_time: i64,
    pub amr: Vec<String>,
    pub acr: String,
    pub risk_score: Option<f64>,
    
    /// Federation metadata
    pub federation_type: String, // "sso", "step_up", "delegation"
    pub source_app: String,
    pub target_apps: Vec<String>,
}

/// Cross-application authentication request
#[derive(Debug, Deserialize)]
pub struct CrossAppAuthRequest {
    /// Source application ID
    pub source_app_id: String,
    /// Target application ID
    pub target_app_id: String,
    /// Federation token from source app
    pub federation_token: String,
    /// Requested scopes for target app
    pub requested_scopes: Vec<String>,
    /// Optional step-up authentication requirement
    pub step_up_required: Option<bool>,
    /// Client information
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Cross-application authentication response
#[derive(Debug, Serialize)]
pub struct CrossAppAuthResponse {
    /// New access token for target application
    pub access_token: String,
    /// Token type
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: i64,
    /// Token expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Granted scopes
    pub scope: String,
    /// Global session ID
    pub global_session_id: String,
    /// Application session ID
    pub app_session_id: String,
    /// Whether step-up authentication was performed
    pub step_up_completed: bool,
}

/// Session federation configuration
#[derive(Debug, Clone)]
pub struct SessionFederationConfig {
    /// JWT signing key for federation tokens
    pub federation_jwt_secret: String,
    /// Issuer for federation tokens
    pub issuer: String,
    /// Maximum session lifetime
    pub max_session_lifetime_hours: i64,
    /// Federation token TTL
    pub federation_token_ttl_minutes: i64,
    /// Enable cross-tenant federation (dangerous!)
    pub enable_cross_tenant_federation: bool,
    /// Require step-up authentication for admin apps
    pub require_admin_step_up: bool,
    /// Maximum risk score for federation
    pub max_federation_risk_score: f64,
    /// Enable session sharing between apps
    pub enable_session_sharing: bool,
}

impl Default for SessionFederationConfig {
    fn default() -> Self {
        Self {
            federation_jwt_secret: std::env::var("FEDERATION_JWT_SECRET")
                .or_else(|_| std::env::var("NEXTAUTH_SECRET"))
                .unwrap_or_else(|_| "federation-secret".to_string()),
            issuer: std::env::var("FEDERATION_ISSUER")
                .unwrap_or_else(|_| "http://localhost:8080/federation".to_string()),
            max_session_lifetime_hours: 8,
            federation_token_ttl_minutes: 5,
            enable_cross_tenant_federation: false,
            require_admin_step_up: true,
            max_federation_risk_score: 0.5,
            enable_session_sharing: true,
        }
    }
}

/// Session federation trait
#[async_trait]
pub trait SessionFederationTrait: Send + Sync {
    /// Create a new federated session
    async fn create_federated_session(
        &self,
        user_id: String,
        tenant_id: String,
        provider_id: String,
        email: String,
        role: String,
        permissions: Vec<String>,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<FederatedSession, AppError>;
    
    /// Add application session to federated session
    async fn add_app_session(
        &self,
        global_session_id: &str,
        app_session: AppSessionInfo,
    ) -> Result<(), AppError>;
    
    /// Generate federation token for cross-app authentication
    async fn generate_federation_token(
        &self,
        global_session_id: &str,
        source_app: &str,
        target_apps: Vec<String>,
    ) -> Result<String, AppError>;
    
    /// Validate federation token and perform cross-app authentication
    async fn authenticate_cross_app(
        &self,
        request: CrossAppAuthRequest,
    ) -> Result<CrossAppAuthResponse, AppError>;
    
    /// Get federated session by ID
    async fn get_session(&self, global_session_id: &str) -> Result<Option<FederatedSession>, AppError>;
    
    /// Update session last access time
    async fn update_session_access(
        &self,
        global_session_id: &str,
        app_id: Option<&str>,
    ) -> Result<(), AppError>;
    
    /// Terminate federated session (logout from all apps)
    async fn terminate_session(&self, global_session_id: &str) -> Result<(), AppError>;
    
    /// Get active sessions for user
    async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<FederatedSession>, AppError>;
    
    /// Cleanup expired sessions
    async fn cleanup_expired_sessions(&self) -> Result<u64, AppError>;
}

/// In-memory session federation implementation
pub struct InMemorySessionFederation {
    config: SessionFederationConfig,
    sessions: Arc<RwLock<HashMap<String, FederatedSession>>>,
    user_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> session_ids
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl InMemorySessionFederation {
    pub fn new(config: SessionFederationConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.federation_jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.federation_jwt_secret.as_bytes());
        
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            encoding_key,
            decoding_key,
        }
    }
    
    /// Calculate risk score based on session data
    fn calculate_risk_score(
        &self,
        ip_address: &Option<String>,
        user_agent: &Option<String>,
        _previous_sessions: &[FederatedSession],
    ) -> f64 {
        let mut risk_score: f64 = 0.0;
        
        // IP address risk (simplified)
        if ip_address.is_none() {
            risk_score += 0.1;
        }
        
        // User agent risk (simplified)
        if user_agent.is_none() {
            risk_score += 0.1;
        }
        
        // TODO: Add more sophisticated risk assessment
        // - Geolocation changes
        // - Device fingerprinting
        // - Behavioral analysis
        // - Time-based patterns
        
        risk_score.min(1.0)
    }
    
    /// Check if step-up authentication is required
    fn requires_step_up(&self, role: &str, target_app: &str) -> bool {
        if !self.config.require_admin_step_up {
            return false;
        }
        
        // Require step-up for admin roles accessing admin apps
        let is_admin_role = matches!(role, "admin" | "super_admin" | "admin-full-004");
        let is_admin_app = target_app.contains("admin");
        
        is_admin_role && is_admin_app
    }
}

#[async_trait]
impl SessionFederationTrait for InMemorySessionFederation {
    async fn create_federated_session(
        &self,
        user_id: String,
        tenant_id: String,
        provider_id: String,
        email: String,
        role: String,
        permissions: Vec<String>,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<FederatedSession, AppError> {
        let global_session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Get previous sessions for risk assessment
        let user_sessions = self.get_user_sessions(&user_id).await?;
        let risk_score = self.calculate_risk_score(&client_ip, &user_agent, &user_sessions);
        
        let session = FederatedSession {
            global_session_id: global_session_id.clone(),
            user_id: user_id.clone(),
            tenant_id,
            provider_id,
            email,
            role,
            permissions,
            app_sessions: HashMap::new(),
            created_at: now,
            last_accessed: now,
            expires_at: now + Duration::hours(self.config.max_session_lifetime_hours),
            ip_address: client_ip,
            user_agent,
            risk_score: Some(risk_score),
            is_active: true,
            requires_mfa: false, // TODO: Determine based on policy
            mfa_completed: false,
            step_up_required: false,
        };
        
        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(global_session_id.clone(), session.clone());
        }
        
        // Update user session mapping
        {
            let mut user_sessions_map = self.user_sessions.write().await;
            user_sessions_map
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(global_session_id.clone());
        }
        
        tracing::info!(
            global_session_id = %global_session_id,
            user_id = %session.user_id,
            tenant_id = %session.tenant_id,
            risk_score = ?session.risk_score,
            "Created federated session"
        );
        
        Ok(session)
    }
    
    async fn add_app_session(
        &self,
        global_session_id: &str,
        app_session: AppSessionInfo,
    ) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(global_session_id) {
            session.app_sessions.insert(app_session.app_id.clone(), app_session.clone());
            session.last_accessed = Utc::now();
            
            tracing::info!(
                global_session_id = %global_session_id,
                app_id = %app_session.app_id,
                "Added app session to federated session"
            );
            
            Ok(())
        } else {
            Err(AppError::not_found(format!("Session {} not found", global_session_id)))
        }
    }
    
    async fn generate_federation_token(
        &self,
        global_session_id: &str,
        source_app: &str,
        target_apps: Vec<String>,
    ) -> Result<String, AppError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(global_session_id)
            .ok_or_else(|| AppError::not_found(format!("Session {} not found", global_session_id)))?;
        
        if !session.is_active {
            return Err(AppError::security_error("Session is not active".to_string()));
        }
        
        if let Some(risk_score) = session.risk_score {
            if risk_score > self.config.max_federation_risk_score {
                return Err(AppError::security_error("Session risk score too high for federation".to_string()));
            }
        }
        
        let now = Utc::now();
        let exp = now + Duration::minutes(self.config.federation_token_ttl_minutes);
        let jti = Uuid::new_v4().to_string();
        
        let app_sessions: Vec<String> = session.app_sessions.keys().cloned().collect();
        
        let federation_token = FederationToken {
            iss: self.config.issuer.clone(),
            sub: session.user_id.clone(),
            aud: target_apps.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti,
            
            global_session_id: global_session_id.to_string(),
            tenant_id: session.tenant_id.clone(),
            provider_id: session.provider_id.clone(),
            email: session.email.clone(),
            role: session.role.clone(),
            permissions: session.permissions.clone(),
            app_sessions,
            
            auth_time: session.created_at.timestamp(),
            amr: vec!["federation".to_string()],
            acr: "1".to_string(),
            risk_score: session.risk_score,
            
            federation_type: "sso".to_string(),
            source_app: source_app.to_string(),
            target_apps,
        };
        
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &federation_token, &self.encoding_key)
            .map_err(|e| AppError::internal_error(format!("Failed to encode federation token: {}", e)))?;
        
        tracing::info!(
            global_session_id = %global_session_id,
            source_app = %source_app,
            target_apps = ?federation_token.target_apps,
            "Generated federation token"
        );
        
        Ok(token)
    }
    
    async fn authenticate_cross_app(
        &self,
        request: CrossAppAuthRequest,
    ) -> Result<CrossAppAuthResponse, AppError> {
        // Validate federation token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&request.target_app_id]);
        
        let token_data = decode::<FederationToken>(
            &request.federation_token,
            &self.decoding_key,
            &validation,
        )
        .map_err(|e| AppError::security_error(format!("Invalid federation token: {}", e)))?;
        
        let federation_token = token_data.claims;
        
        // Validate session is still active
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&federation_token.global_session_id)
            .ok_or_else(|| AppError::not_found("Session not found".to_string()))?;
        
        if !session.is_active {
            return Err(AppError::security_error("Session is not active".to_string()));
        }
        
        // Cross-tenant federation check
        if !self.config.enable_cross_tenant_federation && session.tenant_id != federation_token.tenant_id {
            return Err(AppError::security_error("Cross-tenant federation not allowed".to_string()));
        }
        
        // Check if step-up authentication is required
        let step_up_required = request.step_up_required.unwrap_or_else(|| {
            self.requires_step_up(&session.role, &request.target_app_id)
        });
        
        if step_up_required && !session.mfa_completed {
            return Err(AppError::security_error("Step-up authentication required".to_string()));
        }
        
        // Generate new access token for target application
        let app_session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let token_expires_at = now + Duration::hours(1); // TODO: Configure per-app TTL
        
        // Create application-specific access token
        let access_token = Uuid::new_v4().to_string(); // TODO: Generate proper JWT
        
        // Create app session info
        let app_session = AppSessionInfo {
            app_id: request.target_app_id.clone(),
            app_session_id: app_session_id.clone(),
            access_token: access_token.clone(),
            token_expires_at,
            scopes: request.requested_scopes.clone(),
            last_accessed: now,
            app_metadata: HashMap::new(),
        };
        
        // Add to federated session
        session.app_sessions.insert(request.target_app_id.clone(), app_session);
        session.last_accessed = now;
        
        tracing::info!(
            global_session_id = %federation_token.global_session_id,
            source_app = %request.source_app_id,
            target_app = %request.target_app_id,
            step_up_required = step_up_required,
            "Cross-app authentication successful"
        );
        
        Ok(CrossAppAuthResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600, // 1 hour
            expires_at: token_expires_at,
            scope: request.requested_scopes.join(" "),
            global_session_id: federation_token.global_session_id,
            app_session_id,
            step_up_completed: step_up_required && session.mfa_completed,
        })
    }
    
    async fn get_session(&self, global_session_id: &str) -> Result<Option<FederatedSession>, AppError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(global_session_id).cloned())
    }
    
    async fn update_session_access(
        &self,
        global_session_id: &str,
        app_id: Option<&str>,
    ) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(global_session_id) {
            session.last_accessed = Utc::now();
            
            if let Some(app_id) = app_id {
                if let Some(app_session) = session.app_sessions.get_mut(app_id) {
                    app_session.last_accessed = Utc::now();
                }
            }
            
            Ok(())
        } else {
            Err(AppError::not_found(format!("Session {} not found", global_session_id)))
        }
    }
    
    async fn terminate_session(&self, global_session_id: &str) -> Result<(), AppError> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(mut session) = sessions.remove(global_session_id) {
            session.is_active = false;
            
            // Remove from user session mapping
            let mut user_sessions = self.user_sessions.write().await;
            if let Some(user_session_list) = user_sessions.get_mut(&session.user_id) {
                user_session_list.retain(|id| id != global_session_id);
                if user_session_list.is_empty() {
                    user_sessions.remove(&session.user_id);
                }
            }
            
            tracing::info!(
                global_session_id = %global_session_id,
                user_id = %session.user_id,
                app_count = session.app_sessions.len(),
                "Terminated federated session"
            );
            
            Ok(())
        } else {
            Err(AppError::not_found(format!("Session {} not found", global_session_id)))
        }
    }
    
    async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<FederatedSession>, AppError> {
        let user_sessions = self.user_sessions.read().await;
        let sessions = self.sessions.read().await;
        
        if let Some(session_ids) = user_sessions.get(user_id) {
            let user_session_list: Vec<FederatedSession> = session_ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect();
            Ok(user_session_list)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn cleanup_expired_sessions(&self) -> Result<u64, AppError> {
        let now = Utc::now();
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;
        
        let _initial_count = sessions.len();
        
        // Find expired sessions
        let expired_session_ids: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.expires_at < now || !session.is_active)
            .map(|(id, _)| id.clone())
            .collect();
        
        // Remove expired sessions
        for session_id in &expired_session_ids {
            if let Some(session) = sessions.remove(session_id) {
                // Remove from user session mapping
                if let Some(user_session_list) = user_sessions.get_mut(&session.user_id) {
                    user_session_list.retain(|id| id != session_id);
                    if user_session_list.is_empty() {
                        user_sessions.remove(&session.user_id);
                    }
                }
            }
        }
        
        let cleaned_count = expired_session_ids.len() as u64;
        
        if cleaned_count > 0 {
            tracing::info!(
                cleaned_sessions = cleaned_count,
                remaining_sessions = sessions.len(),
                "Cleaned up expired sessions"
            );
        }
        
        Ok(cleaned_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_federation_creation() {
        let config = SessionFederationConfig::default();
        let federation = InMemorySessionFederation::new(config);
        
        let session = federation
            .create_federated_session(
                "user123".to_string(),
                "tenant1".to_string(),
                "provider1".to_string(),
                "user@example.com".to_string(),
                "user".to_string(),
                vec!["read".to_string()],
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
            )
            .await
            .unwrap();
        
        assert_eq!(session.user_id, "user123");
        assert_eq!(session.tenant_id, "tenant1");
        assert!(session.is_active);
        assert!(session.app_sessions.is_empty());
    }
    
    #[tokio::test]
    async fn test_cross_app_authentication() {
        let config = SessionFederationConfig::default();
        let federation = InMemorySessionFederation::new(config);
        
        // Create a session first
        let session = federation
            .create_federated_session(
                "user123".to_string(),
                "tenant1".to_string(),
                "provider1".to_string(),
                "user@example.com".to_string(),
                "user".to_string(),
                vec!["read".to_string()],
                None,
                None,
            )
            .await
            .unwrap();
        
        // Generate federation token
        let federation_token = federation
            .generate_federation_token(
                &session.global_session_id,
                "frontend",
                vec!["admin".to_string()],
            )
            .await
            .unwrap();
        
        assert!(!federation_token.is_empty());
    }
}