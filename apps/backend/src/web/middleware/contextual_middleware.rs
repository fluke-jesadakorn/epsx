// Contextual middleware stacks for different access patterns
// Enables separation of concerns: internal web app vs external API vs admin interface

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::{HeaderMap, HeaderValue, StatusCode},
    body::Body,
};
use std::time::Instant;
use tracing::{info, warn, debug};
use serde_json::json;

use crate::{
    web::auth::AppState,
    web::routes::AccessContext,
    domain::shared_kernel::value_objects::UserId,
    domain::resource_management::{
        services::{
            RateLimitingService, RateLimitRequest, AccessContext as RateLimitContext,
            rate_limiting_service::IdentifierType,
        },
        ResourceType, ResourceCategory,
    },
};

/// Internal middleware stack for web application users
/// Focus: User experience, session management, feature usage tracking (non-billable)
pub async fn internal_middleware_stack(
    state: State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    let uri = request.uri().clone();
    let method = request.method().clone();
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    debug!(
        target: "internal_middleware",
        method = %method,
        uri = %uri,
        user_agent = %user_agent,
        "Processing internal web app request"
    );

    // Extract user identifier for rate limiting
    let user_id = extract_user_id_from_request(&request)
        .unwrap_or_else(|| "anonymous".to_string());

    // Check internal rate limits
    if let Some(rate_limiter) = &state.rate_limiting_service {
        let rate_limit_request = RateLimitRequest {
            identifier: user_id.clone(),
            identifier_type: IdentifierType::UserId,
            access_context: RateLimitContext::Internal,
            resource_type: ResourceType::WebPageLoad { page: "web_request".to_string() },
            quantity: 1,
            plan_id: None,
            session_metadata: None,
        };

        match rate_limiter.check_rate_limit(rate_limit_request).await {
            Ok(result) => {
                if !result.allowed {
                    return Ok(create_rate_limit_response(result, "internal"));
                }
                // Add rate limit info to request for downstream usage
                request.headers_mut().insert(
                    "X-Rate-Limit-Remaining-Hour",
                    HeaderValue::from_str(&(result.limits.hour_limit - result.current_usage.current_hour).to_string())
                        .unwrap_or_else(|_| HeaderValue::from_static("0"))
                );
            }
            Err(e) => {
                warn!("Rate limit check failed: {}", e);
                // Continue processing but log the error
            }
        }
    }

    // Process request through middleware chain
    let mut response = next.run(request).await;
    
    // Add internal context headers
    let headers = response.headers_mut();
    headers.insert("X-Access-Context", HeaderValue::from_static("internal"));
    headers.insert("X-Billable", HeaderValue::from_static("false"));
    headers.insert("X-Rate-Limit-Type", HeaderValue::from_static("user-session"));
    
    // Add performance metrics
    let duration = start_time.elapsed();
    headers.insert(
        "X-Processing-Time-MS",
        HeaderValue::from_str(&duration.as_millis().to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
    );

    // Log internal access for user experience analytics
    info!(
        target: "internal_access_log",
        method = %method,
        uri = %uri,
        duration_ms = duration.as_millis(),
        context = "internal",
        billable = false,
        user_agent = %user_agent,
        "Internal web app access - user experience tracking"
    );

    Ok(response)
}

/// External middleware stack for API developers
/// Focus: API key authentication, plan enforcement, billable usage tracking
pub async fn external_middleware_stack(
    state: State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    let uri = request.uri().clone();
    let method = request.method().clone();
    let headers = request.headers();

    // Extract API key for billing attribution
    let api_key = extract_api_key_from_headers(headers)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    debug!(
        target: "external_middleware",
        method = %method,
        uri = %uri,
        api_key = %api_key[..8.min(api_key.len())], // Log first 8 chars only
        "Processing external API request"
    );

    // Check external API rate limits with plan-based enforcement
    if let Some(rate_limiter) = &state.rate_limiting_service {
        // TODO: Get plan_id from API key lookup
        let plan_id = Some(2); // Default to plan_id 2 for API access
        
        let rate_limit_request = RateLimitRequest {
            identifier: api_key.clone(),
            identifier_type: IdentifierType::ApiKey,
            access_context: RateLimitContext::External,
            resource_type: ResourceType::ApiCall { 
                endpoint: uri.path().to_string(), 
                method: method.to_string(),
                response_size_bytes: 0,
            },
            quantity: 1,
            plan_id,
            session_metadata: None,
        };

        match rate_limiter.check_rate_limit(rate_limit_request).await {
            Ok(result) => {
                if !result.allowed {
                    return Ok(create_rate_limit_response(result, "external"));
                }
                // Add comprehensive rate limit headers for API developers
                request.headers_mut().insert(
                    "X-Rate-Limit-Remaining-Day",
                    HeaderValue::from_str(&(result.limits.day_limit - result.current_usage.current_day).to_string())
                        .unwrap_or_else(|_| HeaderValue::from_static("0"))
                );
                request.headers_mut().insert(
                    "X-Rate-Limit-Reset-Day",
                    HeaderValue::from_str(&next_day_reset_timestamp().to_string())
                        .unwrap_or_else(|_| HeaderValue::from_static("0"))
                );
                if let Some(cost_impact) = &result.cost_impact {
                    request.headers_mut().insert(
                        "X-Quota-Used-Percentage",
                        HeaderValue::from_str(&cost_impact.quota_percentage_used.to_string())
                            .unwrap_or_else(|_| HeaderValue::from_static("0"))
                    );
                }
            }
            Err(e) => {
                warn!("External API rate limit check failed: {}", e);
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }
    }

    let mut response = next.run(request).await;
    
    // Add external context headers for billing
    let response_headers = response.headers_mut();
    response_headers.insert("X-Access-Context", HeaderValue::from_static("external"));
    response_headers.insert("X-Billable", HeaderValue::from_static("true"));
    response_headers.insert("X-Rate-Limit-Type", HeaderValue::from_static("api-key"));
    response_headers.insert("X-API-Version", HeaderValue::from_static("v1"));
    
    // Add billing-relevant metrics
    let duration = start_time.elapsed();
    response_headers.insert(
        "X-Processing-Time-MS",
        HeaderValue::from_str(&duration.as_millis().to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
    );
    response_headers.insert(
        "X-Resource-Cost",
        HeaderValue::from_str("0.0001").unwrap_or_else(|_| HeaderValue::from_static("0")) // Placeholder cost per request
    );

    // Log billable API access for revenue tracking
    warn!(
        target: "external_api_billing_log",
        method = %method,
        uri = %uri,
        duration_ms = duration.as_millis(),
        context = "external",
        billable = true,
        api_key = %api_key[..8.min(api_key.len())],
        estimated_cost = 0.0001,
        "External API access - BILLABLE usage"
    );

    Ok(response)
}

/// Admin middleware stack for administrative interface
/// Focus: Admin permissions, audit logging, resource monitoring
pub async fn admin_middleware_stack(
    state: State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    let uri = request.uri().clone();
    let method = request.method().clone();
    let client_ip = extract_client_ip(&request);

    warn!(
        target: "admin_middleware",
        method = %method,
        uri = %uri,
        client_ip = client_ip,
        "Processing admin interface request - AUDIT REQUIRED"
    );

    // Extract admin session identifier for rate limiting
    let admin_session = extract_admin_session_from_request(&request)
        .unwrap_or_else(|| format!("admin_{}", client_ip));

    // Check admin rate limits with audit focus
    if let Some(rate_limiter) = &state.rate_limiting_service {
        let resource_category = determine_admin_resource_category(&uri.path());
        
        let rate_limit_request = RateLimitRequest {
            identifier: admin_session.clone(),
            identifier_type: IdentifierType::AdminSession,
            access_context: RateLimitContext::Admin,
            resource_type: ResourceType::AdminSystemOperation { operation: "admin_operation".to_string() },
            quantity: 1,
            plan_id: None,
            session_metadata: None,
        };

        match rate_limiter.check_rate_limit(rate_limit_request).await {
            Ok(result) => {
                if !result.allowed {
                    return Ok(create_rate_limit_response(result, "admin"));
                }
                // Add admin-specific headers for monitoring
                request.headers_mut().insert(
                    "X-Admin-Rate-Limit-Remaining-Hour",
                    HeaderValue::from_str(&(result.limits.hour_limit - result.current_usage.current_hour).to_string())
                        .unwrap_or_else(|_| HeaderValue::from_static("0"))
                );
            }
            Err(e) => {
                warn!("Admin rate limit check failed: {}", e);
                // For admin, be less strict but still log
            }
        }
    }

    let mut response = next.run(request).await;
    
    // Add admin context headers for audit trails
    let headers = response.headers_mut();
    headers.insert("X-Access-Context", HeaderValue::from_static("admin"));
    headers.insert("X-Billable", HeaderValue::from_static("false"));
    headers.insert("X-Requires-Audit", HeaderValue::from_static("true"));
    headers.insert("X-Security-Level", HeaderValue::from_static("high"));
    
    // Add audit-relevant metrics
    let duration = start_time.elapsed();
    headers.insert(
        "X-Processing-Time-MS",
        HeaderValue::from_str(&duration.as_millis().to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
    );

    // Log admin access for security audit trails
    warn!(
        target: "admin_access_audit_log",
        method = %method,
        uri = %uri,
        duration_ms = duration.as_millis(),
        context = "admin",
        billable = false,
        client_ip = client_ip,
        requires_audit = true,
        security_level = "high",
        "Admin interface access - SECURITY AUDIT REQUIRED"
    );

    Ok(response)
}


/// Resource tracking for billing and analytics
pub struct ResourceTracker;

impl ResourceTracker {
    /// Track internal web app usage (non-billable)
    pub async fn track_internal_usage(
        endpoint: &str,
        duration_ms: u128,
        user_id: Option<&UserId>,
    ) {
        debug!(
            target: "internal_resource_tracking",
            endpoint = endpoint,
            duration_ms = duration_ms,
            user_id = ?user_id,
            billable = false,
            resource_type = "web_app_usage",
            "Internal resource usage tracked"
        );
    }

    /// Track external API usage (billable)
    pub async fn track_external_usage(
        endpoint: &str,
        duration_ms: u128,
        api_key: &str,
        estimated_cost: f64,
    ) {
        warn!(
            target: "external_resource_billing",
            endpoint = endpoint,
            duration_ms = duration_ms,
            api_key = &api_key[..8.min(api_key.len())],
            estimated_cost = estimated_cost,
            billable = true,
            resource_type = "api_usage",
            "External API usage tracked - BILLABLE"
        );
    }

    /// Track admin operations (audit-level)
    pub async fn track_admin_usage(
        endpoint: &str,
        duration_ms: u128,
        admin_user_id: Option<&UserId>,
        client_ip: &str,
    ) {
        warn!(
            target: "admin_resource_audit",
            endpoint = endpoint,
            duration_ms = duration_ms,
            admin_user_id = ?admin_user_id,
            client_ip = client_ip,
            billable = false,
            resource_type = "admin_operation",
            audit_required = true,
            "Admin operation tracked - AUDIT TRAIL"
        );
    }
}

// Helper functions for rate limiting integration

fn create_rate_limit_response(result: crate::domain::resource_management::services::RateLimitResult, context: &str) -> Response {
    let body = serde_json::json!({
        "error": "Rate limit exceeded",
        "context": context,
        "current_usage": result.current_usage,
        "limits": result.limits,
        "retry_after_seconds": result.retry_after_seconds,
        "warnings": result.warnings,
    }).to_string();

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Content-Type", "application/json")
        .header("X-Rate-Limit-Context", context)
        .header("Retry-After", result.retry_after_seconds.unwrap_or(60).to_string())
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("Rate limit exceeded")))
}

fn extract_user_id_from_request(request: &Request) -> Option<String> {
    // Try to extract user ID from various sources:
    // 1. Authorization header (OIDC token)
    // 2. Session cookie
    // 3. Custom header
    
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                // TODO: Decode OIDC token to get user_id
                return Some("user_from_token".to_string());
            }
        }
    }
    
    None
}

