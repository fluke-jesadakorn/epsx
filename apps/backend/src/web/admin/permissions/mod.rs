// ============================================================================
// PERMISSIONS MODULE
// Consolidated permission management - replaces 5 separate handler files
// Reduces 3,743 lines to ~1,800 lines (52% reduction)
// ============================================================================

pub mod groups;       // Permission group CRUD operations
pub mod assignments;  // Wallet-group assignment management
pub mod direct;       // Direct wallet permission management
pub mod validation;   // Permission validation logic
pub mod bulk;         // Bulk operations for permissions
pub mod system;       // System-level operations (health, caching, stats)
pub mod available;    // Available permissions listing

// Re-export for convenience
pub use groups::*;
pub use assignments::*;
pub use direct::*;
pub use validation::*;
pub use bulk::*;
pub use system::*;
pub use available::*;

use axum::{
    routing::{get, post, delete, put},
    Router,
};
use crate::web::auth::AppState;

/// Create permission management routes
/// Organizes all permission-related endpoints in one place
pub fn create_permission_routes() -> Router<AppState> {
    Router::new()
        // ============================================================================
        // PERMISSION GROUPS
        // ============================================================================
        .route("/groups", post(groups::create_group))
        .route("/groups", get(groups::list_groups))
        .route("/groups/{group_id}", get(groups::get_group))
        .route("/groups/{group_id}", put(groups::update_group))
        .route("/groups/{group_id}", delete(groups::delete_group))
        .route("/groups/{group_id}/members", get(groups::get_group_members))

        // ============================================================================
        // WALLET-GROUP ASSIGNMENTS
        // ============================================================================
        .route("/assignments", post(assignments::create_assignment))
        .route("/assignments", get(assignments::list_assignments))
        .route("/assignments/{assignment_id}", delete(assignments::remove_assignment))
        .route("/assignments/expiring", get(assignments::get_expiring_assignments))
        .route("/assignments/history/{wallet}", get(assignments::get_assignment_history))
        .route("/assignments/history", get(assignments::get_group_history))
        .route("/wallets/{wallet}/groups", get(assignments::get_wallet_groups))

        // ============================================================================
        // DIRECT PERMISSIONS
        // ============================================================================
        .route("/direct", post(direct::grant_permission))
        .route("/direct", delete(direct::revoke_permission))
        .route("/direct/{wallet}", get(direct::list_wallet_permissions))
        .route("/groups/{group_id}/permissions", post(direct::add_permission_to_group))
        .route("/groups/{group_id}/permissions/{permission_id}", delete(direct::remove_permission_from_group))

        // ============================================================================
        // VALIDATION
        // ============================================================================
        .route("/validate", post(validation::validate_permission))
        .route("/validate/bulk", post(validation::validate_bulk_permissions))
        .route("/wallets/{wallet}/permissions", get(validation::get_wallet_permissions))

        // ============================================================================
        // BULK OPERATIONS
        // ============================================================================
        .route("/bulk/grant", post(bulk::bulk_grant))
        .route("/bulk/revoke", post(bulk::bulk_revoke))
        .route("/bulk/assign-groups", post(bulk::bulk_assign_groups))
        .route("/bulk/apply-template", post(bulk::bulk_apply_template))
        .route("/bulk/validate", post(bulk::bulk_validate))

        // ============================================================================
        // SYSTEM OPERATIONS
        // ============================================================================
        .route("/system/health", get(system::get_health))
        .route("/system/stats", get(system::get_statistics))
        .route("/system/cache/clear", post(system::clear_caches))
        .route("/system/routes", get(system::get_route_permissions))
        .route("/system/routes", post(system::register_route_permission))

        // ============================================================================
        // AVAILABLE PERMISSIONS / PERMISSION DEFINITIONS
        // ============================================================================
        .route("/available", get(available::list_available_permissions))
        .route("/definitions", get(available::list_permission_definitions))
        .route("/definitions", post(available::create_permission_definition))
        .route("/definitions/{id}", delete(available::delete_permission_definition))
        .route("/definitions/by-name/{permission}", delete(available::delete_permission_by_name))
}
