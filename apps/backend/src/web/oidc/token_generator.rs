/// Token Generation Service
/// 
/// Handles JWT token creation and signing for access tokens, ID tokens, and refresh tokens.
/// Supports both admin and user token generation with proper claims processing.

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use uuid::Uuid;

use crate::config::env::get_env_var;
use crate::web::auth::AppState;
use crate::web::oidc::authorization::AuthorizationCodeData;
use crate::infrastructure::adapters::services::firebase::FirebaseUser;

use super::TokenErrorResponse;

/// JWT Claims for ID token
#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub jti: String,
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub auth_time: i64,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub role: String,
    pub admin: Option<bool>,
    pub access_level: Option<String>,
    pub permissions: Vec<String>,
    pub package_tier: String,
}

/// JWT Claims for Access token
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub jti: String,
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub scope: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub package_tier: String,
}

/// Token generation service
pub struct TokenGenerator {
    jwt_secret: String,
    issuer_url: String,
}

impl TokenGenerator {
    pub fn new() -> Self {
        Self {
            jwt_secret: get_jwt_secret(),
            issuer_url: get_issuer_url(),
        }
    }

    /// Generate access token with database user data and admin/user context detection
    pub async fn generate_access_token(
        &self,
        app_state: &AppState,
        firebase_user: &FirebaseUser,
        scope: &str,
        now: DateTime<Utc>,
        expires_in: i64,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        // Get user data from database for accurate permissions
        let (_, package_tier, database_permissions) = 
            self.get_user_database_info(app_state, &firebase_user.uid, firebase_user.email.as_deref().unwrap_or("")).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(TokenErrorResponse::server_error())))?;
        
        // Combine Firebase role with database permissions
        let firebase_role = self.get_role_from_custom_claims(&firebase_user.custom_claims);
        let effective_permissions = if !database_permissions.is_empty() {
            database_permissions
        } else {
            self.get_user_permissions_from_role(&firebase_user.custom_claims)
        };
        
        // Detect admin context based on role and permissions
        let is_admin_context = self.is_admin_user(&firebase_role, &effective_permissions, scope);
        
        if is_admin_context {
            self.generate_admin_access_token(firebase_user, &firebase_role, &effective_permissions, now, expires_in)
        } else {
            self.generate_user_access_token(firebase_user, &firebase_role, &effective_permissions, &package_tier, now, expires_in).await
        }
    }

    /// Generate access token without database lookup (for refresh token flow)
    pub fn generate_access_token_simple(
        &self,
        firebase_user: &FirebaseUser,
        scope: &str,
        now: DateTime<Utc>,
        expires_in: i64,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        let permissions = self.get_user_permissions_from_role(&firebase_user.custom_claims);

        let claims = AccessTokenClaims {
            jti: generate_jti(),
            iss: self.issuer_url.clone(),
            sub: firebase_user.uid.clone(),
            aud: "epsx-api".to_string(),
            exp: (now + Duration::seconds(expires_in)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            scope: scope.to_string(),
            email: firebase_user.email.clone().unwrap_or_default(),
            role: self.get_role_from_custom_claims(&firebase_user.custom_claims),
            permissions,
            package_tier: "FREE".to_string(),
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| {
                tracing::error!("Failed to generate access token: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TokenErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Failed to generate access token".to_string()),
                        error_uri: None,
                    }),
                )
            })
    }

    /// Generate ID token with database user data
    pub async fn generate_id_token(
        &self,
        app_state: &AppState,
        firebase_user: &FirebaseUser,
        client_id: &str,
        now: DateTime<Utc>,
        expires_in: i64,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        // Get user data from database for accurate information
        let (_, package_tier, database_permissions) = 
            self.get_user_database_info(app_state, &firebase_user.uid, firebase_user.email.as_deref().unwrap_or("")).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(TokenErrorResponse::server_error())))?;
        
        let claims = IdTokenClaims {
            jti: generate_jti(),
            iss: self.issuer_url.clone(),
            sub: firebase_user.uid.clone(),
            aud: client_id.to_string(),
            exp: (now + Duration::seconds(expires_in)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            auth_time: now.timestamp(),
            email: firebase_user.email.clone().unwrap_or_default(),
            email_verified: firebase_user.email_verified,
            name: firebase_user.display_name.clone(),
            role: self.get_role_from_custom_claims(&firebase_user.custom_claims),
            admin: firebase_user.custom_claims.get("admin").and_then(|v| v.as_bool()),
            access_level: firebase_user.custom_claims.get("access_level").and_then(|v| v.as_str()).map(|s| s.to_string()),
            permissions: database_permissions,
            package_tier,
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| {
                tracing::error!("Failed to generate ID token: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TokenErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Failed to generate ID token".to_string()),
                        error_uri: None,
                    }),
                )
            })
    }

    /// Generate ID token without database lookup (for refresh token flow)
    pub fn generate_id_token_simple(
        &self,
        firebase_user: &FirebaseUser,
        client_id: &str,
        now: DateTime<Utc>,
        expires_in: i64,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        let claims = IdTokenClaims {
            jti: generate_jti(),
            iss: self.issuer_url.clone(),
            sub: firebase_user.uid.clone(),
            aud: client_id.to_string(),
            exp: (now + Duration::seconds(expires_in)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            auth_time: now.timestamp(),
            email: firebase_user.email.clone().unwrap_or_default(),
            email_verified: firebase_user.email_verified,
            name: firebase_user.display_name.clone(),
            role: self.get_role_from_custom_claims(&firebase_user.custom_claims),
            admin: firebase_user.custom_claims.get("admin").and_then(|v| v.as_bool()),
            access_level: firebase_user.custom_claims.get("access_level").and_then(|v| v.as_str()).map(|s| s.to_string()),
            permissions: vec![],
            package_tier: "FREE".to_string(),
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| {
                tracing::error!("Failed to generate ID token: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TokenErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Failed to generate ID token".to_string()),
                        error_uri: None,
                    }),
                )
            })
    }

    /// Generate refresh token using new rotation service
    pub async fn generate_refresh_token_v2(
        &self,
        auth_data: &AuthorizationCodeData,
        client_id: &str,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        let user_id = &auth_data.firebase_user.uid;
        let scope = &auth_data.scope;
        
        let device_info = Some("Web Browser".to_string());
        
        REFRESH_TOKEN_SERVICE
            .createrefresh_token(user_id, client_id, scope, device_info)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create refresh token: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(TokenErrorResponse {
                        error: "server_error".to_string(),
                        error_description: Some("Failed to generate refresh token".to_string()),
                        error_uri: None,
                    }),
                )
            })
    }

    // Private helper methods

    /// Generate admin access token using AdminJWTService
    fn generate_admin_access_token(
        &self,
        firebase_user: &FirebaseUser,
        role: &str,
        permissions: &[String],
        now: DateTime<Utc>,
        _expires_in: i64,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        use crate::auth::admin_jwt::{AdminJWTService, AdminSecurityContext, AdminPermissionMatrix};
        use std::collections::HashMap;
        
        let admin_service = AdminJWTService::new(self.jwt_secret.as_bytes(), self.issuer_url.clone());
        
        let security_context = AdminSecurityContext {
            mfa_verified: true,
            mfa_timestamp: Some(now.timestamp() as u64),
            risk_score: 0.1,
            risk_factors: vec![],
            device_binding: "web-session".to_string(),
            ip_restrictions: vec![],
            current_ip: "0.0.0.0".to_string(),
            location_hash: "unknown".to_string(),
            session_start: now.timestamp() as u64,
            last_activity: now.timestamp() as u64,
        };
        
        let admin_permissions = AdminPermissionMatrix {
            platforms: HashMap::new(),
            system_access: crate::auth::admin_jwt::SystemAccessLevel {
                level: role.to_string(),
                capabilities: permissions.to_vec(),
                restrictions: vec![],
                monitoring_level: "standard".to_string(),
            },
            delegation_rights: vec![],
            emergency_access: None,
            version: 1,
            hash: "admin-permissions-v1".to_string(),
        };
        
        admin_service.generate_admin_token(
            firebase_user.uid.clone(),
            firebase_user.email.clone().unwrap_or_default(),
            firebase_user.display_name.clone().unwrap_or_else(|| "Admin".to_string()),
            security_context,
            admin_permissions,
        ).map_err(|e| {
            tracing::error!("Failed to generate admin access token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to generate admin access token".to_string()),
                    error_uri: None,
                }),
            )
        })
    }

    /// Generate user access token using UserJWTService
    async fn generate_user_access_token(
        &self,
        firebase_user: &FirebaseUser,
        _role: &str,
        permissions: &[String],
        package_tier: &str,
        now: DateTime<Utc>,
        _expires_in: i64,
    ) -> Result<String, (StatusCode, Json<TokenErrorResponse>)> {
        use crate::auth::user_jwt::{UserJWTService, UserContext, UserPreferences, UserSubscription};
        use std::collections::HashMap;
        
        let user_service = UserJWTService::new(self.jwt_secret.as_bytes(), self.issuer_url.clone());
        
        let user_context = UserContext {
            tier: package_tier.to_string(),
            verified: firebase_user.email_verified,
            created_at: now.timestamp() as u64,
            last_login: now.timestamp() as u64,
            preferences: UserPreferences {
                language: "en".to_string(),
                timezone: "UTC".to_string(),
                currency: "USD".to_string(),
                theme: Some("light".to_string()),
            },
        };
        
        let subscription = if package_tier != "FREE" {
            Some(UserSubscription {
                tier: package_tier.to_string(),
                status: "active".to_string(),
                expires_at: None,
                features: determine_features_for_tier(package_tier),
                limits: determine_limits_for_tier(package_tier),
                usage: HashMap::new(),
            })
        } else {
            None
        };
        
        user_service.generate_user_token(
            firebase_user.uid.clone(),
            firebase_user.email.clone().unwrap_or_default(),
            firebase_user.display_name.clone(),
            user_context,
            permissions.to_vec(),
            subscription,
        ).map_err(|e| {
            tracing::error!("Failed to generate user access token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenErrorResponse {
                    error: "server_error".to_string(),
                    error_description: Some("Failed to generate user access token".to_string()),
                    error_uri: None,
                }),
            )
        })
    }

    /// Get comprehensive user database information for JWT token generation
    async fn get_user_database_info(
        &self,
        app_state: &AppState,
        firebase_uid: &str,
        _email: &str
    ) -> Result<(Vec<String>, String, Vec<String>), Box<dyn std::error::Error>> {
        let _admin_modules_placeholder = vec![];
        
        let firebase_uid_vo = crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid)
            .map_err(|e| format!("Invalid Firebase UID: {}", e))?;
            
        let (package_tier, permissions) = match app_state.user_repo.find_by_firebase_uid(&firebase_uid_vo).await {
            Ok(Some(_user)) => {
                let tier = "FREE".to_string();
                
                let user_permissions: Vec<String> = match self.get_user_role_from_db(app_state, firebase_uid).await {
                    Ok(Some(role)) => {
                        match role.as_str() {
                            "admin" => vec!["admin:*:*".to_string()],
                            "user" => vec!["epsx:basic:read".to_string()],
                            _ => vec!["epsx:basic:read".to_string()],
                        }
                    },
                    Ok(None) => {
                        tracing::warn!("No role found for user {}, using default permissions", firebase_uid);
                        vec!["epsx:basic:read".to_string()]
                    },
                    Err(e) => {
                        tracing::error!("Failed to get user role for {}: {}", firebase_uid, e);
                        vec!["epsx:basic:read".to_string()]
                    }
                };
                
                tracing::debug!("Retrieved user data for {}: tier={}, {} permissions: {:?}", 
                    firebase_uid, tier, user_permissions.len(), user_permissions);
                (tier, user_permissions)
            },
            Ok(None) => {
                tracing::warn!("User not found in database for Firebase UID: {}", firebase_uid);
                ("FREE".to_string(), vec![])
            },
            Err(e) => {
                tracing::error!("Database error getting user data for {}: {}", firebase_uid, e);
                ("FREE".to_string(), vec![])
            }
        };
        
        tracing::info!("User database info for {}: {} tier, {} permissions", 
                      firebase_uid, package_tier, permissions.len());
        
        Ok((_admin_modules_placeholder, package_tier, permissions))
    }

    /// Get user role directly from database
    async fn get_user_role_from_db(
        &self,
        app_state: &AppState,
        firebase_uid: &str
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let query = "SELECT role FROM users WHERE firebase_uid = $1";
        
        match sqlx::query_scalar::<_, Option<String>>(query)
            .bind(firebase_uid)
            .fetch_optional(&*app_state.db_pool)
            .await
        {
            Ok(role) => {
                tracing::debug!("Retrieved role for {}: {:?}", firebase_uid, role);
                Ok(role.flatten())
            },
            Err(e) => {
                tracing::error!("Database error getting role for {}: {}", firebase_uid, e);
                Err(Box::new(e))
            }
        }
    }

    fn get_role_from_custom_claims(&self, custom_claims: &HashMap<String, serde_json::Value>) -> String {
        custom_claims.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string()
    }

    fn get_user_permissions_from_role(&self, custom_claims: &HashMap<String, serde_json::Value>) -> Vec<String> {
        let role = self.get_role_from_custom_claims(custom_claims);
        
        match role.as_str() {
            "admin" => vec![
                "api:admin:*".to_string(),
                "route:*".to_string(),
                "users:manage".to_string(),
                "system:configure".to_string(),
                "security:full".to_string(),
            ],
            "moderator" => vec![
                "api:moderate:*".to_string(),
                "route:/moderate/*".to_string(),
                "content:moderate".to_string(),
                "users:view".to_string(),
            ],
            "premium" => vec![
                "api:premium:*".to_string(),
                "route:/premium/*".to_string(),
                "analytics:read".to_string(),
                "alerts:manage".to_string(),
            ],
            _ => vec![
                "api:basic:read".to_string(),
                "route:/dashboard".to_string(),
                "profile:manage:own".to_string(),
            ],
        }
    }

    /// Detect if user should get admin-level JWT tokens
    fn is_admin_user(&self, role: &str, permissions: &[String], scope: &str) -> bool {
        let is_admin_role = matches!(role, "admin" | "super_admin" | "moderator");
        let has_admin_permissions = permissions.iter().any(|p| {
            p.starts_with("admin:") || p.starts_with("system:") || p.contains("admin")
        });
        let has_admin_scope = scope.contains("admin") || scope.contains("system");
        
        is_admin_role || has_admin_permissions || has_admin_scope
    }
}

