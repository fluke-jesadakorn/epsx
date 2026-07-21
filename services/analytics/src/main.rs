use clap::{Parser, ValueEnum};
use epsx_analytics::{build_auth_verifier, build_router, init_schema, SqlAnalyticsStore};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{net::SocketAddr, sync::Arc};

#[derive(Parser)]
#[command(name = "epsx-analytics", about = "EPSX Analytics Service")]
struct Args {
    #[arg(long, default_value = "8107")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(
        long,
        default_value = "postgres://epsx:epsx@localhost:5432/epsx_analytics"
    )]
    database_url: String,
    #[arg(long, env = "OIDC_ISSUER")]
    oidc_issuer: String,
    #[arg(long, env = "OIDC_JWKS_URL")]
    jwks_url: Option<String>,
    #[arg(long, env = "EPSX_ENV", value_enum, default_value = "development")]
    environment: Environment,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Environment {
    Development,
    Production,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("analytics");
    let args = Args::parse();
    let production = matches!(args.environment, Environment::Production);
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("analytics OIDC configuration must be valid");

    let db = sqlx::PgPool::connect(&args.database_url)
        .await
        .expect("Failed to connect to database");
    init_schema(&db)
        .await
        .expect("Failed to initialize analytics schema");
    let _prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    let app = build_router(Arc::new(SqlAnalyticsStore::new(db)), verifier);
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("analytics listen address must be valid");
    tracing::info!(%addr, "analytics service listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("analytics listener must bind");
    axum::serve(listener, app)
        .await
        .expect("analytics server failed");
}
