use crate::infrastructure::container::DomainContainer;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};

/// Helper function to get user's plan tier from their plan assignments
async fn get_user_plan_from_plans(
    container: &Arc<DomainContainer>,
    wallet_address: &str,
) -> Option<String> {
    #[derive(sqlx::FromRow)]
    struct PlanInfo {
        plan_type: String,
    }

    let result: Result<Vec<PlanInfo>, sqlx::Error> = sqlx::query_as(
        "SELECT g.plan_type 
         FROM wallet_plan_assignments wga
         JOIN plans g ON g.id = wga.plan_id
         WHERE wga.wallet_address = $1 
           AND wga.is_active = true
           AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
         ORDER BY CASE g.plan_type 
           WHEN 'elite' THEN 1
           WHEN 'enterprise' THEN 2
           WHEN 'premium' THEN 3
           WHEN 'professional' THEN 4
           WHEN 'basic' THEN 5
           WHEN 'starter' THEN 6
           ELSE 7
         END
         LIMIT 1",
    )
    .bind(wallet_address)
    .fetch_all(container.db_pool.as_ref())
    .await;

    match result {
        Ok(plans) if !plans.is_empty() => {
            let plan = plans[0].plan_type.clone();
            debug!("Found plan '{}' for wallet {}", plan, wallet_address);
            Some(plan)
        }
        Ok(_) => {
            debug!("No active plans found for wallet {}", wallet_address);
            None
        }
        Err(e) => {
            warn!("Failed to query user plans for rate limiting: {}", e);
            None
        }
    }
}

/// Middleware function for Web3 rate limiting
pub async fn web3_rate_limit_middleware(
    State(container): State<Arc<DomainContainer>>,
    _headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Authentication runs before this middleware. API-key identity therefore
    // carries live scopes and plan-derived limits rather than trusting a
    // caller-controlled wallet header or persisted key snapshot.
    let authenticated = request
        .extensions()
        .get::<crate::web::middleware::OpenIDUserContext>()
        .cloned();
    let wallet_address = authenticated
        .as_ref()
        .map(|context| context.wallet_address.clone());
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();

    // Create rate limiter (lightweight instantiation)
    // In production, this should be in the container, but for now we create it here
    // to ensure we have the latest config/cache reference
    let cache = container
        .cache
        .as_ref()
        .cloned()
        .unwrap_or_else(|| Arc::new(crate::infrastructure::cache::MemoryCache::new()));

    // Create config securely - panic if env vars are missing/invalid in critical path
    // or use safe defaults manually if Config::default() is not available
    let config = match crate::config::Config::from_env() {
        Ok(c) => Arc::new(c),
        Err(_) => {
            // Fallback to minimal config or panic depending on strictness
            // For middleware, we often want to fail open or use hardcoded defaults if config fails
            // But here we need Arc<Config> for the rate limiter.
            // Since we can't easily create a default Config struct without impl Default,
            // we'll unwrap causing a panic if config is broken (acceptable for server startup/runtime)
            // taking the risk that `from_env` works.
            // A safer approach might be to skip rate limiting if config fails, but that's insecure.
            // Let's assume from_env works or panic.
            Arc::new(
                crate::config::Config::from_env().expect("Failed to load config for rate limiter"),
            )
        }
    };

    let rate_limiter =
        crate::web::middleware::multi_level_rate_limiter::MultiLevelRateLimiter::new(cache, config);

    // Determine plan limits based on wallet's plan assignments
    let plan_limits = if let Some(identity) = authenticated
        .as_ref()
        .and_then(|context| context.api_key.as_ref())
    {
        crate::web::middleware::multi_level_rate_limiter::PlanRateLimits {
            plan_id: None,
            plan_name: "effective-api-entitlement".to_string(),
            requests_per_minute: identity.rate_limits.per_minute,
            requests_per_hour: identity.rate_limits.per_hour,
            requests_per_day: identity.rate_limits.per_day,
            burst_capacity: identity.rate_limits.burst,
        }
    } else if let Some(wallet) = &wallet_address {
        // Query user's plans from database to determine rate limit tier
        let user_plan = match get_user_plan_from_plans(&container, wallet).await {
            Some(plan) => plan,
            None => "free".to_string(),
        };

        match user_plan.to_lowercase().as_str() {
            "elite" | "enterprise" | "whale" => {
                crate::web::middleware::multi_level_rate_limiter::PlanRateLimits::elite()
            }
            "premium" | "professional" | "pro" => {
                crate::web::middleware::multi_level_rate_limiter::PlanRateLimits::premium()
            }
            "basic" | "starter" => {
                crate::web::middleware::multi_level_rate_limiter::PlanRateLimits::basic()
            }
            _ => crate::web::middleware::multi_level_rate_limiter::PlanRateLimits::free(),
        }
    } else {
        crate::web::middleware::multi_level_rate_limiter::PlanRateLimits::free()
    };

    // Check rate limits
    let api_key_id = authenticated
        .as_ref()
        .and_then(|context| context.api_key.as_ref())
        .map(|identity| identity.id.to_string());
    let api_key_config = authenticated
        .as_ref()
        .and_then(|context| context.api_key.as_ref())
        .map(|identity| crate::web::middleware::RateLimitConfig {
            requests_per_minute: Some(identity.rate_limits.per_minute),
            requests_per_hour: Some(identity.rate_limits.per_hour),
            requests_per_day: Some(identity.rate_limits.per_day),
        });
    let result = rate_limiter
        .check(
            wallet_address.as_deref(),
            Some(&plan_limits),
            api_key_id.as_deref(),
            api_key_config.as_ref(),
            &path,
            &method,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !result.allowed {
        let level = result
            .blocked_at_level
            .map(|l| l.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        warn!(
            "Rate limit exceeded at level '{}' for wallet: {:?}",
            level, wallet_address
        );

        // Return 429 with Retry-After header
        let retry_after = result.retry_after_seconds.unwrap_or(60).to_string();
        let body = serde_json::json!({
            "error": "rate_limit_exceeded",
            "message": format!("Rate limit exceeded at {} level", level),
            "retry_after": result.retry_after_seconds,
            "blocked_at": level
        });

        let response = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Retry-After", retry_after)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body.to_string()))
            .unwrap();

        return Ok(response);
    }

    // Continue with request
    let mut response = next.run(request).await;

    // Add rate limit headers for the most relevant limit (Plan)
    if let Some((current, limit)) = result.plan_remaining {
        let remaining = limit.saturating_sub(current);
        response
            .headers_mut()
            .insert("X-RateLimit-Limit", limit.to_string().parse().unwrap());
        response.headers_mut().insert(
            "X-RateLimit-Remaining",
            remaining.to_string().parse().unwrap(),
        );
    } else if let Some((current, limit)) = result.global_remaining {
        let remaining = limit.saturating_sub(current);
        response.headers_mut().insert(
            "X-RateLimit-Limit-Global",
            limit.to_string().parse().unwrap(),
        );
        response.headers_mut().insert(
            "X-RateLimit-Remaining-Global",
            remaining.to_string().parse().unwrap(),
        );
    }

    Ok(response)
}

/// Unified rate limit middleware - alias for web3_rate_limit_middleware
/// Provides a unified entry point for rate limiting across the application
pub async fn unified_rate_limit_middleware(
    State(container): State<Arc<DomainContainer>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Delegate to web3_rate_limit_middleware for unified behavior
    web3_rate_limit_middleware(State(container), headers, request, next).await
}

#[cfg(test)]
mod tests {
    // Tests removed - tested functionality has been removed
    // Web3RateLimitService, RateLimitTier, and RateLimitBucket are no longer used
}
