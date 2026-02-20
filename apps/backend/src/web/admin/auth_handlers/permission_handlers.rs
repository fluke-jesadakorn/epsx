// Permission management handlers for Web3 admin

use axum::{extract::{Query, State}, http::StatusCode, response::Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{error, info};

use crate::application::shared::CommandHandler;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};
use crate::web::auth::AppState;

use super::types::*;

// Handler: Get all wallet permissions with filtering
pub async fn get_user_permissions(
  Query(params): Query<PermissionsQuery>,
  State(app_state): State<AppState>
) -> Result<Json<AdminApiResponse<PermissionsResponse>>, StatusCode> {
  info!("Admin: Fetching wallet permissions with filters: {:?}", params);

  // Get database connection
  let _db_pool = app_state.db_pool.as_ref();

  let limit = params.limit.unwrap_or(50);
  let offset = params.offset.unwrap_or(0);

  // Build dynamic query with parameterized bindings
  let has_wallet_filter = params.wallet_address.is_some();
  let wallet_pattern = params.wallet_address.as_ref()
    .map(|w| format!("%{}%", w))
    .unwrap_or_default();

  let where_clause = if has_wallet_filter {
    "WHERE wu.wallet_address ILIKE $3"
  } else {
    ""
  };

  let query_str = format!(
    r#"SELECT
        wu.wallet_address,
        wu.created_at,
        wu.last_auth_at,
        wu.is_active,
        COALESCE(COUNT(DISTINCT CASE
          WHEN p.is_active = true
            AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
            AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
          THEN p.id END), 0) as active_permissions_count
     FROM wallet_users wu
     LEFT JOIN wallet_plan_assignments wga ON wu.wallet_address = wga.wallet_address
     LEFT JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
     LEFT JOIN permissions p ON pgm.permission_id = p.id
     LEFT JOIN wallet_direct_permissions wdp ON wu.wallet_address = wdp.wallet_address
     {}
     GROUP BY wu.wallet_address, wu.created_at, wu.last_auth_at, wu.is_active
     ORDER BY wu.last_auth_at DESC NULLS LAST
     LIMIT $1 OFFSET $2"#,
    where_clause
  );

  #[derive(QueryableByName)]
  struct WalletPermRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    wallet_address: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    active_permissions_count: i64,
  }

  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("Admin: Failed to get database connection: {}", e);
      return Ok(Json(AdminApiResponse::server_error()));
    }
  };

  let sql = diesel::sql_query(&query_str)
    .bind::<diesel::sql_types::BigInt, _>(limit as i64)
    .bind::<diesel::sql_types::BigInt, _>(offset as i64);

  // Conditionally bind wallet filter
  let wallets = if has_wallet_filter {
    sql.bind::<diesel::sql_types::Text, _>(&wallet_pattern)
      .load::<WalletPermRow>(&mut conn)
      .await
  } else {
    sql.load::<WalletPermRow>(&mut conn).await
  };

  let wallets = match wallets {
    Ok(rows) => rows,
    Err(e) => {
      error!("Admin: Failed to fetch wallet permissions: {}", e);
      return Ok(Json(AdminApiResponse::server_error()));
    }
  };

  // For each wallet, get their first permission as primary
  let mut permissions: Vec<WalletPermissionResponse> = Vec::new();

  for (i, row) in wallets.into_iter().enumerate() {
    let wallet_address = &row.wallet_address;
    let active_perms_count = row.active_permissions_count;

    // Get first permission for this wallet as primary permission
    #[derive(QueryableByName)]
    struct PrimaryPermRow {
      #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
      permission_string: Option<String>,
      #[diesel(sql_type = diesel::sql_types::Text)]
      source: String,
    }

    let primary_perm_query = diesel::sql_query(
      r#"
      SELECT DISTINCT p.permission_string, 'plan' as source
      FROM wallet_plan_assignments wga
      JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
      JOIN permissions p ON pgm.permission_id = p.id
      WHERE wga.wallet_address = $1
        AND wga.is_active = true
        AND p.is_active = true
        AND (wga.expires_at IS NULL OR wga.expires_at > NOW())

      UNION

      SELECT DISTINCT p.permission_string, 'direct' as source
      FROM wallet_direct_permissions wdp
      JOIN permissions p ON wdp.permission_id = p.id
      WHERE wdp.wallet_address = $1
        AND wdp.is_active = true
        AND p.is_active = true
        AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

      ORDER BY permission_string
      LIMIT 1
      "#
    )
    .bind::<diesel::sql_types::Text, _>(wallet_address)
    .get_result::<PrimaryPermRow>(&mut conn)
    .await
    .optional();

    let (primary_permission, permission_source) = match primary_perm_query {
      Ok(Some(rec)) => (
        rec.permission_string,
        rec.source
      ),
      _ => (Some("epsx:basic:view".to_string()), "default".to_string())
    };

    permissions.push(WalletPermissionResponse {
      id: format!("perm_{}", i),
      wallet_address: wallet_address.clone(),
      permission: primary_permission.unwrap_or_else(|| "epsx:basic:view".to_string()),
      source: permission_source,
      expires_at: None,
      granted_at: row.created_at.to_rfc3339(),
      granted_by: "system".to_string(),
      is_active: row.is_active,
      metadata: Some(serde_json::json!({
        "permissions_count": active_perms_count,
        "last_auth_at": row.last_auth_at
      })),
    });
  }

  let response = PermissionsResponse {
    permissions,
    total_count: params.limit.unwrap_or(50),
  };

  let metadata = AdminMetadata::list_operation(
    "get_user_permissions",
    crate::web::pagination::PaginationInfo {
      page: (offset / limit + 1) as u32,
      limit: limit as u32,
      total_count: response.total_count as u64,
      total_pages: ((response.total_count as f64) / (limit as f64)).ceil() as u32,
      has_next: response.total_count > (offset + limit),
      has_prev: offset > 0,
    }
  );

  info!("Admin: Successfully fetched {} wallet permissions", response.permissions.len());
  Ok(Json(AdminApiResponse::success_with_meta(response, "Wallet permissions retrieved successfully", metadata)))
}

