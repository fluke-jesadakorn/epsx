// Wallet query handlers for admin
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with sqlx::query_as.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use tracing::{error, info};

use crate::web::auth::AppState;

use super::types::*;

/// Handler: Get recently connected wallets with analytics
pub async fn get_recent_wallets(
    Query(query): Query<RecentWalletsQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: Fetching recent wallet connections");

    let limit = query.limit.unwrap_or(50).min(100);
    let days_back = query.days.unwrap_or(7).min(30);

    #[derive(sqlx::FromRow)]
    struct RecentWalletRow {
        wallet_address: String,
        #[allow(dead_code)]
        wallet_metadata: Option<serde_json::Value>,
        created_at: chrono::DateTime<chrono::Utc>,
        last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
        is_active: bool,
        active_permissions_count: Option<i32>,
    }

    let recent_wallets: Result<Vec<RecentWalletRow>, _> = sqlx::query_as(
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
        "#,
    )
    .bind(limit as i64)
    .bind(days_back)
    .fetch_all(app_state.db_pool.as_ref())
    .await;

    let recent_wallets = match recent_wallets {
        Ok(rows) => rows,
        Err(e) => {
            error!("Admin: Failed to fetch recent wallets: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get total count for pagination info
    #[derive(sqlx::FromRow)]
    struct CountRow {
        count: Option<i64>,
    }

    let total_count: Option<i64> = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count
        FROM wallet_users
        WHERE created_at >= NOW() - make_interval(days => $1)
        "#,
    )
    .bind(days_back)
    .fetch_one(app_state.db_pool.as_ref())
    .await
    .ok()
    .and_then(|r: Option<CountRow>| r)
    .and_then(|r| r.count)
    .unwrap_or(Some(0))
    .unwrap_or(0);

    // Get analytics data
    #[derive(sqlx::FromRow)]
    struct AnalyticsRow {
        connection_date: Option<chrono::NaiveDate>,
        daily_count: Option<i64>,
    }

    let analytics: Vec<AnalyticsRow> = sqlx::query_as(
        r#"
        SELECT
          DATE(created_at) as connection_date,
          COUNT(*) as daily_count
        FROM wallet_users
        WHERE created_at >= NOW() - make_interval(days => $1)
        GROUP BY DATE(created_at)
        ORDER BY connection_date DESC
        "#,
    )
    .bind(days_back)
    .fetch_all(app_state.db_pool.as_ref())
    .await
    .unwrap_or_default();

    // Format wallet data for response
    let formatted_wallets: Vec<serde_json::Value> = recent_wallets
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "wallet_address": row.wallet_address,
                "metadata": serde_json::json!({}),
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

    info!(
        "Admin: Successfully fetched {} recent wallets",
        formatted_wallets.len()
    );
    Ok(Json(response))
}

/// Handler: Search wallets with advanced filtering
pub async fn search_wallets(
    Query(query): Query<WalletSearchQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Admin: Searching wallets with filters");

    let pg = crate::web::pagination::Pagination::from_signed(query.page, query.limit, 20, 100);

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

    #[derive(sqlx::FromRow)]
    struct SearchWalletRow {
        wallet_address: String,
        #[allow(dead_code)]
        wallet_metadata: Option<serde_json::Value>,
        created_at: chrono::DateTime<chrono::Utc>,
        last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
        is_active: bool,
        active_permissions_count: Option<i32>,
    }

    let has_search = query.search.is_some();
    let has_exclude_plan = query.exclude_plan_id.is_some();
    let search_pattern = query
        .search
        .as_ref()
        .map(|s| format!("%{}%", s))
        .unwrap_or_default();

    let exclude_plan_uuid = query
        .exclude_plan_id
        .as_ref()
        .and_then(|id| uuid::Uuid::parse_str(id).ok());
    let has_valid_exclude = exclude_plan_uuid.is_some();

    if has_exclude_plan && !has_valid_exclude {
        info!("Invalid exclude_plan_id format, ignoring filter");
    }

    let mut where_parts = Vec::new();
    let mut param_idx = 3;

    if has_search {
        where_parts.push(format!("wu.wallet_address ILIKE ${}", param_idx));
        param_idx += 1;
    }
    if has_valid_exclude {
        where_parts.push(format!(
            "LOWER(wu.wallet_address) NOT IN (SELECT LOWER(wallet_address) FROM wallet_plan_assignments WHERE plan_id = ${} AND is_active = true)",
            param_idx
        ));
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

    let mut q = sqlx::query_as::<_, SearchWalletRow>(&search_query);
    q = q.bind(pg.limit as i64).bind(pg.offset);
    if has_search {
        q = q.bind(&search_pattern);
    }
    if has_valid_exclude {
        q = q.bind(exclude_plan_uuid.unwrap());
    }
    let wallets = match q.fetch_all(app_state.db_pool.as_ref()).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to search wallets: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Total count
    let count_query = format!("SELECT COUNT(*) as count FROM wallet_users wu {}", where_clause);
    let mut cq = sqlx::query_as::<_, CountRow>(&count_query);
    if has_search {
        cq = cq.bind(&search_pattern);
    }
    if has_valid_exclude {
        cq = cq.bind(exclude_plan_uuid.unwrap());
    }
    let total_count = match cq.fetch_one(app_state.db_pool.as_ref()).await {
        Ok(row) => row.count.unwrap_or(0),
        Err(e) => {
            error!("Admin: Failed to count wallets: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get plan memberships for each wallet
    #[derive(sqlx::FromRow)]
    struct PlanRow {
        plan_name: String,
        slug: String,
        assigned_at: chrono::DateTime<chrono::Utc>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        is_active: bool,
    }

    let mut formatted_wallets: Vec<serde_json::Value> = Vec::new();

    for row in wallets {
        let metadata = serde_json::json!({});
        let permissions: Vec<serde_json::Value> = vec![];

        let plans_result: Result<Vec<PlanRow>, _> = sqlx::query_as(
            r#"
            SELECT pg.name as plan_name, pg.slug, wga.assigned_at, wga.expires_at, wga.is_active
            FROM wallet_plan_assignments wga
            JOIN plans pg ON wga.plan_id = pg.id
            WHERE wga.wallet_address = $1
            ORDER BY wga.assigned_at DESC
            "#,
        )
        .bind(&row.wallet_address)
        .fetch_all(app_state.db_pool.as_ref())
        .await;

        let plans: Vec<serde_json::Value> = match plans_result {
            Ok(plan_rows) => plan_rows
                .into_iter()
                .map(|g| {
                    serde_json::json!({
                        "name": g.plan_name,
                        "slug": g.slug,
                        "assigned_at": g.assigned_at,
                        "expires_at": g.expires_at,
                        "is_active": g.is_active
                    })
                })
                .collect(),
            Err(_) => Vec::new(),
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

    info!(
        "Admin: Successfully searched {} wallets (page {} of {})",
        formatted_wallets.len(),
        pg.page,
        total_pages
    );
    Ok(Json(response))
}

/// Handler: Get available tier levels
pub async fn get_tiers(State(app_state): State<AppState>) -> Result<Json<Vec<String>>, StatusCode> {
    info!("Admin: Fetching available tier levels");

    #[derive(sqlx::FromRow)]
    struct TierRow {
        tier_level: String,
    }

    let result: Result<Vec<TierRow>, _> = sqlx::query_as(
        r#"
        SELECT DISTINCT tier_level
        FROM wallet_users
        WHERE tier_level IS NOT NULL
        ORDER BY tier_level
        "#,
    )
    .fetch_all(app_state.db_pool.as_ref())
    .await;

    let tiers = match result {
        Ok(rows) => rows.into_iter().map(|r| r.tier_level).collect(),
        Err(e) => {
            error!("Admin: Failed to fetch tiers: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    info!("Admin: Successfully fetched {} tier levels", tiers.len());
    Ok(Json(tiers))
}
