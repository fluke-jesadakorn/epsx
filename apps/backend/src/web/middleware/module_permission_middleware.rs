// Module-based permission middleware - replaces the old permission profile system
// Validates access and quotas at the module level with comprehensive logging

use axum::{
    extract::{Request, State},
    http::{Method, StatusCode, HeaderMap},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use chrono::{TimeZone, Datelike, Timelike};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

use crate::web::{
    auth::AppState,
    middleware::module_auth_middleware::{ModuleAuthCtx, AccessLevel},
};
use crate::dom::entities::module::ModuleUsageLog;

// ========================================
// PERMISSION CHECK RESULT TYPES
// ========================================

#[derive(Debug, Clone)]
pub struct ModulePermissionResult {
    pub allowed: bool,
    pub reason: String,
    pub quota_consumed: i32,
    pub remaining_quota: Option<i32>,
    pub retry_after_seconds: Option<u64>,
    pub access_level: Option<AccessLevel>,
}

#[derive(Debug)]
pub struct QuotaCheckResult {
    pub available: bool,
    pub quota_type: String,
    pub consumed: i32,
    pub limit: i32,
    pub reset_time: Option<chrono::DateTime<chrono::Utc>>,
}

// ========================================
// MAIN MIDDLEWARE FUNCTION
// ========================================

/// Module-aware permission middleware that enforces access control and quotas
pub async fn module_permission_middleware(
    State(app_state): State<AppState>,
    module_auth: Option<ModuleAuthCtx>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let headers = req.headers().clone();

    // Skip permission check for public endpoints
    if is_public_endpoint(&path, &method) {
        return Ok(next.run(req).await);
    }

    // Require module authentication for protected endpoints
    let module_auth = module_auth.ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract module name from path
    let module_name = extract_module_from_path(&path)
        .unwrap_or_else(|| "unknown".to_string());

    // Skip module checks for non-module endpoints
    if module_name == "unknown" {
        // Fall back to legacy permission checking for non-module endpoints
        return handle_legacy_endpoints(module_auth, req, next, app_state).await;
    }

    // Check module access and permissions
    let permission_result = check_module_permissions(
        &app_state,
        &module_auth,
        &module_name,
        &path,
        &method,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !permission_result.allowed {
        tracing::warn!(
            "Module access denied: user={}, module={}, path={}, reason={}",
            module_auth.user_id,
            module_name,
            path,
            permission_result.reason
        );

        return create_permission_denied_response(&permission_result);
    }

    // Check and consume quota
    let quota_result = check_and_consume_quota(
        &app_state,
        &module_auth,
        &module_name,
        &path,
        &method,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !quota_result.available {
        tracing::warn!(
            "Quota exceeded: user={}, module={}, quota_type={}, consumed={}, limit={}",
            module_auth.user_id,
            module_name,
            quota_result.quota_type,
            quota_result.consumed,
            quota_result.limit
        );

        return create_quota_exceeded_response(&quota_result);
    }

    // Add module context to request extensions
    req.extensions_mut().insert(module_auth.clone());
    req.extensions_mut().insert(ModuleRequestContext {
        module_name: module_name.clone(),
        access_level: permission_result.access_level.clone(),
        quota_consumed: permission_result.quota_consumed,
        start_time,
    });

    // Execute the request
    let response = next.run(req).await;
    let duration = start_time.elapsed();

    // Log usage for analytics and billing
    tokio::spawn(log_module_usage(
        app_state.clone(),
        module_auth,
        module_name,
        path,
        method.as_str().to_string(),
        response.status().as_u16(),
        permission_result.quota_consumed,
        duration,
        extract_client_info(&headers),
    ));

    Ok(response)
}

// ========================================
// PERMISSION CHECKING FUNCTIONS
// ========================================

/// Check if user has permission to access a module endpoint
async fn check_module_permissions(
    _app_state: &AppState,
    module_auth: &ModuleAuthCtx,
    module_name: &str,
    path: &str,
    method: &Method,
) -> Result<ModulePermissionResult, ModulePermissionError> {
    
    // Check if user has access to the module
    if !module_auth.has_module_access(module_name) {
        return Ok(ModulePermissionResult {
            allowed: false,
            reason: format!("No access to module: {}", module_name),
            quota_consumed: 0,
            remaining_quota: None,
            retry_after_seconds: None,
            access_level: None,
        });
    }

    // Get user's access level for this module
    let access_level = module_auth.get_access_level(module_name)
        .ok_or(ModulePermissionError::AccessLevelNotFound)?;

    // Check endpoint-specific permissions
    let endpoint_config = get_endpoint_config(module_name, path, method)?;
    
    if !access_level.has_access_to(&endpoint_config.required_level) {
        return Ok(ModulePermissionResult {
            allowed: false,
            reason: format!(
                "Insufficient access level: {} required, {} provided",
                endpoint_config.required_level.to_string(),
                access_level.to_string()
            ),
            quota_consumed: 0,
            remaining_quota: None,
            retry_after_seconds: None,
            access_level: Some(access_level.clone()),
        });
    }

    // Check feature-specific restrictions
    if let Some(required_action) = &endpoint_config.required_action {
        if !module_auth.can_perform_action(module_name, required_action, endpoint_config.required_level.clone()) {
            return Ok(ModulePermissionResult {
                allowed: false,
                reason: format!("Action not permitted: {}", required_action),
                quota_consumed: 0,
                remaining_quota: None,
                retry_after_seconds: None,
                access_level: Some(access_level.clone()),
            });
        }
    }

    // Check time-based restrictions
    if let Some(module_access) = module_auth.assigned_modules.iter().find(|m| m.module_name == module_name) {
        if let Some(expires_at) = module_access.expires_at {
            if expires_at <= chrono::Utc::now() {
                return Ok(ModulePermissionResult {
                    allowed: false,
                    reason: "Module access has expired".to_string(),
                    quota_consumed: 0,
                    remaining_quota: None,
                    retry_after_seconds: None,
                    access_level: Some(access_level.clone()),
                });
            }
        }
    }

    Ok(ModulePermissionResult {
        allowed: true,
        reason: "Access granted".to_string(),
        quota_consumed: endpoint_config.quota_cost,
        remaining_quota: None, // Will be calculated in quota check
        retry_after_seconds: None,
        access_level: Some(access_level.clone()),
    })
}

/// Check and consume quota for the request
async fn check_and_consume_quota(
    app_state: &AppState,
    module_auth: &ModuleAuthCtx,
    module_name: &str,
    path: &str,
    method: &Method,
) -> Result<QuotaCheckResult, ModulePermissionError> {
    
    let endpoint_config = get_endpoint_config(module_name, path, method)?;
    let quota_type = &endpoint_config.quota_type;
    let quota_cost = endpoint_config.quota_cost;

    // Check if user has sufficient quota
    let has_quota = module_auth.check_quota(quota_type, quota_cost);
    
    if !has_quota {
        // Get current usage to provide detailed error
        let current_usage = app_state.module_repo
            .get_current_usage(&module_auth.user_id, module_name, quota_type)
            .await
            .unwrap_or(0);

        let quota_limits = app_state.module_repo
            .get_quota_limits(&module_auth.user_id, module_name)
            .await
            .unwrap_or_default();

        let limit = quota_limits.get(quota_type).copied().unwrap_or(0);

        return Ok(QuotaCheckResult {
            available: false,
            quota_type: quota_type.clone(),
            consumed: current_usage,
            limit,
            reset_time: calculate_quota_reset_time(quota_type),
        });
    }

    // For API key requests, we still need to check against the key's limits
    if module_auth.is_api_key_auth() {
        if let Some(api_key_access) = &module_auth.api_key_access {
            let module_limit = api_key_access.rate_limits.get(module_name);
            if let Some(&limit) = module_limit {
                // Check API key specific rate limiting
                let current_usage = get_api_key_current_usage(
                    app_state, 
                    api_key_access.key_id, 
                    module_name, 
                    quota_type
                ).await?;
                
                if current_usage + quota_cost > limit {
                    return Ok(QuotaCheckResult {
                        available: false,
                        quota_type: quota_type.clone(),
                        consumed: current_usage,
                        limit,
                        reset_time: calculate_quota_reset_time(quota_type),
                    });
                }
            }
        }
    }

    Ok(QuotaCheckResult {
        available: true,
        quota_type: quota_type.clone(),
        consumed: quota_cost,
        limit: get_user_quota_limit(module_auth, module_name, quota_type).unwrap_or(-1),
        reset_time: calculate_quota_reset_time(quota_type),
    })
}

// ========================================
// ENDPOINT CONFIGURATION
// ========================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EndpointConfig {
    required_level: AccessLevel,
    required_action: Option<String>,
    quota_type: String,
    quota_cost: i32,
    rate_limit_per_minute: Option<i32>,
}

/// Get configuration for a specific endpoint
fn get_endpoint_config(
    module_name: &str,
    path: &str,
    method: &Method,
) -> Result<EndpointConfig, ModulePermissionError> {
    
    match module_name {
        "stock-ranking" => get_stock_ranking_endpoint_config(path, method),
        "portfolio-analysis" => get_portfolio_analysis_endpoint_config(path, method),
        "market-data" => get_market_data_endpoint_config(path, method),
        "trading-signals" => get_trading_signals_endpoint_config(path, method),
        _ => Err(ModulePermissionError::UnknownModule(module_name.to_string())),
    }
}

/// Stock ranking module endpoint configurations
fn get_stock_ranking_endpoint_config(
    path: &str,
    method: &Method,
) -> Result<EndpointConfig, ModulePermissionError> {
    
    match (path, method.as_str()) {
        // Basic rankings (Bronze+)
        ("/api/v1/stock-ranking/rankings", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: Some("view_rankings".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(10),
        }),
        
        ("/api/v1/stock-ranking/rankings/top-performers", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: Some("view_rankings".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(10),
        }),

        ("/api/v1/stock-ranking/eps/growth", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: Some("view_eps_analysis".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 2,
            rate_limit_per_minute: Some(10),
        }),

        // AI insights (Silver+)
        ("/api/v1/stock-ranking/rankings/ai-insights", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("ai_insights".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 5,
            rate_limit_per_minute: Some(20),
        }),

        ("/api/v1/stock-ranking/rankings/pattern-analysis", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("pattern_recognition".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 3,
            rate_limit_per_minute: Some(20),
        }),

        // Real-time features (Silver+)
        ("/api/v1/stock-ranking/rankings/live", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("real_time_updates".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 2,
            rate_limit_per_minute: Some(30),
        }),

        // Custom algorithms (Gold+)
        ("/api/v1/stock-ranking/rankings/custom", "POST") => Ok(EndpointConfig {
            required_level: AccessLevel::Gold,
            required_action: Some("custom_algorithms".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 10,
            rate_limit_per_minute: Some(5),
        }),

        ("/api/v1/stock-ranking/algorithms", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Gold,
            required_action: Some("custom_algorithms".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(20),
        }),

        // Export functionality (Bronze+, quota limited)
        ("/api/v1/stock-ranking/export/csv", "POST") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: Some("export".to_string()),
            quota_type: "exports_per_day".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(2),
        }),

        ("/api/v1/stock-ranking/export/excel", "POST") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("export".to_string()),
            quota_type: "exports_per_day".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(2),
        }),

        // Enterprise features (Platinum+)
        ("/api/v1/stock-ranking/models/custom", "POST") => Ok(EndpointConfig {
            required_level: AccessLevel::Platinum,
            required_action: Some("custom_models".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 50,
            rate_limit_per_minute: Some(2),
        }),

        ("/api/v1/stock-ranking/bulk/analyze", "POST") => Ok(EndpointConfig {
            required_level: AccessLevel::Platinum,
            required_action: Some("bulk_operations".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 100,
            rate_limit_per_minute: Some(1),
        }),

        // Catch-all for undefined endpoints
        _ => {
            if path.starts_with("/api/v1/stock-ranking/") {
                // Default configuration for unknown stock-ranking endpoints
                Ok(EndpointConfig {
                    required_level: AccessLevel::Bronze,
                    required_action: None,
                    quota_type: "api_calls".to_string(),
                    quota_cost: 1,
                    rate_limit_per_minute: Some(10),
                })
            } else {
                Err(ModulePermissionError::UnknownEndpoint(path.to_string()))
            }
        }
    }
}

/// Portfolio analysis module endpoint configurations  
fn get_portfolio_analysis_endpoint_config(
    path: &str,
    method: &Method,
) -> Result<EndpointConfig, ModulePermissionError> {
    
    match (path, method.as_str()) {
        // Basic portfolio operations (Bronze+)
        ("/api/v1/portfolio-analysis/portfolios", "GET" | "POST") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: Some("manage_portfolios".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(20),
        }),

        // Risk analysis (Silver+)
        (path, "GET") if path.contains("/risk") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("risk_analysis".to_string()),
            quota_type: "analyses_per_day".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(10),
        }),

        // Benchmarking (Gold+)
        (path, "POST") if path.contains("/benchmark") => Ok(EndpointConfig {
            required_level: AccessLevel::Gold,
            required_action: Some("benchmarking".to_string()),
            quota_type: "analyses_per_day".to_string(),
            quota_cost: 3,
            rate_limit_per_minute: Some(5),
        }),

        // Default for portfolio analysis
        _ if path.starts_with("/api/v1/portfolio-analysis/") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: None,
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(10),
        }),

        _ => Err(ModulePermissionError::UnknownEndpoint(path.to_string())),
    }
}

