use super::{bad, hash, missing, Error, Platform, Result};
use axum::http::{HeaderMap, StatusCode};
use epsx_service_auth::VerifiedPrincipal;
use serde::Serialize;
use sqlx::Row;

#[derive(Clone, Serialize)]
pub struct Actor {
    pub merchant_id: String,
    pub wallet: String,
    pub environment: String,
    pub owner_session: bool,
    pub admin: bool,
}
pub async fn principal(p: &Platform, h: &HeaderMap) -> Result<VerifiedPrincipal> {
    let principal = epsx_service_auth::authenticate_headers(p.verifier.as_ref(), h)
        .await
        .map_err(|_| Error(StatusCode::UNAUTHORIZED, "authentication_required"))?;
    if !["epsx-pay", "epsx-admin"].contains(&principal.audience.as_str()) {
        return Err(Error(StatusCode::FORBIDDEN, "wrong_audience"));
    }
    Ok(principal)
}
pub fn environment(h: &HeaderMap) -> Result<String> {
    let env = h
        .get("x-pay-environment")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("test");
    if !["test", "live"].contains(&env) {
        return Err(bad("invalid_environment"));
    }
    Ok(env.into())
}
pub async fn actor(p: &Platform, h: &HeaderMap) -> Result<Actor> {
    let bearer = epsx_service_auth::extract_bearer(h)
        .map_err(|_| Error(StatusCode::UNAUTHORIZED, "authentication_required"))?;
    if bearer.starts_with("epsxpay_") {
        let row=sqlx::query("SELECT k.merchant_id,k.environment,m.owner FROM pay_merchant_keys k JOIN pay_merchants m ON m.id=k.merchant_id WHERE k.key_hash=$1 AND k.revoked_at IS NULL")
            .bind(hash(bearer.as_bytes())).fetch_optional(&p.db).await?.ok_or(Error(StatusCode::UNAUTHORIZED,"invalid_api_key"))?;
        let env: String = row.get("environment");
        if h.contains_key("x-pay-environment") && environment(h)? != env {
            return Err(Error(StatusCode::FORBIDDEN, "environment_mismatch"));
        }
        Ok(Actor {
            merchant_id: row.get("merchant_id"),
            wallet: row.get("owner"),
            environment: env,
            owner_session: false,
            admin: false,
        })
    } else {
        let principal = principal(p, h).await?;
        let wallet = principal.wallet_address.to_ascii_lowercase();
        let admin =
            principal.audience == "epsx-admin" && principal.has_permission("admin:payments:manage");
        let merchant_id = sqlx::query_scalar("SELECT id FROM pay_merchants WHERE owner=$1")
            .bind(&wallet)
            .fetch_optional(&p.db)
            .await?;
        let merchant_id = match merchant_id {
            Some(id) => id,
            None if admin => String::new(),
            None => return Err(missing()),
        };
        Ok(Actor {
            merchant_id,
            wallet,
            environment: environment(h)?,
            owner_session: true,
            admin,
        })
    }
}
pub fn require_owner(a: &Actor) -> Result<()> {
    if a.owner_session {
        Ok(())
    } else {
        Err(Error(
            StatusCode::FORBIDDEN,
            "signed_merchant_session_required",
        ))
    }
}
pub async fn rate(p: &Platform, scope: &str, limit: i32) -> Result<()> {
    let window = chrono::Utc::now().timestamp() / 60;
    let count:i32=sqlx::query_scalar("INSERT INTO pay_merchant_rate_limits(scope,window_start,requests) VALUES($1,$2,1) ON CONFLICT(scope,window_start) DO UPDATE SET requests=pay_merchant_rate_limits.requests+1 RETURNING requests")
        .bind(scope).bind(window).fetch_one(&p.db).await?;
    if count > limit {
        return Err(Error(StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded"));
    }
    Ok(())
}
