use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{error, warn, info};
use chrono::Utc;
use crate::infrastructure::container::DomainContainer;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::schemas::primary::api_keys;
use crate::schemas::analytics::api_key_usage_logs;
use crate::schemas::analytics::analytics_events; // Import the new table
use crate::web::middleware::auth_middleware::Web3AuthContext;
use crate::web::middleware::bearer_middleware::OpenIDUserContext;

#[derive(Insertable)]
#[diesel(table_name = analytics_events)]
pub struct NewAnalyticsEvent {
    pub id: uuid::Uuid,
    pub event_type: String,
    pub wallet_address: Option<String>,
    pub resource_path: String,
    pub method: String,
    pub status_code: i32,
    pub duration_ms: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<Utc>,
}

/// Middleware to track API key usage
pub async fn usage_tracking_middleware(
    State(container): State<Arc<DomainContainer>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = std::time::Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Extract API key ID from OpenIDUserContext if auth_method == "api_key"
    let api_key_id_from_ctx = request.extensions()
        .get::<OpenIDUserContext>()
        .filter(|ctx| ctx.auth_method == "api_key")
        .and_then(|ctx| uuid::Uuid::parse_str(&ctx.jti).ok());

    // Fallback: legacy x-api-key header
    let api_key_id = api_key_id_from_ctx.or_else(|| {
        request.headers()
            .get("x-api-key")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
    });

    // Execute request first to get response status
    let response = next.run(request).await;
    let duration = start_time.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i32;

    // 1. EXTRACT WALLET CONTEXT
    let wallet_address = response.extensions()
        .get::<Web3AuthContext>()
        .map(|ctx| ctx.wallet_address.clone())
        .or_else(|| {
             response.extensions()
                .get::<String>()
                .cloned()
        });

    // 2. API KEY USAGE TRACKING
    if let Some(key_id) = api_key_id {
        let container_clone = container.clone();
        let method_clone = method.clone();
        let path_clone = path.clone();

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

    // 3. NEW ANALYTICS DB LOGGING
    let container_for_db = container.clone();
    let method_for_db = method.clone();
    let path_for_db = path.clone();
    let wallet_for_db = wallet_address.clone();
    
    tokio::spawn(async move {
        // Construct the event
        let event = NewAnalyticsEvent {
            id: uuid::Uuid::new_v4(),
            event_type: "API_REQUEST".to_string(),
            wallet_address: wallet_for_db,
            resource_path: path_for_db,
            method: method_for_db,
            status_code,
            duration_ms: duration,
            metadata: None, // Can populate with IP/User-Agent later if needed
            created_at: Utc::now(),
        };

        // Get connection
        let pool = container_for_db.get_analytics_pool()
            .unwrap_or_else(|| container_for_db.db_pool());

        if let Ok(mut conn) = pool.get().await {
            let result = diesel::insert_into(analytics_events::table)
                .values(&event)
                .execute(&mut conn)
                .await;
                
            if let Err(e) = result {
                error!("Failed to insert analytics event: {}", e);
            }
        } else {
             error!("Failed to get DB connection for analytics logging");
        }
    });

    // 4. TRACING LOGGING (Operational Logs)
    let event_name = format!("API_{}_{}", method, status_code); // e.g., API_GET_200
    
    // Structured logging for analytics ingestion
    info!(
        target: "analytics",
        event = "api_request",
        name = %event_name,
        path = %path,
        method = %method,
        status = status_code,
        duration_ms = duration,
        wallet = ?wallet_address,
        timestamp = ?Utc::now().to_rfc3339()
    );

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
    // Use analytics pool if available, otherwise fallback to primary usage logs
    // ideally we should treat them separately, but for now this ensures we write to the right place
    let pool = container.get_analytics_pool()
        .unwrap_or_else(|| container.db_pool());

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
