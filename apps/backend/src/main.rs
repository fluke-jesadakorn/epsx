mod auth;
mod config;
mod payment;

use crate::{ auth::AuthService, config::Config, payment::PaymentService };
use std::net::SocketAddr;
use axum::{ routing::get, Router };
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::{ trace::TraceLayer, cors::CorsLayer };
use tracing::{ info, error, Level };
use tracing_subscriber::{ self, EnvFilter };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    tracing_subscriber
        ::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    info!("Starting API server...");

    // Load configuration
    let config = Config::from_env()?;

    // Get address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // Initialize services
    let auth_service = AuthService::new(config.clone()).expect("Failed to initialize auth service");
    let payment_service = PaymentService::new(config.clone()).expect(
        "Failed to initialize payment service"
    );

    #[derive(OpenApi)]
    #[openapi(
        paths(
            auth::handlers::email_sign_up,
            auth::handlers::email_sign_in,
            auth::handlers::google_oauth_init,
            auth::handlers::google_oauth_callback,
            auth::handlers::logout
        ),
        components(
            schemas(
                auth::handlers::EmailSignUpRequest,
                auth::handlers::EmailSignInRequest,
                auth::handlers::AuthResponse,
                auth::handlers::OAuthUrlResponse
            )
        ),
        tags((name = "Auth", description = "Authentication endpoints"))
    )]
    struct ApiDoc;

    // Create router with API v1 routes
    let mut openapi = ApiDoc::openapi();
    openapi.merge(payment::routes::PaymentApiDoc::openapi());
    info!("OpenAPI documentation initialized with payment routes");

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .route(
            "/",
            get(|| async { "API is running" })
        )
        .nest(
            "/v1",
            Router::new()
                .nest("/auth", auth::auth_router(&auth_service))
                .nest("/payment", payment::payment_router(payment_service, auth_service.clone()))
        )
        .layer(
            CorsLayer::new()
                .allow_origin(
                    config.frontend_url
                        .parse::<axum::http::HeaderValue>()
                        .unwrap_or_else(|_| "http://localhost:3000".parse().unwrap())
                )
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                    "X-Client-Type".parse().unwrap(),
                    "X-Frontend-URL".parse().unwrap(),
                ])
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_credentials(true)
                .expose_headers([axum::http::header::SET_COOKIE])
        )
        .layer(TraceLayer::new_for_http());

    // Start the server
    info!("Server initialized successfully, starting to serve requests...");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.map_err(|e| {
        error!("Server error: {}", e);
        Box::new(e)
    })?;

    Ok(())
}
