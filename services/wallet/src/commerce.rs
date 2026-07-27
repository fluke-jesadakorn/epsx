//! Backend-owned admin wallet and credit operations.
//!
//! These handlers are deliberately separate from owner wallet custody. Every
//! admin route is authorized by `protect_router`, validates the canonical
//! resource key again, and returns an evidence-bearing DTO after the database
//! transaction commits.

use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use epsx_service_auth::VerifiedPrincipal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use super::AppState;

const MAX_LIMIT: i64 = 100;
const MAX_OFFSET: i64 = 10_000_000;
const MAX_REASON_CHARS: usize = 500;
const MAX_METADATA_BYTES: usize = 4096;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletMutationRequest {
    pub expected_version: i64,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalletMetadataRequest {
    pub expected_version: i64,
    pub metadata: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditMutationRequest {
    pub expected_version: i64,
    pub amount_minor: i64,
    pub reason: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct WalletListQuery {
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AdminWallet {
    pub address: String,
    pub chain_id: String,
    pub label: Option<String>,
    pub role: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub version: i64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct WalletListResponse {
    pub items: Vec<AdminWallet>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub correlation_id: String,
}

#[derive(Debug, Serialize)]
pub struct WalletStatsResponse {
    pub total: i64,
    pub active: i64,
    pub disabled: i64,
    pub new_30_days: i64,
    pub correlation_id: String,
}

#[derive(Debug, Serialize)]
pub struct Evidence {
    pub operation_id: Uuid,
    pub version: i64,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WalletMutationResponse {
    pub wallet: AdminWallet,
    pub evidence: Evidence,
    pub correlation_id: String,
}

#[derive(Debug, Serialize)]
pub struct CreditAccountResponse {
    pub address: String,
    pub balance_minor: i64,
    pub version: i64,
    pub entries: Vec<CreditEntry>,
    pub correlation_id: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CreditEntry {
    pub entry_id: Uuid,
    pub operation: String,
    pub delta_minor: i64,
    pub balance_after_minor: i64,
    pub reason: String,
    pub actor: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreditStatsResponse {
    pub outstanding_minor: i64,
    pub granted_today_minor: i64,
    pub revoked_today_minor: i64,
    pub active_accounts: i64,
    pub correlation_id: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorInfo,
}

#[derive(Debug, Serialize)]
struct ErrorInfo {
    code: &'static str,
    correlation_id: String,
}

fn correlation(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| Uuid::parse_str(value).is_ok())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn error(headers: &HeaderMap, status: StatusCode, code: &'static str) -> Response {
    let correlation_id = correlation(headers);
    let mut response = (
        status,
        Json(ErrorBody {
            error: ErrorInfo {
                code,
                correlation_id: correlation_id.clone(),
            },
        }),
    )
        .into_response();
    if let Ok(value) = correlation_id.parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn json<T: Serialize>(headers: &HeaderMap, status: StatusCode, body: T) -> Response {
    let correlation_id = correlation(headers);
    let mut response = (status, Json(body)).into_response();
    if let Ok(value) = correlation_id.parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn idempotency_key(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("idempotency-key")?.to_str().ok()?.trim();
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|b| b.is_ascii_graphic() && b != b' ')
    {
        return None;
    }
    Some(value.to_owned())
}

fn canonical_address(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() != 42
        || bytes[0] != b'0'
        || !matches!(bytes[1], b'x' | b'X')
        || !bytes[2..].iter().all(u8::is_ascii_hexdigit)
    {
        return None;
    }
    Some(format!("0x{}", value[2..].to_ascii_lowercase()))
}

fn bounded_page(query: &WalletListQuery) -> Result<(i64, i64), &'static str> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=MAX_LIMIT).contains(&limit) || !(0..=MAX_OFFSET).contains(&offset) {
        return Err("invalid_pagination");
    }
    if query.status.as_ref().is_some_and(|value| {
        value.len() > 16
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    }) {
        return Err("invalid_status");
    }
    if query.search.as_ref().is_some_and(|value| {
        value.len() > 42
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'x' | b'X' | b'_'))
    }) {
        return Err("invalid_search");
    }
    Ok((limit, offset))
}

async fn wallet_by_address(
    db: &sqlx::PgPool,
    address: &str,
) -> Result<Option<AdminWallet>, sqlx::Error> {
    sqlx::query_as::<_, AdminWallet>(
        "SELECT a.address, a.chain_id, a.label, a.role,
                COALESCE(s.status, 'active') AS status,
                COALESCE(s.metadata, '{}'::jsonb) AS metadata,
                COALESCE(s.version, 0) AS version,
                a.created_at
           FROM public.accounts a
           LEFT JOIN public.wallet_admin_state s
             ON lower(s.address) = lower(a.address) AND s.chain_id = a.chain_id
          WHERE lower(a.address) = $1
          ORDER BY a.created_at DESC
          LIMIT 1",
    )
    .bind(address)
    .fetch_optional(db)
    .await
}

pub async fn list_admin_wallets(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WalletListQuery>,
) -> Response {
    let (limit, offset) = match bounded_page(&query) {
        Ok(page) => page,
        Err(code) => return error(&headers, StatusCode::BAD_REQUEST, code),
    };
    let status = query.status.as_deref();
    let search = query.search.as_deref();
    let rows = sqlx::query_as::<_, AdminWallet>(
        "SELECT a.address, a.chain_id, a.label, a.role,
                COALESCE(s.status, 'active') AS status,
                COALESCE(s.metadata, '{}'::jsonb) AS metadata,
                COALESCE(s.version, 0) AS version,
                a.created_at
           FROM public.accounts a
           LEFT JOIN public.wallet_admin_state s ON lower(s.address) = lower(a.address) AND s.chain_id = a.chain_id
          WHERE ($1::text IS NULL OR COALESCE(s.status, 'active') = $1)
            AND ($2::text IS NULL OR lower(a.address) LIKE lower($2))
          ORDER BY a.created_at DESC LIMIT $3 OFFSET $4",
    )
    .bind(status)
    .bind(search.map(|value| format!("%{value}%")))
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await;
    let items = match rows {
        Ok(items) => items,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet_read_unavailable",
            )
        }
    };
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM public.accounts a LEFT JOIN public.wallet_admin_state s ON lower(s.address) = lower(a.address) AND s.chain_id = a.chain_id WHERE ($1::text IS NULL OR COALESCE(s.status, 'active') = $1) AND ($2::text IS NULL OR lower(a.address) LIKE lower($2))",
    ).bind(status).bind(search.map(|value| format!("%{value}%"))).fetch_one(&state.db).await;
    let total = match total {
        Ok(total) => total,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet_read_unavailable",
            )
        }
    };
    let correlation_id = correlation(&headers);
    json(
        &headers,
        StatusCode::OK,
        WalletListResponse {
            items,
            total,
            limit,
            offset,
            correlation_id,
        },
    )
}