/// Market data module endpoint configurations
fn get_market_data_endpoint_config(
    path: &str,
    method: &Method,
) -> Result<EndpointConfig, ModulePermissionError> {
    
    match (path, method.as_str()) {
        // Basic quotes (Bronze+, delayed)
        (path, "GET") if path.contains("/quotes/") && !path.contains("/live") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: None,
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(20),
        }),

        // Real-time data (Silver+)
        (path, "GET") if path.contains("/live") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("real_time_data".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 2,
            rate_limit_per_minute: Some(50),
        }),

        // Technical indicators (Silver+)
        (path, "GET") if path.contains("/indicators/") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("technical_indicators".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 3,
            rate_limit_per_minute: Some(30),
        }),

        // Premium data (Gold+)
        (path, "GET") if path.contains("/level2/") || path.contains("/options/") => Ok(EndpointConfig {
            required_level: AccessLevel::Gold,
            required_action: Some("premium_data".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 5,
            rate_limit_per_minute: Some(20),
        }),

        // International markets (Platinum+)
        (path, "GET") if path.contains("/international/") => Ok(EndpointConfig {
            required_level: AccessLevel::Platinum,
            required_action: Some("international_data".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 10,
            rate_limit_per_minute: Some(10),
        }),

        // Default for market data
        _ if path.starts_with("/api/v1/market-data/") => Ok(EndpointConfig {
            required_level: AccessLevel::Bronze,
            required_action: None,
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(10),
        }),

        _ => Err(ModulePermissionError::UnknownEndpoint(path.to_string())),
    }
}

