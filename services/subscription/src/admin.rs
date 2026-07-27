//! Backend-owned administrator plans and access assignments.

use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use epsx_service_auth::VerifiedPrincipal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use super::AppState;

const MAX_LIMIT: i64 = 100;
const MAX_OFFSET: i64 = 10_000_000;

#[derive(Debug, Serialize, FromRow)]
pub struct AdminPlan {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub amount: String,
    pub currency: String,
    pub chain_id: String,
    pub interval: i32,
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRequest {
    pub merchant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub amount: String,
    pub currency: String,
    pub chain_id: String,
    pub interval: i32,
    pub active: Option<bool>,
    pub expected_version: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessRequest {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub permission: String,
    pub expected_version: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct PlanQuery {
    pub active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PlanListResponse {
    pub items: Vec<AdminPlan>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub correlation_id: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AccessAssignment {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub permission: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: i64,
    pub assigned_by: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AccessResponse {
    pub items: Vec<AccessAssignment>,
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

fn response<T: Serialize>(headers: &HeaderMap, status: StatusCode, value: T) -> Response {
    let id = correlation(headers);
    let mut response = (status, Json(value)).into_response();
    if let Ok(value) = id.parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn error(headers: &HeaderMap, status: StatusCode, code: &'static str) -> Response {
    let id = correlation(headers);
    let body = ErrorBody {
        error: ErrorInfo {
            code,
            correlation_id: id.clone(),
        },
    };
    let mut response = (status, Json(body)).into_response();
    if let Ok(value) = id.parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn idempotency_key(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("idempotency-key")?.to_str().ok()?.trim();
    if !(1..=128).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b' ')
    {
        return None;
    }
    Some(value.to_owned())
}

fn validate_plan(request: &PlanRequest, update: bool) -> Result<(), &'static str> {
    let valid_name = !request.name.trim().is_empty() && request.name.chars().count() <= 100;
    let valid_amount = !request.amount.is_empty()
        && request.amount.len() <= 78
        && request.amount.bytes().all(|byte| byte.is_ascii_digit());
    let valid_currency = !request.currency.is_empty()
        && request.currency.len() <= 10
        && request
            .currency
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
    let valid_chain = !request.chain_id.is_empty()
        && request.chain_id.len() <= 10
        && request.chain_id.bytes().all(|byte| byte.is_ascii_digit());
    let valid_description = request
        .description
        .as_ref()
        .is_none_or(|value| value.chars().count() <= 2_000);
    if !valid_name
        || !valid_amount
        || !valid_currency
        || !valid_chain
        || !(1..=366).contains(&request.interval)
        || !valid_description
    {
        return Err("invalid_plan");
    }
    if update && request.expected_version.is_none_or(|version| version < 0) {
        return Err("invalid_version");
    }
    Ok(())
}

fn canonical_wallet(value: &str) -> Option<String> {
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

pub async fn list_plans(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PlanQuery>,
) -> Response {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=MAX_LIMIT).contains(&limit) || !(0..=MAX_OFFSET).contains(&offset) {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_pagination");
    }
    let items = sqlx::query_as::<_, AdminPlan>(
        "SELECT p.id,p.merchant_id,p.name,p.description,p.amount,p.currency,p.chain_id,
                p.interval,p.active,p.created_at,COALESCE(s.version,0)
           FROM public.subscription_plans p
           LEFT JOIN public.subscription_plan_state s ON s.plan_id=p.id
          WHERE ($1::bool IS NULL OR p.active=$1)
          ORDER BY p.created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(query.active)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM public.subscription_plans WHERE ($1::bool IS NULL OR active=$1)",
    )
    .bind(query.active)
    .fetch_one(&state.db)
    .await;
    match (items, total) {
        (Ok(items), Ok(total)) => response(
            &headers,
            StatusCode::OK,
            PlanListResponse {
                items,
                total,
                limit,
                offset,
                correlation_id: correlation(&headers),
            },
        ),
        _ => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "plan_read_unavailable",
        ),
    }
}

pub async fn get_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Ok(id) = Uuid::parse_str(&id) else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_plan_id");
    };
    match sqlx::query_as::<_, AdminPlan>(
        "SELECT p.id,p.merchant_id,p.name,p.description,p.amount,p.currency,p.chain_id,
                p.interval,p.active,p.created_at,COALESCE(s.version,0)
           FROM public.subscription_plans p
           LEFT JOIN public.subscription_plan_state s ON s.plan_id=p.id
          WHERE p.id=$1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(plan)) => response(&headers, StatusCode::OK, plan),
        Ok(None) => error(&headers, StatusCode::NOT_FOUND, "plan_not_found"),
        Err(_) => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "plan_read_unavailable",
        ),
    }
}

pub async fn create_plan(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(request): Json<PlanRequest>,
) -> Response {
    if validate_plan(&request, false).is_err() {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_plan");
    }
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if let Ok(Some(value)) = sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.subscription_admin_operations WHERE idempotency_key=$1",
    )
    .bind(&key)
    .fetch_optional(&state.db)
    .await
    {
        return response(&headers, StatusCode::OK, value);
    }
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "plan_write_unavailable",
            )
        }
    };
    let plan = sqlx::query_as::<_, AdminPlan>(
        "INSERT INTO public.subscription_plans
            (merchant_id,name,description,amount,currency,chain_id,interval,active)
         VALUES($1,$2,$3,$4,$5,$6,$7,COALESCE($8,true))
         RETURNING id,merchant_id,name,description,amount,currency,chain_id,interval,active,created_at,0::bigint AS version",
    )
    .bind(request.merchant_id)
    .bind(request.name.trim())
    .bind(&request.description)
    .bind(&request.amount)
    .bind(&request.currency)
    .bind(&request.chain_id)
    .bind(request.interval)
    .bind(request.active)
    .fetch_one(&mut *tx)
    .await;
    let plan = match plan {
        Ok(plan) => plan,
        Err(_) => {
            return error(
                &headers,
                StatusCode::UNPROCESSABLE_ENTITY,
                "plan_write_rejected",
            )
        }
    };
    let plan_id = plan.id;
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({
        "plan": plan,
        "evidence": {"operation_id": operation_id, "version": 0}
    });
    let state_ok =
        sqlx::query("INSERT INTO public.subscription_plan_state(plan_id,version) VALUES($1,0)")
            .bind(plan_id)
            .execute(&mut *tx)
            .await
            .is_ok();
    let audit_ok = sqlx::query(
        "INSERT INTO public.subscription_admin_operations
            (operation_id,idempotency_key,action,resource_key,actor,version_before,version_after,result)
         VALUES($1,$2,'plan.create',$3,$4,0,0,$5)",
    )
    .bind(operation_id)
    .bind(&key)
    .bind(plan_id.to_string())
    .bind(&principal.subject)
    .bind(&result)
    .execute(&mut *tx)
    .await
    .is_ok();
    if !state_ok || !audit_ok {
        return error(&headers, StatusCode::CONFLICT, "idempotency_conflict");
    }
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "plan_write_unavailable",
        );
    }
    response(&headers, StatusCode::CREATED, result)
}

