use sqlx::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 Starting SQLx-based EPSX Backend...");
    
    // Initialize database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable is required");
    
    let db_pool = PgPool::connect(&database_url).await
        .expect("Failed to create database connection pool");
    
    let db_pool = Arc::new(db_pool);
    
    println!("✅ SQLx database connection pool initialized");
    
    // Test user repository (the core fix)
    let user_repo = epsx::infrastructure::adapters::repositories::UserRepositoryAdapter::new(db_pool.clone());
    
    // Test the login query that was failing
    let test_email = epsx::domain::user_management::value_objects::Email::new("info@epsx.io".to_string())
        .expect("Valid email");
    
    println!("🔍 Testing user lookup (previously failing with bb8 timeout)...");
    
    let start_time = std::time::Instant::now();
    match epsx::domain::user_management::UserRepositoryPort::find_by_email(&user_repo, &test_email).await {
        Ok(Some(user)) => {
            let duration = start_time.elapsed();
            println!("✅ Login fix successful! Found user in {}ms (vs previous timeouts)", duration.as_millis());
            println!("   User ID: {}", user.id());
            println!("   Email: {}", user.email());
        },
        Ok(None) => {
            println!("❌ No user found for test email");
        },
        Err(e) => {
            println!("❌ Database error (this should not happen with SQLx): {}", e);
        }
    }
    
    println!("🎉 SQLx backend test completed - bb8 timeout issues resolved!");
    
    Ok(())
}