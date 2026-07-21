use clap::{Parser, ValueEnum};
use epsx_gateway::{
    auth::{JwksVerifier, JwksVerifierConfig},
    build_http_client, build_router, AppState, GatewayUrls,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};

#[derive(Parser)]
#[command(name = "epsx-gateway", about = "EPSX API Gateway")]
struct Args {
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, env = "OIDC_ISSUER")]
    oidc_issuer: String,
    #[arg(long, env = "OIDC_JWKS_URL")]
    jwks_url: Option<String>,
    #[arg(long, env = "EPSX_ENV", value_enum, default_value = "development")]
    environment: Environment,
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Environment {
    Development,
    Production,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("gateway");
    let args = Args::parse();

    let client = build_http_client().expect("gateway HTTP client configuration must be valid");
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let production = matches!(args.environment, Environment::Production);
    let verifier_config = JwksVerifierConfig::new(
        args.oidc_issuer,
        jwks_url,
        Duration::from_secs(5 * 60),
        production,
    )
    .expect("OIDC_ISSUER and OIDC_JWKS_URL must be valid URLs");
    let verifier = Arc::new(JwksVerifier::new(verifier_config, client.clone()));

    let state = AppState::new(
        GatewayUrls {
            identity: args.identity_url,
            wallet: args.wallet_url,
            payment: args.payment_url,
            subscription: args.subscription_url,
            content: args.content_url,
            notification: args.notification_url,
            analytics: args.analytics_url,
            indexer: args.indexer_url,
        },
        client,
        verifier,
        production,
    )
    .expect("gateway upstream URLs must be valid and non-local in production");
    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("gateway listen address must be valid");
    tracing::info!(%addr, "gateway listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("gateway listener must bind");
    axum::serve(listener, app)
        .await
        .expect("gateway server failed");
}
