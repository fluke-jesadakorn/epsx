use crate::infrastructure::container::DomainContainer;
use crate::schemas::infra_logs::{analytics_events, api_key_usage_logs};
use crate::schemas::primary::api_keys;
use crate::web::middleware::bearer_middleware::OpenIDUserContext;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Insertable)]
#[diesel(table_name = analytics_events)]
struct NewAnalyticsEvent {
    id: uuid::Uuid,
    event_type: String,
    wallet_address: Option<String>,
    resource_path: String,
    method: String,
    status_code: i32,
    duration_ms: i32,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
}

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
        let Ok(mut conn) = pool.get().await else {
            error!("analytics connection unavailable; request analytics not persisted");
            return;
        };
        let event = NewAnalyticsEvent {
            id: uuid::Uuid::new_v4(),
            event_type: "API_REQUEST".to_string(),
            wallet_address: analytics_wallet,
            resource_path: analytics_path,
            method: analytics_method,
            status_code,
            duration_ms: duration,
            metadata: None,
            created_at: Utc::now(),
        };
        if let Err(error) = diesel::insert_into(analytics_events::table)
            .values(&event)
            .execute(&mut conn)
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
    let mut analytics_conn = match analytics_pool.get().await {
        Ok(connection) => connection,
        Err(error) => {
            error!(api_key_id = %api_key_id, "analytics connection failed: {error}");
            return;
        }
    };

    let inserted = diesel::insert_into(api_key_usage_logs::table)
        .values((
            api_key_usage_logs::api_key_id.eq(api_key_id),
            api_key_usage_logs::method.eq(&method),
            api_key_usage_logs::endpoint.eq(&endpoint),
            api_key_usage_logs::response_status.eq(status_code),
            api_key_usage_logs::response_time_ms.eq(duration_ms),
            api_key_usage_logs::request_at.eq(Utc::now()),
        ))
        .execute(&mut analytics_conn)
        .await;
    if let Err(error) = inserted {
        warn!(api_key_id = %api_key_id, "failed to insert usage log: {error}");
        return;
    }

    // Metadata remains in core; it advances only after the authoritative
    // analytics row was accepted, so totals cannot count validation twice.
    let core_pool = container.db_pool();
    let Ok(mut core_conn) = core_pool.get().await else {
        warn!(api_key_id = %api_key_id, "core connection failed after usage insert");
        return;
    };
    if let Err(error) = diesel::update(api_keys::table.find(api_key_id))
        .set((
            api_keys::last_used_at.eq(Utc::now()),
            api_keys::total_requests.eq(api_keys::total_requests + 1),
        ))
        .execute(&mut core_conn)
        .await
    {
        warn!(api_key_id = %api_key_id, "failed to update API-key metadata: {error}");
    }
}
