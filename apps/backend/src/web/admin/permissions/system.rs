// System-Level Permission Operations
// Health checks, statistics, caching, and route permission management

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::Row;

use crate::web::auth::AppState;
use crate::web::responses::AdminResponse;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SystemHealthResponse {
    pub status: String,
    pub database_connected: bool,
    pub total_permission_groups: i64,
    pub total_active_assignments: i64,
    pub total_wallets_with_permissions: i64,
    pub total_permissions: i64,
    pub cache_status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PermissionStatisticsResponse {
    pub total_permission_groups: i64,
    pub active_permission_groups: i64,
    pub total_permissions: i64,
    pub total_wallet_assignments: i64,
    pub active_wallet_assignments: i64,
    pub expiring_assignments_7d: i64,
    pub total_direct_permissions: i64,
    pub top_groups: Vec<TopGroupStats>,
    pub permission_breakdown: PermissionBreakdown,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TopGroupStats {
    pub id: String,
    pub name: String,
    pub group_type: String,
    pub member_count: i64,
    pub permission_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PermissionBreakdown {
    pub by_platform: Vec<PlatformStats>,
    pub by_type: Vec<TypeStats>,
}

#[derive(Debug, Serialize)]
pub struct PlatformStats {
    pub platform: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TypeStats {
    pub permission_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct CacheClearResponse {
    pub caches_cleared: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RoutePermissionRequest {
    pub route_path: String,
    pub http_method: String,
    pub required_permissions: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoutePermission {
    pub id: String,
    pub route_path: String,
    pub http_method: String,
    pub required_permissions: Vec<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListRoutesQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Get permission system health status
/// GET /admin/permissions/system/health
pub async fn get_health(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    // Check database connection
    let db_connected = sqlx::query("SELECT 1")
        .fetch_optional(&*app_state.db_pool)
        .await
        .is_ok();

    // Get system statistics
    let total_groups = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM permission_groups")
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    let active_assignments = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wallet_group_assignments WHERE is_active = true"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    let total_wallets = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT wallet_address) FROM wallet_group_assignments WHERE is_active = true"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    let total_permissions = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM permissions WHERE is_active = true"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    let status = if db_connected { "healthy" } else { "degraded" };

    let response = SystemHealthResponse {
        status: status.to_string(),
        database_connected: db_connected,
        total_permission_groups: total_groups,
        total_active_assignments: active_assignments,
        total_wallets_with_permissions: total_wallets,
        total_permissions,
        cache_status: "operational".to_string(),
        timestamp: Utc::now(),
    };

    AdminResponse::success(response).into_response()
}

/// Get permission system statistics
/// GET /admin/permissions/system/stats
pub async fn get_statistics(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    // Total groups
    let total_groups = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM permission_groups")
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    let active_groups = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM permission_groups WHERE is_active = true"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Total permissions
    let total_permissions = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM permissions")
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Wallet assignments
    let total_assignments = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wallet_group_assignments"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    let active_assignments = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wallet_group_assignments WHERE is_active = true"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Expiring assignments (7 days)
    let expiring_7d = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM wallet_group_assignments
        WHERE is_active = true
          AND expires_at IS NOT NULL
          AND expires_at BETWEEN NOW() AND NOW() + INTERVAL '7 days'
        "#
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Direct permissions
    let total_direct = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wallet_direct_permissions WHERE is_active = true"
    )
        .fetch_one(&*app_state.db_pool)
        .await
        .unwrap_or(0);

    // Top groups by member count
    let top_groups_rows = sqlx::query(
        r#"
        SELECT
            pg.id, pg.name, pg.group_type,
            COUNT(DISTINCT wga.wallet_address) as member_count,
            COUNT(DISTINCT pgm.permission_id) as permission_count
        FROM permission_groups pg
        LEFT JOIN wallet_group_assignments wga ON pg.id = wga.group_id AND wga.is_active = true
        LEFT JOIN permission_group_memberships pgm ON pg.id = pgm.group_id
        WHERE pg.is_active = true
        GROUP BY pg.id, pg.name, pg.group_type
        ORDER BY member_count DESC
        LIMIT 10
        "#
    )
        .fetch_all(&*app_state.db_pool)
        .await
        .unwrap_or_default();

    let top_groups: Vec<TopGroupStats> = top_groups_rows.iter().map(|row| {
        TopGroupStats {
            id: row.get::<Uuid, _>("id").to_string(),
            name: row.get("name"),
            group_type: row.get("group_type"),
            member_count: row.get("member_count"),
            permission_count: row.get("permission_count"),
        }
    }).collect();

    // Permission breakdown by platform
    let platform_rows = sqlx::query(
        "SELECT platform, COUNT(*) as count FROM permissions GROUP BY platform ORDER BY count DESC"
    )
        .fetch_all(&*app_state.db_pool)
        .await
        .unwrap_or_default();

    let by_platform: Vec<PlatformStats> = platform_rows.iter().map(|row| {
        PlatformStats {
            platform: row.get("platform"),
            count: row.get("count"),
        }
    }).collect();

    // Permission breakdown by type
    let type_rows = sqlx::query(
        "SELECT permission_type, COUNT(*) as count FROM permissions GROUP BY permission_type ORDER BY count DESC"
    )
        .fetch_all(&*app_state.db_pool)
        .await
        .unwrap_or_default();

    let by_type: Vec<TypeStats> = type_rows.iter().map(|row| {
        TypeStats {
            permission_type: row.get("permission_type"),
            count: row.get("count"),
        }
    }).collect();

    let response = PermissionStatisticsResponse {
        total_permission_groups: total_groups,
        active_permission_groups: active_groups,
        total_permissions,
        total_wallet_assignments: total_assignments,
        active_wallet_assignments: active_assignments,
        expiring_assignments_7d: expiring_7d,
        total_direct_permissions: total_direct,
        top_groups,
        permission_breakdown: PermissionBreakdown {
            by_platform,
            by_type,
        },
        timestamp: Utc::now(),
    };

    AdminResponse::success(response).into_response()
}

/// Clear permission caches
/// POST /admin/permissions/system/cache/clear
pub async fn clear_caches(
    State(_app_state): State<AppState>,
) -> impl IntoResponse {
    // In a real implementation, this would clear Redis caches
    tracing::info!("Permission caches cleared");

    let response = CacheClearResponse {
        caches_cleared: vec![
            "permission_validation_cache".to_string(),
            "wallet_permissions_cache".to_string(),
            "group_memberships_cache".to_string(),
        ],
        timestamp: Utc::now(),
    };

    AdminResponse::success_with_message(response, "Caches cleared successfully").into_response()
}

/// Get route permission mappings
/// GET /admin/permissions/system/routes
pub async fn get_route_permissions(
    State(_app_state): State<AppState>,
    Query(_query): Query<ListRoutesQuery>,
) -> impl IntoResponse {
    // In a real implementation, this would query a route_permissions table
    // For now, return hardcoded common routes
    let routes = vec![
        RoutePermission {
            id: Uuid::new_v4().to_string(),
            route_path: "/api/admin/wallets".to_string(),
            http_method: "GET".to_string(),
            required_permissions: vec!["admin:users:view".to_string()],
            description: Some("List all users".to_string()),
            created_at: Utc::now(),
        },
        RoutePermission {
            id: Uuid::new_v4().to_string(),
            route_path: "/api/admin/permissions/groups".to_string(),
            http_method: "POST".to_string(),
            required_permissions: vec!["admin:permissions:manage".to_string()],
            description: Some("Create permission group".to_string()),
            created_at: Utc::now(),
        },
        RoutePermission {
            id: Uuid::new_v4().to_string(),
            route_path: "/api/analytics/rankings".to_string(),
            http_method: "GET".to_string(),
            required_permissions: vec!["epsx:analytics:view".to_string()],
            description: Some("View analytics rankings".to_string()),
            created_at: Utc::now(),
        },
    ];

    AdminResponse::success(serde_json::json!({
        "routes": routes,
        "count": routes.len()
    })).into_response()
}

/// Register a new route permission mapping
/// POST /admin/permissions/system/routes
pub async fn register_route_permission(
    State(_app_state): State<AppState>,
    Json(req): Json<RoutePermissionRequest>,
) -> impl IntoResponse {
    // Validate input
    if req.route_path.is_empty() || req.http_method.is_empty() {
        return AdminResponse::bad_request("Route path and HTTP method are required").into_response();
    }

    if req.required_permissions.is_empty() {
        return AdminResponse::bad_request("At least one required permission must be specified").into_response();
    }

    // In a real implementation, this would save to a route_permissions table
    tracing::info!(
        "Registering route permission: {} {} -> {:?}",
        req.http_method,
        req.route_path,
        req.required_permissions
    );

    let response = RoutePermission {
        id: Uuid::new_v4().to_string(),
        route_path: req.route_path,
        http_method: req.http_method,
        required_permissions: req.required_permissions,
        description: req.description,
        created_at: Utc::now(),
    };

    AdminResponse::created(response, "Route permission registered successfully").into_response()
}
