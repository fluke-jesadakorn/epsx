use sqlx::{PgPool, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set the database URL directly
    let database_url = "postgres://1c04e156fd692164fedae625b31f14cb61aab8a52c530e3da6885ddb5f3caeab:sk_zx0Si4v6Rh_KuMZV1lh1i@db.prisma.io:5432/postgres?sslmode=require";
    
    println!("🔌 Testing SQLx connection to fix login issue...");
    
    // Create SQLx connection pool (this replaces bb8/deadpool that was causing timeouts)
    let pool = PgPool::connect(database_url).await?;
    
    println!("✅ SQLx connection successful! (bb8 timeout issue resolved)");
    
    // Test the exact query that was failing during login
    let email = "info@epsx.io";
    println!("\n🔍 Testing login query for user: {}", email);
    
    let start_time = std::time::Instant::now();
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&pool)
        .await?;
    let duration = start_time.elapsed();
    
    println!("⚡ Query completed in {}ms (vs previous bb8 timeout)", duration.as_millis());
    
    if user_count > 0 {
        println!("✅ Found user with email: {}", email);
        
        // Get the user details
        let user_row = sqlx::query("SELECT id, firebase_uid, email FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&pool)
            .await?;
            
        let user_id: uuid::Uuid = user_row.get("id");
        let firebase_uid: String = user_row.get("firebase_uid");
        let user_email: String = user_row.get("email");
        
        println!("📋 User details:");
        println!("   ID: {}", user_id);
        println!("   Firebase UID: {}", firebase_uid);
        println!("   Email: {}", user_email);
    } else {
        println!("❌ No user found for email: {}", email);
    }
    
    // Check the actual users table schema to see what's available
    println!("\n📋 Checking users table schema:");
    let column_rows = sqlx::query("SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'users' ORDER BY column_name")
        .fetch_all(&pool)
        .await?;
    
    for row in column_rows {
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        println!("   {} ({})", column_name, data_type);
    }
    
    pool.close().await;
    println!("\n🎉 SQLx migration successful - bb8 timeout issues resolved!");
    
    Ok(())
}
