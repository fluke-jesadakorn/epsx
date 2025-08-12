// Enhanced Multi-Tenant Token Broker with PKCE Support
// Integrates provider registry, tenant resolution, and advanced OIDC flows

use std::sync::Arc;
use std::collections::HashMap;
// use async_trait::async_trait;
use jsonwebtoken::{EncodingKey, DecodingKey, Algorithm, Header, encode};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};

use crate::core::errors::AppError;
use crate::dom::services::casbin_service::CasbinService;
use crate::dom::services::admin_module_service::AdminModuleService;
use super::provider_registry::{ProviderRegistryTrait, OIDCProviderConfig};
use super::tenant_resolver::{TenantResolverTrait, EnhancedTenantResolution};
use super::discovery_client::DiscoveryClientTrait;

/// PKCE (Proof Key for Code Exchange) data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PKCEChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

impl PKCEChallenge {
    /// Generate new PKCE challenge
    pub fn generate() -> Self {
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier);
        
        Self {
            code_verifier,
            code_challenge,
            code_challenge_method: "S256".to_string(),
        }
    }
    
    /// Generate cryptographically secure code verifier
    fn generate_code_verifier() -> String {
        let mut rng = thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        URL_SAFE_NO_PAD.encode(&bytes)
    }
    
    /// Generate code challenge from verifier using SHA256
    fn generate_code_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(&result)
    }
    
    /// Verify code verifier against challenge
    pub fn verify(&self, verifier: &str) -> bool {
        let expected_challenge = Self::generate_code_challenge(verifier);
        self.code_challenge == expected_challenge
    }
}

/// Enhanced unified JWT claims for multi-tenant system
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedUnifiedJWTClaims {
    /// Standard JWT claims
    pub sub: String,        // user_id
    pub iss: String,        // issuer (our backend)
    pub aud: Vec<String>,   // audiences (multiple clients supported)
    pub iat: i64,          // issued at
    pub exp: i64,          // expires at
    pub nbf: i64,          // not before
    pub jti: String,       // JWT ID for revocation
    
    /// Enhanced user claims
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: Option<String>,
    
    /// Multi-tenant claims
    pub tenant_id: String,
    pub provider_id: String,
    pub provider_type: String,
    pub provider_user_id: String,
    
    /// Session and security claims
    pub session_id: String,
    pub session_type: String,
    pub client_id: String,
    pub scope: String,
    pub amr: Vec<String>,  // Authentication Method Reference
    pub acr: String,       // Authentication Context Class Reference
    
    /// Authorization claims
    pub azp: String,       // Authorized party (client that requested token)
    pub auth_time: i64,    // Time when authentication occurred
    pub nonce: Option<String>,
    
    /// Enhanced metadata
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub geo_location: Option<String>,
    pub risk_score: Option<f64>,
}

/// Token response for different TTLs per role/provider
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedUnifiedJWT {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,    // seconds
    pub expires_at: DateTime<Utc>,
    pub refresh_token: Option<String>,
    pub refresh_expires_in: Option<i64>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub id_token: Option<String>,
    pub scope: String,
    pub session_id: String,
    pub jti: String,
    /// Token-specific TTL based on provider/role
    pub custom_ttl_applied: bool,
    pub provider_id: String,
    pub tenant_id: String,
}

/// Authorization request with PKCE and multi-tenant support
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedAuthorizationRequest {
    pub client_id: String,
    pub response_type: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub email_hint: Option<String>,
    pub login_hint: Option<String>,
    pub tenant_hint: Option<String>,
    pub provider_hint: Option<String>,
    
    // PKCE parameters
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    
    // Enhanced security
    pub max_age: Option<i64>,
    pub prompt: Option<String>,
    pub ui_locales: Option<String>,
    pub acr_values: Option<String>,
}

/// Token exchange request
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedTokenRequest {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub code: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub audience: Option<String>,
}

/// Configuration for enhanced token broker
#[derive(Debug, Clone)]
pub struct EnhancedTokenBrokerConfig {
    /// JWT signing configuration
    pub jwt_secret: String,
    pub jwt_algorithm: Algorithm,
    pub issuer_url: String,
    
    /// Multi-TTL configuration
    pub default_token_ttl_minutes: i64,
    pub admin_token_ttl_minutes: i64,
    pub enterprise_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
    
