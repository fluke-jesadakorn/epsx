use crate::auth::{ handlers, AuthService };
use axum::{ middleware, routing::{ get, post }, Router };
use tower_http::cors::{ Any, CorsLayer };
use tracing::{ info, debug };

pub fn auth_router(auth_service: AuthService) -> Router {
    info!("Initializing authentication router");

    // Configure CORS
    debug!("Configuring CORS policy");
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // Public routes (no auth required)
    debug!("Setting up public routes");
    let public_routes = Router::new()
        .route("/register", post(handlers::email_sign_up))
        .route("/login", post(handlers::email_sign_in))
        .route("/google/init", get(handlers::google_oauth_init))
        .route("/google/callback", get(handlers::google_oauth_callback))
        .with_state(auth_service.clone());

    // Protected routes (auth required)
    debug!("Setting up protected routes");
    let protected_routes = Router::new()
        .route("/protected/example", get(handlers::email_sign_in)) // Temporary route for testing
        .with_state(auth_service.clone())
        .layer(middleware::from_fn_with_state(auth_service, super::middleware::auth_middleware));

    // Combine routes
    info!("Finalizing router setup");
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::http::StatusCode;
    use serde_json::json;
    use tower::ServiceExt;

    async fn setup() -> Router {
        let config = Config {
            firebase_project_id: "test-project".to_string(),
            firebase_private_key: "-----BEGIN PRIVATE KEY-----\nMIIE...\\n-----END PRIVATE KEY-----\n".to_string(),
            firebase_client_email: "firebase-adminsdk@test-project.iam.gserviceaccount.com".to_string(),
            // Other required fields with dummy values
            port: 3001,
            host: "localhost".to_string(),
            mongodb_uri: "mongodb://localhost".to_string(),
            firebase_api_key: "test-api-key".to_string(),
            jwt_secret: "test-secret".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            google_client_id: "test-client-id".to_string(),
            google_client_secret: "test-client-secret".to_string(),
            google_redirect_uri: "http://localhost:3000/auth/oauth/google/callback".to_string(),
        };

        let auth_service = AuthService::new(config).expect("Failed to create auth service");
        auth_router(auth_service)
    }

    #[tokio::test]
    async fn test_register_endpoint() {
        let app = setup().await;

        let response = app
            .oneshot(
                axum::http::Request
                    ::builder()
                    .method("POST")
                    .uri("/register")
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "email": "test@example.com",
                            "password": "password123"
                        }).to_string()
                    )
                    .unwrap()
            ).await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