pub async fn admin_wallet_stats(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT COUNT(*)::bigint,
                COUNT(*) FILTER (WHERE COALESCE(s.status, 'active') = 'active')::bigint,
                COUNT(*) FILTER (WHERE COALESCE(s.status, 'active') = 'disabled')::bigint,
                COUNT(*) FILTER (WHERE a.created_at >= NOW() - INTERVAL '30 days')::bigint
           FROM public.accounts a
           LEFT JOIN public.wallet_admin_state s ON lower(s.address) = lower(a.address) AND s.chain_id = a.chain_id",
    ).fetch_one(&state.db).await;
    match counts {
        Ok((total, active, disabled, new_30_days)) => json(
            &headers,
            StatusCode::OK,
            WalletStatsResponse {
                total,
                active,
                disabled,
                new_30_days,
                correlation_id: correlation(&headers),
            },
        ),
        Err(_) => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet_stats_unavailable",
        ),
    }
}

pub async fn get_admin_wallet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Response {
    let Some(address) = canonical_address(&address) else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    match wallet_by_address(&state.db, &address).await {
        Ok(Some(wallet)) => json(&headers, StatusCode::OK, wallet),
        Ok(None) => error(&headers, StatusCode::NOT_FOUND, "wallet_not_found"),
        Err(_) => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet_read_unavailable",
        ),
    }
}

