use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

/// Remove a wallet-plan assignment
/// DELETE /admin/permissions/assignments/:assignment_id
pub async fn remove_assignment(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Path(assignment_id): Path<String>,
) -> impl IntoResponse {
    let assignment_uuid = match Uuid::parse_str(&assignment_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid assignment ID format").into_response(),
    };

    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    // Fetch assignment details before deactivation (for notification)
    #[derive(QueryableByName)]
    struct AssignmentInfo {
        #[diesel(sql_type = diesel::sql_types::Text)]
        wallet_address: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_name: String,
    }

    let info = diesel::sql_query(
        "SELECT wpa.wallet_address, p.name as plan_name FROM wallet_plan_assignments wpa JOIN plans p ON wpa.plan_id = p.id WHERE wpa.id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(assignment_uuid)
    .get_result::<AssignmentInfo>(&mut conn)
    .await
    .ok();

    match diesel::sql_query(
        "UPDATE wallet_plan_assignments SET is_active = false, updated_at = NOW() WHERE id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(assignment_uuid)
    .execute(&mut conn)
    .await
    {
        Ok(rows) if rows > 0 => {
            let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
            app_state.audit.log(ctx, AuditEntry::new("plan_assignment", "delete", "plan")
                .id(&assignment_id));

            // Notify user about plan removal
            if let Some(info) = info {
                let notif_state = app_state.clone();
                tokio::spawn(async move {
                    use crate::infrastructure::services::NotificationService;
                    use crate::web::notifications::{NotificationType, NotificationPriority};
                    let _ = NotificationService::send(
                        &notif_state,
                        &info.wallet_address,
                        NotificationType::Permission,
                        NotificationPriority::Normal,
                        "Plan Removed",
                        &format!("Your {} plan has been removed", info.plan_name),
                        Some(serde_json::json!({ "plan_name": info.plan_name })),
                        Some("/plans".to_string()),
                    ).await;
                });
            }

            AdminResponse::success_with_message(
                serde_json::json!({"deleted": true}),
                "Assignment removed successfully"
            ).into_response()
        },
        Ok(_) => AdminResponse::not_found("Assignment").into_response(),
        Err(e) => {
            tracing::error!("Failed to remove assignment: {}", e);
            AdminResponse::server_error("Failed to remove assignment").into_response()
        }
    }
}
