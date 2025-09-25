use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::payment::value_objects::CryptoNetwork;

/// Security service for payment verification
#[async_trait]
pub trait PaymentSecurityService: Send + Sync {
    /// Check if transaction hash has already been processed (replay protection)
    async fn check_transaction_replay(&self, tx_hash: &str) -> Result<(), SecurityError>;
    
    /// Record processed transaction to prevent replay attacks
    async fn record_processed_transaction(
        &self,
        tx_hash: &str,
        user_id: &UserId,
        amount: Decimal,
        network: &CryptoNetwork,
    ) -> Result<(), SecurityError>;
    
    /// Check rate limiting for user payment attempts
    async fn check_rate_limit(&self, user_id: &UserId) -> Result<(), SecurityError>;
    
    /// Record payment attempt for rate limiting
    async fn record_payment_attempt(&self, user_id: &UserId, success: bool) -> Result<(), SecurityError>;
    
    /// Detect potential fraud based on payment patterns
    async fn detect_fraud(
        &self,
        user_id: &UserId,
        amount: Decimal,
        network: &CryptoNetwork,
        user_wallet: Option<&str>,
    ) -> Result<FraudAnalysis, SecurityError>;
    
    /// Clean up old security records
    async fn cleanup_old_records(&self) -> Result<(), SecurityError>;
}

#[derive(Debug, Clone)]
pub struct PaymentSecurityConfig {
    /// Maximum payment attempts per user per hour
    pub max_attempts_per_hour: u32,
    /// Maximum successful payments per user per day
    pub max_payments_per_day: u32,
    /// Maximum amount per transaction for new users
    pub max_amount_new_user: Decimal,
    /// Maximum total amount per user per day
    pub max_daily_amount: Decimal,
    /// Minimum time between payment attempts (seconds)
    pub min_attempt_interval: u64,
    /// Days to keep security records
    pub cleanup_days: i64,
    /// Enable fraud detection
    pub fraud_detection_enabled: bool,
}