pub async fn disable_admin_wallet(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(address): Path<String>,
    Json(request): Json<WalletMutationRequest>,
) -> Response {
    mutate_wallet_status(&state, &principal, &headers, &address, request, "disabled").await
}

pub async fn enable_admin_wallet(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(address): Path<String>,
    Json(request): Json<WalletMutationRequest>,
) -> Response {
    mutate_wallet_status(&state, &principal, &headers, &address, request, "active").await
}

async fn mutate_wallet_status(
    state: &AppState,
    principal: &VerifiedPrincipal,
    headers: &HeaderMap,
    raw_address: &str,
    request: WalletMutationRequest,
    status: &str,
) -> Response {
    let Some(address) = canonical_address(raw_address) else {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    let Some(key) = idempotency_key(headers) else {
        return error(headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if request.expected_version < 0
        || request.reason.as_deref().is_some_and(|reason| {
            reason.trim().is_empty()
                || reason.chars().count() > MAX_REASON_CHARS
                || reason.chars().any(char::is_control)
        })
    {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_wallet_request");
    }
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet_write_unavailable",
            )
        }
    };
    if let Ok(Some(result)) = existing_operation(&mut tx, &key).await {
        return json(headers, StatusCode::OK, result);
    }
    let current = match locked_wallet_state(&mut tx, &address).await {
        Ok(Some(value)) => value,
        Ok(None) => return error(headers, StatusCode::NOT_FOUND, "wallet_not_found"),
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet_write_unavailable",
            )
        }
    };
    if current.1 != request.expected_version {
        return error(headers, StatusCode::CONFLICT, "stale_wallet_version");
    }
    let Some(next_version) = current.1.checked_add(1) else {
        return error(headers, StatusCode::CONFLICT, "wallet_version_exhausted");
    };
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({"address": address, "chain_id": current.0, "status": status, "version": next_version, "operation_id": operation_id, "reason": request.reason});
    if sqlx::query("INSERT INTO public.wallet_admin_state (address, chain_id, status, metadata, version, updated_at) VALUES ($1, $2, $3, '{}'::jsonb, $4, NOW()) ON CONFLICT (address, chain_id) DO UPDATE SET status = EXCLUDED.status, version = EXCLUDED.version, updated_at = NOW()")
        .bind(&address).bind(&current.0).bind(status).bind(next_version).execute(&mut *tx).await.is_err() { return error(headers, StatusCode::SERVICE_UNAVAILABLE, "wallet_write_unavailable"); }
    if sqlx::query("INSERT INTO public.wallet_admin_operations (operation_id, idempotency_key, address, chain_id, action, actor, version_before, version_after, result) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
        .bind(operation_id).bind(&key).bind(&address).bind(&current.0).bind(status).bind(&principal.subject).bind(current.1).bind(next_version).bind(&result).execute(&mut *tx).await.is_err() { return error(headers, StatusCode::CONFLICT, "idempotency_conflict"); }
    if tx.commit().await.is_err() {
        return error(
            headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet_write_unavailable",
        );
    }
    let wallet = match wallet_by_address(&state.db, &address).await {
        Ok(Some(wallet)) => wallet,
        _ => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "read_after_write_unavailable",
            )
        }
    };
    json(
        headers,
        StatusCode::OK,
        WalletMutationResponse {
            wallet,
            evidence: Evidence {
                operation_id,
                version: next_version,
                observed_at: Utc::now(),
            },
            correlation_id: correlation(headers),
        },
    )
}

