use crate::infrastructure::container::DomainContainer;
use crate::web::middleware::bearer_middleware::OpenIDUserContext;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Track completed API-key requests exactly once. This middleware must wrap
/// the rate limiter and permission guard so rejected 429/4xx/5xx responses are
/// attributed to the authenticated key as well as successful responses.
pub async fn usage_tracking_middleware(
    State(container): State<Arc<DomainContainer>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = std::time::Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let identity = request
        .extensions()
        .get::<OpenIDUserContext>()
        .map(|context| {
            (
                context.wallet_address.clone(),
                context.api_key.as_ref().map(|identity| identity.id),
            )
        });

    let response = next.run(request).await;
    let duration = start_time.elapsed().as_millis().min(i32::MAX as u128) as i32;
    let status_code = i32::from(response.status().as_u16());
    let wallet_address = identity.as_ref().map(|(wallet, _)| wallet.clone());
    let api_key_id = identity.and_then(|(_, key)| key);

    let analytics_container = Arc::clone(&container);
    let analytics_method = method.clone();
    let analytics_path = path.clone();
    let analytics_wallet = wallet_address.clone();
    tokio::spawn(async move {
        let Some(pool) = analytics_container.get_analytics_pool() else {
            error!("analytics pool unavailable; request analytics not persisted");
            return;
        };
        if let Err(error) = sqlx::query(
            "INSERT INTO analytics_events (\
                id, event_type, wallet_address, resource_path, method, status_code, \
                duration_ms, metadata, created_at\
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind("API_REQUEST")
        .bind(analytics_wallet)
        .bind(&analytics_path)
        .bind(&analytics_method)
        .bind(status_code)
        .bind(duration)
        .bind(serde_json::Value::Null)
        .bind(Utc::now())
        .execute(pool.as_ref())
        .await
        {
            warn!("failed to persist request analytics: {error}");
        }
    });

    if let Some(key_id) = api_key_id {
        let usage_container = Arc::clone(&container);
        let usage_method = method.clone();
        let usage_path = path.clone();
        tokio::spawn(async move {
            persist_api_key_usage(
                usage_container,
                key_id,
                usage_method,
                usage_path,
                status_code,
                duration,
            )
            .await;
        });
    }

    info!(
        target: "analytics",
        event = "api_request",
        path = %path,
        method = %method,
        status = status_code,
        duration_ms = duration,
        wallet = ?wallet_address,
        api_key_id = ?api_key_id,
        timestamp = %Utc::now().to_rfc3339(),
    );

    response
}

async fn persist_api_key_usage(
    container: Arc<DomainContainer>,
    api_key_id: uuid::Uuid,
    method: String,
    endpoint: String,
    status_code: i32,
    duration_ms: i32,
) {
    let Some(analytics_pool) = container.get_analytics_pool() else {
        error!(api_key_id = %api_key_id, "analytics pool unavailable for API-key usage");
        return;
    };

    if let Err(error) = sqlx::query(
        "INSERT INTO api_key_usage_logs (\
            id, api_key_id, method, endpoint, response_status, response_time_ms, request_at\
        ) VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW())",
    )
    .bind(api_key_id)
    .bind(&method)
    .bind(&endpoint)
    .bind(status_code)
    .bind(duration_ms)
    .execute(analytics_pool.as_ref())
    .await
    {
        warn!(api_key_id = %api_key_id, "failed to insert usage log: {error}");
        return;
    }

    // Metadata remains in core; it advances only after the authoritative
    // analytics row was accepted, so totals cannot count validation twice.
    let core_pool: PgPool = container.db_pool().clone();
    if let Err(error) = sqlx::query(
        "UPDATE api_keys SET last_used_at = NOW(), total_requests = total_requests + 1 WHERE id = $1",
    )
    .bind(api_key_id)
    .execute(&core_pool)
    .await
    {
        warn!(api_key_id = %api_key_id, "failed to update API-key metadata: {error}");
    }
}