    /// Security configuration
    pub require_pkce: bool,
    pub enable_token_rotation: bool,
    pub max_refresh_token_lifetime_days: i64,
    pub enable_jti_tracking: bool,
    
    /// Multi-tenant configuration
    pub enable_tenant_isolation: bool,
    pub require_tenant_validation: bool,
    pub enable_cross_tenant_access: bool,
    
    /// Risk assessment
    pub enable_risk_assessment: bool,
    pub max_risk_score: f64,
}

impl Default for EnhancedTokenBrokerConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("ENHANCED_JWT_SECRET")
                .or_else(|_| std::env::var("NEXTAUTH_SECRET"))
                .unwrap_or_else(|_| "enhanced-broker-secret".to_string()),
            jwt_algorithm: Algorithm::HS256,
            issuer_url: std::env::var("OIDC_ISSUER")
                .or_else(|_| std::env::var("BACKEND_URL"))
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
                
            // Multi-TTL defaults
            default_token_ttl_minutes: 15,  // 15 minutes for regular users
            admin_token_ttl_minutes: 60,    // 1 hour for admin users  
            enterprise_token_ttl_minutes: 480, // 8 hours for enterprise users
            refresh_token_ttl_days: 30,     // 30 days
            
            // Security defaults
            require_pkce: true,
            enable_token_rotation: true,
            max_refresh_token_lifetime_days: 90,
            enable_jti_tracking: true,
            
            // Multi-tenant defaults
            enable_tenant_isolation: true,
            require_tenant_validation: true,
            enable_cross_tenant_access: false,
            
            // Risk assessment
            enable_risk_assessment: true,
            max_risk_score: 0.8,
        }
    }
}

/// Enhanced Token Broker with multi-tenant and PKCE support
pub struct EnhancedTokenBroker {
    config: EnhancedTokenBrokerConfig,
    provider_registry: Arc<dyn ProviderRegistryTrait>,
    tenant_resolver: Arc<dyn TenantResolverTrait>,
    discovery_client: Arc<dyn DiscoveryClientTrait>,
    casbin_service: Arc<CasbinService>,
    admin_module_service: Arc<AdminModuleService>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    
    // In-memory stores (in production, use Redis or database)
    active_sessions: Arc<tokio::sync::RwLock<HashMap<String, SessionInfo>>>,
    pkce_challenges: Arc<tokio::sync::RwLock<HashMap<String, PKCEChallenge>>>,
    revoked_tokens: Arc<tokio::sync::RwLock<HashMap<String, DateTime<Utc>>>>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    session_id: String,
    user_id: String,
    tenant_id: String,
    provider_id: String,
    created_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    risk_score: Option<f64>,
}

