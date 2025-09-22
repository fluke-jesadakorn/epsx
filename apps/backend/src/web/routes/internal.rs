// Internal routes for web application users
// Context: OIDC authenticated users, session-based, feature usage tracking

use axum::{
  routing::{ get, put },
  Router,
  middleware as axum_middleware,
  Extension,
};
use std::sync::Arc;
use crate::{
  infrastructure::AppContainer,
  web::middleware::{ contextual_middleware::internal_middleware_stack },
};

pub struct InternalRoutes;

impl InternalRoutes {
  /// Create internal web app routes with OIDC authentication and user-centric middleware
  pub async fn create_routes(
    container: Arc<AppContainer>
  ) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    // Create internal-specific services
    let app_state = container.create_app_state().await?;

    // Internal user routes - web app functionality
    let user_routes = Router::new()
      .route("/profile", get(crate::web::user::handlers::get_profile_handler))
      .route(
        "/profile",
        put(crate::web::user::handlers::update_profile_handler)
      );
    // .route("/me", get(crate::web::user::handlers::me_handler)) // Handler missing

    // Internal analytics routes - feature usage (not billable)
    let analytics_routes = Router::new()
      .route(
        "/rankings",
        get(
          crate::web::analytics::eps_handlers::get_unified_analytics_rankings_cached
        )
      )
      .route(
        "/countries",
        get(crate::web::analytics::eps_handlers::get_available_countries)
      )
      .route(
        "/sectors",
        get(crate::web::analytics::eps_handlers::get_sectors_by_country)
      );

    // Internal settings routes (simplified)
    let settings_routes = Router::new()
      .route(
        "/config",
        get(crate::web::settings::handlers::get_system_config_handler)
      )
      .route(
        "/features",
        get(crate::web::settings::handlers::get_feature_flags_handler)
      );

    // Internal notification routes (simplified)
    let notification_routes = Router::new()
      .route(
        "/",
        get(crate::web::notifications::handlers::get_user_notifications)
      )
      .route(
        "/unread",
        get(crate::web::notifications::handlers::get_unread_notifications)
      )
      .route(
        "/preferences",
        get(crate::web::notifications::handlers::get_preferences)
      )
      .route(
        "/preferences",
        put(crate::web::notifications::handlers::update_preferences)
      );

    // Combine all internal routes
    let internal_router = Router::new()
      .nest("/user", user_routes)
      .nest("/analytics", analytics_routes)
      .nest("/settings", settings_routes)
      .nest("/notifications", notification_routes)
      // Health endpoint removed - handled by main router
      // Add internal-specific middleware stack
      .layer(
        axum_middleware::from_fn_with_state(
          app_state.clone(),
          internal_middleware_stack
        )
      )
      .layer(Extension(container.fcm_service.clone()))
      .layer(Extension(container.user_notification_repo.clone()))
      .with_state(app_state);

    Ok(internal_router)
  }
}
