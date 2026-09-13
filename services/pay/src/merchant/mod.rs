//! Public merchant platform. Separate storage and contracts preserve legacy payment semantics.
pub mod api;
mod auth;
mod catalog;
pub mod chain;
mod controls;
mod qr;
mod reconcile;
mod webhooks;

use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use epsx_service_auth::AccessTokenVerifier;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct Platform {
    pub db: sqlx::PgPool,
    pub verifier: Arc<dyn AccessTokenVerifier>,
    pub chains: Arc<Vec<chain::Network>>,
    pub secret: Arc<String>,
    #[cfg(test)]
    pub webhook_test_url: Option<String>,
}
#[derive(Debug)]
pub struct Error(pub axum::http::StatusCode, pub &'static str);
pub type Result<T> = std::result::Result<T, Error>;
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, axum::Json(json!({"error":self.1}))).into_response()
    }
}
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error=%e,"merchant storage failed");
        Self(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "storage_unavailable",
        )
    }
}
impl From<crate::native_chain::Error> for Error {
    fn from(e: crate::native_chain::Error) -> Self {
        tracing::warn!(error=%e,"merchant chain unavailable");
        Self(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "chain_unavailable",
        )
    }
}
pub fn bad(message: &'static str) -> Error {
    Error(axum::http::StatusCode::BAD_REQUEST, message)
}
pub fn missing() -> Error {
    Error(axum::http::StatusCode::NOT_FOUND, "not_found")
}
pub fn conflict() -> Error {
    Error(axum::http::StatusCode::CONFLICT, "operation_conflict")
}
pub fn id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}
pub fn hash(value: &[u8]) -> String {
    use sha2::Digest;
    hex::encode(sha2::Sha256::digest(value))
}
pub fn capability(p: &Platform, id: &str) -> String {
    webhooks::derive_secret(&p.secret, &format!("checkout:{id}"))
}

pub fn requested(request: &Request) -> bool {
    let path = request.uri().path().trim_start_matches("/api/v1/pay/");
    request.uri().path().starts_with("/api/v1/pay/")
        && (request
            .headers()
            .get("x-pay-api-version")
            .is_some_and(|v| v == "2026-09-08")
            || request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("Bearer epsxpay_"))
            || path.starts_with("merchants")
            || path.starts_with("checkout-sessions")
            || path.starts_with("api-keys")
            || path.starts_with("webhook-endpoints")
            || path.starts_with("deliveries")
            || path.starts_with("events")
            || path
                .split('/')
                .any(|s| s.starts_with("pi_") || s.starts_with("plink_") || s.starts_with("mop_")))
}
async fn dispatch(State(p): State<Platform>, request: Request, next: Next) -> Response {
    if request.uri().path() == "/ready" && !p.chains.is_empty() {
        for n in p.chains.iter() {
            for mode in ["direct", "escrow"]
                .into_iter()
                .chain(n.qr.is_some().then_some("qr"))
            {
                if api::healthy(&p, n, mode).await.is_err() {
                    return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response();
                }
            }
        }
    }
    if !requested(&request) {
        return next.run(request).await;
    }
    let result = api::handle(&p, request).await;
    let mut response = match result {
        Ok(v) => axum::Json(v).into_response(),
        Err(e) => e.into_response(),
    };
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("x-pay-api-version", "2026-09-08".parse().unwrap());
    response
}
pub fn wrap(app: Router, db: sqlx::PgPool, verifier: Arc<dyn AccessTokenVerifier>) -> Router {
    let secret = std::env::var("PAY_MERCHANT_SECRET").unwrap_or_default();
    let chains = chain::Network::from_env().expect("invalid PAY_MERCHANT_NETWORKS");
    if !chains.is_empty() {
        assert!(
            secret.len() >= 32,
            "PAY_MERCHANT_SECRET must have at least 32 bytes"
        );
    }
    let p = Platform {
        db,
        verifier,
        chains: Arc::new(chains),
        secret: Arc::new(secret),
        #[cfg(test)]
        webhook_test_url: None,
    };
    if !p.chains.is_empty() {
        reconcile::spawn(p.clone());
        webhooks::spawn(p.clone());
    }
    app.layer(middleware::from_fn_with_state(p, dispatch))
}

#[cfg(test)]
mod tests;

pub async fn emit(
    conn: &mut sqlx::PgConnection,
    payment: &api::Payment,
    event_type: &str,
) -> Result<()> {
    let event_id = id("evt");
    let payload: Value = json!({"id":event_id,"api_version":"2026-09-08","type":event_type,"created_at":chrono::Utc::now(),"merchant_id":payment.merchant_id,"environment":payment.environment,"revision":payment.revision,"data":api::public_payment(payment,false)});
    sqlx::query("INSERT INTO pay_merchant_events(id,merchant_id,environment,intent_id,revision,event_type,payload) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(&event_id).bind(&payment.merchant_id).bind(&payment.environment).bind(&payment.id).bind(payment.revision).bind(event_type).bind(payload).execute(&mut *conn).await?;
    let endpoints: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM pay_merchant_endpoints WHERE merchant_id=$1 AND environment=$2 AND enabled",
    )
    .bind(&payment.merchant_id)
    .bind(&payment.environment)
    .fetch_all(&mut *conn)
    .await?;
    for endpoint in endpoints {
        sqlx::query(
            "INSERT INTO pay_merchant_deliveries(id,event_id,endpoint_id) VALUES($1,$2,$3)",
        )
        .bind(id("del"))
        .bind(&event_id)
        .bind(endpoint)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
