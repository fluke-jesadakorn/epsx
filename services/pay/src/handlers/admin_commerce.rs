//! Audited admin payment-link and pending-intent operations.

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

use crate::{types::PayIntent, AppState};

const MAX_LIMIT: i64 = 100;
const MAX_OFFSET: i64 = 10_000_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateAdminLinkRequest {
    pub intent_id: String,
    pub max_uses: Option<i32>,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRequest {
    pub expected_version: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct LinkQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AdminPayLink {
    pub id: String,
    pub slug: String,
    pub intent_id: String,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub status: String,
    pub version: i64,
}

#[derive(Debug, Serialize)]
pub struct LinkListResponse {
    pub items: Vec<AdminPayLink>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
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
    let mut response = (
        status,
        Json(ErrorBody {
            error: ErrorInfo {
                code,
                correlation_id: id.clone(),
            },
        }),
    )
        .into_response();
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

fn valid_resource_id(value: &str) -> bool {
    (1..=66).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'x' || byte == b'-' || byte == b'_')
}

pub async fn list_admin_pay_links(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LinkQuery>,
) -> Response {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    if !(1..=MAX_LIMIT).contains(&limit) || !(0..=MAX_OFFSET).contains(&offset) {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_pagination");
    };
    if query
        .status
        .as_ref()
        .is_some_and(|value| !matches!(value.as_str(), "active" | "disabled"))
    {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_status");
    };
    let items=sqlx::query_as::<_,AdminPayLink>("SELECT l.id,l.slug,l.intent_id,l.max_uses,l.current_uses,l.expires_at,l.created_at,COALESCE(s.status,'active'),COALESCE(s.version,0) FROM public.pay_links l LEFT JOIN public.pay_link_admin_state s ON s.link_id=l.id WHERE ($1::text IS NULL OR COALESCE(s.status,'active')=$1) ORDER BY l.created_at DESC LIMIT $2 OFFSET $3").bind(query.status.as_deref()).bind(limit).bind(offset).fetch_all(&state.db).await;
    let total=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM public.pay_links l LEFT JOIN public.pay_link_admin_state s ON s.link_id=l.id WHERE ($1::text IS NULL OR COALESCE(s.status,'active')=$1)").bind(query.status.as_deref()).fetch_one(&state.db).await;
    match (items, total) {
        (Ok(items), Ok(total)) => response(
            &headers,
            StatusCode::OK,
            LinkListResponse {
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
            "payment_link_read_unavailable",
        ),
    }
}

pub async fn create_admin_pay_link(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Json(request): Json<CreateAdminLinkRequest>,
) -> Response {
    if !valid_resource_id(&request.intent_id)
        || request
            .max_uses
            .is_some_and(|value| !(0..=1_000_000).contains(&value))
        || request
            .expires_in
            .is_some_and(|value| !(60..=31_536_000).contains(&value))
    {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_payment_link");
    };
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if let Ok(Some(value)) = sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.pay_admin_operations WHERE idempotency_key=$1",
    )
    .bind(&key)
    .fetch_optional(&state.db)
    .await
    {
        return response(&headers, StatusCode::OK, value);
    };
    let intent=sqlx::query_as::<_,PayIntent>("SELECT id,chain_id,payer,payee,amount,token_address,status,escrow_id,tx_hash,description,expires_at,created_at,updated_at FROM public.pay_intents WHERE id=$1").bind(&request.intent_id).fetch_optional(&state.db).await;
    let intent = match intent {
        Ok(Some(intent)) => intent,
        Ok(None) => return error(&headers, StatusCode::NOT_FOUND, "payment_intent_not_found"),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_link_write_unavailable",
            )
        }
    };
    let id = format!("0x{}", Uuid::new_v4().simple());
    let slug = format!(
        "epsx-{}",
        Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(12)
            .collect::<String>()
    );
    let now = Utc::now();
    let expires = request
        .expires_in
        .and_then(|seconds| now.checked_add_signed(chrono::Duration::seconds(seconds)));
    let max_uses = request.max_uses.unwrap_or(1);
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_link_write_unavailable",
            )
        }
    };
    if sqlx::query("INSERT INTO public.pay_links(id,slug,intent_id,max_uses,current_uses,expires_at,created_at) VALUES($1,$2,$3,$4,0,$5,$6)").bind(&id).bind(&slug).bind(&intent.id).bind(max_uses).bind(expires).bind(now).execute(&mut *tx).await.is_err(){return error(&headers,StatusCode::UNPROCESSABLE_ENTITY,"payment_link_write_rejected")};
    if sqlx::query(
        "INSERT INTO public.pay_link_admin_state(link_id,status,version) VALUES($1,'active',0)",
    )
    .bind(&id)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "payment_link_write_rejected",
        );
    };
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({"link":{"id":id,"slug":slug,"intent_id":intent.id,"max_uses":max_uses,"current_uses":0,"expires_at":expires,"created_at":now,"status":"active","version":0},"evidence":{"operation_id":operation_id,"financial_finality":"not_applicable"},"actor":principal.subject});
    if sqlx::query("INSERT INTO public.pay_admin_operations(operation_id,idempotency_key,action,resource_id,actor,result) VALUES($1,$2,'link.create',$3,$4,$5)").bind(operation_id).bind(&key).bind(&request.intent_id).bind(&principal.subject).bind(&result).execute(&mut *tx).await.is_err(){return error(&headers,StatusCode::CONFLICT,"idempotency_conflict")};
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_link_write_unavailable",
        );
    };
    response(&headers, StatusCode::CREATED, result)
}

