//! Simple test server for API documentation without database dependencies

use axum::{routing::get, Router, Json};
use serde_json::json;
use std::net::SocketAddr;

use epsx::web::docs::{create_docs_routes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting EPSX API Documentation Test Server...");
    
    // Create basic router
    let mut app = Router::new()
        .route("/health", get(|| async { 
            Json(json!({
                "status": "healthy",
                "service": "epsx-docs-test",
                "timestamp": chrono::Utc::now()
            }))
        }));
    
    // Add documentation routes (always available)
    app = app.merge(create_docs_routes());
    println!("📚 API Documentation available at: http://localhost:8082/docs");
    println!("📄 OpenAPI spec available at: http://localhost:8082/api-docs/openapi.json");
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8082));
    println!("🌐 Test server listening on {}", addr);
    
    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}