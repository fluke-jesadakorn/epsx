/// Refresh Token Handler
/// 
/// Handles refresh token operations including validation, rotation, and new token generation.
/// Implements secure token rotation to prevent token reuse attacks.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use axum::{http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

use crate::web::auth::AppState;
use crate::infrastructure::adapters::services::firebase::FirebaseUser;
use super::{TokenErrorResponse, token_generator::TokenGenerator};

/// Refresh token data stored in Redis
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenData {
    pub firebase_user: FirebaseUser,
    pub client_id: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
}

/// Token request parameters for refresh grant
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub grant_type: String,
    pub refresh_token: String,
    pub client_id: Option<String>,
    pub scope: Option<String>,
}

use super::token::TokenResponse;

/// Refresh token handler service
pub struct RefreshHandler {
    token_generator: TokenGenerator,
}

impl RefreshHandler {
    pub fn new() -> Self {
        Self {
            token_generator: TokenGenerator::new(),
        }
    }

    /// Handle refresh token grant with secure token rotation
    pub async fn handle_refresh_token_grant(
        &self,
        _app_state: AppState,
        refresh_token: String,
        client_id: String,
    ) -> Result<Json<TokenResponse>, (StatusCode, Json<TokenErrorResponse>)> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        // Validate and rotate the refresh token using our new service
        let rotation = REFRESH_TOKEN_SERVICE.rotaterefresh_token(
            &refresh_token,
            None, // TODO: Add device info from request headers
        ).await.map_err(|e| {
            tracing::error!("Refresh token rotation failed: {}", e);
            
            let (error_code, error_description) = match e {
                crate::auth::RefreshTokenError::TokenNotFound { .. } => {
                    ("invalid_grant", "Refresh token not found")
                }
                crate::auth::RefreshTokenError::TokenExpired { .. } => {
                    ("invalid_grant", "Refresh token has expired")
                }
                crate::auth::RefreshTokenError::TokenRevoked { .. } => {
                    ("invalid_grant", "Refresh token has been revoked")
                }
                crate::auth::RefreshTokenError::RotationLimitExceeded { .. } => {
                    ("invalid_grant", "Token rotation limit exceeded")
                }
                _ => ("server_error", "Internal server error during token refresh")
            };
            
            (
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: error_code.to_string(),
                    error_description: Some(error_description.to_string()),
                    error_uri: None,
                }),
            )
        })?;

        // Validate client_id matches the stored data
        if rotation.new_token_data.client_id != client_id {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(TokenErrorResponse {
                    error: "invalid_client".to_string(),
                    error_description: Some("Client ID mismatch".to_string()),
                    error_uri: None,
                }),
            ));
        }

        // Get user data for token generation
        let firebase_user = self.create_firebase_user_from_token_data(&rotation.new_token_data);

        // Generate new access and ID tokens
        let now = Utc::now();
        let expires_in = 7200; // 2 hours (frontend client policy)

        let access_token = self.token_generator.generate_access_token_simple(
            &firebase_user, 
            &rotation.new_token_data.scope, 
            now, 
            expires_in
        )?;
        
        let id_token = self.token_generator.generate_id_token_simple(
            &firebase_user, 
            &client_id, 
            now, 
            expires_in
        )?;

        tracing::info!(
            old_token_id = %rotation.old_token_id,
            new_token_id = %rotation.new_token_data.token_id,
            user_id = %rotation.new_token_data.user_id,
            client_id = %client_id,
            rotation_count = %rotation.new_token_data.rotation_count,
            "Refresh token rotated successfully"
        );

        Ok(Json(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            id_token,
            refresh_token: Some(rotation.new_token), // Return the NEW rotated refresh token
            scope: rotation.new_token_data.scope,
        }))
    }

    /// Validate refresh token request parameters
    pub fn validate_refresh_request(
        &self,
        request: &RefreshTokenRequest,
    ) -> Result<(), (StatusCode, Json<TokenErrorResponse>)> {
        // Validate grant type
        if request.grant_type != "refresh_token" {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(TokenErrorResponse {
                    error: "unsupported_grant_type".to_string(),
                    error_description: Some("Only refresh_token grant type is supported".to_string()),
                    error_uri: None,
                }),
            ));
        }

        // Validate refresh token is present
        if request.refresh_token.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(TokenErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some("refresh_token parameter is required".to_string()),
                    error_uri: None,
                }),
            ));
        }

        // Validate refresh token format (basic check)
        if request.refresh_token.len() < 32 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(TokenErrorResponse {
                    error: "invalid_request".to_string(),
                    error_description: Some("Invalid refresh token format".to_string()),
                    error_uri: None,
                }),
            ));
        }

        Ok(())
    }

    /// Revoke a refresh token
    pub async fn revoke_refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<(), (StatusCode, Json<TokenErrorResponse>)> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        // Revoke the refresh token using the service
        REFRESH_TOKEN_SERVICE.revoke_refresh_token(refresh_token, "user_request", "Token revocation requested")
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke refresh token: {}", e);
                
                let (error_code, error_description) = match e {
                    crate::auth::RefreshTokenError::TokenNotFound { .. } => {
                        ("invalid_request", "Refresh token not found")
                    }
                    _ => ("server_error", "Internal server error during token revocation")
                };
                
                (
                    StatusCode::BAD_REQUEST,
                    Json(TokenErrorResponse {
                        error: error_code.to_string(),
                        error_description: Some(error_description.to_string()),
                        error_uri: None,
                    }),
                )
            })?;

        tracing::info!("Refresh token revoked successfully for client: {}", client_id);
        Ok(())
    }

    /// Get refresh token metadata
    pub async fn get_refresh_token_info(
        &self,
        refresh_token: &str,
    ) -> Result<crate::auth::RefreshTokenData, (StatusCode, Json<TokenErrorResponse>)> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        REFRESH_TOKEN_SERVICE.validate_refresh_token(refresh_token)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get refresh token info: {}", e);
                
                let (error_code, error_description) = match e {
                    crate::auth::RefreshTokenError::TokenNotFound { .. } => {
                        ("invalid_request", "Refresh token not found")
                    }
                    crate::auth::RefreshTokenError::TokenExpired { .. } => {
                        ("invalid_request", "Refresh token has expired")
                    }
                    _ => ("server_error", "Internal server error")
                };
                
                (
                    StatusCode::BAD_REQUEST,
                    Json(TokenErrorResponse {
                        error: error_code.to_string(),
                        error_description: Some(error_description.to_string()),
                        error_uri: None,
                    }),
                )
            })
    }

    /// Check if refresh token is valid and not expired
    pub async fn is_refresh_token_valid(
        &self,
        refresh_token: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        match REFRESH_TOKEN_SERVICE.validate_refresh_token(refresh_token).await {
            Ok(_) => Ok(true),
            Err(crate::auth::RefreshTokenError::TokenNotFound { .. }) => Ok(false),
            Err(crate::auth::RefreshTokenError::TokenExpired { .. }) => Ok(false),
            Err(crate::auth::RefreshTokenError::TokenRevoked { .. }) => Ok(false),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Get refresh token statistics
    pub async fn get_refresh_token_stats(
        &self,
        user_id: &str,
    ) -> Result<RefreshTokenStats, Box<dyn std::error::Error>> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        // Get general stats from the service
        let stats = REFRESH_TOKEN_SERVICE.get_stats().await;
        
        Ok(RefreshTokenStats {
            user_id: user_id.to_string(),
            total_active_tokens: stats.total_tokens as usize,
            average_rotation_count: 0.0, // Not available from current stats
            max_rotation_count: 0, // Not available from current stats
            oldest_token_age_hours: 0.0, // Not available from current stats
        })
    }

    // Private helper methods

    /// Helper function to create FirebaseUser from refresh token data
    fn create_firebase_user_from_token_data(&self, token_data: &crate::auth::RefreshTokenData) -> FirebaseUser {
        FirebaseUser {
            uid: token_data.user_id.clone(),
            email: Some(format!("user-{}@example.com", token_data.user_id)), // Placeholder
            email_verified: true,
            display_name: None,
            photo_url: None,
            provider_id: "custom".to_string(),
            custom_claims: HashMap::new(),
        }
    }

    // calculate_oldest_token_age method removed since we don't have access to individual tokens
}