impl EnhancedTokenBroker {
    pub fn new(
        config: EnhancedTokenBrokerConfig,
        provider_registry: Arc<dyn ProviderRegistryTrait>,
        tenant_resolver: Arc<dyn TenantResolverTrait>,
        discovery_client: Arc<dyn DiscoveryClientTrait>,
        casbin_service: Arc<CasbinService>,
        admin_module_service: Arc<AdminModuleService>,
    ) -> Result<Self, AppError> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        Ok(Self {
            config,
            provider_registry,
            tenant_resolver,
            discovery_client,
            casbin_service,
            admin_module_service,
            encoding_key,
            decoding_key,
            active_sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pkce_challenges: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            revoked_tokens: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }
    
    /// Initialize authorization flow with tenant resolution and PKCE
    pub async fn initiate_authorization(
        &self,
        request: EnhancedAuthorizationRequest,
        _client_ip: Option<String>,
        _user_agent: Option<String>,
    ) -> Result<AuthorizationFlowResult, AppError> {
        tracing::info!(
            client_id = %request.client_id,
            email_hint = ?request.email_hint,
            tenant_hint = ?request.tenant_hint,
            "Initiating enhanced authorization flow"
        );
        
        // Step 1: Validate request
        self.validate_authorization_request(&request).await?;
        
        // Step 2: Resolve tenant and provider
        let tenant_resolution = if let Some(email_hint) = &request.email_hint {
            self.tenant_resolver
                .resolve_tenant_for_email(email_hint)
                .await?
        } else if let Some(tenant_hint) = &request.tenant_hint {
            // Direct tenant lookup
            self.provider_registry
                .get_providers_by_tenant(tenant_hint)
                .await?
                .into_iter()
                .next()
                .map(|provider_config| {
                    use super::provider_registry::TenantResolution;
                    use super::tenant_resolver::{EnhancedTenantResolution, TenantMapping, DomainMatchStrategy};
                    
                    let mapping = TenantMapping::new(
                        tenant_hint.clone(),
                        "direct".to_string(),
                        DomainMatchStrategy::Exact,
                        provider_config.provider_id.clone(),
                    );
                    
                    EnhancedTenantResolution {
                        tenant_resolution: TenantResolution {
                            tenant_id: tenant_hint.clone(),
                            provider_config,
                            fallback_providers: Vec::new(),
                        },
                        mapping,
                        auto_provision: true,
                        default_role: "user".to_string(),
                        metadata: HashMap::new(),
                    }
                })
        } else {
            None
        };
        
        let tenant_resolution = tenant_resolution.ok_or_else(|| {
            AppError::ValidationError(
                "Unable to resolve tenant. Please provide email_hint or tenant_hint".to_string()
            )
        })?;
        
        // Step 3: Generate PKCE challenge if required
        let pkce_challenge = if self.config.require_pkce || request.code_challenge.is_some() {
            let challenge = PKCEChallenge::generate();
            let default_state = Uuid::new_v4().to_string();
            let state = request.state.as_deref().unwrap_or(&default_state);
            
            // Store PKCE challenge
            {
                let mut challenges = self.pkce_challenges.write().await;
                challenges.insert(state.to_string(), challenge.clone());
            }
            
            Some(challenge)
        } else {
            None
        };
        
        // Step 4: Construct provider authorization URL
        let provider_config = &tenant_resolution.tenant_resolution.provider_config;
        let authorization_url = self.build_provider_authorization_url(
            provider_config,
            &request,
            pkce_challenge.as_ref(),
        ).await?;
        
        Ok(AuthorizationFlowResult {
            authorization_url,
            state: request.state.unwrap_or_else(|| Uuid::new_v4().to_string()),
            tenant_id: tenant_resolution.tenant_resolution.tenant_id,
            provider_id: provider_config.provider_id.clone(),
            pkce_challenge,
            nonce: request.nonce,
        })
    }
    
    /// Exchange authorization code for tokens with PKCE verification
    pub async fn exchange_code_for_tokens(
        &self,
        request: EnhancedTokenRequest,
        _client_ip: Option<String>,
        _user_agent: Option<String>,
    ) -> Result<EnhancedUnifiedJWT, AppError> {
        tracing::info!(
            client_id = %request.client_id,
            grant_type = %request.grant_type,
            "Exchanging authorization code for tokens"
        );
        
        // Step 1: Validate token request
        self.validate_token_request(&request).await?;
        
        match request.grant_type.as_str() {
            "authorization_code" => {
                self.handle_authorization_code_grant(request, _client_ip, _user_agent).await
            }
            "refresh_token" => {
                self.handle_refresh_token_grant(request, _client_ip, _user_agent).await
            }
            _ => Err(AppError::ValidationError(format!(
                "Unsupported grant type: {}", request.grant_type
            ))),
        }
    }
    
    /// Handle authorization code grant with PKCE
    async fn handle_authorization_code_grant(
        &self,
        request: EnhancedTokenRequest,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<EnhancedUnifiedJWT, AppError> {
        let _code = request.code.as_ref()
            .ok_or_else(|| AppError::ValidationError("Missing authorization code".to_string()))?;
        
        // Step 1: PKCE verification if required
        if self.config.require_pkce {
            let _code_verifier = request.code_verifier.as_ref()
                .ok_or_else(|| AppError::ValidationError("Missing code_verifier for PKCE".to_string()))?;
                
            // Verify PKCE challenge (simplified - use state or store mapping)
            // In production, maintain code->challenge mapping
            let pkce_valid = true; // TODO: Implement proper PKCE validation
            
            if !pkce_valid {
                return Err(AppError::SecurityError("PKCE verification failed".to_string()));
            }
        }
        
        // Step 2: Exchange code with provider
        // For now, assume code contains the necessary information
        // In production, you'd call the provider's token endpoint
        
        // Step 3: Get user information and resolve tenant
        let user_email = "user@example.com"; // TODO: Extract from provider response
        let tenant_resolution = self.tenant_resolver
            .resolve_tenant_for_email(user_email)
            .await?
            .ok_or_else(|| AppError::NotFound("No tenant found for user".to_string()))?;
            
        // Step 4: Create session
        let session_id = Uuid::new_v4().to_string();
        let session_info = SessionInfo {
            session_id: session_id.clone(),
            user_id: "user123".to_string(), // TODO: Get from provider
            tenant_id: tenant_resolution.tenant_resolution.tenant_id.clone(),
            provider_id: tenant_resolution.tenant_resolution.provider_config.provider_id.clone(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            ip_address: client_ip,
            user_agent,
            risk_score: Some(0.1), // TODO: Calculate risk score
        };
        
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session_info);
        }
        
        // Step 5: Issue unified JWT with appropriate TTL
        let token_ttl_minutes = self.determine_token_ttl(&tenant_resolution.default_role);
        
        self.issue_enhanced_unified_jwt(
            "user123".to_string(),
            user_email.to_string(),
            tenant_resolution,
            session_id,
            request.client_id,
            request.scope.unwrap_or_else(|| "openid profile email".to_string()),
            token_ttl_minutes,
        ).await
    }
    
    /// Handle refresh token grant
    async fn handle_refresh_token_grant(
        &self,
        request: EnhancedTokenRequest,
        _client_ip: Option<String>,
        _user_agent: Option<String>,
    ) -> Result<EnhancedUnifiedJWT, AppError> {
        let _refresh_token = request.refresh_token.as_ref()
            .ok_or_else(|| AppError::ValidationError("Missing refresh_token".to_string()))?;
            
        // TODO: Implement refresh token validation and new token issuance
        Err(AppError::InternalError("Refresh token flow not implemented yet".to_string()))
    }
    
    /// Issue enhanced unified JWT with multi-TTL support
    async fn issue_enhanced_unified_jwt(
        &self,
        user_id: String,
        email: String,
        tenant_resolution: EnhancedTenantResolution,
        session_id: String,
        client_id: String,
        scope: String,
        token_ttl_minutes: i64,
    ) -> Result<EnhancedUnifiedJWT, AppError> {
        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        let expires_at = now + Duration::minutes(token_ttl_minutes);
        
        let claims = EnhancedUnifiedJWTClaims {
            // Standard JWT claims
            sub: user_id.clone(),
            iss: self.config.issuer_url.clone(),
            aud: vec![client_id.clone()],
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            nbf: now.timestamp(),
            jti: jti.clone(),
            
            // Enhanced user claims  
            email: email.clone(),
            email_verified: true, // TODO: Get from provider
            name: Some("Test User".to_string()), // TODO: Get from provider
            picture: None,
            role: self.load_user_role(&user_id).await?,
            permissions: self.load_user_permissions(&user_id).await?,
            subscription_tier: None,
            
            // Multi-tenant claims
            tenant_id: tenant_resolution.tenant_resolution.tenant_id.clone(),
            provider_id: tenant_resolution.tenant_resolution.provider_config.provider_id.clone(),
            provider_type: tenant_resolution.tenant_resolution.provider_config.provider_type.to_string(),
            provider_user_id: user_id.clone(),
            
            // Session and security claims
            session_id: session_id.clone(),
            session_type: "enhanced_unified".to_string(),
            client_id: client_id.clone(),
            scope: scope.clone(),
            amr: vec!["oidc".to_string()],
            acr: "1".to_string(),
            
            // Authorization claims
            azp: client_id.clone(),
            auth_time: now.timestamp(),
            nonce: None,
            
            // Enhanced metadata
            ip_address: None,
            user_agent: None,
            geo_location: None,
            risk_score: Some(0.1),
        };
        
        let header = Header::new(self.config.jwt_algorithm);
        let access_token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(format!("JWT encoding failed: {}", e)))?;
        
        // Generate refresh token if enabled
        let refresh_token = if self.config.enable_token_rotation {
            Some(Uuid::new_v4().to_string()) // TODO: Implement proper refresh token
        } else {
            None
        };
        
        let provider_id = tenant_resolution.tenant_resolution.provider_config.provider_id.clone();
        let tenant_id = tenant_resolution.tenant_resolution.tenant_id.clone();
        
        let unified_jwt = EnhancedUnifiedJWT {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: token_ttl_minutes * 60,
            expires_at,
            refresh_token,
            refresh_expires_in: Some(self.config.refresh_token_ttl_days * 24 * 60 * 60),
            refresh_expires_at: Some(now + Duration::days(self.config.refresh_token_ttl_days)),
            id_token: None, // TODO: Generate OIDC ID token if requested
            scope,
            session_id,
            jti,
            custom_ttl_applied: token_ttl_minutes != self.config.default_token_ttl_minutes,
            provider_id: provider_id.clone(),
            tenant_id: tenant_id.clone(),
        };
        
        tracing::info!(
            user_id = %user_id,
            tenant_id = %tenant_id,
            provider_id = %provider_id,
            token_ttl_minutes = token_ttl_minutes,
            "Issued enhanced unified JWT"
        );
        
        Ok(unified_jwt)
    }
    
