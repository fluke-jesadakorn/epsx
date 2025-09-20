use crate::domain::shared_kernel::value_objects::UserId;
use std::collections::HashMap;
// Multi-Provider Token Broker
// Core orchestration component for handling authentication across multiple providers

use jsonwebtoken::{ EncodingKey, Algorithm, Header, encode };
use serde::{ Serialize, Deserialize };
use chrono::{ DateTime, Utc, Duration };

use crate::config::env::get_env_var;


// Casbin service removed - using modern JWT auth
use super::providers::{

  ProviderRegistry,
  UserClaims,
  AuthProviderError,
  ProviderType,
};
use super::providers::firebase_provider::FirebaseProvider;

use super::providers::oidc_provider::OIDCProvider;


/// Unified JWT claims issued by the token broker
/// These JWTs are what the frontend and backend will use consistently
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedJWTClaims {
  /// Standard JWT claims
  pub sub: String, // user_id
  pub iss: String, // issuer (our backend)
  pub aud: String, // audience (client_id)
  pub iat: i64, // issued at
  pub exp: i64, // expires at
  pub jti: String, // JWT ID for revocation

  /// Custom claims for our application
  pub email: String,
  pub permissions: Vec<String>,
  pub subscription_tier: Option<String>,

  /// Provider information
  pub provider: String, // Which provider authenticated the user
  pub provider_user_id: String, // Original provider user ID

  /// Session information
  pub session_id: String, // Backend session ID
  pub session_type: String, // "unified" for broker-issued tokens
}

/// Configuration for the token broker
#[derive(Debug, Clone)]
pub struct TokenBrokerConfig {
  /// JWT signing secret
  pub jwt_secret: String,
  /// Issuer URL (our backend)
  pub issuer_url: String,
  /// Default audience
  pub default_audience: String,
  /// Token validity duration
  pub token_ttl_hours: i64,
  /// Refresh token TTL
  pub refresh_token_ttl_days: i64,
  /// Enable provider auto-detection
  pub auto_detect_provider: bool,
}

impl Default for TokenBrokerConfig {
  fn default() -> Self {
    Self {
      jwt_secret: get_env_var("NEXTAUTH_SECRET")
        .or_else(|_| get_env_var("AUTH_SECRET"))
        .unwrap_or_else(|_| "default-broker-secret".to_string()),
      issuer_url: get_env_var("BACKEND_URL").unwrap_or_else(|_|
        "http://localhost:8080".to_string()
      ),
      default_audience: "frontend-client".to_string(),
      token_ttl_hours: 24,
      refresh_token_ttl_days: 30,
      auto_detect_provider: true,
    }
  }
}

/// Multi-provider token broker
/// Central component that orchestrates authentication across different providers
pub struct TokenBroker {
  /// Provider registry with all available auth providers
  provider_registry: ProviderRegistry,
  /// JWT encoding key for unified tokens
  encoding_key: EncodingKey,
  /// Casbin service for permission resolution
  // casbin_service: Arc<CasbinService>, // Removed
  /// Broker configuration
  config: TokenBrokerConfig,
}

impl TokenBroker {
  /// Create new token broker with configuration
  pub fn new(
    config: TokenBrokerConfig
    // casbin_service: Arc<CasbinService>, // Removed
  ) -> Self {
    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());