pub async fn update_admin_wallet_metadata(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(address): Path<String>,
    Json(request): Json<WalletMetadataRequest>,
) -> Response {
    let Some(address) = canonical_address(&address) else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    if request.expected_version < 0
        || !request.metadata.is_object()
        || serde_json::to_vec(&request.metadata)
            .map_or(true, |bytes| bytes.len() > MAX_METADATA_BYTES)
    {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_metadata");
    }
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet_write_unavailable",
            )
        }
    };
    if let Ok(Some(result)) = existing_operation(&mut tx, &key).await {
        return json(&headers, StatusCode::OK, result);
    }
    let current = match locked_wallet_state(&mut tx, &address).await {
        Ok(Some(value)) => value,
        Ok(None) => return error(&headers, StatusCode::NOT_FOUND, "wallet_not_found"),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet_write_unavailable",
            )
        }
    };
    if current.1 != request.expected_version {
        return error(&headers, StatusCode::CONFLICT, "stale_wallet_version");
    }
    let Some(next_version) = current.1.checked_add(1) else {
        return error(&headers, StatusCode::CONFLICT, "wallet_version_exhausted");
    };
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({"address": address, "chain_id": current.0, "version": next_version, "operation_id": operation_id});
    if sqlx::query("INSERT INTO public.wallet_admin_state (address, chain_id, status, metadata, version, updated_at) VALUES ($1,$2,'active',$3,$4,NOW()) ON CONFLICT (address,chain_id) DO UPDATE SET metadata=EXCLUDED.metadata, version=EXCLUDED.version, updated_at=NOW()")
        .bind(&address).bind(&current.0).bind(&request.metadata).bind(next_version).execute(&mut *tx).await.is_err() { return error(&headers, StatusCode::SERVICE_UNAVAILABLE, "wallet_write_unavailable") }
    if sqlx::query("INSERT INTO public.wallet_admin_operations (operation_id,idempotency_key,address,chain_id,action,actor,version_before,version_after,result) VALUES ($1,$2,$3,$4,'metadata',$5,$6,$7,$8)")
        .bind(operation_id).bind(&key).bind(&address).bind(&current.0).bind(&principal.subject).bind(current.1).bind(next_version).bind(&result).execute(&mut *tx).await.is_err() { return error(&headers, StatusCode::CONFLICT, "idempotency_conflict") }
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet_write_unavailable",
        );
    }
    let wallet = match wallet_by_address(&state.db, &address).await {
        Ok(Some(wallet)) => wallet,
        _ => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "read_after_write_unavailable",
            )
        }
    };
    json(
        &headers,
        StatusCode::OK,
        WalletMutationResponse {
            wallet,
            evidence: Evidence {
                operation_id,
                version: next_version,
                observed_at: Utc::now(),
            },
            correlation_id: correlation(&headers),
        },
    )
}

async fn locked_wallet_state(
    tx: &mut Transaction<'_, Postgres>,
    address: &str,
) -> Result<Option<(String, i64)>, sqlx::Error> {
    let account = sqlx::query_as::<_, (String,)>(
        "SELECT chain_id
           FROM public.accounts
          WHERE lower(address)=$1
          ORDER BY created_at DESC
          LIMIT 1
          FOR UPDATE",
    )
    .bind(address)
    .fetch_optional(&mut **tx)
    .await?;
    let Some((chain_id,)) = account else {
        return Ok(None);
    };
    let version = sqlx::query_scalar::<_, i64>(
        "SELECT version
           FROM public.wallet_admin_state
          WHERE lower(address)=lower($1) AND chain_id=$2
          FOR UPDATE",
    )
    .bind(address)
    .bind(&chain_id)
    .fetch_optional(&mut **tx)
    .await?
    .unwrap_or(0);
    Ok(Some((chain_id, version)))
}

async fn existing_operation(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
) -> Result<Option<Value>, sqlx::Error> {
    sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.wallet_admin_operations WHERE idempotency_key=$1",
    )
    .bind(key)
    .fetch_optional(&mut **tx)
    .await
}

