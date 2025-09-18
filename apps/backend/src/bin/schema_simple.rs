use sqlx::{PgPool, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        
    println!("Connecting to database...");
    
    let pool = PgPool::connect(&database_url).await?;
    
    // Test basic connection
    let _row = sqlx::query("SELECT 1 as test_value")
        .fetch_one(&pool)
        .await?;
        
    println!("✅ Basic connection works");
    
    // Test if user exists (critical query for login)
    let email = "info@epsx.io";
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&pool)
        .await?;
        
    println!("Found {} users with email {}", user_count, email);
    
    // Show actual users table columns 
    println!("\n=== CHECKING users TABLE SCHEMA ===");
    let column_rows = sqlx::query("SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'users' ORDER BY column_name")
        .fetch_all(&pool)
        .await?;
    
    for row in column_rows {
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        println!("  {} ({})", column_name, data_type);
    }
    
    pool.close().await;
    
    Ok(())
}