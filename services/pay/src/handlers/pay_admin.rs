//! Authoritative administrator payment reads and lifecycle mutations.
//!
//! Every mutation in this module is an optimistic, idempotent database
//! transition.  Escrow settlement is deliberately reported as pending
//! external settlement: changing the service state is durable evidence, but
//! it is not an on-chain finality claim.

use axum::{
    extract::{Extension, Path as AxPath, Query, State},
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

use crate::types::{EscrowRecord, PayIntent, PayIntentListResponse};
use crate::AppState;

const MAX_LIMIT: i64 = 100;
const MAX_OFFSET: i64 = 10_000_000;

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AdminPayIntentQuery {
    pub payer: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminPaymentMutationRequest {
    pub expected_version: i64,
}

#[derive(Debug, Serialize, FromRow)]
struct AdminEscrowVersion {
    status: String,
    updated_at: DateTime<Utc>,
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
        Json(serde_json::json!({
            "error": {"code": code, "correlation_id": correlation_id}
        })),
    )
        .into_response();
    if let Ok(value) = correlation_id.parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn response<T: Serialize>(headers: &HeaderMap, status: StatusCode, body: T) -> Response {
    let correlation_id = correlation(headers);
    let mut response = (status, Json(body)).into_response();
    if let Ok(value) = correlation_id.parse() {
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
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn canonical_payer(value: &str) -> Option<String> {
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

fn valid_status(value: &str) -> bool {
    matches!(
        value,
        "pending" | "escrowed" | "released" | "cancelled" | "expired"
    )
}

fn version_matches(expected: i64, actual: DateTime<Utc>) -> bool {
    expected > 0 && expected == actual.timestamp_micros()
}

enum ExistingOperation {
    Replay(Value),
    Absent,
    Conflict,
}

async fn existing_operation(
    executor: &mut sqlx::PgConnection,
    key: &str,
    action: &str,
    resource_id: &str,
) -> Result<ExistingOperation, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, Value)>(
        "SELECT action, resource_id, result
           FROM public.pay_admin_operations
          WHERE idempotency_key=$1",
    )
    .bind(key)
    .fetch_optional(&mut *executor)
    .await?;
    match row {
        Some((stored_action, stored_resource, result))
            if stored_action == action && stored_resource == resource_id =>
        {
            Ok(ExistingOperation::Replay(result))
        }
        Some(_) => Ok(ExistingOperation::Conflict),
        None => Ok(ExistingOperation::Absent),
    }
}

// ============================================================================
// GET /api/v1/admin/pay/intents
// ============================================================================

pub async fn admin_list_pay_intents(
    State(state): State<AppState>,
    Extension(_principal): Extension<VerifiedPrincipal>,
    Query(params): Query<AdminPayIntentQuery>,
) -> Result<Json<PayIntentListResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(1..=MAX_LIMIT).contains(&limit) || !(0..=MAX_OFFSET).contains(&offset) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if params
        .status
        .as_deref()
        .is_some_and(|status| !valid_status(status))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let payer = match params.payer.as_deref() {
        Some(value) => Some(canonical_payer(value).ok_or(StatusCode::BAD_REQUEST)?),
        None => None,
    };
    let items = sqlx::query_as::<_, PayIntent>(
        "SELECT id, chain_id, payer, payee, amount, token_address, status, escrow_id, tx_hash,
                description, expires_at, created_at, updated_at
           FROM public.pay_intents
          WHERE ($1::text IS NULL OR payer=$1)
            AND ($2::text IS NULL OR status=$2)
          ORDER BY created_at DESC LIMIT $3 OFFSET $4",
    )
    .bind(payer.as_deref())
    .bind(params.status.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM public.pay_intents
          WHERE ($1::text IS NULL OR payer=$1)
            AND ($2::text IS NULL OR status=$2)",
    )
    .bind(payer.as_deref())
    .bind(params.status.as_deref())
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(PayIntentListResponse { items, total }))
}

// ============================================================================
// POST /api/v1/admin/pay/intents/:id/force-cancel
// ============================================================================

pub async fn admin_force_cancel_pay_intent(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(request): Json<AdminPaymentMutationRequest>,
) -> Response {
    let Some(key) = idempotency_key(&headers) else {
        return error(&headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if !valid_resource_id(&id) || request.expected_version <= 0 {
        return error(&headers, StatusCode::BAD_REQUEST, "invalid_payment_intent");
    }
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
    match existing_operation(&mut tx, &key, "intent.force-cancel", &id).await {
        Ok(ExistingOperation::Replay(result)) => return response(&headers, StatusCode::OK, result),
        Ok(ExistingOperation::Absent) => {}
        Ok(ExistingOperation::Conflict) | Err(_) => {
            return error(&headers, StatusCode::CONFLICT, "idempotency_conflict")
        }
    }
    let intent = sqlx::query_as::<_, PayIntent>(
        "SELECT id,chain_id,payer,payee,amount,token_address,status,escrow_id,tx_hash,
                description,expires_at,created_at,updated_at
           FROM public.pay_intents WHERE id=$1 FOR UPDATE",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await;
    let intent = match intent {
        Ok(Some(intent)) => intent,
        Ok(None) => return error(&headers, StatusCode::NOT_FOUND, "payment_intent_not_found"),
        Err(_) => {
            return error(
                &headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_write_unavailable",
            )
        }
    };
    if !version_matches(request.expected_version, intent.updated_at) {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "stale_payment_intent_version",
        );
    }
    if intent.status != "pending" {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "payment_intent_not_cancellable",
        );
    }
    let version_after = sqlx::query_scalar::<_, DateTime<Utc>>(
        "UPDATE public.pay_intents SET status='cancelled',updated_at=NOW()
          WHERE id=$1 AND status='pending'
       RETURNING updated_at",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await;
    let version_after = match version_after {
        Ok(Some(value)) => value,
        _ => {
            return error(
                &headers,
                StatusCode::CONFLICT,
                "payment_intent_write_rejected",
            );
        }
    };
    if version_after <= intent.updated_at {
        return error(
            &headers,
            StatusCode::CONFLICT,
            "payment_intent_write_rejected",
        );
    }
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({
        "id": id,
        "status": "cancelled",
        "evidence": {
            "operation_id": operation_id,
            "version_before": intent.updated_at,
            "version_after": version_after,
            "ledger_entry": "finalized",
            "financial_finality": "not_applicable",
            "correlation_id": correlation(&headers)
        }
    });
    if sqlx::query(
        "INSERT INTO public.pay_admin_operations
            (operation_id,idempotency_key,action,resource_id,actor,version_before,version_after,
             resource_version_before,resource_version_after,result)
         VALUES($1,$2,'intent.force-cancel',$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(operation_id)
    .bind(&key)
    .bind(&id)
    .bind(&principal.subject)
    .bind(intent.updated_at)
    .bind(version_after)
    .bind(intent.updated_at.timestamp_micros())
    .bind(version_after.timestamp_micros())
    .bind(&result)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(&headers, StatusCode::CONFLICT, "idempotency_conflict");
    }
    if sqlx::query(
        "INSERT INTO public.pay_ledger_entries
            (operation_id,resource_id,entry_type,status,amount,token_address,chain_id,finalized_at)
         VALUES($1,$2,'intent.force-cancel','finalized',$3,$4,$5,$6)",
    )
    .bind(operation_id)
    .bind(&id)
    .bind(&intent.amount)
    .bind(&intent.token_address)
    .bind(&intent.chain_id)
    .bind(version_after)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(&headers, StatusCode::CONFLICT, "ledger_write_rejected");
    }
    if tx.commit().await.is_err() {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_write_unavailable",
        );
    }
    let observed =
        sqlx::query_scalar::<_, String>("SELECT status FROM public.pay_intents WHERE id=$1")
            .bind(&id)
            .fetch_optional(&state.db)
            .await;
    if !matches!(observed, Ok(Some(status)) if status == "cancelled") {
        return error(
            &headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_read_after_write_unavailable",
        );
    }
    response(&headers, StatusCode::OK, result)
}

// ============================================================================
// POST /api/v1/admin/pay/escrows/:id/force-{release,refund}
// ============================================================================

pub async fn admin_force_release_escrow(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(request): Json<AdminPaymentMutationRequest>,
) -> Response {
    mutate_admin_escrow(&state, &principal, &headers, &id, request, "released").await
}

pub async fn admin_force_refund_escrow(
    State(state): State<AppState>,
    Extension(principal): Extension<VerifiedPrincipal>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(request): Json<AdminPaymentMutationRequest>,
) -> Response {
    mutate_admin_escrow(&state, &principal, &headers, &id, request, "refunded").await
}

async fn mutate_admin_escrow(
    state: &AppState,
    principal: &VerifiedPrincipal,
    headers: &HeaderMap,
    id: &str,
    request: AdminPaymentMutationRequest,
    status: &str,
) -> Response {
    let Some(key) = idempotency_key(headers) else {
        return error(headers, StatusCode::BAD_REQUEST, "missing_idempotency_key");
    };
    if !valid_resource_id(id) || request.expected_version <= 0 {
        return error(headers, StatusCode::BAD_REQUEST, "invalid_escrow");
    }
    let action = if status == "released" {
        "escrow.force-release"
    } else {
        "escrow.force-refund"
    };
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_write_unavailable",
            )
        }
    };
    match existing_operation(&mut tx, &key, action, id).await {
        Ok(ExistingOperation::Replay(result)) => return response(headers, StatusCode::OK, result),
        Ok(ExistingOperation::Absent) => {}
        Ok(ExistingOperation::Conflict) | Err(_) => {
            return error(headers, StatusCode::CONFLICT, "idempotency_conflict")
        }
    }
    let escrow = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id,chain_id,payer,payee,amount,token_address,fee_amount,status,on_chain_id,
                tx_hash,dispute_reason,created_at,updated_at
           FROM public.escrows WHERE id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await;
    let escrow = match escrow {
        Ok(Some(escrow)) => escrow,
        Ok(None) => return error(headers, StatusCode::NOT_FOUND, "escrow_not_found"),
        Err(_) => {
            return error(
                headers,
                StatusCode::SERVICE_UNAVAILABLE,
                "payment_write_unavailable",
            )
        }
    };
    if !version_matches(request.expected_version, escrow.updated_at) {
        return error(headers, StatusCode::CONFLICT, "stale_escrow_version");
    }
    if !matches!(escrow.status.as_str(), "active" | "disputed") {
        return error(headers, StatusCode::CONFLICT, "escrow_not_mutable");
    }
    let version_after = sqlx::query_scalar::<_, DateTime<Utc>>(
        "UPDATE public.escrows SET status=$1,updated_at=NOW()
          WHERE id=$2 AND status IN ('active','disputed')
       RETURNING updated_at",
    )
    .bind(status)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .ok()
    .flatten();
    let Some(version_after) = version_after.filter(|value| *value > escrow.updated_at) else {
        return error(headers, StatusCode::CONFLICT, "escrow_write_rejected");
    };
    let operation_id = Uuid::new_v4();
    let result = serde_json::json!({
        "escrow_id": id,
        "status": status,
        "evidence": {
            "operation_id": operation_id,
            "version_before": escrow.updated_at,
            "version_after": version_after,
            "ledger_entry": "pending",
            "financial_finality": "pending_external_settlement",
            "correlation_id": correlation(headers)
        }
    });
    if sqlx::query(
        "INSERT INTO public.pay_admin_operations
            (operation_id,idempotency_key,action,resource_id,actor,version_before,version_after,
             resource_version_before,resource_version_after,result)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(operation_id)
    .bind(&key)
    .bind(action)
    .bind(id)
    .bind(&principal.subject)
    .bind(escrow.updated_at)
    .bind(version_after)
    .bind(escrow.updated_at.timestamp_micros())
    .bind(version_after.timestamp_micros())
    .bind(&result)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(headers, StatusCode::CONFLICT, "idempotency_conflict");
    }
    if sqlx::query(
        "INSERT INTO public.pay_ledger_entries
            (operation_id,resource_id,entry_type,status,amount,token_address,chain_id,finalized_at)
         VALUES($1,$2,$3,'pending',$4,$5,$6,NULL)",
    )
    .bind(operation_id)
    .bind(id)
    .bind(action)
    .bind(&escrow.amount)
    .bind(&escrow.token_address)
    .bind(&escrow.chain_id)
    .execute(&mut *tx)
    .await
    .is_err()
    {
        return error(headers, StatusCode::CONFLICT, "ledger_write_rejected");
    }
    if tx.commit().await.is_err() {
        return error(
            headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_write_unavailable",
        );
    }
    let observed = sqlx::query_as::<_, AdminEscrowVersion>(
        "SELECT status,updated_at FROM public.escrows WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;
    if !matches!(observed, Ok(Some(value)) if value.status == status && value.updated_at == version_after)
    {
        return error(
            headers,
            StatusCode::SERVICE_UNAVAILABLE,
            "payment_read_after_write_unavailable",
        );
    }
    response(headers, StatusCode::OK, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_query_and_mutation_identifiers_are_strictly_bounded() {
        assert!(canonical_payer("0x1111111111111111111111111111111111111111").is_some());
        for value in [
            "0x1111111111111111111111111111111111111111/../x",
            "0x1111111111111111111111111111111111111111%2f",
            "0x111111111111111111111111111111111111111",
        ] {
            assert!(canonical_payer(value).is_none());
        }
        assert!(valid_resource_id("0xintent-1"));
        assert!(!valid_resource_id("intent.id"));
        assert!(!valid_resource_id("intent/../x"));
    }

    #[test]
    fn mutation_versions_use_microsecond_timestamps_and_allow_initial_zero_only() {
        let value = Utc::now();
        assert!(!version_matches(0, value));
        assert!(version_matches(value.timestamp_micros(), value));
        assert!(!version_matches(value.timestamp_micros() - 1, value));
        assert!(!version_matches(-1, value));
    }
}