pub async fn update_plan(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<PlanRequest>,
) -> Response {
    let Ok(id) = Uuid::parse_str(&id) else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_plan_id");
    };
    if validate_plan(&request, true).is_err() {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_plan");
    }
    let Some(expected) = request.expected_version else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_version");
    };
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "plan_write_unavailable",
            )
        }
    };
    if let Ok(Some(value)) = sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.subscription_admin_operations WHERE idempotency_key=$1",
    )
    .bind(&key)
    .fetch_optional(&mut *tx)
    .await
    {
        return response(&headers, StatusCode::OK, value);
    }
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM public.subscription_plan_state WHERE plan_id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await;
    let current = match current {
        Ok(Some(value)) => value,
        Ok(None) => return error(&headers, StatusCode::NOT_FOUND, "plan_not_found"),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "plan_write_unavailable",
            )
        }
    };
    if current != expected {
        return error(&headers, StatusCode::CONFLICT, "stale_plan_version");
    }
    let next = current.saturating_add(1);
    let plan = sqlx::query_as::<_, AdminPlan>(
        "UPDATE public.subscription_plans
            SET name=$1,description=$2,amount=$3,currency=$4,chain_id=$5,
                interval=$6,active=COALESCE($7,active)
          WHERE id=$8
         RETURNING id,merchant_id,name,description,amount,currency,chain_id,interval,active,created_at,$9::bigint AS version",
    )
    .bind(request.name.trim())
    .bind(&request.description)
    .bind(&request.amount)
    .bind(&request.currency)
    .bind(&request.chain_id)
    .bind(request.interval)
    .bind(request.active)
    .bind(id)
    .bind(next)
    .fetch_one(&mut *tx)
    .await;
    let plan = match plan {
        Ok(plan) => plan,
        Err(_) => {
            return error(
                &headers,
                StatusCode::UNPROCESSABLE_ENTITY,
                "plan_write_rejected",
            )
        }
    };
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({
        "plan": plan,
        "evidence": {"operation_id": operation_id, "version": next}
    });
    let state_ok = sqlx::query(
        "UPDATE public.subscription_plan_state SET version=$1,updated_at=NOW() WHERE plan_id=$2 AND version=$3",
    )
    .bind(next)
    .bind(id)
    .bind(current)
    .execute(&mut *tx)
    .await
    .map_or(false, |result| result.rows_affected() == 1);
    let audit_ok = sqlx::query(
        "INSERT INTO public.subscription_admin_operations
            (operation_id,idempotency_key,action,resource_key,actor,version_before,version_after,result)
         VALUES($1,$2,'plan.update',$3,$4,$5,$6,$7)",
    )
    .bind(operation_id)
    .bind(&key)
    .bind(id.to_string())
    .bind(&principal.subject)
    .bind(current)
    .bind(next)
    .bind(&result)
    .execute(&mut *tx)
    .await
    .is_ok();
    if !state_ok || !audit_ok {
        return error(&headers, StatusCode::CONFLICT, "idempotency_conflict");
    }
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "plan_write_unavailable",
        );
    }
    response(&headers, StatusCode::OK, result)
}