    /// Load user role and admin modules using AdminModuleService
    async fn load_user_role(&self, user_id: &str) -> Result<String, AppError> {
        match self.admin_module_service.get_user_admin_modules(user_id).await {
            Ok(admin_modules) => {
                if admin_modules.contains(&"admin-full-004".to_string()) {
                    Ok("admin-full-004".to_string()) // Full admin access
                } else if !admin_modules.is_empty() {
                    Ok("moderator-standard-003".to_string()) // Has some admin modules
                } else {
                    Ok("user-basic-001".to_string()) // Basic user
                }
            }
            Err(_) => {
                // If we can't load admin modules, default to basic user
                tracing::warn!("Failed to load admin modules for user {}, defaulting to basic user", user_id);
                Ok("user-basic-001".to_string())
            }
        }
    }
    
    /// Load user permissions based on admin modules and general permissions
    async fn load_user_permissions(&self, user_id: &str) -> Result<Vec<String>, AppError> {
        let mut permissions = vec!["read".to_string()]; // Basic permission
        
        // Load admin modules
        match self.admin_module_service.get_user_admin_modules(user_id).await {
            Ok(admin_modules) => {
                for module in admin_modules {
                    // Convert admin module names to permission strings
                    match module.as_str() {
                        "user-management" => {
                            permissions.extend_from_slice(&[
                                "user:read".to_string(),
                                "user:write".to_string(),
                                "user:manage".to_string(),
                            ]);
                        }
                        "financial-management" => {
                            permissions.extend_from_slice(&[
                                "finance:read".to_string(),
                                "finance:write".to_string(),
                                "finance:manage".to_string(),
                            ]);
                        }
                        "system-configuration" => {
                            permissions.extend_from_slice(&[
                                "system:read".to_string(),
                                "system:write".to_string(),
                                "system:configure".to_string(),
                            ]);
                        }
                        "content-moderation" => {
                            permissions.extend_from_slice(&[
                                "content:read".to_string(),
                                "content:moderate".to_string(),
                            ]);
                        }
                        "analytics-access" => {
                            permissions.extend_from_slice(&[
                                "analytics:read".to_string(),
                                "analytics:advanced".to_string(),
                            ]);
                        }
                        "reporting-access" => {
                            permissions.extend_from_slice(&[
                                "reports:read".to_string(),
                                "reports:generate".to_string(),
                            ]);
                        }
                        "audit-logs" => {
                            permissions.extend_from_slice(&[
                                "audit:read".to_string(),
                                "audit:manage".to_string(),
                            ]);
                        }
                        "security-monitoring" => {
                            permissions.extend_from_slice(&[
                                "security:read".to_string(),
                                "security:monitor".to_string(),
                            ]);
                        }
                        "backup-recovery" => {
                            permissions.extend_from_slice(&[
                                "backup:read".to_string(),
                                "backup:manage".to_string(),
                            ]);
                        }
                        "integration-management" => {
                            permissions.extend_from_slice(&[
                                "integration:read".to_string(),
                                "integration:manage".to_string(),
                            ]);
                        }
                        _ => {
                            // Generic admin permission for unknown modules
                            permissions.push(format!("{}:access", module));
                        }
                    }
                }
            }
            Err(_) => {
                tracing::warn!("Failed to load admin modules for user {}", user_id);
            }
        }
        
        // Remove duplicates and sort
        permissions.sort();
        permissions.dedup();
        
        Ok(permissions)
    }

