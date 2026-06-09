use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
    routing::any,
    Router,
};
use epsx_auth::{jwt_middleware, JwtAuth};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Parser)]
#[command(name = "epsx-gateway", about = "EPSX API Gateway")]
struct Args {
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "super-secret-jwt-key")]
    jwt_secret: String,
    #[arg(long, default_value = "http://localhost:8101")]
    identity_url: String,
    #[arg(long, default_value = "http://localhost:8102")]
    wallet_url: String,
    #[arg(long, default_value = "http://localhost:8103")]
    payment_url: String,
    #[arg(long, default_value = "http://localhost:8104")]
    subscription_url: String,
    #[arg(long, default_value = "http://localhost:8105")]
    content_url: String,
    #[arg(long, default_value = "http://localhost:8106")]
    notification_url: String,
    #[arg(long, default_value = "http://localhost:8107")]
    analytics_url: String,
    #[arg(long, default_value = "http://localhost:8108")]
    indexer_url: String,
}

#[derive(Clone)]
struct AppState {
    identity_url: String,
    wallet_url: String,
    payment_url: String,
    subscription_url: String,
    content_url: String,
    notification_url: String,
    analytics_url: String,
    indexer_url: String,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("gateway");
    let args = Args::parse();

    let auth = Arc::new(JwtAuth::from_secret(&args.jwt_secret));

    let state = AppState {
        identity_url: args.identity_url.clone(),
        wallet_url: args.wallet_url.clone(),
        payment_url: args.payment_url.clone(),
        subscription_url: args.subscription_url.clone(),
        content_url: args.content_url.clone(),
        notification_url: args.notification_url.clone(),
        analytics_url: args.analytics_url.clone(),
        indexer_url: args.indexer_url.clone(),
    };

    let app = Router::new()
        .route("/health", any(health))
        .route("/api/v1/identity/{*path}", any(proxy_identity))
        .route("/api/v1/wallet/{*path}", any(proxy_wallet))
        .route("/api/v1/payment/{*path}", any(proxy_payment))
        .route("/api/v1/subscription/{*path}", any(proxy_subscription))
        .route("/api/v1/content/{*path}", any(proxy_content))
        .route("/api/v1/news", any(proxy_news))
        .route("/api/v1/news/{*path}", any(proxy_news))
        .route("/api/v1/portfolio/{*path}", any(proxy_portfolio))
        .route("/api/v1/plans", any(proxy_plans))
        .route("/api/v1/plans/{*path}", any(proxy_plans))
        .route("/api/v1/rankings", any(proxy_rankings))
        .route("/api/v1/rankings/{*path}", any(proxy_rankings))
        .route("/api/v1/notification/{*path}", any(proxy_notification))
        .route("/api/v1/analytics/{*path}", any(proxy_analytics))
        .route("/api/v1/indexer/{*path}", any(proxy_indexer))
        .layer(axum::middleware::from_fn_with_state(auth.clone(), jwt_middleware))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS, Method::PATCH])
                .allow_headers(Any)
                .allow_credentials(false),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    tracing::info!("Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

macro_rules! proxy_fn {
    ($name:ident, $state:ident, $svc:ident) => {
        async fn $name(
            axum::extract::State($state): axum::extract::State<AppState>,
            uri: axum::http::Uri,
            req: axum::http::Request<axum::body::Body>,
        ) -> Result<axum::http::Response<axum::body::Body>, StatusCode> {
            proxy_to_service(&$state.$svc, uri, req, None).await
        }
    };
}

macro_rules! proxy_rewrite_fn {
    ($name:ident, $state:ident, $svc:ident, $from:literal, $to:literal) => {
        async fn $name(
            axum::extract::State($state): axum::extract::State<AppState>,
            uri: axum::http::Uri,
            req: axum::http::Request<axum::body::Body>,
        ) -> Result<axum::http::Response<axum::body::Body>, StatusCode> {
            proxy_to_service(&$state.$svc, uri, req, Some(($from, $to))).await
        }
    };
}

proxy_fn!(proxy_identity, state, identity_url);
proxy_fn!(proxy_wallet, state, wallet_url);
proxy_fn!(proxy_payment, state, payment_url);
proxy_fn!(proxy_subscription, state, subscription_url);
proxy_fn!(proxy_content, state, content_url);
proxy_rewrite_fn!(proxy_news, state, content_url, "/api/v1/news", "/api/v1/content/news");
proxy_rewrite_fn!(proxy_portfolio, state, content_url, "/api/v1/portfolio", "/api/v1/content/portfolio");
proxy_rewrite_fn!(proxy_plans, state, content_url, "/api/v1/plans", "/api/v1/content/plans");
proxy_rewrite_fn!(proxy_rankings, state, content_url, "/api/v1/rankings", "/api/v1/content/rankings");
proxy_fn!(proxy_notification, state, notification_url);
proxy_fn!(proxy_analytics, state, analytics_url);
proxy_fn!(proxy_indexer, state, indexer_url);

async fn proxy_to_service(
    base_url: &str,
    uri: axum::http::Uri,
    req: Request,
    rewrite: Option<(&str, &str)>,
) -> Result<Response, StatusCode> {
    let client = reqwest::Client::new();
    let raw_path = uri.path();
    let path = match rewrite {
        Some((from, to)) if raw_path.starts_with(from) => {
            format!("{}{}", to, &raw_path[from.len()..])
        }
        _ => raw_path.to_string(),
    };
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
    let url = format!("{}{}{}", base_url.trim_end_matches('/'), path, query);

    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut request = client.request(method, &url);
    for (key, value) in headers.iter() {
        if key != "host" && key != "content-length" {
            request = request.header(key, value);
        }
    }

    let response = request
        .body(body)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("proxy error: {} -> {}: {}", base_url, url, e);
            StatusCode::BAD_GATEWAY
        })?;

    let mut builder = Response::builder().status(response.status().as_u16());

    for (key, value) in response.headers() {
        if key != "content-length" {
            builder = builder.header(key, value);
        }
    }

    let resp_body = response.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    builder
        .body(axum::body::Body::from(resp_body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
