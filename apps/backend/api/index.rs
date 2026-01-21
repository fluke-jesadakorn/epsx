use std::sync::Arc;
use tracing::{info, error};
use vercel_runtime::{run, Request, Response, Body, Error};
use tower::ServiceExt;

use epsx::{
    infrastructure::container::DomainContainer,
    create_router,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Initializing EPSX Backend for Vercel Serverless...");

    // Get database pool (using the global static pool optimized for serverless)
    let db_pool = match epsx::infrastructure::database::get_diesel_pool().await {
        Ok(pool) => Arc::new(pool),
        Err(e) => {
            error!("❌ Failed to initialize database pool: {}", e);
            return Err(format!("Database initialization failed: {}", e).into());
        }
    };

    // Create container (stateless for Vercel)
    let container = Arc::new(DomainContainer::new(db_pool));
    
    // Create the unified router
    let app = create_router(container);

    info!("✅ Vercel serverless function initialized");

    // Run the service with Vercel runtime
    // We wrap the router in a closure that calls oneshot and converts the body
    run(move |req: Request| {
        let app = app.clone();
        async move {
            let response = app.oneshot(req).await.map_err(|e: axum::BoxError| Error::from(e.to_string()))?;
            let (parts, body) = response.into_parts();
            
            // Convert Axum body to bytes
            let bytes = axum::body::to_bytes(body, usize::MAX)
                .await
                .map_err(|e: axum::Error| Error::from(e.to_string()))?;
            
            // Create Vercel response with Vercel body
            let vercel_response = Response::from_parts(parts, Body::from(bytes.to_vec()));
            Ok(vercel_response)
        }
    }).await
}
