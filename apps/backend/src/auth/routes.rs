use crate::auth::{ handlers, AuthService };
use axum::{ extract::{ FromRef, State }, middleware, routing::get, Router, Json, http::StatusCode };
use serde::Serialize;
use tracing::{ info, debug };

#[derive(Clone, FromRef)]
struct AppState {
    auth_service: AuthService,
}

#[derive(Serialize)]
struct AdminDashboardResponse {
    message: String,
    status: String,
}

// Example admin handler
async fn admin_dashboard(State(_state): State<AppState>) -> Result<
    Json<AdminDashboardResponse>,
    StatusCode
> {
    Ok(
        Json(AdminDashboardResponse {
            message: "Welcome to admin dashboard".to_string(),
            status: "success".to_string(),
        })
    )
}

pub fn auth_router(auth_service: AuthService) -> Router {
    info!("Initializing authentication router");

    // Public routes (no auth required)
    debug!("Setting up public routes");
    let state = AppState { auth_service: auth_service.clone() };

    let public_routes = Router::new().with_state(state.clone());

    // Protected routes (auth required)
    debug!("Setting up protected routes");
    let protected_routes = Router::new()
        .route("/session/validate", get(handlers::session_validate))
        .route("/protected", get(handlers::protected_example))
        .route("/sign-in", axum::routing::post(handlers::sign_in))
        .route("/sign-out", get(handlers::sign_out))
        .layer(middleware::from_fn(super::middleware::auth_middleware))
        .with_state(state.clone());

    // Admin routes (admin role required)
    debug!("Setting up admin routes");
    let admin_routes = Router::new()
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin-only", get(handlers::admin_only_example))
        .layer(middleware::from_fn(super::middleware::auth_middleware))
        .with_state(state.clone());

    // Return router with all routes
    info!("Finalizing router setup");
    Router::new().merge(public_routes).merge(protected_routes).merge(admin_routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Config;

    #[allow(dead_code)]
    async fn setup() -> Router {
        let config = Config {
            firebase_service_account_path: "test-service-account.json".to_string(),
            port: 8080,
            frontend_url: "http://localhost:3000".to_string(),
            musepay_partner_id: "test_partner_id".to_string(),
            musepay_private_key: "test_private_key".to_string(),
            musepay_api_url: "http://localhost:8081".to_string(),
        };
        let auth_service = AuthService::new(config).expect("Failed to create auth service");
        Router::new().merge(auth_router(auth_service))
    }
}