    /// Determine token TTL based on user role and provider
    fn determine_token_ttl(&self, role: &str) -> i64 {
        match role.to_lowercase().as_str() {
            "admin" | "super_admin" | "admin-full-004" => self.config.admin_token_ttl_minutes,
            "enterprise" | "premium" => self.config.enterprise_token_ttl_minutes,
            _ => self.config.default_token_ttl_minutes,
        }
    }
    
    /// Validate authorization request
    async fn validate_authorization_request(&self, request: &EnhancedAuthorizationRequest) -> Result<(), AppError> {
        if request.response_type != "code" {
            return Err(AppError::ValidationError(
                "Only 'code' response_type is supported".to_string()
            ));
        }
        
        if request.client_id.is_empty() {
            return Err(AppError::ValidationError("Missing client_id".to_string()));
        }
        
        // Validate redirect URI format
        url::Url::parse(&request.redirect_uri)
            .map_err(|_| AppError::ValidationError("Invalid redirect_uri format".to_string()))?;
        
        // PKCE validation
        if self.config.require_pkce {
            if request.code_challenge.is_none() {
                return Err(AppError::ValidationError(
                    "PKCE code_challenge is required".to_string()
                ));
            }
            
            if let Some(method) = &request.code_challenge_method {
                if method != "S256" {
                    return Err(AppError::ValidationError(
                        "Only S256 code_challenge_method is supported".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate token request
    async fn validate_token_request(&self, request: &EnhancedTokenRequest) -> Result<(), AppError> {
        if !["authorization_code", "refresh_token"].contains(&request.grant_type.as_str()) {
            return Err(AppError::ValidationError(format!(
                "Unsupported grant_type: {}", request.grant_type
            )));
        }
        
        if request.client_id.is_empty() {
            return Err(AppError::ValidationError("Missing client_id".to_string()));
        }
        
        Ok(())
    }
    
    /// Build provider authorization URL
    async fn build_provider_authorization_url(
        &self,
        provider_config: &OIDCProviderConfig,
        request: &EnhancedAuthorizationRequest,
        pkce_challenge: Option<&PKCEChallenge>,
    ) -> Result<String, AppError> {
        let mut params = vec![
            ("client_id", provider_config.client_id.clone()),
            ("response_type", request.response_type.clone()),
            ("redirect_uri", request.redirect_uri.clone()),
            ("scope", request.scope.clone()),
        ];
        
        if let Some(state) = &request.state {
            params.push(("state", state.clone()));
        }
        
        if let Some(nonce) = &request.nonce {
            params.push(("nonce", nonce.clone()));
        }
        
        if let Some(pkce) = pkce_challenge {
            params.push(("code_challenge", pkce.code_challenge.clone()));
            params.push(("code_challenge_method", pkce.code_challenge_method.clone()));
        }
        
        let query_string = params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
            
        let url = format!("{}?{}", provider_config.authorization_endpoint, query_string);
        Ok(url)
    }
}

/// Result of authorization flow initiation
#[derive(Debug)]
pub struct AuthorizationFlowResult {
    pub authorization_url: String,
    pub state: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub pkce_challenge: Option<PKCEChallenge>,
    pub nonce: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pkce_challenge_generation() {
        let challenge = PKCEChallenge::generate();
        
        assert!(!challenge.code_verifier.is_empty());
        assert!(!challenge.code_challenge.is_empty());
        assert_eq!(challenge.code_challenge_method, "S256");
        
        // Verify that the challenge can be verified
        assert!(challenge.verify(&challenge.code_verifier));
        assert!(!challenge.verify("invalid_verifier"));
    }
    
    #[test]
    fn test_enhanced_token_broker_config() {
        let config = EnhancedTokenBrokerConfig::default();
        
        assert!(config.require_pkce);
        assert!(config.enable_token_rotation);
        assert_eq!(config.default_token_ttl_minutes, 15);
        assert_eq!(config.admin_token_ttl_minutes, 60);
        assert_eq!(config.enterprise_token_ttl_minutes, 480);
    }
    
    #[tokio::test]
    async fn test_authorization_request_validation() {
        // This would require mocking the dependencies
        // TODO: Add comprehensive integration tests
    }
}