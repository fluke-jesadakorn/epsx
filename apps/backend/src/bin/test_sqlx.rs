use sqlx::{PgPool, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        
    println!("Connecting to database...");
    
    // Create SQLx connection pool
    let pool = PgPool::connect(&database_url).await?;
    
    println!("✅ SQLx connection successful!");
    
    // Test a simple query
    let row = sqlx::query("SELECT 1 as test_value")
        .fetch_one(&pool)
        .await?;
        
    let test_value: i32 = row.get("test_value");
    println!("✅ Test query successful: {}", test_value);
    
    // Test user lookup by email (the failing query)
    let email = "info@epsx.io";
    println!("Testing user lookup for: {}", email);
    
    let user_row = sqlx::query("SELECT id, email FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&pool)
        .await?;
        
    match user_row {
        Some(row) => {
            let user_id: uuid::Uuid = row.get("id");
            let user_email: String = row.get("email");
            println!("✅ Found user: {} ({})", user_email, user_id);
        },
        None => {
            println!("❌ No user found for email: {}", email);
        }
    }
    
    pool.close().await;
    println!("✅ Connection closed successfully");
    
    Ok(())
}