/// Trading signals module endpoint configurations
fn get_trading_signals_endpoint_config(
    path: &str,
    method: &Method,
) -> Result<EndpointConfig, ModulePermissionError> {
    
    match (path, method.as_str()) {
        // Basic signals (Silver+)
        ("/api/v1/trading-signals/signals", "GET") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("view_signals".to_string()),
            quota_type: "signals_per_day".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(10),
        }),

        // AI signals (Silver+)
        (path, "GET") if path.contains("/ai") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: Some("ai_signals".to_string()),
            quota_type: "signals_per_day".to_string(),
            quota_cost: 3,
            rate_limit_per_minute: Some(5),
        }),

        // Strategy management (Gold+)
        (path, _) if path.contains("/strategies") => Ok(EndpointConfig {
            required_level: AccessLevel::Gold,
            required_action: Some("manage_strategies".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 5,
            rate_limit_per_minute: Some(10),
        }),

        // Backtesting (Gold+)
        (path, "POST") if path.contains("/backtest") => Ok(EndpointConfig {
            required_level: AccessLevel::Gold,
            required_action: Some("backtesting".to_string()),
            quota_type: "backtests".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(2),
        }),

        // Strategy optimization (Platinum+)
        (path, "POST") if path.contains("/optimize") => Ok(EndpointConfig {
            required_level: AccessLevel::Platinum,
            required_action: Some("strategy_optimization".to_string()),
            quota_type: "optimizations".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(1),
        }),

        // Live trading (Platinum+)
        (path, _) if path.contains("/deploy") || path.contains("/live") => Ok(EndpointConfig {
            required_level: AccessLevel::Platinum,
            required_action: Some("live_trading".to_string()),
            quota_type: "api_calls".to_string(),
            quota_cost: 10,
            rate_limit_per_minute: Some(5),
        }),

        // Default for trading signals
        _ if path.starts_with("/api/v1/trading-signals/") => Ok(EndpointConfig {
            required_level: AccessLevel::Silver,
            required_action: None,
            quota_type: "api_calls".to_string(),
            quota_cost: 1,
            rate_limit_per_minute: Some(5),
        }),

        _ => Err(ModulePermissionError::UnknownEndpoint(path.to_string())),
    }
}

