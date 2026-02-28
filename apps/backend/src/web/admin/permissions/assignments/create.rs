use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;
use super::{CreateAssignmentRequest, AssignmentResponse};

/// Create a new wallet-plan assignment
/// POST /admin/permissions/assignments
pub async fn create_assignment(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<crate::web::middleware::bearer_middleware::OpenIDUserContext>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAssignmentRequest>,
) -> impl IntoResponse {
    // Validate wallet address format
    let wallet = req.wallet_address.to_lowercase();
    if !wallet.starts_with("0x") || wallet.len() != 42 {
        return AdminResponse::bad_request("Invalid wallet address format (must be 42 characters starting with 0x)").into_response();
    }

    // Parse plan ID
    let plan_uuid = match Uuid::parse_str(&req.plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    // Get database connection
    let mut conn = match app_state.db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return AdminResponse::server_error("Database connection failed").into_response();
        }
    };

    #[derive(QueryableByName)]
    struct AssignmentId {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    #[derive(QueryableByName)]
    struct PlanDetails {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_type: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        plan_metadata: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan_group: String,
    }

    // Run transaction
    let assignment_metadata = req.assignment_metadata.clone().unwrap_or(serde_json::json!({}));
    let wallet_clone = wallet.clone();
    let wallet_for_user_insert = wallet.clone();
    let assignment_source_clone = req.assignment_source.clone();
    let assignment_reason_clone = req.assignment_reason.clone();
    let payment_reference_clone = req.payment_reference.clone();
    let subscription_id_clone = req.subscription_id.clone();
    let req_expires_at = req.expires_at;
    let req_auto_renew = req.auto_renew.unwrap_or(false);

    let result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        Box::pin(async move {
            // CRITICAL: Ensure wallet_users entry exists before assignment (FK constraint)
            diesel::sql_query(
                r#"
                INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata)
                VALUES ($1, true, 'Bronze', '{}')
                ON CONFLICT (wallet_address) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_for_user_insert)
            .execute(conn)
            .await?;

            // Fetch plan details early to get expiry settings
            let plan = diesel::sql_query(
                "SELECT name, plan_type, plan_metadata, plan_group FROM plans WHERE id = $1"
            )
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .get_result::<PlanDetails>(conn)
            .await
            .optional()?;

            let plan_ref = plan.as_ref().ok_or(diesel::result::Error::NotFound)?;

            // Cross-group validation: reject if wallet has plans from a different group (excluding 'custom')
            if plan_ref.plan_group != "custom" {
                #[derive(QueryableByName)]
                struct ExistingGroup {
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    plan_group: String,
                }

                let existing_groups: Vec<ExistingGroup> = diesel::sql_query(
                    r#"
                    SELECT DISTINCT p.plan_group
                    FROM wallet_plan_assignments wpa
                    JOIN plans p ON wpa.plan_id = p.id
                    WHERE wpa.wallet_address = $1 AND wpa.is_active = true AND wpa.plan_id != $2 AND p.plan_group != 'custom'
                    "#
                )
                .bind::<diesel::sql_types::Text, _>(&wallet_clone)
                .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
                .load(conn)
                .await?;

                for eg in &existing_groups {
                    if eg.plan_group != plan_ref.plan_group {
                        return Err(diesel::result::Error::RollbackTransaction);
                    }
                }
            }

            // Calculate expiry
            let expires_at = match req_expires_at {
                Some(at) => Some(at),
                None => {
                    let days = plan_ref.plan_metadata.get("default_expiry_days")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(30);

                    if days == -1 {
                        None
                    } else {
                        Some(Utc::now() + chrono::Duration::try_days(days).unwrap_or_else(chrono::Duration::zero))
                    }
                }
            };

            // Deactivate existing subscription plan assignments for this wallet
            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE wallet_address = $1
                  AND is_active = true
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                  AND plan_id != $2
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_clone)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .execute(conn)
            .await?;

            // Insert or update assignment
            let assignment_id = diesel::sql_query(
                r#"
                INSERT INTO wallet_plan_assignments (
                    wallet_address, plan_id, assigned_at, expires_at, is_active,
                    assignment_source, assignment_reason, payment_reference, subscription_id,
                    auto_renew, next_billing_date, assignment_metadata
                )
                VALUES ($1, $2, NOW(), $3, true, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (wallet_address, plan_id) DO UPDATE
                SET is_active = true, expires_at = EXCLUDED.expires_at, updated_at = NOW()
                RETURNING id
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_clone)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
            .bind::<diesel::sql_types::Text, _>(&assignment_source_clone)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&assignment_reason_clone)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payment_reference_clone)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&subscription_id_clone)
            .bind::<diesel::sql_types::Bool, _>(req_auto_renew)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
            .bind::<diesel::sql_types::Jsonb, _>(&assignment_metadata)
            .get_result::<AssignmentId>(conn)
            .await?
            .id;

            Ok((assignment_id, plan, expires_at))
        })
    }).await;

    let (assignment_id, plan, final_expires_at) = match result {
        Ok((id, Some(g), exp)) => (id, g, exp),
        Ok((_, None, _)) => return AdminResponse::not_found("Permission plan").into_response(),
        Err(e) => {
            if matches!(e, diesel::result::Error::NotFound) {
                return AdminResponse::not_found("Permission plan").into_response();
            }
            if matches!(e, diesel::result::Error::RollbackTransaction) {
                return AdminResponse::bad_request("Cannot mix plan groups. Wallet already has plans from a different group.").into_response();
            }
            tracing::error!("Transaction failed: {}", e);
            return AdminResponse::server_error("Failed to create assignment").into_response();
        }
    };

    // Build response
    let response = AssignmentResponse {
        id: assignment_id.to_string(),
        wallet_address: wallet,
        plan_id: req.plan_id,
        plan_name: plan.name,
        plan_type: plan.plan_type,
        assigned_at: Utc::now(),
        expires_at: final_expires_at,
        is_active: true,
        assignment_source: req.assignment_source,
        assignment_reason: req.assignment_reason,
        assigned_by: None,
        payment_reference: req.payment_reference,
        subscription_id: req.subscription_id,
        auto_renew: req_auto_renew,
        next_billing_date: final_expires_at,
        assignment_metadata: req.assignment_metadata.unwrap_or(serde_json::json!({})),
    };

    let ctx = AuditCtx::from_wallet(&user_ctx.wallet_address, &headers);
    app_state.audit.log(ctx, AuditEntry::new("plan_assignment", "create", "plan")
        .id(&response.id)
        .after(serde_json::json!({
            "wallet": &response.wallet_address,
            "plan_id": &response.plan_id,
            "plan_name": &response.plan_name,
            "source": &response.assignment_source,
        })));

    // Notify user about plan assignment
    let notif_wallet = response.wallet_address.clone();
    let notif_plan = response.plan_name.clone();
    let notif_plan_id = response.plan_id.clone();
    let notif_state = app_state.clone();
    tokio::spawn(async move {
        use crate::infrastructure::services::NotificationService;
        use crate::web::notifications::{NotificationType, NotificationPriority};
        let _ = NotificationService::send(
            &notif_state,
            &notif_wallet,
            NotificationType::Permission,
            NotificationPriority::Normal,
            "Plan Updated",
            &format!("You have been assigned to the {} plan", notif_plan),
            Some(serde_json::json!({ "plan_id": notif_plan_id, "plan_name": notif_plan })),
            Some("/plans".to_string()),
        ).await;
    });

    AdminResponse::created(response, "Wallet assigned to permission plan successfully").into_response()
}