// Handler: Grant manual permission to wallet
pub async fn grant_manual_permission(
  State(app_state): State<AppState>,
  Json(request): Json<GrantPermissionRequest>
) -> Result<Json<AdminApiResponse<serde_json::Value>>, StatusCode> {
  info!(
    "Admin: Granting manual permissions to wallet: {}",
    request.wallet_address
  );

  // Normalize wallet address
  let wallet_address = request.wallet_address.to_lowercase();

  // Parse expiration date if provided
  let expires_at = if let Some(expires_str) = &request.expires_at {
    match chrono::DateTime::parse_from_rfc3339(expires_str) {
      Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
      Err(e) => {
        error!("Admin: Invalid expires_at format: {}", e);
        return Err(StatusCode::BAD_REQUEST);
      }
    }
  } else {
    None
  };

  // Get wallet repository from domain container
  let wallet_repository = match &app_state.domain_container.wallet_user_repository {
    Some(repo) => repo.clone(),
    None => {
      error!("Admin: Wallet repository not available");
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Get TransactionalOutbox from domain container (for CQRS event persistence)
  let outbox = match &app_state.domain_container.transactional_outbox {
    Some(outbox) => outbox.clone(),
    None => {
      error!("Admin: TransactionalOutbox not available");
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Create the grant permission handler (CQRS-enabled)
  let handler = crate::application::wallet_management::commands::handlers::GrantPermissionCommandHandler::new(
    wallet_repository,
    outbox,
  );

  // Process each permission
  let mut granted_permissions = Vec::new();
  let mut failed_permissions = Vec::new();

  for permission_str in &request.permissions {
    // Create the grant permission command
    let command = crate::application::wallet_management::commands::models::GrantPermissionCommand::new(
      wallet_address.clone(),
      permission_str.clone(),
    )
    .with_expiration(expires_at.unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(365)))
    .granted_by("admin".to_string())
    .with_reason(request.grant_reason.clone().unwrap_or_else(|| "Manual admin grant".to_string()));

    // Execute the command
    match handler.handle(command).await {
      Ok(response) => {
        info!("Admin: Successfully granted permission {} to wallet {}", permission_str, wallet_address);
        granted_permissions.push(serde_json::json!({
          "permission": permission_str,
          "granted_at": response.granted_at,
          "expires_at": response.expires_at
        }));
      }
      Err(e) => {
        error!("Admin: Failed to grant permission {} to wallet {}: {}", permission_str, wallet_address, e);
        failed_permissions.push(serde_json::json!({
          "permission": permission_str,
          "error": e.to_string()
        }));
      }
    }
  }

  let success = !granted_permissions.is_empty();
  let response_data = serde_json::json!({
    "wallet_address": wallet_address,
    "granted_permissions": granted_permissions,
    "failed_permissions": failed_permissions,
    "total_requested": request.permissions.len(),
    "total_granted": granted_permissions.len(),
    "total_failed": failed_permissions.len()
  });

  let metadata = AdminMetadata::crud_operation(
    "grant_manual_permission",
    Some("admin".to_string())
  );

  if success {
    Ok(Json(AdminApiResponse::success_with_meta(response_data, "Permissions granted successfully", metadata)))
  } else {
    Ok(Json(AdminApiResponse::error("Failed to grant permissions", "All permission grants failed")))
  }
}