impl Default for RefreshHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Refresh token statistics
#[derive(Debug, Serialize)]
pub struct RefreshTokenStats {
    pub user_id: String,
    pub total_active_tokens: usize,
    pub average_rotation_count: f64,
    pub max_rotation_count: u32,
    pub oldest_token_age_hours: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_refresh_request() {
        let handler = RefreshHandler::new();
        
        // Valid request
        let valid_request = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: "valid_refresh_token_here_1234567890".to_string(),
            client_id: Some("test-client".to_string()),
            scope: Some("read write".to_string()),
        };
        assert!(handler.validate_refresh_request(&valid_request).is_ok());
        
        // Invalid grant type
        let invalid_grant = RefreshTokenRequest {
            grant_type: "authorization_code".to_string(),
            refresh_token: "valid_refresh_token_here_1234567890".to_string(),
            client_id: Some("test-client".to_string()),
            scope: Some("read write".to_string()),
        };
        assert!(handler.validate_refresh_request(&invalid_grant).is_err());
        
        // Empty refresh token
        let empty_token = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: "".to_string(),
            client_id: Some("test-client".to_string()),
            scope: Some("read write".to_string()),
        };
        assert!(handler.validate_refresh_request(&empty_token).is_err());
        
        // Short refresh token
        let short_token = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: "short".to_string(),
            client_id: Some("test-client".to_string()),
            scope: Some("read write".to_string()),
        };
        assert!(handler.validate_refresh_request(&short_token).is_err());
    }

    #[test]
    fn test_calculate_oldest_token_age() {
        let handler = RefreshHandler::new();
        
        // Empty tokens
        let empty_tokens = vec![];
        assert_eq!(handler.calculate_oldest_token_age(&empty_tokens), 0.0);
        
        // Single token created 2 hours ago
        let now = Utc::now();
        let two_hours_ago = now - chrono::Duration::hours(2);
        let tokens = vec![
            crate::auth::RefreshTokenData {
                token_id: "test1".to_string(),
                user_id: "user1".to_string(),
                client_id: "client1".to_string(),
                scope: "read".to_string(),
                created_at: two_hours_ago,
                expires_at: now + chrono::Duration::days(30),
                last_used_at: None,
                rotation_count: 0,
                device_info: None,
                is_revoked: false,
            }
        ];
        
        let age = handler.calculate_oldest_token_age(&tokens);
        assert!((age - 2.0).abs() < 0.1); // Should be approximately 2 hours
    }
}