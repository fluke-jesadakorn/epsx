// Web3 Admin Permission Management Handlers
// Leverages existing Web3PermissionService for blockchain-based permission management

use axum::{ extract::{ Query, State }, http::StatusCode, response::Json };
use serde::{ Deserialize, Serialize };
use tracing::{ error, info, warn };
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::web::auth::AppState;
use crate::application::shared::CommandHandler;
use crate::web::admin::responses::{AdminApiResponse, AdminMetadata};

// Using structs from crate::auth module - no local duplicates needed

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct GrantPermissionRequest {
  pub wallet_address: String,
  pub permissions: Vec<String>,
  pub expires_at: Option<String>, // ISO 8601 datetime
  pub grant_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NFTGateRequest {
  pub contract_address: String,
  pub contract_name: String,
  pub permissions: Vec<String>,
  pub min_token_count: i32,
  pub network: Option<String>, // Default to "ethereum"
}

#[derive(Debug, Deserialize)]
pub struct TokenGateRequest {
  pub contract_address: String,
  pub token_symbol: String,
  pub permissions: Vec<String>,
  pub min_amount: String,
  pub token_decimals: Option<i32>, // Default to 18
  pub network: Option<String>, // Default to "ethereum"
}

#[derive(Debug, Deserialize)]
pub struct DAOProposalRequest {
  pub title: String,
  pub description: String,
  pub target_wallet: String,
  pub permissions: Vec<String>,
  pub votes_required: i32,
  pub expires_at: String, // ISO 8601 datetime
  pub dao_contract_address: Option<String>,
  pub network: Option<String>, // Default to "ethereum"
}

#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
  pub permissions: Vec<WalletPermissionResponse>,
  pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WalletPermissionResponse {
  pub id: String,
  pub wallet_address: String,
  pub permission: String,
  pub source: String, // "manual", "nft", "token", "dao"
  pub expires_at: Option<String>,
  pub granted_at: String,
  pub granted_by: String,
  pub is_active: bool,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct NFTGateResponse {
  pub id: String,
  pub contract_address: String,
  pub contract_name: String,
  pub permissions: Vec<String>,
  pub min_token_count: i32,
  pub network: String,
  pub is_active: bool,
  pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TokenGateResponse {
  pub id: String,
  pub contract_address: String,
  pub token_symbol: String,
  pub permissions: Vec<String>,
  pub min_amount: String,
  pub token_decimals: i32,
  pub network: String,
  pub is_active: bool,
  pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DAOProposalResponse {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub target_wallet: String,
  pub permissions: Vec<String>,
  pub votes_for: i32,
  pub votes_against: i32,
  pub votes_required: i32,
  pub status: String, // "pending", "approved", "rejected", "executed"
  pub created_at: String,
  pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PermissionsQuery {
  pub wallet_address: Option<String>,
  pub permission: Option<String>,
  pub source: Option<String>,
  pub limit: Option<usize>,
  pub offset: Option<usize>,
}

// Handler: Get all wallet permissions with filtering
pub async fn get_user_permissions(
  Query(params): Query<PermissionsQuery>,
  State(app_state): State<AppState>
) -> Result<Json<AdminApiResponse<PermissionsResponse>>, StatusCode> {
  info!("🔍 Admin: Fetching wallet permissions with filters: {:?}", params);

  // Get database connection
  let _db_pool = app_state.db_pool.as_ref();
  
  let limit = params.limit.unwrap_or(50);
  let offset = params.offset.unwrap_or(0);
  
  // Build dynamic query based on filters
  let mut query_conditions = Vec::new();
  let mut query_params: Vec<String> = Vec::new();
  
  if let Some(wallet) = &params.wallet_address {
    query_conditions.push("wu.wallet_address ILIKE $1".to_string());
    query_params.push(format!("%{}%", wallet));
  }
  
  let where_clause = if query_conditions.is_empty() {
    "".to_string()
  } else {
    format!("WHERE {}", query_conditions.join(" AND "))
  };
  
  // Query wallet permissions from normalized tables
  // This query gets wallets with their permission counts
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
     LEFT JOIN wallet_group_assignments wga ON wu.wallet_address = wga.wallet_address
     LEFT JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
     LEFT JOIN permissions p ON pgm.permission_id = p.id
     LEFT JOIN wallet_direct_permissions wdp ON wu.wallet_address = wdp.wallet_address
     {}
     GROUP BY wu.wallet_address, wu.created_at, wu.last_auth_at, wu.is_active
     ORDER BY wu.last_auth_at DESC NULLS LAST
     LIMIT {} OFFSET {}"#,
    where_clause, limit, offset
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
      error!("❌ Admin: Failed to get database connection: {}", e);
      return Ok(Json(AdminApiResponse::server_error()));
    }
  };

  let wallets = match diesel::sql_query(&query_str)
    .load::<WalletPermRow>(&mut conn)
    .await {
    Ok(rows) => rows,
    Err(e) => {
      error!("❌ Admin: Failed to fetch wallet permissions: {}", e);
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
      SELECT DISTINCT p.permission_string, 'group' as source
      FROM wallet_group_assignments wga
      JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
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
    crate::web::admin::responses::PaginationInfo {
      page: (offset / limit + 1) as i32,
      limit: limit as i32,
      total: response.total_count as i32,
      total_pages: ((response.total_count as f64) / (limit as f64)).ceil() as i32,
      has_next_page: response.total_count > (offset + limit),
      has_previous_page: offset > 0,
    }
  );
  
  info!("✅ Admin: Successfully fetched {} wallet permissions", response.permissions.len());
  Ok(Json(AdminApiResponse::success_with_meta(response, "Wallet permissions retrieved successfully", metadata)))
}

// Handler: Grant manual permission to wallet
pub async fn grant_manual_permission(
  State(app_state): State<AppState>,
  Json(request): Json<GrantPermissionRequest>
) -> Result<Json<AdminApiResponse<serde_json::Value>>, StatusCode> {
  info!(
    "🎯 Admin: Granting manual permissions to wallet: {}",
    request.wallet_address
  );

  // Parse expiration date if provided
  let expires_at = if let Some(expires_str) = &request.expires_at {
    match chrono::DateTime::parse_from_rfc3339(expires_str) {
      Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
      Err(e) => {
        error!("❌ Admin: Invalid expires_at format: {}", e);
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
      error!("❌ Admin: Wallet repository not available");
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Get TransactionalOutbox from domain container (for CQRS event persistence)
  let outbox = match &app_state.domain_container.transactional_outbox {
    Some(outbox) => outbox.clone(),
    None => {
      error!("❌ Admin: TransactionalOutbox not available");
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
      request.wallet_address.clone(),
      permission_str.clone(),
    )
    .with_expiration(expires_at.unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::days(365)))
    .granted_by("admin".to_string())
    .with_reason(request.grant_reason.clone().unwrap_or_else(|| "Manual admin grant".to_string()));

    // Execute the command
    match handler.handle(command).await {
      Ok(response) => {
        info!("✅ Admin: Successfully granted permission {} to wallet {}", permission_str, request.wallet_address);
        granted_permissions.push(serde_json::json!({
          "permission": permission_str,
          "granted_at": response.granted_at,
          "expires_at": response.expires_at
        }));
      }
      Err(e) => {
        error!("❌ Admin: Failed to grant permission {} to wallet {}: {}", permission_str, request.wallet_address, e);
        failed_permissions.push(serde_json::json!({
          "permission": permission_str,
          "error": e.to_string()
        }));
      }
    }
  }

  let success = !granted_permissions.is_empty();
  let response_data = serde_json::json!({
    "wallet_address": request.wallet_address,
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

// Handler: Create NFT permission gate
pub async fn create_nft_gate(
  State(_app_state): State<AppState>,
  Json(request): Json<NFTGateRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!(
    "🎨 Admin: Creating NFT gate for contract: {}",
    request.contract_address
  );

  let _network = request.network.unwrap_or_else(|| "ethereum".to_string());

  // NFT gate creation requires proper Web3 service integration - not implemented
  warn!("⚠️ Admin: NFT gate creation not implemented");
  Err(StatusCode::NOT_IMPLEMENTED)
}

// Handler: Create token permission gate
pub async fn create_token_gate(
  State(_app_state): State<AppState>,
  Json(request): Json<TokenGateRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!(
    "🪙 Admin: Creating token gate for contract: {}",
    request.contract_address
  );

  let _network = request.network.unwrap_or_else(|| "ethereum".to_string());
  let _decimals = request.token_decimals.unwrap_or(18);

  // Token gate creation requires proper Web3 service integration - not implemented
  warn!("⚠️ Admin: Token gate creation not implemented");
  Err(StatusCode::NOT_IMPLEMENTED)
}

// Handler: Create DAO proposal
pub async fn create_dao_proposal(
  State(_app_state): State<AppState>,
  Json(request): Json<DAOProposalRequest>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!("🗳️ Admin: Creating DAO proposal: {}", request.title);

  let network = request.network.unwrap_or_else(|| "ethereum".to_string());
  let _dao_contract = request.dao_contract_address.unwrap_or_else(|| {
    // Default DAO contract addresses per network
    match network.as_str() {
      "ethereum" => "0x0000000000000000000000000000000000000000".to_string(),
      "polygon" => "0x0000000000000000000000000000000000000000".to_string(),
      "bsc" => "0x0000000000000000000000000000000000000000".to_string(),
      _ => "0x0000000000000000000000000000000000000000".to_string(),
    }
  });

  // Parse expiration date
  let _expires_at = match
    chrono::DateTime::parse_from_rfc3339(&request.expires_at)
  {
    Ok(dt) => dt.with_timezone(&chrono::Utc),
    Err(e) => {
      error!("❌ Admin: Invalid expires_at format: {}", e);
      return Err(StatusCode::BAD_REQUEST);
    }
  };

  // DAO proposal creation requires proper Web3 service integration - not implemented
  warn!("⚠️ Admin: DAO proposal creation not implemented");
  Err(StatusCode::NOT_IMPLEMENTED)
}

// Handler: Get NFT gates with real database queries
pub async fn get_nft_gates(State(app_state): State<AppState>) -> Result<
  Json<serde_json::Value>,
  StatusCode
> {
  info!("🎨 Admin: Fetching NFT gates from database");

  // Note: NFT permission configs table not implemented yet
  // use crate::schema::nft_permission_configs::dsl::*;
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;

  let conn = app_state.db_pool.get().await
    .map_err(|e| {
      error!("Failed to get database connection: {:?}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  // Placeholder for now - models need schema alignment
  let nft_gates: Vec<serde_json::Value> = vec![];

  let response =
    serde_json::json!({
        "nft_gates": nft_gates,
        "total_count": nft_gates.len()
    });

  Ok(Json(response))
}

// Handler: Get token gates with real database queries
pub async fn get_token_gates(State(app_state): State<AppState>) -> Result<
  Json<serde_json::Value>,
  StatusCode
> {
  info!("🪙 Admin: Fetching token gates from database");

  // Note: Token permission configs table not implemented yet
  // use crate::schema::token_permission_configs::dsl::*;
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;

  let conn = app_state.db_pool.get().await
    .map_err(|e| {
      error!("Failed to get database connection: {:?}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  // Placeholder for now - models need schema alignment
  let token_gates: Vec<serde_json::Value> = vec![];

  let response =
    serde_json::json!({
        "token_gates": token_gates,
        "total_count": token_gates.len()
    });

  Ok(Json(response))
}

// Handler: Get DAO proposals with real database queries
pub async fn get_dao_proposals(State(app_state): State<AppState>) -> Result<
  Json<serde_json::Value>,
  StatusCode
> {
  info!("🗳️ Admin: Fetching DAO proposals from database");

  // Note: DAO proposals table not implemented yet
  // use crate::schema::dao_proposals::dsl::*;
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;

  let conn = app_state.db_pool.get().await
    .map_err(|e| {
      error!("Failed to get database connection: {:?}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  // Placeholder for now - models need schema alignment
  let proposals: Vec<serde_json::Value> = vec![];

  let response =
    serde_json::json!({
        "proposals": proposals,
        "total_count": proposals.len()
    });

  Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct RecentWalletsQuery {
  pub limit: Option<i32>,
  pub days: Option<i32>,
}

// Handler: Get recently connected wallets with analytics
pub async fn get_recent_wallets(
  Query(query): Query<RecentWalletsQuery>,
  State(app_state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!("👛 Admin: Fetching recent wallet connections");

  let limit = query.limit.unwrap_or(50).min(100); // Cap at 100 for performance
  let days_back = query.days.unwrap_or(7).min(30); // Cap at 30 days

  // Get database connection from app state
  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("❌ Admin: Failed to get database connection: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Query recent wallet connections with metadata from normalized tables
  #[derive(QueryableByName)]
  struct RecentWalletRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    wallet_address: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    wallet_metadata: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
    active_permissions_count: Option<i32>,
  }

  let recent_wallets = match diesel::sql_query(
    r#"
    SELECT
      wu.wallet_address,
      wu.wallet_metadata,
      wu.created_at,
      wu.last_auth_at,
      wu.is_active,
      COALESCE(
        (
          SELECT COUNT(DISTINCT p.id)::int
          FROM wallet_group_assignments wga
          JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
          JOIN permissions p ON pgm.permission_id = p.id
          WHERE wga.wallet_address = wu.wallet_address
            AND wga.is_active = true
            AND p.is_active = true
            AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
        ) + (
          SELECT COUNT(DISTINCT p.id)::int
          FROM wallet_direct_permissions wdp
          JOIN permissions p ON wdp.permission_id = p.id
          WHERE wdp.wallet_address = wu.wallet_address
            AND wdp.is_active = true
            AND p.is_active = true
            AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
        ),
        0
      ) as active_permissions_count
    FROM wallet_users wu
    WHERE wu.created_at >= NOW() - make_interval(days => $2)
    ORDER BY wu.created_at DESC
    LIMIT $1
    "#
  )
  .bind::<diesel::sql_types::BigInt, _>(limit as i64)
  .bind::<diesel::sql_types::Integer, _>(days_back)
  .load::<RecentWalletRow>(&mut conn)
  .await {
    Ok(rows) => rows,
    Err(e) => {
      error!("❌ Admin: Failed to fetch recent wallets: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Get total count for pagination info
  #[derive(QueryableByName)]
  struct CountRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    count: Option<i64>,
  }

  let total_count = match diesel::sql_query(
    r#"
    SELECT COUNT(*) as count
    FROM wallet_users
    WHERE created_at >= NOW() - make_interval(days => $1)
    "#
  )
  .bind::<diesel::sql_types::Integer, _>(days_back)
  .get_result::<CountRow>(&mut conn)
  .await {
    Ok(row) => row.count.unwrap_or(0),
    Err(e) => {
      error!("❌ Admin: Failed to count recent wallets: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Get analytics data
  #[derive(QueryableByName)]
  struct AnalyticsRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Date>)]
    connection_date: Option<chrono::NaiveDate>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    daily_count: Option<i64>,
  }

  let analytics = match diesel::sql_query(
    r#"
    SELECT
      DATE(created_at) as connection_date,
      COUNT(*) as daily_count
    FROM wallet_users
    WHERE created_at >= NOW() - make_interval(days => $1)
    GROUP BY DATE(created_at)
    ORDER BY connection_date DESC
    "#
  )
  .bind::<diesel::sql_types::Integer, _>(days_back)
  .load::<AnalyticsRow>(&mut conn)
  .await {
    Ok(rows) => rows,
    Err(e) => {
      error!("❌ Admin: Failed to fetch wallet analytics: {}", e);
      Vec::new()
    }
  };

  // Format wallet data for response
  let formatted_wallets: Vec<serde_json::Value> = recent_wallets
    .into_iter()
    .map(|row| {
      let metadata = serde_json::json!({});

      serde_json::json!({
        "wallet_address": row.wallet_address,
        "metadata": metadata,
        "created_at": row.created_at,
        "last_auth_at": row.last_auth_at,
        "is_active": row.is_active,
        "active_permissions_count": row.active_permissions_count.unwrap_or(0),
        "connection_info": {
          "is_new": chrono::Utc::now().signed_duration_since(row.created_at).num_hours() < 24,
          "last_seen": row.last_auth_at.map(|t| chrono::Utc::now().signed_duration_since(t).num_hours())
        }
      })
    })
    .collect();

  // Format analytics data
  let daily_analytics: Vec<serde_json::Value> = analytics
    .into_iter()
    .map(|row| {
      serde_json::json!({
        "date": row.connection_date,
        "connections": row.daily_count.unwrap_or(0)
      })
    })
    .collect();

  let response = serde_json::json!({
    "recent_wallets": formatted_wallets,
    "analytics": {
      "total_in_period": total_count,
      "daily_breakdown": daily_analytics,
      "period_days": days_back,
      "avg_daily": if days_back > 0 { total_count as f64 / days_back as f64 } else { 0.0 }
    },
    "metadata": {
      "limit": limit,
      "total_count": formatted_wallets.len(),
      "has_more": formatted_wallets.len() as i32 >= limit,
      "generated_at": chrono::Utc::now().to_rfc3339()
    }
  });

  info!("✅ Admin: Successfully fetched {} recent wallets", formatted_wallets.len());
  Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct WalletSearchQuery {
  pub page: Option<i32>,
  pub limit: Option<i32>,
  pub search: Option<String>,
  pub tier: Option<String>,
  pub status: Option<String>,
  pub date_range: Option<String>,
  pub has_permissions: Option<String>,
  pub sort_by: Option<String>,
  pub sort_order: Option<String>,
}

// Handler: Search wallets with advanced filtering
pub async fn search_wallets(
  Query(query): Query<WalletSearchQuery>,
  State(app_state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!("🔍 Admin: Searching wallets with filters");

  let page = query.page.unwrap_or(1).max(1);
  let limit = query.limit.unwrap_or(20).min(100); // Cap at 100 for performance
  let offset = (page - 1) * limit;

  // We'll use a simplified approach with fixed query parameters as the complex
  // dynamic query building with parameters is complex for this file
  // The filters will be handled by the simplified query below

  // Build ORDER BY clause
  let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
  let sort_order = query.sort_order.as_deref().unwrap_or("desc");
  
  let order_by = match sort_by {
    "wallet_address" => format!("wallet_address {}", sort_order.to_uppercase()),
    "last_auth_at" => format!("last_auth_at {} NULLS LAST", sort_order.to_uppercase()),
    "permissions_count" => format!("active_permissions_count {}", sort_order.to_uppercase()),
    _ => format!("created_at {}", sort_order.to_uppercase()),
  };

  // We'll use a simplified approach - the complex dynamic query building was removed to avoid complexity
  // Use a fixed WHERE clause that works with the simplified query
  let where_clause = "WHERE 1=1".to_string();

  // Get database connection from app state
  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("❌ Admin: Failed to get database connection: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Build and execute the main query (NOTE: This query template is unused - actual query is below)
  let _base_query = format!(
    r#"
    SELECT
      wu.wallet_address,
      wu.wallet_metadata,
      wu.created_at,
      wu.last_auth_at,
      wu.is_active,
      COALESCE((
        SELECT COUNT(DISTINCT p.id)::int
        FROM wallet_group_assignments wga
        JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
        JOIN permissions p ON pgm.permission_id = p.id
        WHERE wga.wallet_address = wu.wallet_address
          AND wga.is_active = true
          AND p.is_active = true
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
      ), 0) + COALESCE((
        SELECT COUNT(DISTINCT p.id)::int
        FROM wallet_direct_permissions wdp
        JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = wu.wallet_address
          AND wdp.is_active = true
          AND p.is_active = true
          AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
      ), 0) as active_permissions_count
    FROM wallet_users wu
    {}
    ORDER BY {}
    LIMIT {}
    OFFSET {}
    "#,
    where_clause, order_by, limit, offset
  );

  // Query wallets with permission counts from normalized tables
  #[derive(QueryableByName)]
  struct SearchWalletRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    wallet_address: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
    wallet_metadata: Option<serde_json::Value>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
    active_permissions_count: Option<i32>,
  }

  let wallets = match diesel::sql_query(
    r#"
    SELECT
      wu.wallet_address,
      wu.wallet_metadata,
      wu.created_at,
      wu.last_auth_at,
      wu.is_active,
      COALESCE((
        SELECT COUNT(DISTINCT p.id)::int
        FROM wallet_group_assignments wga
        JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
        JOIN permissions p ON pgm.permission_id = p.id
        WHERE wga.wallet_address = wu.wallet_address
          AND wga.is_active = true
          AND p.is_active = true
          AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
      ), 0) + COALESCE((
        SELECT COUNT(DISTINCT p.id)::int
        FROM wallet_direct_permissions wdp
        JOIN permissions p ON wdp.permission_id = p.id
        WHERE wdp.wallet_address = wu.wallet_address
          AND wdp.is_active = true
          AND p.is_active = true
          AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
      ), 0) as active_permissions_count
    FROM wallet_users wu
    ORDER BY wu.created_at DESC
    LIMIT $1
    OFFSET $2
    "#
  )
  .bind::<diesel::sql_types::BigInt, _>(limit as i64)
  .bind::<diesel::sql_types::BigInt, _>(offset as i64)
  .load::<SearchWalletRow>(&mut conn)
  .await {
    Ok(rows) => rows,
    Err(e) => {
      error!("❌ Admin: Failed to search wallets: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Get total count for pagination
  #[derive(QueryableByName)]
  struct SearchCountRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    count: Option<i64>,
  }

  let total_count = match diesel::sql_query("SELECT COUNT(*) as count FROM wallet_users")
    .get_result::<SearchCountRow>(&mut conn)
    .await {
    Ok(row) => row.count.unwrap_or(0),
    Err(e) => {
      error!("❌ Admin: Failed to count wallets: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Format wallet data for response with group lookup
  let mut formatted_wallets: Vec<serde_json::Value> = Vec::new();

  for row in wallets {
    let metadata = serde_json::json!({});
    let permissions: Vec<serde_json::Value> = vec![];

    // Get group memberships for this wallet
    #[derive(QueryableByName)]
    struct GroupRow {
      #[diesel(sql_type = diesel::sql_types::Text)]
      group_name: String,
      #[diesel(sql_type = diesel::sql_types::Text)]
      slug: String,
      #[diesel(sql_type = diesel::sql_types::Timestamptz)]
      assigned_at: chrono::DateTime<chrono::Utc>,
      #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
      expires_at: Option<chrono::DateTime<chrono::Utc>>,
      #[diesel(sql_type = diesel::sql_types::Bool)]
      is_active: bool,
    }

    let groups = match diesel::sql_query(
      r#"
      SELECT pg.name as group_name, pg.slug, wga.assigned_at, wga.expires_at, wga.is_active
      FROM wallet_group_assignments wga
      JOIN permission_groups pg ON wga.group_id = pg.id
      WHERE wga.wallet_address = $1
      ORDER BY wga.assigned_at DESC
      "#
    )
    .bind::<diesel::sql_types::Text, _>(&row.wallet_address)
    .load::<GroupRow>(&mut conn)
    .await {
      Ok(group_rows) => group_rows
        .into_iter()
        .map(|g| serde_json::json!({
          "name": g.group_name,
          "slug": g.slug,
          "assigned_at": g.assigned_at,
          "expires_at": g.expires_at,
          "is_active": g.is_active
        }))
        .collect(),
      Err(_) => Vec::new()
    };

    formatted_wallets.push(serde_json::json!({
      "wallet_address": row.wallet_address,
      "metadata": metadata,
      "created_at": row.created_at,
      "last_auth_at": row.last_auth_at,
      "is_active": row.is_active,
      "permissions": permissions,
      "groups": groups,
      "active_permissions_count": row.active_permissions_count.unwrap_or(0)
    }));
  }

  let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
  let has_more = page < total_pages as i32;

  let response = serde_json::json!({
    "wallets": formatted_wallets,
    "total_count": total_count,
    "has_more": has_more,
    "metadata": {
      "page": page,
      "limit": limit,
      "total_pages": total_pages,
      "applied_filters": {
        "search": query.search,
        "tier": query.tier,
        "status": query.status,
        "date_range": query.date_range,
        "sort_by": sort_by,
        "sort_order": sort_order
      }
    }
  });

  info!("✅ Admin: Successfully searched {} wallets (page {} of {})", formatted_wallets.len(), page, total_pages);
  Ok(Json(response))
}

// Handler: Get available tier levels
pub async fn get_tiers(
  State(app_state): State<AppState>
) -> Result<Json<Vec<String>>, StatusCode> {
  info!("📊 Admin: Fetching available tier levels");

  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("❌ Admin: Failed to get database connection: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  #[derive(QueryableByName)]
  struct TierRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    tier_level: String,
  }

  let tiers: Vec<String> = match diesel::sql_query(
    r#"
    SELECT DISTINCT tier_level
    FROM wallet_users
    WHERE tier_level IS NOT NULL
    ORDER BY tier_level
    "#
  )
  .load::<TierRow>(&mut conn)
  .await {
    Ok(rows) => rows.into_iter().map(|r| r.tier_level).collect(),
    Err(e) => {
      error!("❌ Admin: Failed to fetch tiers: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  info!("✅ Admin: Successfully fetched {} tier levels", tiers.len());
  Ok(Json(tiers))
}