pub async fn admin_credit_stats(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let result = sqlx::query_as::<_, (i64, i64, i64, i64)>("SELECT COALESCE((SELECT SUM(balance_minor) FROM public.wallet_credit_accounts),0)::bigint, COALESCE((SELECT SUM(delta_minor) FROM public.wallet_credit_ledger WHERE operation='grant' AND created_at >= CURRENT_DATE),0)::bigint, COALESCE((SELECT SUM(ABS(delta_minor)) FROM public.wallet_credit_ledger WHERE operation='revoke' AND created_at >= CURRENT_DATE),0)::bigint, COUNT(*)::bigint FROM public.wallet_credit_accounts WHERE balance_minor > 0").fetch_one(&state.db).await;
    match result {
        Ok((outstanding_minor, granted_today_minor, revoked_today_minor, active_accounts)) => json(
            &headers,
            StatusCode::OK,
            CreditStatsResponse {
                outstanding_minor,
                granted_today_minor,
                revoked_today_minor,
                active_accounts,
                correlation_id: correlation(&headers),
            },
        ),
        Err(_) => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "credit_stats_unavailable",
        ),
    }
}

pub async fn get_admin_credits(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Response {
    let Some(address) = canonical_address(&address) else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    let wallet_exists = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM public.accounts WHERE lower(address)=lower($1) LIMIT 1",
    )
    .bind(&address)
    .fetch_optional(&state.db)
    .await;
    match wallet_exists {
        Ok(Some(1)) => {}
        Ok(_) => return error(&headers, StatusCode::NOT_FOUND, "wallet_not_found"),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "credit_read_unavailable",
            )
        }
    }
    let account = sqlx::query_as::<_, (i64, i64)>(
        "SELECT balance_minor, version FROM public.wallet_credit_accounts WHERE address=$1",
    )
    .bind(&address)
    .fetch_optional(&state.db)
    .await;
    let (balance_minor, version) = match account {
        Ok(Some(value)) => value,
        Ok(None) => (0, 0),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "credit_read_unavailable",
            )
        }
    };
    let entries = sqlx::query_as::<_, CreditEntry>("SELECT entry_id, operation, delta_minor, balance_after_minor, reason, actor, created_at FROM public.wallet_credit_ledger WHERE address=$1 ORDER BY created_at DESC LIMIT 100").bind(&address).fetch_all(&state.db).await;
    match entries {
        Ok(entries) => json(
            &headers,
            StatusCode::OK,
            CreditAccountResponse {
                address,
                balance_minor,
                version,
                entries,
                correlation_id: correlation(&headers),
            },
        ),
        Err(_) => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "credit_read_unavailable",
        ),
    }
}

pub async fn grant_admin_credits(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(address): Path<String>,
    Json(request): Json<CreditMutationRequest>,
) -> Response {
    mutate_credits(&state, &principal, &headers, &address, request, "grant").await
}
pub async fn revoke_admin_credits(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(address): Path<String>,
    Json(request): Json<CreditMutationRequest>,
) -> Response {
    mutate_credits(&state, &principal, &headers, &address, request, "revoke").await
}