    Self {
      provider_registry: ProviderRegistry::new(),
      encoding_key,
      // casbin_service, // Removed
      config,
    }
  }

  /// Register authentication providers
  pub fn with_providers(mut self) -> Self {
    // Register Firebase provider
    let firebase_provider = FirebaseProvider::new();

    // Register OIDC provider
    let oidc_config =
      super::providers::oidc_provider::OIDCProviderConfig::default();
    let oidc_provider = OIDCProvider::new(oidc_config);

    self.provider_registry = self.provider_registry
      .register(firebase_provider)
      .register(oidc_provider);

    self
  }

  /// Process a token from any provider and return a unified JWT
  /// This is the main entry point for token processing
  pub async fn process_token(
    &self,
    token: &str,
    provider_hint: Option<ProviderType>
  ) -> Result<UnifiedJWT, AuthProviderError> {
    // Step 1: Find the appropriate provider
    let provider = if let Some(hint) = provider_hint {
      self.provider_registry
        .get_provider(&hint)
        .ok_or_else(||
          AuthProviderError::ConfigurationError(
            format!("Provider {:?} not found", hint)
          )
        )?
    } else if self.config.auto_detect_provider {
      self.provider_registry
        .find_provider_for_token(token)
        .ok_or_else(|| AuthProviderError::InvalidTokenFormat)?
    } else {
      return Err(
        AuthProviderError::ConfigurationError(
          "No provider specified and auto-detection disabled".to_string()
        )
      );
    };

    tracing::info!(
      "Processing token with provider: {}",
      provider.provider_name()
    );

    // Step 2: Validate token with the selected provider
    let user_claims = provider.validate_token(token).await?;

    // Step 3: Enhance claims with Casbin permissions if needed
    let enhanced_claims = self.enhance_with_casbin(&user_claims).await?;

    // Step 4: Issue unified JWT
    let unified_jwt = self.issue_unified_jwt(enhanced_claims).await?;

    Ok(unified_jwt)
  }

  /// Enhance user claims with additional Casbin permissions
  async fn enhance_with_casbin(
    &self,
    claims: &UserClaims
  ) -> Result<UserClaims, AuthProviderError> {
    // Get additional permissions from Casbin for the user
    // This allows for dynamic permission assignment beyond what the provider knows

    let enhanced_claims = claims.clone();

    // TODO: Query Casbin for additional permissions
    // For now, keep the original permissions from the provider
    tracing::debug!(
      "User {} has {} permissions from provider",
      claims.user_id,
      claims.permissions.len()
    );

    Ok(enhanced_claims)
  }

  /// Issue a unified JWT token that can be used across the application
  async fn issue_unified_jwt(
    &self,
    claims: UserClaims
  ) -> Result<UnifiedJWT, AuthProviderError> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(self.config.token_ttl_hours);
    let jti = uuid::Uuid::new_v4().to_string();
    let session_id = uuid::Uuid::new_v4().to_string();

    let jwt_claims = UnifiedJWTClaims {
      // Standard claims
      sub: claims.user_id.to_string(),
      iss: self.config.issuer_url.clone(),
      aud: self.config.default_audience.clone(),
      iat: now.timestamp(),
      exp: expires_at.timestamp(),
      jti: jti.clone(),

      // Application claims
      email: claims.email.to_string(),
      permissions: claims.permissions,
      subscription_tier: claims.extra_claims
        .get("subscription_tier")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()),

      // Provider information
      provider: claims.provider.to_string(),
      provider_user_id: claims.provider_user_id,

      // Session information
      session_id: session_id.clone(),
      session_type: "unified".to_string(),
    };

    // Create JWT header
    let header = Header::new(Algorithm::HS256);

    // Encode the JWT
    let token = encode(&header, &jwt_claims, &self.encoding_key).map_err(|e|
      AuthProviderError::InternalError(format!("JWT encoding failed: {}", e))
    )?;

    let unified_jwt = UnifiedJWT {
      access_token: token,
      token_type: "Bearer".to_string(),
      expires_at,
      expires_in: self.config.token_ttl_hours * 3600, // seconds
      session_id: session_id.clone(),
      jti,
      refresh_token: Some(self.generaterefresh_token(&claims.user_id.to_string(), &session_id)?),
    };

    tracing::info!(
      "Issued unified JWT for user {} via JWT system",
      claims.user_id.to_string()
    );

    Ok(unified_jwt)
  }

  /// Validate a token using multi-provider approach
  /// Returns UserClaims for compatibility with casbin middleware
  pub async fn validate_token(
    &self,
    token: &str
  ) -> Result<UserClaims, AuthProviderError> {
    // First try to validate as a unified JWT
    if let Ok(jwt_claims) = self.validate_unified_jwt(token).await {
      // Convert UnifiedJWTClaims to UserClaims
      return Ok(UserClaims::new(
        UserId::from_string(jwt_claims.sub).map_err(|e| AuthProviderError::TokenValidationFailed(format!("Invalid user ID: {}", e)))?,
        crate::domain::shared_kernel::value_objects::Email
          ::new(jwt_claims.email)
          .unwrap_or_else(|_|
            crate::domain::shared_kernel::value_objects::Email
              ::new("unknown@example.com".to_string())
              .unwrap()
          ),
        jwt_claims.permissions,
        jwt_claims.provider_user_id,
        ProviderType::OIDC, // Default to OIDC for unified JWTs
        DateTime::from_timestamp(jwt_claims.exp, 0).unwrap_or_else(
          || Utc::now()
        ),
        jwt_claims.iat as u64,
        jwt_claims.exp as u64,
        jwt_claims.subscription_tier,
      ));
    }

    // If not a unified JWT, try to process through providers
    match self.process_token(token, None).await {
      Ok(unified_jwt) => {
        // Extract claims from the unified JWT token
        self
          .validate_unified_jwt(&unified_jwt.access_token).await
          .map(|jwt_claims| UserClaims::new(
            UserId::from_string(jwt_claims.sub).unwrap_or_else(|_| UserId::new()),
            crate::domain::shared_kernel::value_objects::Email
              ::new(jwt_claims.email)
              .unwrap_or_else(|_|
                crate::domain::shared_kernel::value_objects::Email
                  ::new("unknown@example.com".to_string())
                  .unwrap()
              ),
            jwt_claims.permissions,
            jwt_claims.provider_user_id,
            ProviderType::OIDC,
            DateTime::from_timestamp(
              jwt_claims.exp,
              0
            ).unwrap_or_else(|| Utc::now()),
            jwt_claims.iat as u64,
            jwt_claims.exp as u64,
            jwt_claims.subscription_tier,
          ))
      }
      Err(e) => Err(e),
    }
  }

  /// Validate a unified JWT issued by this broker
  pub async fn validate_unified_jwt(
    &self,
    token: &str
  ) -> Result<UnifiedJWTClaims, AuthProviderError> {
    use jsonwebtoken::{ decode, DecodingKey, Validation, Algorithm };

    let decoding_key = DecodingKey::from_secret(
      self.config.jwt_secret.as_bytes()
    );
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[&self.config.issuer_url]);
    validation.set_audience(&[&self.config.default_audience]);

    let token_data = decode::<UnifiedJWTClaims>(
      token,
      &decoding_key,
      &validation
    ).map_err(|e| {
      tracing::error!("Unified JWT validation failed: {}", e);
      match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature =>
          AuthProviderError::TokenExpired,
        jsonwebtoken::errors::ErrorKind::InvalidToken =>
          AuthProviderError::InvalidTokenFormat,
        _ =>
          AuthProviderError::TokenValidationFailed(
            format!("JWT validation failed: {}", e)
          ),
      }
    })?;

    Ok(token_data.claims)
  }

  /// Get provider statistics for monitoring
  pub fn get_provider_stats(&self) -> HashMap<String, serde_json::Value> {
    let mut stats = HashMap::new();

    for provider in self.provider_registry.providers() {
      stats.insert(
        provider.provider_name().to_string(),
        serde_json::json!({
                    "type": provider.provider_type(),
                    "priority": provider.priority(),
                    "name": provider.provider_name()
                })
      );
    }

    stats.insert(
      "total_providers".to_string(),
      serde_json::Value::Number(self.provider_registry.providers().len().into())
    );

    stats
  }

  /// Generate a refresh token for a user session
  fn generaterefresh_token(
    &self,
    user_id: &str,
    session_id: &str
  ) -> Result<String, AuthProviderError> {
    use crate::auth::jwt::RefreshTokenClaims;
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let expires_at = now + Duration::days(7); // 7 days for refresh tokens

    let claims = RefreshTokenClaims {
      sub: user_id.to_string(),
      session_id: session_id.to_string(),
      iat: now.timestamp(),
      exp: expires_at.timestamp(),
      aud: "refresh".to_string(),
      iss: "epsx-backend".to_string(),
      token_type: "refresh".to_string(),
    };

    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &self.encoding_key)
      .map_err(|e| AuthProviderError::InternalError(format!("Refresh token encoding failed: {}", e)))
  }

  /// Refresh an access token using a refresh token
  pub async fn refresh_access_token(
    &self,
    refresh_token: &str
  ) -> Result<UnifiedJWT, AuthProviderError> {
    use crate::auth::jwt::RefreshTokenClaims;
    use jsonwebtoken::{decode, DecodingKey, Validation};

    // Decode and validate refresh token
    let decoding_key = DecodingKey::from_secret(self.config.jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["refresh"]);
    validation.set_issuer(&["epsx-backend"]);

    let token_data = decode::<RefreshTokenClaims>(
      refresh_token, 
      &decoding_key, 
      &validation
    ).map_err(|e| AuthProviderError::ValidationError(format!("Invalid refresh token: {}", e)))?;

    let refresh_claims = token_data.claims;

    // TODO: Verify session is still valid in database
    // For now, proceed with token refresh

    // Create new access token with same user data
    // This is a simplified flow - in production you'd fetch fresh user data
    let now = Utc::now();
    let expires_at = now + Duration::hours(self.config.token_ttl_hours);
    let jti = uuid::Uuid::new_v4().to_string();

    let permissions = vec!["epsx:dashboard:read".to_string()];
    
    let jwt_claims = crate::auth::jwt::Claims {
      sub: refresh_claims.sub.clone(),
      iss: "epsx-backend".to_string(),
      aud: "epsx-api".to_string(),
      exp: expires_at.timestamp() as usize,
      iat: now.timestamp() as usize,
      nbf: now.timestamp() as usize,
      jti: jti.clone(),
      
      // User information (in production, fetch from database)
      email: format!("user+{}@{}", refresh_claims.sub, 
        get_env_var("DEFAULT_EMAIL_DOMAIN").unwrap_or_else(|_| "epsx.io".to_string())), // Placeholder
      name: None,
      
      // ENHANCED: Embedded permissions with versioning
      permissions: permissions.clone(),
      permission_version: 1, // Default version for token broker
      permission_last_updated: now.timestamp() as u64,
      
      // ENHANCED: Pre-computed context (fallback values for token broker)
      platforms: crate::auth::jwt::derive_accessible_platforms_from_permissions(&permissions),
      verified: false, // Conservative default
      
      // ENHANCED: Refresh hints
      needs_refresh: false, // Token broker generates longer-lived tokens
      refresh_after: Some((expires_at.timestamp() - 300) as u64), // Refresh 5 min before expiry
    };

    let header = Header::new(Algorithm::HS256);
    let access_token = encode(&header, &jwt_claims, &self.encoding_key)
      .map_err(|e| AuthProviderError::InternalError(format!("Access token encoding failed: {}", e)))?;

    Ok(UnifiedJWT {
      access_token,
      token_type: "Bearer".to_string(),
      expires_at,
      expires_in: self.config.token_ttl_hours * 3600,
      session_id: refresh_claims.session_id.clone(),
      jti,
      refresh_token: Some(self.generaterefresh_token(&refresh_claims.sub, &refresh_claims.session_id)?),
    })
  }
}

/// Unified JWT response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedJWT {
  pub access_token: String,
  pub token_type: String,
  pub expires_at: DateTime<Utc>,
  pub expires_in: i64, // seconds
  pub session_id: String,
  pub jti: String,
  pub refresh_token: Option<String>,
}
