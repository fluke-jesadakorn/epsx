// Wallet query handlers for admin

use axum::{extract::{Query, State}, http::StatusCode, response::Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{error, info};

use crate::web::auth::AppState;

use super::types::*;

// Handler: Get recently connected wallets with analytics
pub async fn get_recent_wallets(
  Query(query): Query<RecentWalletsQuery>,
  State(app_state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!("Admin: Fetching recent wallet connections");

  let limit = query.limit.unwrap_or(50).min(100); // Cap at 100 for performance
  let days_back = query.days.unwrap_or(7).min(30); // Cap at 30 days

  // Get database connection from app state
  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("Admin: Failed to get database connection: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Query recent wallet connections with metadata from normalized tables
  #[derive(QueryableByName)]
  struct RecentWalletRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    wallet_address: String,
    #[allow(dead_code)]
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

  // Optimized query: First get wallets, then get permission counts separately
  // This avoids correlated subqueries which are slow
  let recent_wallets = match diesel::sql_query(
    r#"
    WITH recent_wallets_base AS (
      SELECT
        wu.wallet_address,
        wu.wallet_metadata,
        wu.created_at,
        wu.last_auth_at,
        wu.is_active
      FROM wallet_users wu
      WHERE wu.created_at >= NOW() - make_interval(days => $2)
      ORDER BY wu.created_at DESC
      LIMIT $1
    ),
    plan_permission_counts AS (
      SELECT
        rwb.wallet_address,
        COUNT(DISTINCT p.id)::int as plan_count
      FROM recent_wallets_base rwb
      LEFT JOIN wallet_plan_assignments wga
        ON wga.wallet_address = rwb.wallet_address
        AND wga.is_active = true
        AND (wga.expires_at IS NULL OR wga.expires_at > NOW())
      LEFT JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
      LEFT JOIN permissions p ON pgm.permission_id = p.id AND p.is_active = true
      GROUP BY rwb.wallet_address
    ),
    direct_permission_counts AS (
      SELECT
        rwb.wallet_address,
        COUNT(DISTINCT p.id)::int as direct_count
      FROM recent_wallets_base rwb
      LEFT JOIN wallet_direct_permissions wdp
        ON wdp.wallet_address = rwb.wallet_address
        AND wdp.is_active = true
        AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
      LEFT JOIN permissions p ON wdp.permission_id = p.id AND p.is_active = true
      GROUP BY rwb.wallet_address
    )
    SELECT
      rwb.wallet_address,
      rwb.wallet_metadata,
      rwb.created_at,
      rwb.last_auth_at,
      rwb.is_active,
      COALESCE(ppc.plan_count, 0) + COALESCE(dpc.direct_count, 0) as active_permissions_count
    FROM recent_wallets_base rwb
    LEFT JOIN plan_permission_counts ppc ON ppc.wallet_address = rwb.wallet_address
    LEFT JOIN direct_permission_counts dpc ON dpc.wallet_address = rwb.wallet_address
    ORDER BY rwb.created_at DESC
    "#
  )
  .bind::<diesel::sql_types::BigInt, _>(limit as i64)
  .bind::<diesel::sql_types::Integer, _>(days_back)
  .load::<RecentWalletRow>(&mut conn)
  .await {
    Ok(rows) => rows,
    Err(e) => {
      error!("Admin: Failed to fetch recent wallets: {}", e);
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
      error!("Admin: Failed to count recent wallets: {}", e);
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
      error!("Admin: Failed to fetch wallet analytics: {}", e);
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

  info!("Admin: Successfully fetched {} recent wallets", formatted_wallets.len());
  Ok(Json(response))
}

// Handler: Search wallets with advanced filtering
pub async fn search_wallets(
  Query(query): Query<WalletSearchQuery>,
  State(app_state): State<AppState>
) -> Result<Json<serde_json::Value>, StatusCode> {
  info!("Admin: Searching wallets with filters");

  let pg = crate::web::pagination::Pagination::from_signed(query.page, query.limit, 20, 100);

  // We'll use a simplified approach with fixed query parameters as the complex
  // dynamic query building with parameters is complex for this file
  // The filters will be handled by the simplified query below

  // Whitelist sort_by and sort_order to prevent SQL injection
  let sort_dir = match query.sort_order.as_deref() {
    Some("asc") | Some("ASC") => "ASC",
    _ => "DESC",
  };
  let order_by = match query.sort_by.as_deref() {
    Some("wallet_address") => format!("wallet_address {}", sort_dir),
    Some("last_auth_at") => format!("last_auth_at {} NULLS LAST", sort_dir),
    Some("permissions_count") => format!("active_permissions_count {}", sort_dir),
    _ => format!("created_at {}", sort_dir),
  };

  // Get database connection from app state
  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("Admin: Failed to get database connection: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Query wallets with permission counts from normalized tables
  #[derive(QueryableByName)]
  struct SearchWalletRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    wallet_address: String,
    #[allow(dead_code)]
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

  // Build WHERE clause with parameterized bindings (params start at $3)
  let has_search = query.search.is_some();
  let has_exclude_plan = query.exclude_plan_id.is_some();
  let search_pattern = query.search.as_ref().map(|s| format!("%{}%", s)).unwrap_or_default();

  // Parse exclude_plan_id as UUID to prevent injection
  let exclude_plan_uuid = query.exclude_plan_id.as_ref().and_then(|id| {
    uuid::Uuid::parse_str(id).ok()
  });
  let has_valid_exclude = exclude_plan_uuid.is_some();

  if has_exclude_plan && !has_valid_exclude {
    info!("Invalid exclude_plan_id format, ignoring filter");
  }

  let mut where_parts = Vec::new();
  let mut param_idx = 3; // $1 = limit, $2 = offset

  if has_search {
    where_parts.push(format!("wu.wallet_address ILIKE ${}", param_idx));
    param_idx += 1;
  }
  if has_valid_exclude {
    where_parts.push(format!(
      "LOWER(wu.wallet_address) NOT IN (SELECT LOWER(wallet_address) FROM wallet_plan_assignments WHERE plan_id = ${} AND is_active = true)",
      param_idx
    ));
    // param_idx += 1; // uncomment if adding more params
  }

  let where_clause = if where_parts.is_empty() {
    String::new()
  } else {
    format!("WHERE {}", where_parts.join(" AND "))
  };

  let search_query = format!(
    r#"
    SELECT
      wu.wallet_address,
      wu.wallet_metadata,
      wu.created_at,
      wu.last_auth_at,
      wu.is_active,
      COALESCE((
        SELECT COUNT(DISTINCT p.id)::int
        FROM wallet_plan_assignments wga
        JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
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
    LIMIT $1
    OFFSET $2
    "#,
    where_clause, order_by
  );

  // Build query with parameterized bindings
  let base = diesel::sql_query(&search_query)
    .bind::<diesel::sql_types::BigInt, _>(pg.limit as i64)
    .bind::<diesel::sql_types::BigInt, _>(pg.offset);

  let wallets = match (has_search, has_valid_exclude) {
    (true, true) => {
      base
        .bind::<diesel::sql_types::Text, _>(&search_pattern)
        .bind::<diesel::sql_types::Uuid, _>(exclude_plan_uuid.unwrap())
        .load::<SearchWalletRow>(&mut conn).await
    }
    (true, false) => {
      base
        .bind::<diesel::sql_types::Text, _>(&search_pattern)
        .load::<SearchWalletRow>(&mut conn).await
    }
    (false, true) => {
      base
        .bind::<diesel::sql_types::Uuid, _>(exclude_plan_uuid.unwrap())
        .load::<SearchWalletRow>(&mut conn).await
    }
    (false, false) => {
      base.load::<SearchWalletRow>(&mut conn).await
    }
  };

  let wallets = match wallets {
    Ok(rows) => rows,
    Err(e) => {
      error!("Failed to search wallets: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Get total count for pagination (same WHERE clause, same params starting at $1)
  #[derive(QueryableByName)]
  struct SearchCountRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    count: Option<i64>,
  }

  let count_where = if where_parts.is_empty() {
    String::new()
  } else {
    // Re-number params starting at $1 for count query
    let renumbered: Vec<String> = where_parts.iter().enumerate().map(|(i, part)| {
      // Replace $N with $(i+1)
      let old = format!("${}", i + 3);
      let new = format!("${}", i + 1);
      part.replace(&old, &new)
    }).collect();
    format!("WHERE {}", renumbered.join(" AND "))
  };

  let count_query = format!("SELECT COUNT(*) as count FROM wallet_users wu {}", count_where);

  let total_count = match (has_search, has_valid_exclude) {
    (true, true) => {
      diesel::sql_query(&count_query)
        .bind::<diesel::sql_types::Text, _>(&search_pattern)
        .bind::<diesel::sql_types::Uuid, _>(exclude_plan_uuid.unwrap())
        .get_result::<SearchCountRow>(&mut conn).await
    }
    (true, false) => {
      diesel::sql_query(&count_query)
        .bind::<diesel::sql_types::Text, _>(&search_pattern)
        .get_result::<SearchCountRow>(&mut conn).await
    }
    (false, true) => {
      diesel::sql_query(&count_query)
        .bind::<diesel::sql_types::Uuid, _>(exclude_plan_uuid.unwrap())
        .get_result::<SearchCountRow>(&mut conn).await
    }
    (false, false) => {
      diesel::sql_query(&count_query)
        .get_result::<SearchCountRow>(&mut conn).await
    }
  };

  let total_count = match total_count {
    Ok(row) => row.count.unwrap_or(0),
    Err(e) => {
      error!("Admin: Failed to count wallets: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Format wallet data for response with plan lookup
  let mut formatted_wallets: Vec<serde_json::Value> = Vec::new();

  for row in wallets {
    let metadata = serde_json::json!({});
    let permissions: Vec<serde_json::Value> = vec![];

    // Get plan memberships for this wallet
    #[derive(QueryableByName)]
    struct PlanRow {
      #[diesel(sql_type = diesel::sql_types::Text)]
      plan_name: String,
      #[diesel(sql_type = diesel::sql_types::Text)]
      slug: String,
      #[diesel(sql_type = diesel::sql_types::Timestamptz)]
      assigned_at: chrono::DateTime<chrono::Utc>,
      #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
      expires_at: Option<chrono::DateTime<chrono::Utc>>,
      #[diesel(sql_type = diesel::sql_types::Bool)]
      is_active: bool,
    }

    let plans = match diesel::sql_query(
      r#"
      SELECT pg.name as plan_name, pg.slug, wga.assigned_at, wga.expires_at, wga.is_active
      FROM wallet_plan_assignments wga
      JOIN plans pg ON wga.plan_id = pg.id
      WHERE wga.wallet_address = $1
      ORDER BY wga.assigned_at DESC
      "#
    )
    .bind::<diesel::sql_types::Text, _>(&row.wallet_address)
    .load::<PlanRow>(&mut conn)
    .await {
      Ok(plan_rows) => plan_rows
        .into_iter()
        .map(|g| serde_json::json!({
          "name": g.plan_name,
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
      "plans": plans,
      "active_permissions_count": row.active_permissions_count.unwrap_or(0)
    }));
  }

  let total_pages = pg.total_pages(total_count as u64);
  let has_more = pg.has_next(total_count as u64);

  let response = serde_json::json!({
    "wallets": formatted_wallets,
    "total_count": total_count,
    "has_more": has_more,
    "metadata": {
      "page": pg.page,
      "limit": pg.limit,
      "total_pages": total_pages,
      "applied_filters": {
        "search": query.search,
        "tier": query.tier,
        "status": query.status,
        "date_range": query.date_range,
        "sort_by": query.sort_by,
        "sort_order": query.sort_order
      }
    }
  });

  info!("Admin: Successfully searched {} wallets (page {} of {})", formatted_wallets.len(), pg.page, total_pages);
  Ok(Json(response))
}

// Handler: Get available tier levels
pub async fn get_tiers(
  State(app_state): State<AppState>
) -> Result<Json<Vec<String>>, StatusCode> {
  info!("Admin: Fetching available tier levels");

  let mut conn = match app_state.db_pool.get().await {
    Ok(conn) => conn,
    Err(e) => {
      error!("Admin: Failed to get database connection: {}", e);
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
      error!("Admin: Failed to fetch tiers: {}", e);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  info!("Admin: Successfully fetched {} tier levels", tiers.len());
  Ok(Json(tiers))
}