pub async fn disable_admin_pay_link(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<MutationRequest>,
) -> Response {
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if request.expected_version < 0 || !valid_resource_id(&id) {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_payment_link");
    }
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_link_write_unavailable",
            )
        }
    };
    if let Ok(Some(value)) = sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.pay_admin_operations WHERE idempotency_key=$1",
    )
    .bind(&key)
    .fetch_optional(&mut *tx)
    .await
    {
        return response(&headers, StatusCode::OK, value);
    }
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM public.pay_link_admin_state WHERE link_id=$1 FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await;
    let Some(current) = (match current {
        Ok(value) => value,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_link_write_unavailable",
            )
        }
    }) else {
        return error(&headers, StatusCode::NOT_FOUND, "payment_link_not_found");
    };
    if current != request.expected_version {
        return error(&headers, StatusCode::CONFLICT, "stale_payment_link_version");
    }
    let Some(next) = current.checked_add(1) else {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "payment_link_version_exhausted",
        );
    };
    if sqlx::query(
        "UPDATE public.pay_link_admin_state SET status='disabled',version=$1,updated_at=NOW() WHERE link_id=$2 AND version=$3",
    )
    .bind(next)
    .bind(&id)
    .bind(current)
    .execute(&mut *tx)
    .await
    .map_or(true, |result| result.rows_affected() != 1)
    {
        return error(&headers, StatusCode::CONFLICT, "payment_link_write_rejected");
    }
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({"id":id,"status":"disabled","version":next,"evidence":{"operation_id":operation_id,"financial_finality":"not_applicable"},"actor":principal.subject});
    if sqlx::query(
        "INSERT INTO public.pay_admin_operations(operation_id,idempotency_key,action,resource_id,actor,resource_version_before,resource_version_after,result) VALUES($1,$2,'link.disable',$3,$4,$5,$6,$7)",
    )
    .bind(operation_id)
    .bind(&key)
    .bind(&id)
    .bind(&principal.subject)
    .bind(current)
    .bind(next)
    .bind(&result)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(&headers, StatusCode::CONFLICT, "idempotency_conflict");
    }
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_link_write_unavailable",
        );
    }
    let observed = sqlx::query_as::<_, (String, i64)>(
        "SELECT COALESCE(status, 'active'), version
           FROM public.pay_link_admin_state
          WHERE link_id=$1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;
    let read_after_write_ok = match observed {
        Ok(Some((status, version))) => status == "disabled" && version == next,
        _ => false,
    };
    if !read_after_write_ok {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_link_read_after_write_unavailable",
        );
    }
    response(&headers, StatusCode::OK, result)
}

pub async fn cancel_admin_pay_intent(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<MutationRequest>,
) -> Response {
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if request.expected_version < 0 || !valid_resource_id(&id) {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_payment_intent");
    };
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_write_unavailable",
            )
        }
    };
    if let Ok(Some(value)) = sqlx::query_scalar::<_, Value>(
        "SELECT result FROM public.pay_admin_operations WHERE idempotency_key=$1",
    )
    .bind(&key)
    .fetch_optional(&mut *tx)
    .await
    {
        return response(&headers, StatusCode::OK, value);
    };
    let intent=sqlx::query_as::<_,PayIntent>("SELECT id,chain_id,payer,payee,amount,token_address,status,escrow_id,tx_hash,description,expires_at,created_at,updated_at FROM public.pay_intents WHERE id=$1 FOR UPDATE").bind(&id).fetch_optional(&mut *tx).await;
    let intent = match intent {
        Ok(Some(value)) => value,
        Ok(None) => return error(&headers, StatusCode::NOT_FOUND, "payment_intent_not_found"),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_write_unavailable",
            )
        }
    };
    let version_before = intent.updated_at;
    if request.expected_version > 0 && request.expected_version != version_before.timestamp() {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "stale_payment_intent_version",
        );
    };
    if intent.status != "pending" {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "payment_intent_not_cancellable",
        );
    };
    let updated=sqlx::query("UPDATE public.pay_intents SET status='cancelled',updated_at=NOW() WHERE id=$1 AND status='pending'").bind(&id).execute(&mut *tx).await;
    if updated.map_or(true, |result| result.rows_affected() != 1) {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "payment_intent_write_rejected",
        );
    };
    let operation_id = Uuid::new_v4();
    let now = Utc::now();
    if sqlx::query("INSERT INTO public.pay_admin_operations(operation_id,idempotency_key,action,resource_id,actor,version_before,version_after,result) VALUES($1,$2,'intent.cancel',$3,$4,$5,$6,$7)").bind(operation_id).bind(&key).bind(&id).bind(&principal.subject).bind(version_before).bind(now).bind(serde_json::json!({"id":id,"status":"cancelled"})).execute(&mut *tx).await.is_err(){return error(&headers,StatusCode::CONFLICT,"idempotency_conflict")};
    if sqlx::query("INSERT INTO public.pay_ledger_entries(operation_id,resource_id,entry_type,status,amount,token_address,chain_id,finalized_at) VALUES($1,$2,'intent.cancel','finalized',$3,$4,$5,$6)").bind(operation_id).bind(&id).bind(&intent.amount).bind(&intent.token_address).bind(&intent.chain_id).bind(now).execute(&mut *tx).await.is_err(){return error(&headers,StatusCode::CONFLICT,"ledger_write_rejected")};
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_write_unavailable",
        );
    };
    response(
        &headers,
        StatusCode::OK,
        serde_json::json!({"id":id,"status":"cancelled","evidence":{"operation_id":operation_id,"observed_at":now,"financial_finality":"not_applicable","ledger_entry":"finalized"},"correlation_id":correlation(&headers)}),
    )
}
