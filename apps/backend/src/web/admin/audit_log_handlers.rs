use axum::{
    extract::{Query, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use crate::domain::shared_kernel::entities::audit::{AuditQuery, AuditAction, ResourceType};
use crate::infrastructure::repositories::audit_log_repository::DieselAuditLogRepository;
use crate::domain::audit::repository::AuditLogRepository;
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
    pub search: Option<String>, // Added search param for frontend compatibility
    pub category: Option<String>, // Frontend sends category
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
    let repo = DieselAuditLogRepository::new();
    
    // Parse Date Strings
    let from_date = params.from_date.as_ref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let to_date = params.to_date.as_ref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    // Simple mapping for Action
    // Frontend mock data uses snake_case e.g. "permission_granted"
    // Our enums are now snake_case.
    // However, query params might come in as "permission" (category) or specific action?
    // Frontend sends `category=permission`.
    // We should map category to resource type or set of actions.
    // For now, let's just map specific fields if provided.
    
    let mut action = None;
    if let Some(act) = &params.action {
         action = match act.as_str() {
            "create" => Some(AuditAction::Create),
            "read" => Some(AuditAction::Read),
            "update" => Some(AuditAction::Update),
            "delete" => Some(AuditAction::Delete),
            "login" => Some(AuditAction::Login),
            "logout" => Some(AuditAction::Logout),
            "permission_granted" => Some(AuditAction::PermissionGranted),
            "permission_revoked" => Some(AuditAction::PermissionRevoked),
            "payment_initiated" => Some(AuditAction::PaymentInitiated),
            "payment_completed" => Some(AuditAction::PaymentCompleted),
            "export" => Some(AuditAction::Export),
            _ => None,
        };
    }

    let mut resource_type = None;
    if let Some(rt) = &params.resource_type {
        resource_type = match rt.to_lowercase().as_str() {
            "user" => Some(ResourceType::User),
            "session" => Some(ResourceType::Session),
            "payment" => Some(ResourceType::Payment),
            "notification" => Some(ResourceType::Notification),
            "analytics" => Some(ResourceType::Analytics),
            "admin" => Some(ResourceType::Admin),
            _ => None,
        };
    }

    // Handle Category Filter from Frontend
    // 'all' | 'permission' | 'wallet' | 'plan' | 'system'
    // This is stricter than action/resource_type matching.
    // For MVP, we might just ignore category or map it to resource types?
    if let Some(cat) = &params.category {
        match cat.as_str() {
            "permission" => {
               // Filter by actions related to permissions?
               // Or resource type?
               // Since AuditQuery is strict, maybe filter by ResourceType::Admin or just leave it for now?
               // Ideally we need multiple actions filter or resource type list.
            },
            "wallet" => {
                resource_type = Some(ResourceType::User); 
            },
            "plan" => {
                // subscriptions map to payment?
                resource_type = Some(ResourceType::Payment);
            },
            _ => {}
        }
    }

    // Handle wallet address format
    let mut wallet_address_query = params.wallet_address.as_ref().map(|w| UserId::from_string_unchecked(w.clone()));
    
    // Fallback: Check if search param looks like a wallet address
    if wallet_address_query.is_none() {
        if let Some(search) = &params.search {
            if search.starts_with("0x") {
                wallet_address_query = Some(UserId::from_string_unchecked(search.clone()));
            }
        }
    }

    let query = AuditQuery {
        wallet_address: wallet_address_query,
        action,
        resource_type,
        result: None, 
        from_date,
        to_date,
        limit: params.page_size,
        offset: params.page.map(|p| (p - 1) * params.page_size.unwrap_or(50)),
    };

    match repo.find_all(query).await {
        Ok((logs, total)) => {
             Json(serde_json::json!({
                 "success": true,
                 "data": {
                     "entries": logs,
                     "total_pages": (total as f64 / params.page_size.unwrap_or(50) as f64).ceil() as u64,
                     "total": total,
                     "page": params.page.unwrap_or(1),
                     "page_size": params.page_size.unwrap_or(50)
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