async fn mutate_credits(
    state: &AppState,
    principal: &VerifiedPrincipal,
    headers: &HeaderMap,
    raw_address: &str,
    request: CreditMutationRequest,
    operation: &str,
) -> Response {
    let Some(address) = canonical_address(raw_address) else {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    let Some(key) = idempotency_key(headers) else {
        return error(headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if !(1..=1_000_000_000_000_i64).contains(&request.amount_minor)
        || request.expected_version < 0
        || request.reason.trim().is_empty()
        || request.reason.chars().count() > MAX_REASON_CHARS
    {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_credit_request");
    }
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "credit_write_unavailable",
            )
        }
    };
    if let Ok(Some(result)) = sqlx::query_scalar::<_, Value>(
        "SELECT result
           FROM public.wallet_credit_ledger
          WHERE idempotency_key=$1 AND result IS NOT NULL",
    )
    .bind(&key)
    .fetch_optional(&mut *tx)
    .await
    {
        return json(headers, StatusCode::OK, result);
    }
    let wallet_exists = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM public.accounts WHERE lower(address)=lower($1) LIMIT 1",
    )
    .bind(&address)
    .fetch_optional(&mut *tx)
    .await;
    match wallet_exists {
        Ok(Some(1)) => {}
        Ok(_) => return error(headers, StatusCode::NOT_FOUND, "wallet_not_found"),
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "credit_write_unavailable",
            )
        }
    }
    let current = sqlx::query_as::<_, (i64, i64)>("INSERT INTO public.wallet_credit_accounts(address) VALUES($1) ON CONFLICT(address) DO NOTHING RETURNING balance_minor, version").bind(&address).fetch_optional(&mut *tx).await;
    let current = match current { Ok(Some(value)) => value, Ok(None) => match sqlx::query_as::<_, (i64, i64)>("SELECT balance_minor, version FROM public.wallet_credit_accounts WHERE address=$1 FOR UPDATE").bind(&address).fetch_one(&mut *tx).await { Ok(value) => value, Err(_) => return error(headers, StatusCode::SERVICE_UNAVAILABLE, "credit_write_unavailable") }, Err(_) => return error(headers, StatusCode::SERVICE_UNAVAILABLE, "credit_write_unavailable") };
    if current.1 != request.expected_version {
        return error(headers, StatusCode::CONFLICT, "stale_credit_version");
    }
    let delta = if operation == "grant" {
        request.amount_minor
    } else {
        -request.amount_minor
    };
    let balance_after = match current.0.checked_add(delta) {
        Some(value) if value >= 0 => value,
        _ => {
            return error(
                headers,
                StatusCode::UNPROCESSABLE_ENTITY,
                "insufficient_credits",
            )
        }
    };
    let Some(next_version) = current.1.checked_add(1) else {
        return error(headers, StatusCode::CONFLICT, "credit_version_exhausted");
    };
    let entry_id = Uuid::new_v4();
    let observed_at = Utc::now();
    let result = serde_json::json!({
        "address": address,
        "balance_minor": balance_after,
        "version": next_version,
        "evidence": {
            "operation_id": entry_id,
            "version": next_version,
            "observed_at": observed_at,
        },
        "correlation_id": correlation(headers),
    });
    if sqlx::query("UPDATE public.wallet_credit_accounts SET balance_minor=$1, version=$2, updated_at=NOW() WHERE address=$3 AND version=$4") .bind(balance_after).bind(next_version).bind(&address).bind(current.1).execute(&mut *tx).await.map_or(true, |result| result.rows_affected()!=1) { return error(headers, StatusCode::CONFLICT, "stale_credit_version") }
    if sqlx::query("INSERT INTO public.wallet_credit_ledger(entry_id,idempotency_key,address,operation,delta_minor,balance_after_minor,reason,actor,result) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)").bind(entry_id).bind(&key).bind(&address).bind(operation).bind(delta).bind(balance_after).bind(request.reason.trim()).bind(&principal.subject).bind(&result).execute(&mut *tx).await.is_err() { return error(headers, StatusCode::CONFLICT, "idempotency_conflict") }
    if tx.commit().await.is_err() {
        return error(
            headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "credit_write_unavailable",
        );
    }
    let observed = sqlx::query_as::<_, (i64, i64)>(
        "SELECT balance_minor, version
           FROM public.wallet_credit_accounts
          WHERE address=$1",
    )
    .bind(&address)
    .fetch_optional(&state.db)
    .await;
    let read_after_write_ok = match observed {
        Ok(Some((balance, version))) => balance == balance_after && version == next_version,
        _ => false,
    };
    if !read_after_write_ok {
        return error(
            headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "credit_read_after_write_unavailable",
        );
    }
    json(headers, StatusCode::OK, result)
}

#[allow(dead_code)]
fn decode_strict<T: DeserializeOwned>(value: Value) -> Result<T, &'static str> {
    serde_json::from_value(value).map_err(|_| "malformed_response")
}