fn extract_admin_session_from_request(request: &Request) -> Option<String> {
    // Extract admin session from request
    if let Some(session_header) = request.headers().get("X-Admin-Session") {
        if let Ok(session_str) = session_header.to_str() {
            return Some(session_str.to_string());
        }
    }
    
    None
}

fn determine_admin_resource_category(path: &str) -> ResourceCategory {
    if path.contains("/users/bulk/") {
        ResourceCategory::BulkOperations
    } else if path.contains("/users/") {
        ResourceCategory::UserManagement
    } else if path.contains("/system/") || path.contains("/analytics/") {
        ResourceCategory::SystemQueries
    } else {
        ResourceCategory::Admin
    }
}

fn next_day_reset_timestamp() -> u64 {
    use chrono::{Utc, Duration};
    let now = Utc::now();
    let tomorrow = now + Duration::days(1);
    let tomorrow_start = tomorrow.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    tomorrow_start.timestamp() as u64
}

// Original helper functions

fn extract_api_key_from_headers(headers: &HeaderMap) -> Result<String, &'static str> {
    // Try Authorization header first (Bearer token format)
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Ok(auth_str[7..].to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("X-API-Key") {
        if let Ok(api_key) = api_key_header.to_str() {
            return Ok(api_key.to_string());
        }
    }

    Err("No API key found")
}

fn extract_client_ip(request: &Request) -> String {
    // Try to get real IP from various headers
    let headers = request.headers();
    
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(ip_str) = forwarded_for.to_str() {
            // Take the first IP from comma-separated list
            if let Some(first_ip) = ip_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fallback to connection info
    "unknown".to_string()
}