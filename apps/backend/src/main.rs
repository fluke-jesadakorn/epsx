mod auth;
mod config;
mod payment;
mod db;
mod stock;

use anyhow::{ Context, Result };
use crate::{ auth::AuthService, config::Config, payment::PaymentService, stock::StockService };
use std::sync::Arc;
use std::net::SocketAddr;
use axum::{ routing::get, Router };
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::{ trace::{ TraceLayer, DefaultOnResponse }, cors::CorsLayer };
use tracing::{ info, error, Level };
use tracing_subscriber::{ self, EnvFilter };
use axum::http::{ Method, HeaderName, HeaderValue };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    tracing_subscriber
        ::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .init();

    info!("Starting API server...");

    // Load configuration
    let config = Config::from_env()?;

    // Get address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // Initialize MongoDB
    let mongodb_client = db::connect_db().await?;
    let _mongodb_client = Arc::new(mongodb_client);

    // Initialize services
    let auth_service = AuthService::new(config.clone()).context(
        "Failed to initialize auth service"
    )?;
    let payment_service = PaymentService::new(config.clone()).expect(
        "Failed to initialize payment service"
    );
    let stock_service = Arc::new(StockService::new());

    info!("All services initialized successfully");

    #[derive(OpenApi)]
    #[openapi(
        paths(
            auth::handlers::session_validate,
            auth::handlers::protected_example,
            auth::handlers::admin_only_example,
            auth::handlers::sign_in,
            auth::handlers::sign_out
        ),
        components(
            schemas(
                auth::handlers::SignInRequest,
                auth::handlers::SignInResponse,
                auth::handlers::AuthResponse,
                auth::handlers::ProtectedResponse
            )
        ),
        tags((name = "Auth", description = "Authentication endpoints"))
    )]
    struct ApiDoc;

    // Create OpenAPI documentation
    let api_docs = ApiDoc::openapi();

    // Create router with API v1 routes
    let app = Router::new()
        .route(
            "/",
            get(|| async { "API is running" })
        )
        .nest(
            "/v1",
            Router::new()
                .nest("/auth", auth::auth_router(auth_service.clone()))
                .nest("/payment", payment::payment_router(payment_service, auth_service.clone()))
                .nest("/stock", stock::stock_router(stock_service))
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_response(DefaultOnResponse::new().include_headers(true))
        )
        .layer({
            let frontend_url = std::env
                ::var("FRONTEND_URL")
                .unwrap_or_else(|_| String::from("http://localhost:3000"));

            CorsLayer::new()
                .allow_origin(vec![HeaderValue::from_str(&frontend_url).unwrap()])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::PATCH,
                    Method::HEAD,
                ])
                .allow_headers(
                    Vec::from([
                        HeaderName::from_static("authorization"),
                        HeaderName::from_static("content-type"),
                        HeaderName::from_static("accept"),
                        HeaderName::from_static("origin"),
                        HeaderName::from_static("x-frontend-url"),
                        HeaderName::from_static("user-agent"),
                        HeaderName::from_static("access-control-allow-origin"),
                    ])
                )
                .allow_credentials(true)
                .expose_headers(
                    Vec::from([
                        HeaderName::from_static("authorization"),
                        HeaderName::from_static("content-type"),
                        HeaderName::from_static("accept"),
                        HeaderName::from_static("origin"),
                        HeaderName::from_static("location"),
                        HeaderName::from_static("access-control-expose-headers"),
                    ])
                )
                .max_age(std::time::Duration::from_secs(86400))
        })
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_docs));

    // Start the server
    info!("Server initialized successfully, starting to serve requests...");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await
        .map_err(|e| {
            error!("Server error: {}", e);
            Box::new(e)
        })?;

    Ok(())
}