// ========================================
// HELPER FUNCTIONS
// ========================================

/// Extract module name from URL path
fn extract_module_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 4 && parts[1] == "api" && parts[2] == "v1" {
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

/// Check if endpoint is public (doesn't require authentication)
fn is_public_endpoint(path: &str, method: &Method) -> bool {
    let public_patterns = [
        ("/health", &Method::GET),
        ("/api/v1/auth/login", &Method::POST),
        ("/api/v1/auth/register", &Method::POST),
        ("/api/v1/auth/password-reset", &Method::POST),
        ("/auth/me-public", &Method::GET),
    ];
    
    public_patterns.iter().any(|(pattern, allowed_method)| {
        path == *pattern && method == *allowed_method
    })
}

/// Handle legacy endpoints that don't use the module system
async fn handle_legacy_endpoints(
    _module_auth: ModuleAuthCtx,
    req: Request,
    next: Next,
    _app_state: AppState,
) -> Result<Response, StatusCode> {
    // For now, allow all legacy endpoints to pass through
    // In the future, these should be migrated to the module system
    Ok(next.run(req).await)
}

/// Create permission denied response
fn create_permission_denied_response(result: &ModulePermissionResult) -> Result<Response, StatusCode> {
    let response = Json(json!({
        "error": "Access denied",
        "message": result.reason,
        "required_access_level": result.access_level.as_ref().map(|l| l.to_string()),
        "timestamp": chrono::Utc::now()
    }));
    
    Ok(response.into_response())
}

/// Create quota exceeded response
fn create_quota_exceeded_response(result: &QuotaCheckResult) -> Result<Response, StatusCode> {
    let response = Json(json!({
        "error": "Quota exceeded",
        "message": format!("Quota exceeded for {}: {}/{}", result.quota_type, result.consumed, result.limit),
        "quota_type": result.quota_type,
        "consumed": result.consumed,
        "limit": result.limit,
        "reset_time": result.reset_time,
        "timestamp": chrono::Utc::now()
    }));
    
    Ok(response.into_response())
}

/// Calculate when quota resets
fn calculate_quota_reset_time(quota_type: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match quota_type {
        "api_calls" => {
            // Reset every minute
            let now = chrono::Utc::now();
            Some(now.with_second(0).unwrap() + chrono::Duration::minutes(1))
        }
        "daily_limit" | "exports_per_day" | "analyses_per_day" | "signals_per_day" => {
            // Reset at midnight UTC
            let now = chrono::Utc::now();
            Some(now.date_naive().succ_opt()?.and_hms_opt(0, 0, 0)?.and_utc())
        }
        "monthly_limit" => {
            // Reset on first day of next month
            let now = chrono::Utc::now();
            let next_month = if now.month() == 12 {
                chrono::Utc.with_ymd_and_hms(now.year() + 1, 1, 1, 0, 0, 0).single()
            } else {
                chrono::Utc.with_ymd_and_hms(now.year(), now.month() + 1, 1, 0, 0, 0).single()
            };
            next_month
        }
        _ => None,
    }
}

/// Get user's quota limit for a specific type
fn get_user_quota_limit(
    module_auth: &ModuleAuthCtx,
    module_name: &str,
    quota_type: &str,
) -> Option<i32> {
    module_auth.get_quota_status(module_name)?.custom_limits.get(quota_type).copied()
        .or_else(|| {
            match quota_type {
                "api_calls" => module_auth.get_quota_status(module_name)?.api_calls,
                "daily_limit" => module_auth.get_quota_status(module_name)?.daily_limit,  
                "monthly_limit" => module_auth.get_quota_status(module_name)?.monthly_limit,
                _ => None,
            }
        })
}

/// Get current usage for an API key
async fn get_api_key_current_usage(
    _app_state: &AppState,
    _key_id: Uuid,
    _module_name: &str,
    _quota_type: &str,
) -> Result<i32, ModulePermissionError> {
    // This would query the module_usage_logs table filtered by API key
    // For now, return 0 as placeholder
    Ok(0)
}

/// Extract client information from headers
fn extract_client_info(headers: &HeaderMap) -> ClientInfo {
    ClientInfo {
        ip: headers.get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        user_agent: headers.get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    }
}

/// Log module usage for analytics and billing
async fn log_module_usage(
    app_state: AppState,
    module_auth: ModuleAuthCtx,
    module_name: String,
    path: String,
    method: String,
    status_code: u16,
    quota_consumed: i32,
    duration: std::time::Duration,
    client_info: ClientInfo,
) {
    let usage_log = ModuleUsageLog {
        id: Uuid::new_v4(),
        user_id: if !module_auth.is_api_key_auth() { Some(module_auth.user_id.clone()) } else { None },
        api_key_id: module_auth.api_key_access.as_ref().map(|ak| ak.key_id),
        sub_module_id: module_auth.assigned_modules.iter()
            .find(|m| m.module_name == module_name)
            .map(|m| m.module_id),
        endpoint: path,
        request_method: method,
        response_status: status_code as i32,
        response_time_ms: Some(duration.as_millis() as i32),
        quota_consumed,
        quota_type: Some("api_calls".to_string()),
        client_ip: client_info.ip,
        user_agent: client_info.user_agent,
        request_id: None,
        session_id: None,
        request_size_bytes: None,
        response_size_bytes: None,
        cache_hit: false,
        billable: true,
        cost_units: Some(quota_consumed as f64 * 0.001), // $0.001 per API call
        timestamp: chrono::Utc::now(),
        request_metadata: serde_json::json!({}),
    };

    if let Err(e) = app_state.module_repo.log_usage(&usage_log).await {
        tracing::error!("Failed to log module usage: {:?}", e);
    }
}

// ========================================
// SUPPORTING TYPES
// ========================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ModuleRequestContext {
    module_name: String,
    access_level: Option<AccessLevel>,
    quota_consumed: i32,
    start_time: Instant,
}

