use axum::{
    extract::{Query, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::domain::shared_kernel::entities::audit::AuditQuery;
use crate::infrastructure::repositories::audit_log_repository::DieselAuditLogRepository;
use serde::Deserialize;
use utoipa::{ToSchema, IntoParams};
use crate::web::auth::AppState;
use crate::domain::shared_kernel::value_objects::UserId;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct AuditLogQueryParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub wallet_address: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub search: Option<String>,
    pub category: Option<String>,
}

/// Get audit logs with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/admin/audit-logs",
    params(AuditLogQueryParams),
    responses(
        (status = 200, description = "Audit logs retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_audit_logs_handler(
    State(_state): State<AppState>,
    Query(params): Query<AuditLogQueryParams>,
) -> impl IntoResponse {
    let from_date = params.from_date.as_ref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let to_date = params.to_date.as_ref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    // Resolve wallet search from explicit param or search field
    let mut wallet_address = params.wallet_address.as_ref().map(|w| UserId::from_string_unchecked(w.clone()));
    let mut search = params.search.clone();

    if wallet_address.is_none() {
        if let Some(ref s) = search {
            if s.starts_with("0x") {
                wallet_address = Some(UserId::from_string_unchecked(s.clone()));
                search = None; // Don't double-filter
            }
        }
    }

    // Normalize category
    let category = params.category.as_deref().and_then(|c| match c {
        "all" => None,
        c @ ("permission" | "wallet" | "plan" | "system" | "payment" | "auth" | "developer" | "notification") => Some(c.to_string()),
        _ => None,
    });

    let page_size = params.page_size.unwrap_or(50);
    let query = AuditQuery {
        wallet_address,
        action: None,
        resource_type: None,
        result: None,
        from_date,
        to_date,
        limit: Some(page_size),
        offset: params.page.map(|p| (p.saturating_sub(1)) * page_size),
        category,
        search,
    };

    match DieselAuditLogRepository::find_all_unified(&query).await {
        Ok((logs, total)) => {
             Json(serde_json::json!({
                 "success": true,
                 "data": {
                     "entries": logs,
                     "total_pages": (total as f64 / page_size as f64).ceil() as u64,
                     "total": total,
                     "page": params.page.unwrap_or(1),
                     "page_size": page_size
                 }
             })).into_response()
        },
        Err(e) => {
            tracing::error!("Failed to fetch audit logs: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": "Failed to fetch audit logs"
            }))).into_response()
        }
    }
}
