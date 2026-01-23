//! Payment Repository Adapter (Infrastructure Layer)
//! PostgreSQL implementation of PaymentRepositoryPort using Diesel

use crate::prelude::*;
use tracing::{info, error, debug, warn};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};

use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::domain::payment::{
    Payment, PaymentId, PaymentStatus, PaymentAmount, TransactionHash,
    PaymentReference, PaymentStats
};
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::domain::payment::repository_ports::PaymentRepositoryPort;

use crate::infrastructure::models::payment::{
    PaymentDb, NewPaymentDb,
};
use crate::schemas::payments::payments;

/// PostgreSQL implementation of PaymentRepositoryPort using Diesel
#[derive(Clone)]
pub struct PaymentRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl PaymentRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }

    /// Convert PaymentDb domain model to database model
    fn payment_to_domain(&self, payment_db: PaymentDb) -> Result<Payment, AppError> {
        // Convert payment amount
        // Convert BigDecimal to Decimal
        let amount_decimal = rust_decimal::Decimal::from_str(&payment_db.amount.to_string())
            .unwrap_or(rust_decimal::Decimal::ZERO);

        // Parse currency string to Currency enum
        let currency = match payment_db.currency.as_str() {
            "USD" => crate::domain::payment::value_objects::Currency::USD,
            "USDT" => crate::domain::payment::value_objects::Currency::USDT,
            "USDC" => crate::domain::payment::value_objects::Currency::USDC,
            "ETH" => crate::domain::payment::value_objects::Currency::ETH,
            "BTC" => crate::domain::payment::value_objects::Currency::BTC,
            "BNB" => crate::domain::payment::value_objects::Currency::BNB,
            "TRX" => crate::domain::payment::value_objects::Currency::TRX,
            _ => crate::domain::payment::value_objects::Currency::USD, // Default to USD
        };

        let amount = PaymentAmount::new(amount_decimal, currency)
            .map_err(|e| AppError::validation_error(format!("Invalid payment amount: {}", e)))?;

        // Create payment ID
        let payment_id = PaymentId::from_uuid(payment_db.id);

        // Create payment reference
        let payment_reference = PaymentReference::from_string(&payment_db.payment_reference)
            .map_err(|e| AppError::validation_error(format!("Invalid payment reference: {}", e)))?;

        // Create transaction hash if present
        let transaction_hash = payment_db.transaction_hash
            .map(|hash| TransactionHash::new(hash, crate::domain::payment::value_objects::Network::BinanceSmartChain))
            .transpose()
            .map_err(|e| AppError::validation_error(format!("Invalid transaction hash: {}", e)))?;

        // Parse payment status
        let status = match payment_db.status.as_str() {
            "created" => PaymentStatus::Created,
            "awaiting_payment" | "awaiting" | "pending" => PaymentStatus::AwaitingPayment,
            "pending_verification" => PaymentStatus::PendingVerification,
            "verifying" => PaymentStatus::Verifying,
            "verification_failed" => PaymentStatus::VerificationFailed,
            "confirmed" => PaymentStatus::Confirmed,
            "processing" => PaymentStatus::Processing,
            "completed" => PaymentStatus::Completed,
            "failed" => PaymentStatus::Failed,
            "cancelled" => PaymentStatus::Cancelled,
            "refunding" => PaymentStatus::Refunding,
            "refunded" => PaymentStatus::Refunded,
            _ => return Err(AppError::validation_error("Invalid payment status")),
        };

        // Create wallet address
        let wallet_address = WalletAddress::new(&payment_db.wallet_address)
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        // Create payment aggregate with nullable created_at handling
        let created_at = payment_db.created_at.unwrap_or_else(chrono::Utc::now);
        Payment::new(
            payment_id,
            payment_reference,
            wallet_address,
            amount,
            status,
            transaction_hash,
            payment_db.plan_id.to_string(),
            created_at,
            payment_db.metadata.clone().unwrap_or(serde_json::json!({})),
        )
        .map_err(|e| AppError::validation_error(format!("Failed to create payment aggregate: {}", e)))
    }

    /// Convert domain model to database model
    fn payment_to_db(&self, payment: &Payment) -> Result<NewPaymentDb, AppError> {
        let amount_str = payment.amount().amount().to_string();
        let amount = BigDecimal::from_str(&amount_str)
            .map_err(|e| AppError::validation_error(format!("Invalid amount: {}", e)))?;

        let status_str = match payment.status() {
            PaymentStatus::Created => "created",
            PaymentStatus::AwaitingPayment => "awaiting_payment",
            PaymentStatus::PendingVerification => "pending_verification",
            PaymentStatus::Verifying => "verifying",
            PaymentStatus::VerificationFailed => "verification_failed",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Completed => "completed",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Refunding => "refunding",
            PaymentStatus::Refunded => "refunded",
        };

        let plan_uuid = Uuid::parse_str(&payment.plan_id())
            .map_err(|e| AppError::validation_error(format!("Invalid plan ID: {}", e)))?;

        Ok(NewPaymentDb {
            payment_reference: payment.reference().value().to_string(),
            wallet_address: payment.wallet_address().as_str().to_string(),
            amount,
            currency: payment.amount().currency().to_string(),
            method: "crypto".to_string(), // Default method
            status: status_str.to_string(),
            plan_id: plan_uuid,
            contract_address: None, // Will be set when blockchain transaction is confirmed
            token_address: None,   // Will be set when blockchain transaction is confirmed
            block_number: None,     // Will be set when blockchain transaction is confirmed
            confirmations: Some(0), // Initial value
            expires_at: None,      // Will be set based on payment configuration
            metadata: serde_json::to_value(payment.metadata())
                .map_err(|e| AppError::validation_error(format!("Failed to serialize metadata: {}", e)))?,
        })
    }
}

