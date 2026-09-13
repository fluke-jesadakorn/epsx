use clap::{Parser, ValueEnum};
use epsx_identity::{build_auth_verifier, build_router};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "epsx-identity", about = "EPSX Identity Service")]
struct Args {
    #[arg(long, default_value = "8101")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
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
    epsx_observability::Observability::init("identity");
    let args = Args::parse();
    let production = matches!(args.environment, Environment::Production);
    let jwks_url = args.jwks_url.unwrap_or_else(|| {
        format!(
            "{}/.well-known/jwks.json",
            args.oidc_issuer.trim_end_matches('/')
        )
    });
    let verifier = build_auth_verifier(&args.oidc_issuer, &jwks_url, production)
        .expect("identity OIDC configuration must be valid");

    // Identity persistence, SIWE nonce consumption, refresh rotation, and
    // administrative mutation stay structurally disabled until their audited
    // lifecycle contracts have additive schemas and concurrency proofs.
    let app = build_router(verifier);
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("identity listen address must be valid");
    tracing::info!(%addr, "identity service listening with lifecycle routes disabled");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("identity listener must bind");
    axum::serve(listener, app)
        .await
        .expect("identity server failed");
}
