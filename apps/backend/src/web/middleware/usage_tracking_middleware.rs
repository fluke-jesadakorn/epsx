use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{error, warn};
use chrono::Utc;
use crate::infrastructure::container::DomainContainer;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::schema::{api_key_usage_logs, api_keys};

/// Middleware to track API key usage
pub async fn usage_tracking_middleware(
    State(container): State<Arc<DomainContainer>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = std::time::Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    
    // Extract API Key ID if present (usually set by auth middleware or rate limiter)
    // For now, we'll try to extract from extensions or headers if not present
    // Note: This relies on previous middleware identifying the API key
    // Since we don't have a standardized "ApiKeyContext" yet, we'll look for "x-api-key" header 
    // or try to find it from the Bearer token context if it relates to a specific key
    
    // For this implementation, we will look for the X-API-Key header directly 
    // as that's the transparent way to track specific key usage.
    // In a full implementation, this should be integrated with AuthContext.
    
    let api_key_id_str = request.headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
        
    // Execute request first to get response status
    let response = next.run(request).await;
    let duration = start_time.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i32;

    // Log in background if we have an API key
    if let Some(key_id_str) = api_key_id_str {
        if let Ok(key_id) = uuid::Uuid::parse_str(&key_id_str) {
            let container_clone = container.clone();
            let method_clone = method.clone();
            let path_clone = path.clone();
            
            // Spawn background task to avoid blocking response
            tokio::spawn(async move {
                log_usage(
                    container_clone, 
                    key_id, 
                    method_clone, 
                    path_clone, 
                    status_code, 
                    duration
                ).await;
            });
        }
    }

    response
}

async fn log_usage(
    container: Arc<DomainContainer>,
    api_key_id: uuid::Uuid,
    method: String,
    endpoint: String,
    status_code: i32,
    duration_ms: i32,
) {
    let pool = container.db_pool();
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection for usage logging: {}", e);
            return;
        }
    };
    
    // 1. Log detailed usage
    let new_log = (
        api_key_usage_logs::api_key_id.eq(api_key_id),
        api_key_usage_logs::method.eq(&method),
        api_key_usage_logs::endpoint.eq(&endpoint),
        api_key_usage_logs::response_status.eq(status_code),
        api_key_usage_logs::response_time_ms.eq(duration_ms),
        api_key_usage_logs::request_at.eq(Utc::now()),
    );
    
    if let Err(e) = diesel::insert_into(api_key_usage_logs::table)
        .values(new_log)
        .execute(&mut conn)
        .await 
    {
        warn!("Failed to insert usage log: {}", e);
    }
    
    // 2. Update total requests count on api_key
    // Using simple increment
    if let Err(e) = diesel::update(api_keys::table.find(api_key_id))
        .set(api_keys::total_requests.eq(api_keys::total_requests + 1))
        .execute(&mut conn)
        .await
    {
        warn!("Failed to update total_requests count: {}", e);
    }
}
