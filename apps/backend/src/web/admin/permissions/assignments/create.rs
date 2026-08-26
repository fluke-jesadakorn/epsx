use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use uuid::Uuid;

use super::{AssignmentResponse, CreateAssignmentRequest};
use crate::infrastructure::cache::redis_cache::set_perm_invalidated;
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

/// Create a new wallet-plan assignment
/// POST /admin/permissions/assignments
pub async fn create_assignment(
    State(app_state): State<AppState>,
    axum::Extension(user_ctx): axum::Extension<
        crate::web::middleware::bearer_middleware::OpenIDUserContext,
    >,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateAssignmentRequest>,
) -> impl IntoResponse {
    // Validate wallet address format
    let wallet = req.wallet_address.to_lowercase();
    if !wallet.starts_with("0x") || wallet.len() != 42 {
        return AdminResponse::bad_request(
            "Invalid wallet address format (must be 42 characters starting with 0x)",
        )
        .into_response();
    }

    // Parse plan ID
    let plan_uuid = match Uuid::parse_str(&req.plan_id) {
        Ok(id) => id,
        Err(_) => return AdminResponse::bad_request("Invalid plan ID format").into_response(),
    };

    #[derive(sqlx::FromRow)]
    struct AssignmentId {
        id: Uuid,
    }

    #[derive(sqlx::FromRow)]
    struct PlanDetails {
        name: String,
        plan_type: String,
        plan_metadata: serde_json::Value,
        plan_group: String,
    }

    let assignment_metadata = req
        .assignment_metadata
        .clone()
        .unwrap_or(serde_json::json!({}));
    let assignment_source = req.assignment_source.clone();
    let assignment_reason = req.assignment_reason.clone();
    let payment_reference = req.payment_reference.clone();
    let subscription_id = req.subscription_id.clone();
    let req_auto_renew = req.auto_renew.unwrap_or(false);

    let mut tx = match app_state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to start transaction: {}", e);
            return AdminResponse::server_error("Database error").into_response();
        }
    };

    // CRITICAL: Ensure wallet_users entry exists before assignment (FK constraint)
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata)
        VALUES ($1, true, 'Bronze', '{}')
        ON CONFLICT (wallet_address) DO NOTHING
        "#,
    )
    .bind(&wallet)
    .execute(&mut *tx)
    .await
    {
        tracing::error!("Failed to insert wallet_user: {}", e);
        return AdminResponse::server_error("Database error").into_response();
    }

    // Fetch plan details early to get expiry settings
    let plan: Option<PlanDetails> = match sqlx::query_as::<_, PlanDetails>(
        "SELECT name, plan_type, plan_metadata, plan_group FROM plans WHERE id = $1",
    )
    .bind(plan_uuid)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to fetch plan: {}", e);
            return AdminResponse::server_error("Database error").into_response();
        }
    };

    let plan_ref = match plan {
        Some(ref p) => p,
        None => return AdminResponse::not_found("Permission plan").into_response(),
    };

    // Cross-group validation: reject if wallet has plans from a different group (excluding 'custom')
    if plan_ref.plan_group != "custom" {
        #[derive(sqlx::FromRow)]
        struct ExistingGroup {
            plan_group: String,
        }

        let existing_groups: Vec<ExistingGroup> = match sqlx::query_as::<_, ExistingGroup>(
            r#"
            SELECT DISTINCT p.plan_group
            FROM wallet_plan_assignments wpa
            JOIN plans p ON wpa.plan_id = p.id
            WHERE wpa.wallet_address = $1 AND wpa.is_active = true AND wpa.plan_id != $2 AND p.plan_group != 'custom'
            "#,
        )
        .bind(&wallet)
        .bind(plan_uuid)
        .fetch_all(&mut *tx)
        .await
        {
            Ok(groups) => groups,
            Err(e) => {
                tracing::error!("Failed to fetch existing groups: {}", e);
                return AdminResponse::server_error("Database error").into_response();
            }
        };

        for eg in &existing_groups {
            if eg.plan_group != plan_ref.plan_group {
                return AdminResponse::bad_request(
                    "Cannot mix plan groups. Wallet already has plans from a different group.",
                )
                .into_response();
            }
        }
    }

    // Calculate expiry
    let expires_at = match req.expires_at {
        Some(at) => Some(at),
        None => {
            let days = plan_ref
                .plan_metadata
                .get("default_expiry_days")
                .and_then(|v| v.as_i64())
                .unwrap_or(30);

            if days == -1 {
                None
            } else {
                Some(
                    Utc::now()
                        + chrono::Duration::try_days(days).unwrap_or_else(chrono::Duration::zero),
                )
            }
        }
    };

    // Deactivate existing subscription plan assignments for this wallet
    if let Err(e) = sqlx::query(
        r#"
        UPDATE wallet_plan_assignments
        SET is_active = false, updated_at = NOW()
        WHERE wallet_address = $1
          AND is_active = true
          AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
          AND plan_id != $2
        "#,
    )
    .bind(&wallet)
    .bind(plan_uuid)
    .execute(&mut *tx)
    .await
    {
        tracing::error!("Failed to deactivate existing assignments: {}", e);
        return AdminResponse::server_error("Database error").into_response();
    }

    // Insert or update assignment
    let assignment_id_res = sqlx::query_as::<_, AssignmentId>(
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
        "#,
    )
    .bind(&wallet)
    .bind(plan_uuid)
    .bind(expires_at)
    .bind(&assignment_source)
    .bind(&assignment_reason)
    .bind(&payment_reference)
    .bind(&subscription_id)
    .bind(req_auto_renew)
    .bind(expires_at)
    .bind(&assignment_metadata)
    .fetch_one(&mut *tx)
    .await;

    let assignment_id = match assignment_id_res {
        Ok(row) => row.id,
        Err(e) => {
            tracing::error!("Failed to insert assignment: {}", e);
            return AdminResponse::server_error("Failed to create assignment").into_response();
        }
    };

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit assignment transaction: {}", e);
        return AdminResponse::server_error("Failed to commit transaction").into_response();
    }

    let plan = plan.unwrap();
    let final_expires_at = expires_at;

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
    app_state.audit.log(
        ctx,
        AuditEntry::new("plan_assignment", "create", "plan")
            .id(&response.id)
            .after(serde_json::json!({
                "wallet": &response.wallet_address,
                "plan_id": &response.plan_id,
                "plan_name": &response.plan_name,
                "source": &response.assignment_source,
            })),
    );

    // Invalidate cached permissions so next request gets live DB permissions
    set_perm_invalidated(app_state.cache.as_ref(), &response.wallet_address);

    // Notify user about plan assignment
    let notif_wallet = response.wallet_address.clone();
    let notif_plan = response.plan_name.clone();
    let notif_plan_id = response.plan_id.clone();
    let notif_assignment_id = response.id.clone();
    let notif_state = app_state.clone();
    tokio::spawn(async move {
        // Wave 10 / R3: route through the NotificationPort.
        use epsx_contracts::notification_port::SendNotificationRequest;
        if let Some(port) = notif_state.notification_port.as_ref() {
            let _ = port
                .send_with_event_id_retry(
                    &format!("permission.assignment.created:{notif_assignment_id}"),
                    SendNotificationRequest {
                        recipient_wallet_address: notif_wallet.clone(),
                        notification_type: "permission".to_string(),
                        priority: "normal".to_string(),
                        title: "Plan Updated".to_string(),
                        message: format!("You have been assigned to the {} plan", notif_plan),
                        data: Some(serde_json::json!({
                            "plan_id": notif_plan_id,
                            "plan_name": notif_plan,
                        })),
                        action_url: Some("/plans".to_string()),
                        expires_at: None,
                    },
                )
                .await;
        } else {
            tracing::warn!(
                "notification_port not wired in AppState; plan-assigned \
                 notification for wallet={} dropped",
                notif_wallet
            );
        }
    });

    AdminResponse::created(response, "Wallet assigned to permission plan successfully")
        .into_response()
}
