// Service port interfaces for external integrations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::dom::values::{UserId, Currency, Symbol};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[async_trait]
#[cfg_attr(test, automock)]
pub trait FbAuthSvc: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<FbClaims, AuthServiceError>;
    async fn create_custom_token(&self, uid: &str) -> Result<String, AuthServiceError>;
    async fn get_user(&self, uid: &str) -> Result<FbUser, AuthServiceError>;
    async fn list_users(&self, page_token: Option<String>) -> Result<FbUserList, AuthServiceError>;
    async fn delete_user(&self, uid: &str) -> Result<(), AuthServiceError>;
}

#[async_trait]
pub trait EmailSvc: Send + Sync {
    async fn send_welcome_email(&self, email: &str, name: &str) -> Result<(), EmailServiceError>;
    async fn send_password_reset(&self, email: &str, reset_link: &str) -> Result<(), EmailServiceError>;
    async fn send_payment_confirmation(&self, email: &str, amount: Decimal, currency: &str) -> Result<(), EmailServiceError>;
    async fn send_role_upgrade_notification(&self, email: &str, new_role: &str) -> Result<(), EmailServiceError>;
}

#[async_trait]
pub trait PayGw: Send + Sync {
    async fn create_payment_address(&self, currency: &Currency, user_id: &UserId) -> Result<PaymentAddress, PaymentServiceError>;
    async fn verify_transaction(&self, tx_hash: &str, expected_amount: Decimal, currency: &Currency) -> Result<TransactionDetails, PaymentServiceError>;
    async fn get_exchange_rate(&self, from: &Currency, to: &Currency) -> Result<Decimal, PaymentServiceError>;
    async fn estimate_fees(&self, currency: &Currency, network: &str) -> Result<Decimal, PaymentServiceError>;
}

#[async_trait]
pub trait StockDataSvc: Send + Sync {
    async fn get_real_time_price(&self, symbol: &Symbol) -> Result<StockPrice, StockServiceError>;
    async fn get_historical_data(&self, symbol: &Symbol, period: &str) -> Result<Vec<StockPrice>, StockServiceError>;
    async fn get_market_status(&self, market: &str) -> Result<MarketStatus, StockServiceError>;
    async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolInfo>, StockServiceError>;
}

#[async_trait]
pub trait WebSocketSvc: Send + Sync {
    async fn broadcast_to_user(&self, user_id: &UserId, message: &str) -> Result<(), WebSocketError>;
    async fn broadcast_to_role(&self, role: &str, message: &str) -> Result<(), WebSocketError>;
    async fn broadcast_stock_update(&self, symbol: &Symbol, price: Decimal) -> Result<(), WebSocketError>;
    async fn get_connected_users(&self) -> Result<Vec<UserId>, WebSocketError>;
}

// Supporting types
#[derive(Debug, Clone)]
pub struct FbClaims {
    pub uid: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FbUser {
    pub uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_sign_in: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct FbUserList {
    pub users: Vec<FbUser>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaymentAddress {
    pub address: String,
    pub currency: Currency,
    pub network: String,
    pub qr_code_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransactionDetails {
    pub tx_hash: String,
    pub amount: Decimal,
    pub currency: Currency,
    pub confirmations: u32,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct StockPrice {
    pub symbol: Symbol,
    pub price: Decimal,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
    pub change: Option<Decimal>,
    pub change_percent: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct MarketStatus {
    pub market: String,
    pub is_open: bool,
    pub next_open: Option<DateTime<Utc>>,
    pub next_close: Option<DateTime<Utc>>,
    pub timezone: String,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub symbol: Symbol,
    pub name: String,
    pub market: String,
    pub sector: Option<String>,
    pub currency: String,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum AuthServiceError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    
    #[error("External service error: {0}")]
    ExternalError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EmailServiceError {
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),
    
    #[error("Email delivery failed: {0}")]
    DeliveryFailed(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("External service error: {0}")]
    ExternalError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentServiceError {
    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),
    
    #[error("Address generation failed: {0}")]
    AddressGenerationFailed(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Insufficient confirmations")]
    InsufficientConfirmations,
    
    #[error("Amount mismatch: expected {expected}, got {actual}")]
    AmountMismatch { expected: Decimal, actual: Decimal },
    
    #[error("External service error: {0}")]
    ExternalError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum StockServiceError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Market closed")]
    MarketClosed,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid period: {0}")]
    InvalidPeriod(String),
    
    #[error("External service error: {0}")]
    ExternalError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("User not connected: {0}")]
    UserNotConnected(String),
    
    #[error("Message too large")]
    MessageTooLarge,
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::Symbol;
    
    #[test]
    fn should_create_fb_claims() {
        let claims = FbClaims {
            uid: "test-uid".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            name: Some("Test User".to_string()),
            picture: None,
        };
        
        assert_eq!(claims.uid, "test-uid");
        assert_eq!(claims.email, "test@example.com");
        assert!(claims.email_verified);
    }
    
    #[test]
    fn should_create_payment_address() {
        let address = PaymentAddress {
            address: "0x1234567890abcdef".to_string(),
            currency: Currency::USDT,
            network: "Ethereum".to_string(),
            qr_code_url: Some("https://example.com/qr.png".to_string()),
        };
        
        assert_eq!(address.address, "0x1234567890abcdef");
        assert_eq!(address.currency, Currency::USDT);
    }
    
    #[test]
    fn should_create_stock_price() {
        let price = StockPrice {
            symbol: Symbol::new("AAPL").unwrap(),
            price: rust_decimal_macros::dec!(150.50),
            volume: 1000000,
            timestamp: Utc::now(),
            change: Some(rust_decimal_macros::dec!(2.50)),
            change_percent: Some(rust_decimal_macros::dec!(1.69)),
        };
        
        assert_eq!(price.price, rust_decimal_macros::dec!(150.50));
        assert_eq!(price.volume, 1000000);
    }
}