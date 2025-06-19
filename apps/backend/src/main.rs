use std::{sync::Arc, net::SocketAddr};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod auth;
mod config;
mod db;
mod payment;
mod stock;

use crate::{ auth::AuthService, config::Config, payment::PaymentService, stock::FinancialDataService };

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--websocket" {
        // Run websocket client
        let config = Arc::new(Config::from_env());
        let financial_data_service = FinancialDataService::new(&config);
        financial_data_service.subscribe_to_symbols(vec!["AAPL".to_string()]).await.unwrap();
        tokio::signal::ctrl_c().await.unwrap();
        return;
    }

    // Load configuration
    let config = Arc::new(Config::from_env());
    info!("Configuration loaded");

    // Initialize MongoDB connection
    let db = Arc::new(db::connect_db().await.expect("Failed to connect to MongoDB"));
    info!("Connected to MongoDB");

    // Initialize services
    let auth_service = Arc::new(AuthService::new(db.clone()));
    let payment_service = Arc::new(PaymentService::new(config.clone()));
    info!("Services initialized");

    // CORS middleware
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    // Build the router
    let app = Router::new()
        .merge(payment::payment_router(payment_service, auth_service.clone()))
        .merge(stock::stock_router(&config, db.clone()))
        .merge(auth::auth_router(auth_service))
        .layer(cors);

    // Start the server
    let addr = SocketAddr::new(
        config.host().parse().expect("Invalid host"),
        config.port(),
    );
    info!("Server starting on {}", addr);
    
    axum::serve(
        tokio::net::TcpListener::bind(&addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