#[async_trait]
impl PaymentRepositoryPort for PaymentRepositoryAdapter {
    async fn save(&self, payment: &Payment) -> Result<(), String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let payment_db = self.payment_to_db(payment)
            .map_err(|e| format!("Failed to convert payment to database model: {}", e))?;

        info!(
            "Saving payment {} with reference {} for wallet {}",
            payment.id().value(),
            payment.reference().value(),
            payment.wallet_address().as_str()
        );

        diesel::insert_into(payments::table)
            .values(&payment_db)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save payment {}: {}", payment.id().value(), e);
                format!("Failed to save payment: {}", e)
            })?;

        info!("Successfully saved payment {}", payment.id().value());
        Ok(())
    }

    async fn find_by_id(&self, payment_id: &PaymentId) -> Result<Option<Payment>, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        debug!("Finding payment by ID: {}", payment_id.value());

        let payment_db = payments::table
            .filter(payments::id.eq(payment_id.value()))
            .first::<PaymentDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find payment by ID {}: {}", payment_id.value(), e);
                format!("Failed to find payment: {}", e)
            })?;

        match payment_db {
            Some(row) => {
                let payment = self.payment_to_domain(row)
                    .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
                info!("Found payment: {}", payment_id.value());
                Ok(Some(payment))
            }
            None => {
                debug!("Payment not found: {}", payment_id.value());
                Ok(None)
            }
        }
    }

    async fn find_by_user(&self, wallet_address: &WalletAddress) -> Result<Vec<Payment>, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        debug!("Finding payments for wallet: {}", wallet_address.as_str());

        let payments_db = payments::table
            .filter(payments::wallet_address.eq(wallet_address.as_str()))
            .order(payments::created_at.desc())
            .load::<PaymentDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find payments for wallet {}: {}", wallet_address.as_str(), e);
                format!("Failed to find payments: {}", e)
            })?;

        let mut payments = Vec::new();
        for payment_db in payments_db {
            let payment = self.payment_to_domain(payment_db)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} payments for wallet {}", payments.len(), wallet_address.as_str());
        Ok(payments)
    }

    async fn find_by_status(&self, status: PaymentStatus) -> Result<Vec<Payment>, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let status_str = match status {
            PaymentStatus::Created => "created",
            PaymentStatus::AwaitingPayment => "awaiting_payment",
            PaymentStatus::PendingVerification => "pending_verification",
            PaymentStatus::Verifying => "verifying",
            PaymentStatus::VerificationFailed => "verification_failed",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Completed => "completed",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Refunding => "refunding",
            PaymentStatus::Refunded => "refunded",
        };

        debug!("Finding payments with status: {}", status_str);

        let payments_db = payments::table
            .filter(payments::status.eq(status_str))
            .order(payments::created_at.desc())
            .load::<PaymentDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find payments with status {}: {}", status_str, e);
                format!("Failed to find payments: {}", e)
            })?;

        let mut payments = Vec::new();
        for payment_db in payments_db {
            let payment = self.payment_to_domain(payment_db)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} payments with status {}", payments.len(), status_str);
        Ok(payments)
    }

    async fn find_by_reference(&self, reference: &PaymentReference) -> Result<Option<Payment>, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        debug!("Finding payment by reference: {}", reference.value());

        let payment_db = payments::table
            .filter(payments::payment_reference.eq(reference.value()))
            .first::<PaymentDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find payment by reference {}: {}", reference.value(), e);
                format!("Failed to find payment: {}", e)
            })?;

        match payment_db {
            Some(row) => {
                let payment = self.payment_to_domain(row)
                    .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
                info!("Found payment by reference: {}", reference.value());
                Ok(Some(payment))
            }
            None => {
                debug!("Payment not found by reference: {}", reference.value());
                Ok(None)
            }
        }
    }

    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Result<Vec<Payment>, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        debug!("Finding payments between {} and {}", start, end);

        let payments_db = payments::table
            .filter(payments::created_at.ge(start))
            .filter(payments::created_at.le(end))
            .order(payments::created_at.desc())
            .load::<PaymentDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find payments in date range: {}", e);
                format!("Failed to find payments: {}", e)
            })?;

        let mut payments = Vec::new();
        for payment_db in payments_db {
            let payment = self.payment_to_domain(payment_db)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} payments in date range", payments.len());
        Ok(payments)
    }

    async fn find_expired_pending(&self, threshold: DateTime<Utc>) -> Result<Vec<Payment>, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        debug!("Finding expired pending payments older than {}", threshold);

        let payments_db = payments::table
            .filter(payments::status.eq("pending").or(payments::status.eq("awaiting_payment")))
            .filter(payments::created_at.lt(threshold))
            .filter(payments::expires_at.lt(threshold))
            .order(payments::created_at.asc())
            .load::<PaymentDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find expired pending payments: {}", e);
                format!("Failed to find payments: {}", e)
            })?;

        let mut payments = Vec::new();
        for payment_db in payments_db {
            let payment = self.payment_to_domain(payment_db)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} expired pending payments", payments.len());
        Ok(payments)
    }

    async fn update_status(&self, payment_id: &PaymentId, status: PaymentStatus) -> Result<(), String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let status_str = match status {
            PaymentStatus::Created => "created",
            PaymentStatus::AwaitingPayment => "awaiting_payment",
            PaymentStatus::PendingVerification => "pending_verification",
            PaymentStatus::Verifying => "verifying",
            PaymentStatus::VerificationFailed => "verification_failed",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Completed => "completed",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Refunding => "refunding",
            PaymentStatus::Refunded => "refunded",
        };

        info!("Updating payment {} status to {}", payment_id.value(), status_str);

        let completed_at = match status {
            PaymentStatus::Completed => Some(Utc::now()),
            PaymentStatus::Failed => Some(Utc::now()),
            PaymentStatus::Refunded => Some(Utc::now()),
            _ => None,
        };

        diesel::update(payments::table.filter(payments::id.eq(payment_id.value())))
            .set((
                payments::status.eq(status_str.to_string()),
                payments::updated_at.eq(Utc::now()),
                payments::completed_at.eq(completed_at),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update payment status: {}", e);
                format!("Failed to update payment status: {}", e)
            })?;

        info!("Successfully updated payment {} status to {}", payment_id.value(), status_str);
        Ok(())
    }

    async fn delete(&self, payment_id: &PaymentId) -> Result<(), String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        warn!("Deleting payment {} - this should only be used for testing/debugging", payment_id.value());

        diesel::delete(payments::table.filter(payments::id.eq(payment_id.value())))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete payment: {}", e);
                format!("Failed to delete payment: {}", e)
            })?;

        info!("Successfully deleted payment {}", payment_id.value());
        Ok(())
    }

    async fn get_user_payment_stats(&self, wallet_address: &WalletAddress) -> Result<PaymentStats, String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        debug!("Getting payment stats for wallet: {}", wallet_address.as_str());

        // Query for payment statistics
        let stats_query = r#"
            SELECT
                COUNT(*) as total_payments,
                COUNT(CASE WHEN status IN ('completed', 'confirmed') THEN 1 END) as completed_payments,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_payments,
                COALESCE(SUM(CASE WHEN status IN ('completed', 'confirmed') THEN amount END), 0) as total_amount,
                COALESCE(AVG(amount), 0) as average_amount,
                MAX(created_at) as last_payment_date
            FROM payments
            WHERE wallet_address = $1
        "#;

        #[derive(diesel::QueryableByName)]
        struct StatsRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_payments: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            completed_payments: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            failed_payments: i64,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            total_amount: BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            average_amount: BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            last_payment_date: Option<DateTime<Utc>>,
        }

        let stats_row = diesel::sql_query(stats_query)
            .bind::<diesel::sql_types::Text, _>(wallet_address.as_str())
            .load::<StatsRow>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get payment stats: {}", e);
                format!("Failed to get payment stats: {}", e)
            })?
            .into_iter()
            .next()
            .ok_or_else(|| "No payment stats found".to_string())?;

        // Convert BigDecimal to Decimal
        let total_amount_decimal = rust_decimal::Decimal::from_str(&stats_row.total_amount.to_string())
            .unwrap_or(rust_decimal::Decimal::ZERO);
        let average_amount_decimal = rust_decimal::Decimal::from_str(&stats_row.average_amount.to_string())
            .unwrap_or(rust_decimal::Decimal::ZERO);

        let total_amount = PaymentAmount::new(total_amount_decimal, crate::domain::payment::value_objects::Currency::USD)
            .map_err(|e| format!("Failed to create total amount: {}", e))?;

        let average_amount = PaymentAmount::new(average_amount_decimal, crate::domain::payment::value_objects::Currency::USD)
            .map_err(|e| format!("Failed to create average amount: {}", e))?;

        let stats = PaymentStats {
            total_payments: stats_row.total_payments as u32,
            completed_payments: stats_row.completed_payments as u32,
            failed_payments: stats_row.failed_payments as u32,
            total_amount,
            average_amount,
            last_payment_date: stats_row.last_payment_date,
        };

        info!("Retrieved payment stats for wallet {}: {} total, {} completed",
              wallet_address.as_str(), stats.total_payments, stats.completed_payments);

        Ok(stats)
    }
}