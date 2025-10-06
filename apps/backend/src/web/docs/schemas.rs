//! Common OpenAPI Schemas
//! 
//! Defines reusable schemas for API responses, errors, and common data structures.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    /// Indicates if the request was successful
    #[schema(example = true)]
    pub success: bool,
    
    /// Response data (only present on success)
    pub data: Option<T>,
    
    /// Human-readable message
    #[schema(example = "Operation completed successfully")]
    pub message: Option<String>,
    
    /// Pagination information (for paginated responses)
    pub pagination: Option<PaginationInfo>,
}

/// Error response structure
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Always false for error responses
    #[schema(example = false)]
    pub success: bool,
    
    /// Error code for programmatic handling
    #[schema(example = "invalid_wallet_address")]
    pub error: String,
    
    /// Human-readable error message
    #[schema(example = "The provided wallet address format is invalid")]
    pub message: String,
    
    /// Additional error details (optional)
    pub details: Option<serde_json::Value>,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    #[schema(example = "healthy")]
    pub status: String,
    
    /// Service name
    #[schema(example = "epsx-backend")]
    pub service: String,
    
    /// Current timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pagination information for list responses
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationInfo {
    /// Current page number (1-based)
    #[schema(example = 1)]
    pub page: u32,
    
    /// Number of items per page
    #[schema(example = 20)]
    pub limit: u32,
    
    /// Total number of items
    #[schema(example = 150)]
    pub total: u64,
    
    /// Total number of pages
    #[schema(example = 8)]
    pub total_pages: u32,
    
    /// Whether there is a next page
    #[schema(example = true)]
    pub has_next: bool,
    
    /// Whether there is a previous page
    #[schema(example = false)]
    pub has_prev: bool,
}

/// Web3 Challenge response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChallengeResponse {
    /// Unique nonce for this challenge
    #[schema(example = "abc123def456")]
    pub nonce: String,
    
    /// SIWE message to be signed
    #[schema(example = "epsx.io wants you to sign in with your Ethereum account:\\n0x1234...\\n\\nPlease sign this message to verify your ownership of this wallet.\\n\\nURI: https://epsx.io\\nVersion: 1\\nChain ID: 1\\nNonce: abc123def456")]
    pub message: String,
    
    /// Challenge expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,
    
    /// Wallet address for this challenge
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
}

/// Web3 Verification response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerificationResponse {
    /// Authentication token
    pub token: String,
    
    /// Token expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,
    
    /// User wallet address
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,
    
    /// User permissions
    pub permissions: Vec<String>,
}

/// Analytics ranking item
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RankingItem {
    /// Stock ranking position
    #[schema(example = 1)]
    pub rank: u32,
    
    /// Stock symbol
    #[schema(example = "AAPL")]
    pub symbol: String,
    
    /// Company name
    #[schema(example = "Apple Inc.")]
    pub name: String,
    
    /// EPS growth percentage
    #[schema(example = 15.5)]
    pub eps_growth: f64,
    
    /// Market capitalization
    #[schema(example = "3000000000000")]
    pub market_cap: u64,
    
    /// Industry sector
    #[schema(example = "Technology")]
    pub sector: String,
    
    /// Country code
    #[schema(example = "US")]
    pub country: String,
}

/// Session information response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionInfo {
    /// Session token
    pub token: String,

    /// Wallet address associated with session
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,

    /// Session creation time
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Session expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,

    /// User permissions in this session
    pub permissions: Vec<String>,

    /// Session is active
    #[schema(example = true)]
    pub active: bool,
}

/// Permission group structure
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PermissionGroup {
    /// Group unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,

    /// Group name
    #[schema(example = "Premium Users")]
    pub name: String,

    /// Group description
    #[schema(example = "Users with premium access permissions")]
    pub description: Option<String>,

    /// Permissions included in this group
    #[schema(example = json!(["epsx:rankings:view:50", "epsx:trading:premium"]))]
    pub permissions: Vec<String>,

    /// Group creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Group update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Permission validation request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PermissionCheckRequest {
    /// Wallet address to check
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,

    /// Permission to validate (format: platform:resource:action)
    #[schema(example = "epsx:analytics:read")]
    pub permission: String,
}

