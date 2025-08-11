//! Test if analytics router can be created successfully
use std::sync::Arc;

// Test analytics router creation
async fn test_analytics_router() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing analytics router creation...");
    
    // Try to create minimal infrastructure
    let postgres_url = "postgresql://postgres:password@localhost:5432/epsx_db";
    println!("📊 Connecting to database...");
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(postgres_url)
        .await?;
    
    println!("✅ Database connected");
    
    // Create infrastructure factory
    let infra = epsx::infra::InfraFactory {
        database_backend: epsx::infra::DatabaseBackend::PostgreSQL,
        postgres_pool: epsx::infra::DatabasePool::Postgres(pool),
    };
    
    println!("🏗️ Creating analytics router...");
    
    // Try to create analytics router
    let router = epsx::web::analytics::create_analytics_router(&infra);
    
    println!("✅ Analytics router created successfully!");
    println!("🎉 Test passed - analytics routes should be working");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();
    
    match test_analytics_router().await {
        Ok(()) => {
            println!("✅ Analytics router test PASSED");
        }
        Err(e) => {
            println!("❌ Analytics router test FAILED: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}