/// Token Introspection Service
/// 
/// Provides token introspection functionality for validating and getting metadata
/// about access tokens, ID tokens, and refresh tokens. Implements RFC 7662.

use axum::{http::StatusCode, response::Json, extract::{State, Form}};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::web::auth::AppState;
use super::{TokenErrorResponse, token_validator::TokenValidator};

/// Token introspection request (POST /oauth/introspect)
#[derive(Debug, Deserialize)]
pub struct IntrospectionRequest {
    pub token: String,
    pub token_type_hint: Option<String>, // "access_token", "refresh_token", "id_token"
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

/// Token introspection response (RFC 7662)
#[derive(Debug, Serialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    // Custom fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_tier: Option<String>,
}

/// Extended token metadata for internal use
#[derive(Debug, Serialize)]
pub struct TokenMetadata {
    pub token_type: String,
    pub is_active: bool,
    pub is_expired: bool,
    pub is_revoked: bool,
    pub subject: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub not_before: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

/// Token introspection service
pub struct IntrospectionService {
    token_validator: TokenValidator,
}

impl IntrospectionService {
    pub fn new() -> Self {
        Self {
            token_validator: TokenValidator::new(),
        }
    }

    /// POST /oauth/introspect - Token introspection endpoint (RFC 7662)
    pub async fn introspect_token(
        &self,
        _app_state: &AppState,
        request: IntrospectionRequest,
    ) -> Result<Json<IntrospectionResponse>, (StatusCode, Json<TokenErrorResponse>)> {
        tracing::debug!("Token introspection request for token_type_hint: {:?}", request.token_type_hint);

        // Validate token format first
        if let Err(e) = self.token_validator.validate_token_format(&request.token) {
            tracing::debug!("Token format validation failed: {}", e);
            return Ok(Json(IntrospectionResponse {
                active: false,
                ..Default::default()
            }));
        }

        // Determine token type and validate accordingly
        let introspection_result = match request.token_type_hint.as_deref() {
            Some("access_token") => self.introspect_access_token(&request.token).await,
            Some("id_token") => self.introspect_id_token(&request.token).await,
            Some("refresh_token") => self.introspect_refresh_token(&request.token).await,
            _ => {
                // Try to detect token type automatically
                self.introspect_unknown_token(&request.token).await
            }
        };

        match introspection_result {
            Ok(response) => {
                tracing::debug!("Token introspection successful: active={}", response.active);
                Ok(Json(response))
            }
            Err(e) => {
                tracing::debug!("Token introspection failed: {}", e);
                // Return inactive token response instead of error (per RFC 7662)
                Ok(Json(IntrospectionResponse {
                    active: false,
                    ..Default::default()
                }))
            }
        }
    }

    /// Get detailed token metadata (internal API)
    pub async fn get_token_metadata(
        &self,
        token: &str,
    ) -> Result<TokenMetadata, Box<dyn std::error::Error>> {
        // Try to validate as different token types
        if let Ok(claims) = self.token_validator.validate_access_token(token) {
            let now = Utc::now();
            let is_expired = self.token_validator.is_token_expired(claims.exp);
            let is_revoked = self.token_validator.is_token_revoked(&claims.jti).unwrap_or(false);

            return Ok(TokenMetadata {
                token_type: "access_token".to_string(),
                is_active: !is_expired && !is_revoked,
                is_expired,
                is_revoked,
                subject: claims.sub,
                email: claims.email,
                permissions: claims.permissions,
                package_tier: claims.package_tier,
                issued_at: DateTime::from_timestamp(claims.iat, 0).unwrap_or(now),
                expires_at: DateTime::from_timestamp(claims.exp, 0).unwrap_or(now),
                not_before: DateTime::from_timestamp(claims.nbf, 0).unwrap_or(now),
                last_used: None, // Would need to track this separately
            });
        }

        // Try refresh token
        if let Ok(_token_data) = self.get_refresh_token_data(token).await {
            return Ok(TokenMetadata {
                token_type: "refresh_token".to_string(),
                is_active: true, // If we can get data, it's active
                is_expired: false,
                is_revoked: false,
                subject: _token_data.user_id.clone(),
                email: format!("user-{}@example.com", _token_data.user_id), // Placeholder
                permissions: vec![],
                package_tier: "FREE".to_string(),
                issued_at: _token_data.created_at,
                expires_at: _token_data.expires_at,
                not_before: _token_data.created_at,
                last_used: _token_data.used_at,
            });
        }

        Err("Unable to determine token type or validate token".into())
    }

    /// Check if token is currently active
    pub async fn is_token_active(&self, token: &str) -> bool {
        match self.get_token_metadata(token).await {
            Ok(metadata) => metadata.is_active,
            Err(_) => false,
        }
    }

    /// Get token permissions
    pub async fn get_token_permissions(&self, token: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Try unified access token validation first
        if let Ok((_, _, permissions, _)) = self.token_validator.validate_unified_access_token(token) {
            return Ok(permissions);
        }

        // Try legacy access token
        if let Ok(claims) = self.token_validator.validate_access_token(token) {
            return Ok(claims.permissions);
        }

        Err("Unable to extract permissions from token".into())
    }

    /// Get token expiration info
    pub async fn get_token_expiration(&self, token: &str) -> Result<(DateTime<Utc>, bool), Box<dyn std::error::Error>> {
        let metadata = self.get_token_metadata(token).await?;
        Ok((metadata.expires_at, metadata.is_expired))
    }

    // Private helper methods

