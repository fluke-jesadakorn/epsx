use tokio_postgres::{Client, NoTls};
use std::time::Duration;
use tracing::{info, error, warn};

/// Direct database client for high-latency connections
/// Bypasses diesel-async connection pooling issues
pub struct DirectDbClient {
    database_url: String,
}

impl DirectDbClient {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
    
    /// Create a direct connection with optimized settings for high-latency networks
    async fn create_connection(&self) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔗 Creating direct database connection for high-latency query");
        
        let (client, connection) = tokio_postgres::connect(&self.database_url, NoTls).await
            .map_err(|e| {
                error!("❌ Direct database connection failed: {}", e);
                e
            })?;
            
        // Spawn the connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection error: {}", e);
            }
        });
        
        info!("✅ Direct database connection established successfully");
        Ok(client)
    }
    
    /// Find user by email with direct connection and retry logic
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserData>, Box<dyn std::error::Error + Send + Sync>> {
        let mut attempts = 0;
        let max_attempts = 3;
        let mut backoff_ms = 500;
        
        loop {
            attempts += 1;
            info!("🔍 Direct user lookup attempt {} of {} for email: {}", attempts, max_attempts, email);
            
            match self.try_find_user(email).await {
                Ok(user) => {
                    info!("✅ Direct user lookup successful on attempt {}", attempts);
                    return Ok(user);
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        error!("❌ Direct user lookup failed after {} attempts: {}", max_attempts, e);
                        return Err(e);
                    }
                    
                    warn!("⚠️ Direct user lookup attempt {} failed: {}. Retrying in {}ms...", attempts, e, backoff_ms);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms *= 2;
                }
            }
        }
    }
    
    /// Try to find user with timeout
    async fn try_find_user(&self, email: &str) -> Result<Option<UserData>, Box<dyn std::error::Error + Send + Sync>> {
        // Create connection with timeout
        let connection_future = self.create_connection();
        let client = tokio::time::timeout(Duration::from_secs(10), connection_future)
            .await
            .map_err(|_| "Connection timeout after 10 seconds")?
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        // Execute query with timeout
        let query = "SELECT id, email, firebase_uid, role, created_at, updated_at, subscription_tier, is_active FROM users WHERE email = $1 AND is_active = true";
        let email_param = email.to_string(); // Store email to avoid borrow issues
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&email_param];
        let query_future = client.query_opt(query, params);
        
        let row = tokio::time::timeout(Duration::from_secs(15), query_future)
            .await
            .map_err(|_| "Query timeout after 15 seconds")?
            .map_err(|e| format!("Query failed: {}", e))?;
            
        match row {
            Some(row) => {
                // Read database values with fallback string conversion
                let id = if let Ok(uuid_val) = row.try_get::<_, uuid::Uuid>("id") {
                    uuid_val
                } else {
                    let id_str: String = row.get("id");
                    uuid::Uuid::parse_str(&id_str)
                        .map_err(|e| format!("Invalid UUID in database: {}", e))?
                };
                
                let (created_at, updated_at) = if let (Ok(c), Ok(u)) = (
                    row.try_get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
                    row.try_get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
                ) {
                    (c.naive_utc(), u.naive_utc())
                } else {
                    let created_at_str: String = row.get("created_at");
                    let updated_at_str: String = row.get("updated_at");
                    
                    let created_at = chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S"))
                        .map_err(|e| format!("Invalid created_at timestamp: {}", e))?;
                    
                    let updated_at = chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
                        .or_else(|_| chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S"))
                        .map_err(|e| format!("Invalid updated_at timestamp: {}", e))?;
                    
                    (created_at, updated_at)
                };

                let user_data = UserData {
                    id,
                    email: row.get("email"),
                    firebase_uid: row.get("firebase_uid"),
                    role: row.get("role"),
                    created_at,
                    updated_at,
                    subscription_tier: row.get("subscription_tier"),
                    is_active: row.get("is_active"),
                };
                info!("✅ User found: {} with role: {}", user_data.email, user_data.role);
                Ok(Some(user_data))
            }
            None => {
                info!("ℹ️ No user found for email: {}", email);
                Ok(None)
            }
        }
    }
}

/// Simple user data structure for direct queries
#[derive(Debug)]
pub struct UserData {
    pub id: uuid::Uuid,
    pub email: String,
    pub firebase_uid: Option<String>,
    pub role: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub subscription_tier: String,
    pub is_active: bool,
}