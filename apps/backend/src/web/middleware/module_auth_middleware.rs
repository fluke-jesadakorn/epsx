// Module-aware authentication middleware for IAM system v2
// Extends the existing auth system with module-specific access control

use axum::{
    async_trait,
    extract::{Request, FromRequestParts, Path},
    http::{request::Parts, header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::dom::values::{UserId, Role, SessId};
use super::auth_middleware::AuthCtx;

// ========================================
// MODULE AUTHENTICATION TYPES
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessLevel {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Enterprise,
}

impl AccessLevel {
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        match s.to_lowercase().as_str() {
            "bronze" => Ok(AccessLevel::Bronze),
            "silver" => Ok(AccessLevel::Silver),
            "gold" => Ok(AccessLevel::Gold),
            "platinum" => Ok(AccessLevel::Platinum),
            "enterprise" => Ok(AccessLevel::Enterprise),
            _ => Err("Invalid access level"),
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            AccessLevel::Bronze => "bronze",
            AccessLevel::Silver => "silver",
            AccessLevel::Gold => "gold",
            AccessLevel::Platinum => "platinum",
            AccessLevel::Enterprise => "enterprise",
        }
    }

    pub fn numeric_value(&self) -> u8 {
        match self {
            AccessLevel::Bronze => 1,
            AccessLevel::Silver => 2,
            AccessLevel::Gold => 3,
            AccessLevel::Platinum => 4,
            AccessLevel::Enterprise => 5,
        }
    }

    pub fn has_access_to(&self, required: &AccessLevel) -> bool {
        self.numeric_value() >= required.numeric_value()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleQuotas {
    pub api_calls: Option<i32>, // -1 for unlimited
    pub rate_limit_per_minute: i32,
    pub daily_limit: Option<i32>,
    pub monthly_limit: Option<i32>,
    pub custom_limits: HashMap<String, i32>,
}

impl Default for ModuleQuotas {
    fn default() -> Self {
        Self {
            api_calls: Some(100),
            rate_limit_per_minute: 10,
            daily_limit: Some(1000),
            monthly_limit: Some(30000),
            custom_limits: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRestrictions {
    pub ip_restrictions: Vec<String>,
    pub time_restrictions: Option<String>, // JSON string for time-based rules
    pub feature_restrictions: HashMap<String, bool>,
    pub endpoint_restrictions: Vec<String>,
}

impl Default for ModuleRestrictions {
    fn default() -> Self {
        Self {
            ip_restrictions: vec![],
            time_restrictions: None,
            feature_restrictions: HashMap::new(),
            endpoint_restrictions: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModuleAccess {
    pub assignment_id: Uuid,
    pub module_id: Uuid,
    pub module_name: String,
    pub display_name: String,
    pub access_level: AccessLevel,
    pub quotas: ModuleQuotas,
    pub restrictions: ModuleRestrictions,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAccess {
    pub key_id: Uuid,
    pub client_name: String,
    pub allowed_modules: Vec<UserModuleAccess>,
    pub rate_limits: HashMap<String, i32>, // module_name -> rate_limit
    pub expires_at: Option<DateTime<Utc>>,
    pub total_requests: i32,
    pub last_used_at: Option<DateTime<Utc>>,
}

// ========================================
// MODULE-AWARE AUTH CONTEXT
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthCtx {
    // Existing auth context
    pub user_id: UserId,
    pub role: Role,
    pub sess: SessId,
    
    // Module-specific access
    pub assigned_modules: Vec<UserModuleAccess>,
    pub effective_quotas: HashMap<String, ModuleQuotas>, // module_name -> quotas
    
    // API key context (for third-party access)
    pub api_key_access: Option<ApiKeyAccess>,
    
    // Current context
    pub current_module: Option<String>,
    pub request_timestamp: DateTime<Utc>,
}

impl ModuleAuthCtx {
    // Check if user has access to a specific module
    pub fn has_module_access(&self, module_name: &str) -> bool {
        self.assigned_modules.iter().any(|m| 
            m.module_name == module_name && 
            m.status == "active" &&
            m.expires_at.map_or(true, |exp| exp > Utc::now())
        )
    }

    // Get access level for a specific module
    pub fn get_access_level(&self, module_name: &str) -> Option<&AccessLevel> {
        self.assigned_modules.iter()
            .find(|m| m.module_name == module_name && m.status == "active")
            .map(|m| &m.access_level)
    }

    // Check if user can perform an action on a module
    pub fn can_perform_action(&self, module_name: &str, action: &str, required_level: AccessLevel) -> bool {
        if let Some(access_level) = self.get_access_level(module_name) {
            if !access_level.has_access_to(&required_level) {
                return false;
            }

            // Check feature restrictions
            if let Some(module_access) = self.assigned_modules.iter().find(|m| m.module_name == module_name) {
                if let Some(&restricted) = module_access.restrictions.feature_restrictions.get(action) {
                    return !restricted;
                }
            }

            return true;
        }
        false
    }

    // Get current quota status for a module
    pub fn get_quota_status(&self, module_name: &str) -> Option<&ModuleQuotas> {
        self.effective_quotas.get(module_name)
    }

    // Check if user is using API key authentication
    pub fn is_api_key_auth(&self) -> bool {
        self.api_key_access.is_some()
    }

    // Get available modules for user
    pub fn get_available_modules(&self) -> Vec<&str> {
        self.assigned_modules.iter()
            .filter(|m| m.status == "active" && m.expires_at.map_or(true, |exp| exp > Utc::now()))
            .map(|m| m.module_name.as_str())
            .collect()
    }

    // Check quota availability
    pub fn check_quota(&self, quota_type: &str, amount: i32) -> bool {
        // For API key auth, check against API key specific limits
        if let Some(api_key_access) = &self.api_key_access {
            if let Some(module_name) = &self.current_module {
                if let Some(&limit) = api_key_access.rate_limits.get(module_name) {
                    return limit >= amount;
                }
            }
        }

        // For user auth, check against module quotas
        if let Some(module_name) = &self.current_module {
            if let Some(quotas) = self.get_quota_status(module_name) {
                match quota_type {
                    "api_calls" => quotas.api_calls.map_or(true, |limit| limit == -1 || limit >= amount),
                    "daily_limit" => quotas.daily_limit.map_or(true, |limit| limit == -1 || limit >= amount),
                    "monthly_limit" => quotas.monthly_limit.map_or(true, |limit| limit == -1 || limit >= amount),
                    custom => quotas.custom_limits.get(custom).map_or(true, |&limit| limit >= amount),
                }
            } else {
                false
            }
        } else {
            // If no current module context, allow (will be checked at endpoint level)
            true
        }
    }
}

// ========================================
// AUTHENTICATION EXTRACTOR
// ========================================

#[async_trait]
impl FromRequestParts<crate::web::auth::AppState> for ModuleAuthCtx {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::web::auth::AppState,
    ) -> Result<Self, Self::Rejection> {
        
        // First try API key authentication
        if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(api_key) = auth_str.strip_prefix("Bearer ak_") {
                    return Self::from_api_key(api_key, state).await;
                }
            }
        }

        // Fall back to user authentication
        let auth_ctx = AuthCtx::from_request_parts(parts, state).await?;
        Self::from_user_auth(auth_ctx, state).await
    }
}

impl ModuleAuthCtx {
    // Create ModuleAuthCtx from API key
    async fn from_api_key(
        api_key: &str,
        state: &crate::web::auth::AppState,
    ) -> Result<Self, StatusCode> {
        tracing::debug!("Attempting API key authentication: {}...", &api_key[..std::cmp::min(8, api_key.len())]);
        
        // Hash the API key to match database
        let key_hash = Self::hash_api_key(api_key);
        
        // Validate API key and get access info
        // Note: This would need to be implemented in your repository layer
        match state.module_repo.get_api_key_access(&key_hash).await {
            Ok(Some(api_access)) => {
                // Check if API key is expired
                if let Some(expires_at) = api_access.expires_at {
                    if expires_at <= Utc::now() {
                        tracing::warn!("API key expired: {}", api_access.key_id);
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }

                // Create effective quotas from API key modules
                let mut effective_quotas = HashMap::new();
                for module in &api_access.allowed_modules {
                    effective_quotas.insert(module.module_name.clone(), module.quotas.clone());
                }

                // Create a synthetic user ID for API key
                let synthetic_user_id = UserId::from_str(&format!("api_key_{}", api_access.key_id))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                Ok(ModuleAuthCtx {
                    user_id: synthetic_user_id,
                    role: Role::from_str("api_client").unwrap_or_default(),
                    sess: SessId::generate(),
                    assigned_modules: api_access.allowed_modules.clone(),
                    effective_quotas,
                    api_key_access: Some(api_access),
                    current_module: None,
                    request_timestamp: Utc::now(),
                })
            }
            Ok(None) => {
                tracing::warn!("API key not found");
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(e) => {
                tracing::error!("Failed to validate API key: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    // Create ModuleAuthCtx from user authentication
    async fn from_user_auth(
        auth_ctx: AuthCtx,
        state: &crate::web::auth::AppState,
    ) -> Result<Self, StatusCode> {
        
        // Get user's module assignments
        let user_modules = match state.module_repo.get_user_module_assignments(&auth_ctx.user_id).await {
            Ok(modules) => modules,
            Err(e) => {
                tracing::error!("Failed to get user module assignments: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        // Build effective quotas
        let mut effective_quotas = HashMap::new();
        for module in &user_modules {
            effective_quotas.insert(module.module_name.clone(), module.quotas.clone());
        }

        Ok(ModuleAuthCtx {
            user_id: auth_ctx.user_id,
            role: auth_ctx.role,
            sess: auth_ctx.sess,
            assigned_modules: user_modules,
            effective_quotas,
            api_key_access: None,
            current_module: None,
            request_timestamp: Utc::now(),
        })
    }

    // Helper function to hash API keys
    fn hash_api_key(api_key: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// ========================================
// MODULE-SPECIFIC EXTRACTORS
// ========================================

#[derive(Debug)]
pub struct ModuleAccess {
    pub auth: ModuleAuthCtx,
    pub module_name: String,
}

#[async_trait]
impl FromRequestParts<crate::web::auth::AppState> for ModuleAccess {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::web::auth::AppState,
    ) -> Result<Self, Self::Rejection> {

        // Get module authentication context
        let mut auth = ModuleAuthCtx::from_request_parts(parts, state).await?;

        // Extract module name from path
        let path_params: Result<Path<HashMap<String, String>>, _> = 
            Path::from_request_parts(parts, state).await;
        
        let module_name = match path_params {
            Ok(Path(params)) => {
                params.get("module").cloned()
                    .or_else(|| Self::extract_module_from_path(&parts.uri.path()))
                    .unwrap_or_else(|| "unknown".to_string())
            }
            Err(_) => Self::extract_module_from_path(&parts.uri.path()).unwrap_or_else(|| "unknown".to_string()),
        };

        // Validate access to the specific module
        if module_name != "unknown" && !auth.has_module_access(&module_name) {
            tracing::warn!("User {} attempted to access module {} without permission", 
                auth.user_id, module_name);
            return Err(StatusCode::FORBIDDEN);
        }

        // Set current module in context
        auth.current_module = Some(module_name.clone());

        Ok(ModuleAccess {
            auth,
            module_name,
        })
    }
}

impl ModuleAccess {
    // Extract module name from URL path
    fn extract_module_from_path(path: &str) -> Option<String> {
        // Expected patterns:
        // /api/v1/stock-ranking/* -> stock-ranking
        // /api/v1/portfolio-analysis/* -> portfolio-analysis
        // /api/v1/market-data/* -> market-data
        // /api/v1/trading-signals/* -> trading-signals
        
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 4 && parts[1] == "api" && parts[2] == "v1" {
            // Check if this is a known module
            match parts[3] {
                "stock-ranking" | "portfolio-analysis" | "market-data" | "trading-signals" => {
                    Some(parts[3].to_string())
                }
                _ => None,
            }
        } else {
            None
        }
    }

    // Check if user can perform specific action
    pub fn can_perform(&self, action: &str, required_level: AccessLevel) -> bool {
        self.auth.can_perform_action(&self.module_name, action, required_level)
    }

    // Get access level for current module
    pub fn get_access_level(&self) -> Option<&AccessLevel> {
        self.auth.get_access_level(&self.module_name)
    }

    // Check quota availability
    pub fn check_quota(&self, quota_type: &str, amount: i32) -> bool {
        if let Some(quotas) = self.auth.get_quota_status(&self.module_name) {
            match quota_type {
                "api_calls" => quotas.api_calls.map_or(true, |limit| limit == -1 || limit >= amount),
                "daily_limit" => quotas.daily_limit.map_or(true, |limit| limit == -1 || limit >= amount),
                "monthly_limit" => quotas.monthly_limit.map_or(true, |limit| limit == -1 || limit >= amount),
                custom => quotas.custom_limits.get(custom).map_or(true, |&limit| limit >= amount),
            }
        } else {
            false
        }
    }
}

// ========================================
// MIDDLEWARE FUNCTIONS
// ========================================

pub async fn module_auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // The actual authentication is handled by the ModuleAuthCtx extractor
    // This middleware can log requests or perform additional validation
    
    tracing::debug!("Processing request with module authentication");
    Ok(next.run(req).await)
}

// Middleware to require specific module access
pub fn require_module_access(
    module_name: &str,
    min_access_level: AccessLevel,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    let module_name = module_name.to_string();
    move |req, next| {
        let module_name = module_name.clone();
        let min_access_level = min_access_level.clone();
        Box::pin(async move {
            // Extract module access from request
            // Note: This would need to be integrated with Axum's middleware system
            // For now, we'll proceed and let the route handler validate access
            
            tracing::debug!("Requiring {} access to module: {}", 
                min_access_level.to_string(), module_name);
            Ok(next.run(req).await)
        })
    }
}

// ========================================
// USAGE LOGGING
// ========================================

pub async fn log_module_usage(
    auth: &ModuleAuthCtx,
    module_name: &str,
    endpoint: &str,
    method: &str,
    response_status: u16,
    quota_consumed: i32,
    state: &crate::web::auth::AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let usage_log = crate::dom::entities::ModuleUsageLog {
        id: Uuid::new_v4(),
        user_id: if auth.is_api_key_auth() { None } else { Some(auth.user_id.clone()) },
        api_key_id: auth.api_key_access.as_ref().map(|ak| ak.key_id),
        sub_module_id: auth.assigned_modules.iter()
            .find(|m| m.module_name == module_name)
            .map(|m| m.module_id),
        endpoint: endpoint.to_string(),
        request_method: method.to_string(),
        response_status: response_status as i32,
        response_time_ms: None,
        quota_consumed,
        quota_type: Some("api_calls".to_string()),
        client_ip: None,
        user_agent: None,
        request_id: None,
        session_id: None,
        request_size_bytes: None,
        response_size_bytes: None,
        cache_hit: false,
        billable: true,
        cost_units: Some(quota_consumed as f64 * 0.001),
        timestamp: Utc::now(),
        request_metadata: serde_json::json!({}),
    };

    // Log to database
    match state.usage_repo.log_usage(usage_log).await {
        Ok(_) => tracing::debug!("Logged module usage: {} - {}", module_name, endpoint),
        Err(e) => tracing::error!("Failed to log module usage: {:?}", e),
    }

    Ok(())
}