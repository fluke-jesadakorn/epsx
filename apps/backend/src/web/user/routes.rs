// User management routes

use axum::{ routing::{ get, put }, Router };
use crate::web::auth::routes::AppState;
use super::handlers::{ get_profile_handler, update_profile_handler };
// Legacy permissions module removed for Web3-first migration
// use super::permissions::{ get_user_permissions, check_user_permission };

/// Create v1 API routes for wallet operations with Web3-first RESTful patterns
pub fn wallet_routes_v1() -> Router<AppState> {
  // Web3 wallet routes (available to authenticated wallets)
  let wallet_routes = Router::new()
    .route("/api/v1/wallet/profile", get(get_profile_handler))
    .route("/api/v1/wallet/profile", put(update_profile_handler))
    // Legacy email-based routes removed
    // .route("/api/v1/wallet/permissions", get(get_wallet_permissions)) // Use tier-based permissions
    ;

  // Premium wallet features (Silver tier and above)
  let premium_routes = Router::new()
    .route(
      "/api/v1/health",
      get(|| async { "OK" })
    );

  // Admin features (require admin wallet permissions)
  let admin_routes = Router::new()
    // .route("/api/v1/admin/users", get(list_users_handler)) // Handler missing
    // .route("/api/v1/admin/users/:id", delete(delete_user_handler)) // Handler missing
    .route(
      "/api/v1/admin/health",
      get(|| async { "OK" })
    );

  // Legacy routes for backward compatibility
  let legacy_routes = Router::new()
    .route("/users/profile", get(get_profile_handler))
    .route("/users/profile", put(update_profile_handler));
  // .route("/users/expiration-status", get(get_expiration_status_handler)) // Handler missing
  // .route("/users/notifications", get(get_notifications_handler)) // Handler missing
  // .route("/users/notifications/mark-read", post(mark_notifications_read_handler)) // Handler missing
  // .route("/users/request-expiration-check", post(request_expiration_check_handler)) // Handler missing
  // .route("/users", get(list_users_handler)) // Handler missing
  // .route("/users/:id", delete(delete_user_handler)) // Handler missing

  Router::new()
    .merge(wallet_routes)
    .merge(premium_routes)
    .merge(admin_routes)
    .merge(legacy_routes)
}

/// Create legacy user routes (backward compatibility) - DEPRECATED
/// Use wallet_routes_v1() for Web3-first operations
pub fn user_routes() -> Router<AppState> {
  Router::new()
    // Legacy wallet profile operations (mapped to /wallet/profile internally)
    .route("/me", get(get_profile_handler))
    .route("/me", put(update_profile_handler))

  // Admin operations - handlers missing
  // .route("/users", get(list_users_handler)) // Handler missing
  // .route("/users/:id", delete(delete_user_handler)) // Handler missing

  // Auth operations - handler missing
  // .route("/logout", post(logout_handler)) // Handler missing

  // User-driven expiration management - handlers missing
  // .route("/me/expiration-status", get(get_expiration_status_handler)) // Handler missing
  // .route("/me/request-expiration-check", post(request_expiration_check_handler)) // Handler missing

  // User notifications (deprecated - use /api/v1/notifications instead) - handlers missing
  // .route("/me/notifications", get(get_notifications_handler)) // Handler missing
  // .route("/me/notifications/mark-read", post(mark_notifications_read_handler)) // Handler missing
}
