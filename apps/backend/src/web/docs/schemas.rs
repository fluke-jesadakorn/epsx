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