pub async fn get_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let Some(wallet) = query
        .get("wallet_address")
        .and_then(|value| canonical_wallet(value))
    else {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    let items = sqlx::query_as::<_, AccessAssignment>(
        "SELECT a.wallet_address,a.plan_id,p.name,a.permission,a.expires_at,a.version,
                a.assigned_by,a.updated_at
           FROM public.subscription_access_assignments a
           JOIN public.subscription_plans p ON p.id=a.plan_id
          WHERE lower(a.wallet_address)=lower($1)
          ORDER BY p.name,a.permission",
    )
    .bind(wallet)
    .fetch_all(&state.db)
    .await;
    match items {
        Ok(items) => response(
            &headers,
            StatusCode::OK,
            AccessResponse {
                items,
                correlation_id: correlation(&headers),
            },
        ),
        Err(_) => error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "access_read_unavailable",
        ),
    }
}

pub async fn assign_access(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(request): Json<AccessRequest>,
) -> Response {
    mutate_access(&state, &principal, &headers, request, "assign").await
}

pub async fn revoke_access(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(request): Json<AccessRequest>,
) -> Response {
    mutate_access(&state, &principal, &headers, request, "revoke").await
}

async fn mutate_access(
    state: &AppState,
    principal: &VerifiedPrincipal,
    headers: &HeaderMap,
    request: AccessRequest,
    action: &str,
) -> Response {
    let Some(wallet) = canonical_wallet(&request.wallet_address) else {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_wallet_address");
    };
    if request.permission.is_empty()
        || request.permission.len() > 128
        || !request
            .permission
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-'))
        || request.expected_version < 0
    {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_access_request");
    }
    let Some(key) = idempotency_key(headers) else {
        return error(headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "access_write_unavailable",
            )
        }
    };
    if let Ok(Some(value)) = sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.subscription_admin_operations WHERE idempotency_key=$1",
    )
    .bind(&key)
    .fetch_optional(&mut *tx)
    .await
    {
        return response(headers, StatusCode::OK, value);
    }
    let plan_exists =
        sqlx::query_scalar::<_, i64>("SELECT 1 FROM public.subscription_plans WHERE id=$1")
            .bind(request.plan_id)
            .fetch_optional(&mut *tx)
            .await;
    if !matches!(plan_exists, Ok(Some(1))) {
        return error(headers, StatusCode::NOT_FOUND, "plan_not_found");
    }
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM public.subscription_access_assignments
          WHERE lower(wallet_address)=lower($1) AND plan_id=$2 AND permission=$3 FOR UPDATE",
    )
    .bind(&wallet)
    .bind(request.plan_id)
    .bind(&request.permission)
    .fetch_optional(&mut *tx)
    .await;
    let current = match current {
        Ok(Some(value)) => value,
        Ok(None) => 0,
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "access_write_unavailable",
            )
        }
    };
    if current != request.expected_version {
        return error(headers, StatusCode::CONFLICT, "stale_access_version");
    }
    let next = current.saturating_add(1);
    let changed = if action == "assign" {
        sqlx::query(
            "INSERT INTO public.subscription_access_assignments
                (wallet_address,plan_id,permission,version,assigned_by)
             VALUES($1,$2,$3,$4,$5)
             ON CONFLICT(wallet_address,plan_id,permission) DO UPDATE
                SET version=EXCLUDED.version,assigned_by=EXCLUDED.assigned_by,updated_at=NOW()",
        )
        .bind(&wallet)
        .bind(request.plan_id)
        .bind(&request.permission)
        .bind(next)
        .bind(&principal.subject)
        .execute(&mut *tx)
        .await
        .is_ok()
    } else {
        sqlx::query(
            "DELETE FROM public.subscription_access_assignments
              WHERE lower(wallet_address)=lower($1) AND plan_id=$2 AND permission=$3 AND version=$4",
        )
        .bind(&wallet)
        .bind(request.plan_id)
        .bind(&request.permission)
        .bind(current)
        .execute(&mut *tx)
        .await
        .map_or(false, |result| result.rows_affected() == 1)
    };
    if !changed {
        return error(headers, StatusCode::CONFLICT, "access_write_rejected");
    }
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({
        "wallet_address": wallet,
        "plan_id": request.plan_id,
        "permission": request.permission,
        "action": action,
        "version": next,
        "evidence": {"operation_id": operation_id, "version": next}
    });
    if sqlx::query(
        "INSERT INTO public.subscription_admin_operations
            (operation_id,idempotency_key,action,resource_key,actor,version_before,version_after,result)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(operation_id)
    .bind(&key)
    .bind(format!("access.{action}"))
    .bind(format!("{}:{}:{}", wallet, request.plan_id, request.permission))
    .bind(&principal.subject)
    .bind(current)
    .bind(next)
    .bind(&result)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(headers, StatusCode::CONFLICT, "idempotency_conflict");
    }
    if tx.commit().await.is_err() {
        return error(
            headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "access_write_unavailable",
        );
    }
    response(headers, StatusCode::OK, result)
}