#[derive(Debug)]
struct ClientInfo {
    ip: Option<String>,
    user_agent: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
enum ModulePermissionError {
    #[error("Access level not found")]
    AccessLevelNotFound,
    
    #[error("Unknown module: {0}")]
    UnknownModule(String),
    
    #[error("Unknown endpoint: {0}")]
    UnknownEndpoint(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_module_from_path() {
        assert_eq!(extract_module_from_path("/api/v1/stock-ranking/rankings"), Some("stock-ranking".to_string()));
        assert_eq!(extract_module_from_path("/api/v1/portfolio-analysis/portfolios"), Some("portfolio-analysis".to_string()));
        assert_eq!(extract_module_from_path("/api/v1/market-data/quotes"), Some("market-data".to_string()));
        assert_eq!(extract_module_from_path("/api/v1/trading-signals/signals"), Some("trading-signals".to_string()));
        assert_eq!(extract_module_from_path("/api/v1/auth/login"), None);
        assert_eq!(extract_module_from_path("/health"), None);
    }

    #[test]
    fn test_is_public_endpoint() {
        assert!(is_public_endpoint("/health", &Method::GET));
        assert!(is_public_endpoint("/api/v1/auth/login", &Method::POST));
        assert!(!is_public_endpoint("/api/v1/stock-ranking/rankings", &Method::GET));
    }

    #[test]
    fn test_stock_ranking_endpoint_config() {
        let config = get_stock_ranking_endpoint_config("/api/v1/stock-ranking/rankings", &Method::GET).unwrap();
        assert_eq!(config.required_level, AccessLevel::Bronze);
        assert_eq!(config.quota_cost, 1);

        let ai_config = get_stock_ranking_endpoint_config("/api/v1/stock-ranking/rankings/ai-insights", &Method::GET).unwrap();
        assert_eq!(ai_config.required_level, AccessLevel::Silver);
        assert_eq!(ai_config.quota_cost, 5);
    }
}