impl Default for PaymentSecurityConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_hour: std::env::var("MAX_PAYMENT_ATTEMPTS_PER_HOUR")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            max_payments_per_day: std::env::var("MAX_PAYMENTS_PER_DAY")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_amount_new_user: std::env::var("MAX_AMOUNT_NEW_USER")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(Decimal::from(1000)),
            max_daily_amount: std::env::var("MAX_DAILY_AMOUNT")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(Decimal::from(10000)),
            min_attempt_interval: std::env::var("MIN_PAYMENT_ATTEMPT_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            cleanup_days: std::env::var("SECURITY_CLEANUP_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            fraud_detection_enabled: std::env::var("FRAUD_DETECTION_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudAnalysis {
    pub risk_score: u8, // 0-100, where 100 is highest risk
    pub flags: Vec<FraudFlag>,
    pub allow_payment: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FraudFlag {
    /// High transaction amount for new user
    HighAmountNewUser,
    /// Rapid successive payment attempts
    RapidAttempts,
    /// Unusual payment pattern
    UnusualPattern,
    /// Multiple failed attempts recently
    MultipleFailures,
    /// Amount significantly higher than user's history
    AmountOutlier,
    /// Different network than usual
    NetworkChange,
    /// Payment from different wallet
    WalletChange,
    /// Payment amount is exact round number (potential bot)
    RoundAmount,
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Transaction has already been processed: {tx_hash}")]
    TransactionAlreadyProcessed { tx_hash: String },
    
    #[error("Rate limit exceeded: {limit_type}")]
    RateLimitExceeded { limit_type: String },
    
    #[error("Fraud detected: {reason}")]
    FraudDetected { reason: String },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

pub struct SqlxPaymentSecurityService {
    db_pool: Arc<PgPool>,
    config: PaymentSecurityConfig,
    // In-memory rate limiting cache
    rate_limit_cache: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl SqlxPaymentSecurityService {
    pub fn new(db_pool: Arc<PgPool>, config: PaymentSecurityConfig) -> Self {
        Self {
            db_pool,
            config,
            rate_limit_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if user is new (created less than 7 days ago)
    async fn is_new_user(&self, user_id: &UserId) -> Result<bool, SecurityError> {
        let user_uuid = uuid::Uuid::parse_str(&user_id.to_string())
            .map_err(|_| SecurityError::ConfigError { 
                message: "Invalid user ID format".to_string() 
            })?;
        
        let result = sqlx::query(
            "SELECT created_at FROM users WHERE id = $1"
        )
        .bind(user_uuid)
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?;
        
        match result {
            Some(row) => {
                let created_at: DateTime<Utc> = row.try_get("created_at")
                    .map_err(|e| SecurityError::DatabaseError { 
                        message: e.to_string() 
                    })?;
                Ok(Utc::now() - created_at < Duration::days(7))
            }
            None => Ok(true), // If user not found, treat as new for safety
        }
    }
    
    /// Get user's payment history for fraud analysis
    async fn get_user_payment_history(
        &self,
        user_id: &UserId,
        days: i64,
    ) -> Result<Vec<PaymentHistoryRecord>, SecurityError> {
        let user_uuid = uuid::Uuid::parse_str(&user_id.to_string())
            .map_err(|_| SecurityError::ConfigError { 
                message: "Invalid user ID format".to_string() 
            })?;
        
        let cutoff_date = Utc::now() - Duration::days(days);
        
        let rows = sqlx::query(
            "SELECT amount, network, success, attempted_at, wallet_address 
             FROM payment_attempts 
             WHERE user_id = $1 AND attempted_at > $2 
             ORDER BY attempted_at DESC"
        )
        .bind(user_uuid)
        .bind(cutoff_date)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?;
        
        let mut history = Vec::new();
        for row in rows {
            // Convert BigDecimal from database to Decimal for business logic
            let amount_big: BigDecimal = row.try_get("amount").unwrap_or_default();
            let amount = Decimal::from_str(&amount_big.to_string()).unwrap_or_default();
            
            history.push(PaymentHistoryRecord {
                amount,
                network: row.try_get::<String, _>("network").unwrap_or_default(),
                success: row.try_get::<bool, _>("success").unwrap_or(false),
                attempted_at: row.try_get::<DateTime<Utc>, _>("attempted_at").unwrap_or_else(|_| Utc::now()),
                wallet_address: row.try_get::<Option<String>, _>("wallet_address").unwrap_or(None),
            });
        }
        
        Ok(history)
    }
    
    /// Analyze amount for potential fraud
    fn analyze_amount(&self, amount: Decimal, history: &[PaymentHistoryRecord]) -> Vec<FraudFlag> {
        let mut flags = Vec::new();
        
        // Check for round amounts (potential bot behavior)
        if amount.fract() == Decimal::ZERO && amount % Decimal::from(100) == Decimal::ZERO {
            flags.push(FraudFlag::RoundAmount);
        }
        
        // Check if amount is outlier compared to history
        if !history.is_empty() {
            let successful_payments: Vec<&PaymentHistoryRecord> = history
                .iter()
                .filter(|p| p.success)
                .collect();
            
            if !successful_payments.is_empty() {
                let avg_amount: Decimal = successful_payments
                    .iter()
                    .map(|p| p.amount)
                    .sum::<Decimal>() / Decimal::from(successful_payments.len());
                
                // If amount is 5x higher than average, flag as outlier
                if amount > avg_amount * Decimal::from(5) {
                    flags.push(FraudFlag::AmountOutlier);
                }
            }
        }
        
        flags
    }
    
    /// Analyze timing patterns for fraud
    fn analyze_timing(&self, history: &[PaymentHistoryRecord]) -> Vec<FraudFlag> {
        let mut flags = Vec::new();
        
        // Check for rapid attempts in last hour
        let hour_ago = Utc::now() - Duration::hours(1);
        let recent_attempts = history
            .iter()
            .filter(|p| p.attempted_at > hour_ago)
            .count();
        
        if recent_attempts > 3 {
            flags.push(FraudFlag::RapidAttempts);
        }
        
        // Check for multiple failures recently
        let recent_failures = history
            .iter()
            .filter(|p| p.attempted_at > hour_ago && !p.success)
            .count();
        
        if recent_failures > 2 {
            flags.push(FraudFlag::MultipleFailures);
        }
        
        flags
    }
    
    /// Analyze network and wallet changes
    fn analyze_patterns(
        &self,
        network: &CryptoNetwork,
        user_wallet: Option<&str>,
        history: &[PaymentHistoryRecord],
    ) -> Vec<FraudFlag> {
        let mut flags = Vec::new();
        
        if !history.is_empty() {
            // Check for network changes
            let network_str = format!("{:?}", network);
            let same_network_count = history
                .iter()
                .filter(|p| p.network == network_str)
                .count();
            
            // If less than 50% of payments used this network, flag it
            if same_network_count < history.len() / 2 {
                flags.push(FraudFlag::NetworkChange);
            }
            
            // Check for wallet changes
            if let Some(wallet) = user_wallet {
                let same_wallet_count = history
                    .iter()
                    .filter(|p| {
                        if let Some(ref prev_wallet) = p.wallet_address {
                            prev_wallet.to_lowercase() == wallet.to_lowercase()
                        } else {
                            false
                        }
                    })
                    .count();
                
                // If less than 70% of payments used this wallet, flag it
                if same_wallet_count < (history.len() * 7) / 10 {
                    flags.push(FraudFlag::WalletChange);
                }
            }
        }
        
        flags
    }
    
    /// Calculate risk score based on flags and context
    fn calculate_risk_score(
        &self,
        flags: &[FraudFlag],
        is_new_user: bool,
        amount: Decimal,
    ) -> u8 {
        let mut score = 0u8;
        
        // Base score for new users
        if is_new_user {
            score += 20;
        }
        
        // High amounts increase risk
        if amount > self.config.max_amount_new_user {
            score += 25;
        }
        
        // Add points for each flag
        for flag in flags {
            let points = match flag {
                FraudFlag::HighAmountNewUser => 30,
                FraudFlag::RapidAttempts => 25,
                FraudFlag::MultipleFailures => 20,
                FraudFlag::AmountOutlier => 15,
                FraudFlag::UnusualPattern => 15,
                FraudFlag::NetworkChange => 10,
                FraudFlag::WalletChange => 10,
                FraudFlag::RoundAmount => 5,
            };
            score = score.saturating_add(points);
        }
        
        score.min(100)
    }
}

#[derive(Debug, Clone)]
struct PaymentHistoryRecord {
    amount: Decimal,
    network: String,
    success: bool,
    attempted_at: DateTime<Utc>,
    wallet_address: Option<String>,
}

#[async_trait]
impl PaymentSecurityService for SqlxPaymentSecurityService {
    async fn check_transaction_replay(&self, tx_hash: &str) -> Result<(), SecurityError> {
        debug!("Checking transaction replay for: {}", tx_hash);
        
        let result = sqlx::query(
            "SELECT id FROM processed_transactions WHERE transaction_hash = $1"
        )
        .bind(tx_hash)
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?;
        
        if result.is_some() {
            warn!("Replay attack detected for transaction: {}", tx_hash);
            return Err(SecurityError::TransactionAlreadyProcessed { 
                tx_hash: tx_hash.to_string() 
            });
        }
        
        Ok(())
    }
    
    async fn record_processed_transaction(
        &self,
        tx_hash: &str,
        user_id: &UserId,
        amount: Decimal,
        network: &CryptoNetwork,
    ) -> Result<(), SecurityError> {
        let user_uuid = uuid::Uuid::parse_str(&user_id.to_string())
            .map_err(|_| SecurityError::ConfigError { 
                message: "Invalid user ID format".to_string() 
            })?;
        
        // Convert Decimal to BigDecimal for SQLx compatibility
        let amount_big = BigDecimal::from_str(&amount.to_string())
            .map_err(|_| SecurityError::ConfigError { 
                message: "Invalid amount format".to_string() 
            })?;
        
        sqlx::query(
            "INSERT INTO processed_transactions (transaction_hash, user_id, amount, network, processed_at) 
             VALUES ($1, $2, $3, $4, NOW())"
        )
        .bind(tx_hash)
        .bind(user_uuid)
        .bind(amount_big)
        .bind(format!("{:?}", network))
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?;
        
        info!("Recorded processed transaction: {} for user: {}", tx_hash, user_id);
        Ok(())
    }
    
    async fn check_rate_limit(&self, user_id: &UserId) -> Result<(), SecurityError> {
        let user_key = user_id.to_string();
        let now = Utc::now();
        let hour_ago = now - Duration::hours(1);
        
        // Check in-memory cache first (faster)
        {
            let mut cache = self.rate_limit_cache.write().await;
            let attempts = cache.entry(user_key.clone()).or_insert_with(Vec::new);
            
            // Remove old attempts
            attempts.retain(|&attempt_time| attempt_time > hour_ago);
            
            // Check hourly limit
            if attempts.len() >= self.config.max_attempts_per_hour as usize {
                warn!("Rate limit exceeded for user: {} (hourly)", user_id);
                return Err(SecurityError::RateLimitExceeded { 
                    limit_type: "hourly attempts".to_string() 
                });
            }
            
            // Check minimum interval
            if let Some(&last_attempt) = attempts.last() {
                let time_since_last = now - last_attempt;
                if time_since_last < Duration::seconds(self.config.min_attempt_interval as i64) {
                    warn!("Rate limit exceeded for user: {} (too frequent)", user_id);
                    return Err(SecurityError::RateLimitExceeded { 
                        limit_type: "minimum interval".to_string() 
                    });
                }
            }
        }
        
        // Check daily limits from database
        let user_uuid = uuid::Uuid::parse_str(&user_id.to_string())
            .map_err(|_| SecurityError::ConfigError { 
                message: "Invalid user ID format".to_string() 
            })?;
        
        let day_ago = now - Duration::days(1);
        
        // Check daily payment count
        let daily_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM payment_attempts 
             WHERE user_id = $1 AND attempted_at > $2 AND success = true"
        )
        .bind(user_uuid)
        .bind(day_ago)
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?;
        
        if daily_count >= self.config.max_payments_per_day as i64 {
            warn!("Daily payment limit exceeded for user: {}", user_id);
            return Err(SecurityError::RateLimitExceeded { 
                limit_type: "daily payments".to_string() 
            });
        }
        
        // Check daily amount limit
        let daily_amount: Option<BigDecimal> = sqlx::query_scalar(
            "SELECT SUM(amount) FROM payment_attempts 
             WHERE user_id = $1 AND attempted_at > $2 AND success = true"
        )
        .bind(user_uuid)
        .bind(day_ago)
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?;
        
        if let Some(total_amount_big) = daily_amount {
            // Convert BigDecimal to Decimal for comparison
            let total_amount = Decimal::from_str(&total_amount_big.to_string())
                .unwrap_or_default();
            
            if total_amount >= self.config.max_daily_amount {
                warn!("Daily amount limit exceeded for user: {}", user_id);
                return Err(SecurityError::RateLimitExceeded { 
                    limit_type: "daily amount".to_string() 
                });
            }
        }
        
        debug!("Rate limit check passed for user: {}", user_id);
        Ok(())
    }
    
    async fn record_payment_attempt(&self, user_id: &UserId, success: bool) -> Result<(), SecurityError> {
        let user_key = user_id.to_string();
        
        // Update in-memory cache
        {
            let mut cache = self.rate_limit_cache.write().await;
            let attempts = cache.entry(user_key).or_insert_with(Vec::new);
            attempts.push(Utc::now());
        }
        
        debug!("Recorded payment attempt for user: {}, success: {}", user_id, success);
        Ok(())
    }
    
    async fn detect_fraud(
        &self,
        user_id: &UserId,
        amount: Decimal,
        network: &CryptoNetwork,
        user_wallet: Option<&str>,
    ) -> Result<FraudAnalysis, SecurityError> {
        if !self.config.fraud_detection_enabled {
            return Ok(FraudAnalysis {
                risk_score: 0,
                flags: Vec::new(),
                allow_payment: true,
                reason: None,
            });
        }
        
        debug!("Running fraud detection for user: {}, amount: {}", user_id, amount);
        
        let is_new_user = self.is_new_user(user_id).await?;
        let history = self.get_user_payment_history(user_id, 30).await?;
        
        let mut flags = Vec::new();
        
        // Check if new user with high amount
        if is_new_user && amount > self.config.max_amount_new_user {
            flags.push(FraudFlag::HighAmountNewUser);
        }
        
        // Analyze amount patterns
        flags.extend(self.analyze_amount(amount, &history));
        
        // Analyze timing patterns
        flags.extend(self.analyze_timing(&history));
        
        // Analyze network and wallet patterns
        flags.extend(self.analyze_patterns(network, user_wallet, &history));
        
        let risk_score = self.calculate_risk_score(&flags, is_new_user, amount);
        
        // Determine if payment should be allowed
        let (allow_payment, reason) = if risk_score >= 80 {
            (false, Some("High fraud risk detected".to_string()))
        } else if risk_score >= 60 {
            (true, Some("Medium fraud risk - payment allowed with monitoring".to_string()))
        } else {
            (true, None)
        };
        
        let analysis = FraudAnalysis {
            risk_score,
            flags,
            allow_payment,
            reason,
        };
        
        if risk_score >= 60 {
            warn!("Fraud analysis for user {}: risk_score={}, flags={:?}", 
                  user_id, risk_score, analysis.flags);
        } else {
            debug!("Fraud analysis for user {}: risk_score={}", user_id, risk_score);
        }
        
        Ok(analysis)
    }
    
    async fn cleanup_old_records(&self) -> Result<(), SecurityError> {
        let cutoff_date = Utc::now() - Duration::days(self.config.cleanup_days);
        
        // Clean up processed transactions
        let deleted_transactions = sqlx::query(
            "DELETE FROM processed_transactions WHERE processed_at < $1"
        )
        .bind(cutoff_date)
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?
        .rows_affected();
        
        // Clean up old payment attempts
        let deleted_attempts = sqlx::query(
            "DELETE FROM payment_attempts WHERE attempted_at < $1"
        )
        .bind(cutoff_date)
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| SecurityError::DatabaseError { 
            message: e.to_string() 
        })?
        .rows_affected();
        
        // Clean up in-memory cache
        {
            let mut cache = self.rate_limit_cache.write().await;
            cache.clear();
        }
        
        info!(
            "Cleaned up {} processed transactions and {} payment attempts older than {} days",
            deleted_transactions, deleted_attempts, self.config.cleanup_days
        );
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_security_config_from_env() {
        std::env::set_var("MAX_PAYMENT_ATTEMPTS_PER_HOUR", "15");
        std::env::set_var("FRAUD_DETECTION_ENABLED", "false");
        
        let config = PaymentSecurityConfig::default();
        assert_eq!(config.max_attempts_per_hour, 15);
        assert!(!config.fraud_detection_enabled);
        
        std::env::remove_var("MAX_PAYMENT_ATTEMPTS_PER_HOUR");
        std::env::remove_var("FRAUD_DETECTION_ENABLED");
    }
    
    #[test]
    fn test_fraud_flag_analysis() {
        // This would test the fraud detection logic with mock data
        // Implementation depends on your testing strategy
    }
}