    /// Introspect access token
    async fn introspect_access_token(&self, token: &str) -> Result<IntrospectionResponse, Box<dyn std::error::Error>> {
        // Try unified token validation first
        if let Ok((sub, email, permissions, package_tier)) = self.token_validator.validate_unified_access_token(token) {
            return Ok(IntrospectionResponse {
                active: true,
                scope: Some("read write".to_string()), // Default scope
                token_type: Some("Bearer".to_string()),
                sub: Some(sub),
                email: Some(email),
                permissions: Some(permissions),
                package_tier: Some(package_tier),
                ..Default::default()
            });
        }

        // Try legacy access token
        let claims = self.token_validator.validate_access_token(token)?;
        let is_expired = self.token_validator.is_token_expired(claims.exp);
        let is_revoked = self.token_validator.is_token_revoked(&claims.jti).unwrap_or(false);

        Ok(IntrospectionResponse {
            active: !is_expired && !is_revoked,
            scope: Some(claims.scope),
            token_type: Some("Bearer".to_string()),
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            nbf: Some(claims.nbf),
            sub: Some(claims.sub),
            aud: Some(claims.aud),
            iss: Some(claims.iss),
            jti: Some(claims.jti),
            email: Some(claims.email),
            role: Some(claims.role),
            permissions: Some(claims.permissions),
            package_tier: Some(claims.package_tier),
            ..Default::default()
        })
    }

    /// Introspect ID token
    async fn introspect_id_token(&self, token: &str) -> Result<IntrospectionResponse, Box<dyn std::error::Error>> {
        // ID tokens need audience validation, so we'll use a placeholder
        let claims = self.token_validator.validate_id_token(token, "epsx-frontend")?;
        let is_expired = self.token_validator.is_token_expired(claims.exp);
        let is_revoked = self.token_validator.is_token_revoked(&claims.jti).unwrap_or(false);

        Ok(IntrospectionResponse {
            active: !is_expired && !is_revoked,
            token_type: Some("id_token".to_string()),
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            nbf: Some(claims.nbf),
            sub: Some(claims.sub),
            aud: Some(claims.aud),
            iss: Some(claims.iss),
            jti: Some(claims.jti),
            email: Some(claims.email),
            role: Some(claims.role),
            permissions: Some(claims.permissions),
            package_tier: Some(claims.package_tier),
            ..Default::default()
        })
    }

    /// Introspect refresh token
    async fn introspect_refresh_token(&self, token: &str) -> Result<IntrospectionResponse, Box<dyn std::error::Error>> {
        let token_data = self.get_refresh_token_data(token).await?;
        
        Ok(IntrospectionResponse {
            active: true, // If we can get data, it's active
            token_type: Some("refresh_token".to_string()),
            sub: Some(token_data.user_id),
            client_id: Some(token_data.client_id),
            scope: Some(token_data.scope),
            iat: Some(token_data.created_at.timestamp()),
            exp: Some(token_data.expires_at.timestamp()),
            ..Default::default()
        })
    }

    /// Try to detect and introspect unknown token type
    async fn introspect_unknown_token(&self, token: &str) -> Result<IntrospectionResponse, Box<dyn std::error::Error>> {
        // Try access token first (most common)
        if let Ok(response) = self.introspect_access_token(token).await {
            return Ok(response);
        }

        // Try refresh token
        if let Ok(response) = self.introspect_refresh_token(token).await {
            return Ok(response);
        }

        // Try ID token (least likely due to audience validation)
        if let Ok(response) = self.introspect_id_token(token).await {
            return Ok(response);
        }

        Err("Unable to introspect token: unknown or invalid token type".into())
    }

    /// Get refresh token data from service
    async fn get_refresh_token_data(&self, token: &str) -> Result<crate::auth::RefreshTokenData, Box<dyn std::error::Error>> {
        use crate::auth::REFRESH_TOKEN_SERVICE;
        
        REFRESH_TOKEN_SERVICE.validate_refresh_token(token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

impl Default for IntrospectionService {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for IntrospectionResponse {
    fn default() -> Self {
        Self {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: None,
            iat: None,
            nbf: None,
            sub: None,
            aud: None,
            iss: None,
            jti: None,
            email: None,
            role: None,
            permissions: None,
            package_tier: None,
        }
    }
}

/// Axum handler for token introspection endpoint
pub async fn oidc_introspect(
    State(app_state): State<AppState>,
    Form(request): Form<IntrospectionRequest>,
) -> Result<Json<IntrospectionResponse>, (StatusCode, Json<TokenErrorResponse>)> {
    let service = IntrospectionService::new();
    service.introspect_token(&app_state, request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_introspection_response_serialization() {
        let response = IntrospectionResponse {
            active: true,
            token_type: Some("Bearer".to_string()),
            sub: Some("user123".to_string()),
            email: Some("user@example.com".to_string()),
            exp: Some(1234567890),
            ..Default::default()
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"token_type\":\"Bearer\""));
        assert!(json.contains("\"sub\":\"user123\""));
        assert!(!json.contains("\"aud\":null")); // Should skip None fields
    }

    #[test]
    fn test_inactive_token_response() {
        let response = IntrospectionResponse {
            active: false,
            ..Default::default()
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"active\":false"));
        // Should only contain active field when inactive
        assert!(!json.contains("\"token_type\""));
        assert!(!json.contains("\"sub\""));
    }

    #[tokio::test]
    async fn test_token_metadata_creation() {
        let service = IntrospectionService::new();
        
        // Test with invalid token
        let result = service.get_token_metadata("invalid.token.here").await;
        assert!(result.is_err());
    }
}