/// Permission validation response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PermissionCheckResponse {
    /// Whether the permission is granted
    #[schema(example = true)]
    pub has_permission: bool,

    /// Source of permission (group, direct, or none)
    #[schema(example = "group")]
    pub source: String,

    /// Group name if from group
    pub group_name: Option<String>,

    /// Permission expiry if time-limited
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Plan/Subscription tier
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Plan {
    /// Plan unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,

    /// Plan name
    #[schema(example = "Premium Plan")]
    pub name: String,

    /// Plan description
    #[schema(example = "Access to premium features including advanced analytics")]
    pub description: Option<String>,

    /// Monthly price in cents
    #[schema(example = 2999)]
    pub price_monthly: i64,

    /// Yearly price in cents
    #[schema(example = 29990)]
    pub price_yearly: i64,

    /// Associated permission group ID
    pub group_id: String,

    /// Plan features list
    #[schema(example = json!(["Advanced Analytics", "Premium Support", "API Access"]))]
    pub features: Vec<String>,

    /// Maximum ranking limit
    #[schema(example = 50)]
    pub ranking_limit: Option<i32>,

    /// Plan is active
    #[schema(example = true)]
    pub active: bool,
}

/// Wallet user information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WalletUser {
    /// Wallet address (primary identifier)
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: String,

    /// User's permission groups
    pub groups: Vec<String>,

    /// Direct permissions (non-group)
    pub permissions: Vec<String>,

    /// Account creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last activity timestamp
    pub last_active_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Account is active
    #[schema(example = true)]
    pub active: bool,

    /// Account metadata
    pub metadata: Option<serde_json::Value>,
}

/// Analytics country filter
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Country {
    /// Country code (ISO 3166-1 alpha-2)
    #[schema(example = "US")]
    pub code: String,

    /// Country name
    #[schema(example = "United States")]
    pub name: String,

    /// Available stock count
    #[schema(example = 5000)]
    pub stock_count: u32,
}

/// Analytics sector filter
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Sector {
    /// Sector identifier
    #[schema(example = "technology")]
    pub id: String,

    /// Sector display name
    #[schema(example = "Technology")]
    pub name: String,

    /// Stock count in sector
    #[schema(example = 1200)]
    pub stock_count: u32,
}

/// Notification message
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Notification {
    /// Notification unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,

    /// Notification type
    #[schema(example = "info")]
    pub notification_type: String,

    /// Notification title
    #[schema(example = "System Update")]
    pub title: String,

    /// Notification message
    #[schema(example = "New features are now available")]
    pub message: String,

    /// Target wallet addresses (if specific users)
    pub target_wallets: Option<Vec<String>>,

    /// Notification timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Read status
    #[schema(example = false)]
    pub read: bool,
}

/// Security event
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SecurityEvent {
    /// Event unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,

    /// Event type
    #[schema(example = "authentication_failure")]
    pub event_type: String,

    /// Severity level
    #[schema(example = "medium")]
    pub severity: String,

    /// Source wallet address
    #[schema(example = "0x1234567890123456789012345678901234567890")]
    pub wallet_address: Option<String>,

    /// Event description
    pub description: String,

    /// Event metadata
    pub metadata: Option<serde_json::Value>,

    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Platform statistics
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PlatformStats {
    /// Total registered wallets
    #[schema(example = 15000)]
    pub total_wallets: u64,

    /// Active wallets (last 30 days)
    #[schema(example = 5000)]
    pub active_wallets: u64,

    /// Total permission groups
    #[schema(example = 12)]
    pub total_groups: u32,

    /// Total API calls (last 24 hours)
    #[schema(example = 250000)]
    pub api_calls_24h: u64,

    /// Average response time (ms)
    #[schema(example = 45.5)]
    pub avg_response_time: f64,

    /// Timestamp of statistics
    pub timestamp: chrono::DateTime<chrono::Utc>,
}