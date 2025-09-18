use sqlx::{PgPool, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        
    println!("Connecting to database to check schema...");
    
    let pool = PgPool::connect(&database_url).await?;
    
    // Check users table schema
    println!("\n=== USERS TABLE COLUMNS ===");
    let rows = sqlx::query(
        "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = 'users' ORDER BY column_name;"
    )
    .fetch_all(&pool)
    .await?;
    
    for row in rows {
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");
        println!("{}: {} (nullable: {})", column_name, data_type, is_nullable);
    }
    
    // Check if user_permissions table exists
    println!("\n=== CHECKING USER_PERMISSIONS TABLE ===");
    let table_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'user_permissions'"
    )
    .fetch_one(&pool)
    .await?;
    
    if table_exists > 0 {
        println!("✅ user_permissions table exists");
        
        let rows = sqlx::query(
            "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = 'user_permissions' ORDER BY column_name;"
        )
        .fetch_all(&pool)
        .await?;
        
        for row in rows {
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let is_nullable: String = row.get("is_nullable");
            println!("{}: {} (nullable: {})", column_name, data_type, is_nullable);
        }
    } else {
        println!("❌ user_permissions table does not exist");
    }
    
    // Check what tables do exist
    println!("\n=== ALL TABLES ===");
    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name;"
    )
    .fetch_all(&pool)
    .await?;
    
    for row in rows {
        let table_name: String = row.get("table_name");
        println!("- {}", table_name);
    }
    
    pool.close().await;
    
    Ok(())
}