impl Default for TokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions

fn get_issuer_url() -> String {
    get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
}

fn get_jwt_secret() -> String {
    get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string())
}

/// Generate a unique JWT ID (JTI) for token revocation support
fn generate_jti() -> String {
    Uuid::new_v4().to_string()
}

/// Determine features available for subscription tier
fn determine_features_for_tier(tier: &str) -> Vec<String> {
    match tier {
        "ENTERPRISE" => vec![
            "premium_analytics".to_string(),
            "api_access".to_string(),
            "custom_alerts".to_string(),
            "priority_support".to_string(),
            "white_label".to_string(),
        ],
        "PREMIUM" => vec![
            "premium_analytics".to_string(),
            "api_access".to_string(),
            "custom_alerts".to_string(),
            "priority_support".to_string(),
        ],
        "BASIC" => vec![
            "basic_analytics".to_string(),
            "standard_alerts".to_string(),
        ],
        _ => vec!["basic_access".to_string()],
    }
}

/// Determine usage limits for subscription tier
fn determine_limits_for_tier(tier: &str) -> std::collections::HashMap<String, u32> {
    use std::collections::HashMap;
    
    let mut limits = HashMap::new();
    match tier {
        "ENTERPRISE" => {
            limits.insert("api_calls_per_hour".to_string(), 10000);
            limits.insert("data_exports_per_day".to_string(), 100);
            limits.insert("custom_alerts".to_string(), 1000);
        }
        "PREMIUM" => {
            limits.insert("api_calls_per_hour".to_string(), 1000);
            limits.insert("data_exports_per_day".to_string(), 20);
            limits.insert("custom_alerts".to_string(), 100);
        }
        "BASIC" => {
            limits.insert("api_calls_per_hour".to_string(), 100);
            limits.insert("data_exports_per_day".to_string(), 5);
            limits.insert("custom_alerts".to_string(), 10);
        }
        _ => {
            limits.insert("api_calls_per_hour".to_string(), 20);
            limits.insert("data_exports_per_day".to_string(), 1);
            limits.insert("custom_alerts".to_string(), 1);
        }
    }
    limits
}