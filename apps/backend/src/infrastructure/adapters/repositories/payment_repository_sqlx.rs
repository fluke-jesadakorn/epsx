/// Payment Repository Adapter using SQLx
/// PostgreSQL implementation using SQLx to avoid Diesel field count limitations

use crate::prelude::*;
use tracing::{info, error, debug, warn};
use sqlx::{postgres::PgRow, PgPool, Row, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::domain::payment::{
    Payment, PaymentId, PaymentStatus, PaymentAmount, TransactionHash,
    CryptoAddress, PaymentReference, PaymentStats, TransactionRecord
};
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::domain::payment::repository_ports::PaymentRepositoryPort;

/// SQLx-based Payment Repository implementation
#[derive(Clone)]
pub struct PaymentRepositorySqlx {
    db_pool: &'static PgPool,
}

impl PaymentRepositorySqlx {
    pub fn new(db_pool: &'static PgPool) -> Self {
        Self { db_pool }
    }

    /// Convert SQLx row to Payment domain model
    fn row_to_payment(&self, row: &PgRow) -> Result<Payment, AppError> {
        // Convert payment amount
        let amount_decimal: BigDecimal = row.try_get("amount")
            .unwrap_or(BigDecimal::from(0));
        let amount_decimal = rust_decimal::Decimal::from_str(&amount_decimal.to_string())
            .unwrap_or(rust_decimal::Decimal::ZERO);

        // Parse currency string to Currency enum
        let currency_str: String = row.try_get("currency").unwrap_or_default();
        let currency = match currency_str.as_str() {
            "USD" => crate::domain::payment::value_objects::Currency::USD,
            "USDT" => crate::domain::payment::value_objects::Currency::USDT,
            "USDC" => crate::domain::payment::value_objects::Currency::USDC,
            "ETH" => crate::domain::payment::value_objects::Currency::ETH,
            "BTC" => crate::domain::payment::value_objects::Currency::BTC,
            "BNB" => crate::domain::payment::value_objects::Currency::BNB,
            "TRX" => crate::domain::payment::value_objects::Currency::TRX,
            _ => crate::domain::payment::value_objects::Currency::USD,
        };

        let amount = PaymentAmount::new(amount_decimal, currency)
            .map_err(|e| AppError::validation_error(format!("Invalid payment amount: {}", e)))?;

        // Create payment ID
        let payment_id: Uuid = row.try_get("id").unwrap_or_default();
        let payment_id = PaymentId::from_uuid(payment_id);

        // Create payment reference
        let payment_reference: String = row.try_get("payment_reference").unwrap_or_default();
        let payment_reference = PaymentReference::from_string(&payment_reference)
            .map_err(|e| AppError::validation_error(format!("Invalid payment reference: {}", e)))?;

        // Create transaction hash if present
        let transaction_hash: Option<String> = row.try_get("transaction_hash").ok();
        let transaction_hash = transaction_hash
            .map(|hash| TransactionHash::new(hash, crate::domain::payment::value_objects::Network::BinanceSmartChain))
            .transpose()
            .map_err(|e| AppError::validation_error(format!("Invalid transaction hash: {}", e)))?;

        // Parse payment status
        let status_str: String = row.try_get("status").unwrap_or_default();
        let status = match status_str.as_str() {
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
        let wallet_address: String = row.try_get("wallet_address").unwrap_or_default();
        let wallet_address = WalletAddress::new(&wallet_address)
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        // Handle nullable created_at
        let created_at: Option<DateTime<Utc>> = row.try_get("created_at").ok();
        let created_at = created_at.unwrap_or_else(|| chrono::Utc::now());

        // Get metadata
        let metadata: serde_json::Value = row.try_get("metadata").unwrap_or(serde_json::Value::Null);

        // Create payment aggregate
        Payment::new(
            payment_id,
            payment_reference,
            wallet_address,
            amount,
            status,
            transaction_hash,
            row.try_get::<Uuid, _>("plan_id").unwrap_or_default().to_string(),
            created_at,
            metadata,
        )
        .map_err(|e| AppError::validation_error(format!("Failed to create payment aggregate: {}", e)))
    }
}

#[async_trait]
impl PaymentRepositoryPort for PaymentRepositorySqlx {
    async fn save(&self, payment: &Payment) -> Result<(), String> {
        let amount_str = payment.amount().amount().to_string();
        let amount = BigDecimal::from_str(&amount_str)
            .map_err(|e| format!("Invalid amount: {}", e))?;

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

        let currency_str = match payment.amount().currency() {
            crate::domain::payment::value_objects::Currency::USD => "USD",
            crate::domain::payment::value_objects::Currency::USDT => "USDT",
            crate::domain::payment::value_objects::Currency::USDC => "USDC",
            crate::domain::payment::value_objects::Currency::ETH => "ETH",
            crate::domain::payment::value_objects::Currency::BTC => "BTC",
            crate::domain::payment::value_objects::Currency::BNB => "BNB",
            crate::domain::payment::value_objects::Currency::TRX => "TRX",
        };

        let plan_uuid = Uuid::from_str(&payment.plan_id())
            .map_err(|e| format!("Invalid plan ID: {}", e))?;

        let metadata = serde_json::to_value(payment.metadata())
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO payments (
                payment_reference, wallet_address, amount, currency, method, status,
                plan_id, contract_address, token_address, block_number, confirmations,
                created_at, updated_at, expires_at, completed_at, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW(), NULL, NULL, $12
            )
            ON CONFLICT (id) DO UPDATE SET
                status = EXCLUDED.status,
                updated_at = NOW(),
                completed_at = EXCLUDED.completed_at,
                metadata = EXCLUDED.metadata
            "#,
            payment.payment_reference().value(),
            payment.wallet_address().as_str(),
            amount,
            currency_str,
            "crypto", // Default method
            status_str,
            plan_uuid,
            None::<String>, // contract_address
            None::<String>, // token_address
            None::<i64>,    // block_number
            Some(0i32),     // confirmations
            metadata
        )
        .execute(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| format!("Failed to save payment: {}", e))?;

        info!("Saved payment: {}", payment.payment_reference().value());
        Ok(())
    }

    async fn find_by_id(&self, payment_id: &PaymentId) -> Result<Option<Payment>, String> {
        let row = sqlx::query!(
            "SELECT * FROM payments WHERE id = $1",
            payment_id.value()
        )
        .fetch_optional(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| {
            error!("Failed to find payment by ID {}: {}", payment_id.value(), e);
            format!("Failed to find payment: {}", e)
        })?;

        match row {
            Some(record) => {
                // Convert the query result to a domain model
                // This is a simplified version - you'll need to implement proper conversion
                let payment = self.convert_sqlx_to_payment(record)?;
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
        let rows = sqlx::query!(
            "SELECT * FROM payments WHERE wallet_address = $1 ORDER BY created_at DESC",
            wallet_address.as_str()
        )
        .fetch_all(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| {
            error!("Failed to find payments for wallet {}: {}", wallet_address.as_str(), e);
            format!("Failed to find payments: {}", e)
        })?;

        let mut payments = Vec::new();
        for row in rows {
            let payment = self.convert_sqlx_to_payment(row)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} payments for wallet {}", payments.len(), wallet_address.as_str());
        Ok(payments)
    }

    async fn find_by_status(&self, status: PaymentStatus) -> Result<Vec<Payment>, String> {
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

        let rows = sqlx::query!(
            "SELECT * FROM payments WHERE status = $1 ORDER BY created_at DESC",
            status_str
        )
        .fetch_all(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| {
            error!("Failed to find payments with status {}: {}", status_str, e);
            format!("Failed to find payments: {}", e)
        })?;

        let mut payments = Vec::new();
        for row in rows {
            let payment = self.convert_sqlx_to_payment(row)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} payments with status {}", payments.len(), status_str);
        Ok(payments)
    }

    async fn find_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Payment>, String> {
        let rows = sqlx::query!(
            "SELECT * FROM payments WHERE created_at >= $1 AND created_at <= $2 ORDER BY created_at DESC",
            start,
            end
        )
        .fetch_all(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| {
            error!("Failed to find payments in date range: {}", e);
            format!("Failed to find payments: {}", e)
        })?;

        let mut payments = Vec::new();
        for row in rows {
            let payment = self.convert_sqlx_to_payment(row)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} payments in date range {} to {}", payments.len(), start, end);
        Ok(payments)
    }

    async fn find_expired_pending(&self, threshold: DateTime<Utc>) -> Result<Vec<Payment>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT * FROM payments
            WHERE (status = 'pending' OR status = 'awaiting_payment')
            AND created_at < $1
            AND expires_at < $1
            ORDER BY created_at ASC
            "#,
            threshold
        )
        .fetch_all(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| {
            error!("Failed to find expired pending payments: {}", e);
            format!("Failed to find payments: {}", e)
        })?;

        let mut payments = Vec::new();
        for row in rows {
            let payment = self.convert_sqlx_to_payment(row)
                .map_err(|e| format!("Failed to convert payment to domain model: {}", e))?;
            payments.push(payment);
        }

        info!("Found {} expired pending payments", payments.len());
        Ok(payments)
    }

    async fn get_payment_stats(&self, wallet_address: Option<&WalletAddress>) -> Result<PaymentStats, String> {
        let (count_filter, wallet_param) = if let Some(addr) = wallet_address {
            (format!("WHERE wallet_address = '{}'", addr.as_str()), Some(addr.as_str()))
        } else {
            (String::new(), None)
        };

        let query = format!(
            r#"
            SELECT
                COUNT(*) as total_payments,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_payments,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_payments,
                COUNT(*) FILTER (WHERE status = 'pending') as pending_payments,
                COALESCE(SUM(amount), 0) as total_amount,
                COALESCE(SUM(amount) FILTER (WHERE status = 'completed'), 0) as completed_amount
            FROM payments {}
            "#,
            count_filter
        );

        // Execute raw query since sqlx::query! macro doesn't support dynamic queries
        let row = sqlx::query(&query)
            .fetch_one(self.db_pool as &sqlx::PgPool)
            .await
            .map_err(|e| format!("Failed to get payment stats: {}", e))?;

        let total_payments: i64 = row.try_get("total_payments").unwrap_or(0);
        let completed_payments: i64 = row.try_get("completed_payments").unwrap_or(0);
        let failed_payments: i64 = row.try_get("failed_payments").unwrap_or(0);
        let pending_payments: i64 = row.try_get("pending_payments").unwrap_or(0);

        let total_amount: Option<BigDecimal> = row.try_get("total_amount").ok();
        let completed_amount: Option<BigDecimal> = row.try_get("completed_amount").ok();

        let stats = PaymentStats::new(
            total_payments as u32,
            completed_payments as u32,
            failed_payments as u32,
            pending_payments as u32,
            rust_decimal::Decimal::from_str(&total_amount.unwrap_or_default().to_string()).unwrap_or_default(),
            rust_decimal::Decimal::from_str(&completed_amount.unwrap_or_default().to_string()).unwrap_or_default(),
        ).map_err(|e| format!("Failed to create payment stats: {}", e))?;

        if let Some(addr) = wallet_param {
            info!("Retrieved payment stats for wallet: {}", addr);
        } else {
            info!("Retrieved global payment stats");
        }

        Ok(stats)
    }

    async fn update_transaction(&self, payment_id: &PaymentId, transaction_hash: &TransactionHash, confirmations: i32) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE payments
            SET
                transaction_hash = $1,
                confirmations = $2,
                status = CASE
                    WHEN $2 >= 12 THEN 'confirmed'
                    ELSE 'pending_verification'
                END,
                updated_at = NOW()
            WHERE id = $3
            "#,
            transaction_hash.value(),
            confirmations as i32,
            payment_id.value()
        )
        .execute(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| format!("Failed to update transaction: {}", e))?;

        info!("Updated transaction for payment {}: {} confirmations", payment_id.value(), confirmations);
        Ok(())
    }

    async fn update_status(&self, payment_id: &PaymentId, status: PaymentStatus) -> Result<(), String> {
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

        sqlx::query!(
            r#"
            UPDATE payments
            SET
                status = $1,
                updated_at = NOW(),
                completed_at = CASE
                    WHEN $1 IN ('completed', 'confirmed') THEN NOW()
                    ELSE completed_at
                END
            WHERE id = $2
            "#,
            status_str,
            payment_id.value()
        )
        .execute(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| format!("Failed to update payment status: {}", e))?;

        info!("Updated payment {} status to {}", payment_id.value(), status_str);
        Ok(())
    }

    async fn delete(&self, payment_id: &PaymentId) -> Result<(), String> {
        sqlx::query!(
            "DELETE FROM payments WHERE id = $1",
            payment_id.value()
        )
        .execute(self.db_pool as &sqlx::PgPool)
        .await
        .map_err(|e| format!("Failed to delete payment: {}", e))?;

        info!("Deleted payment: {}", payment_id.value());
        Ok(())
    }
}

// Helper method to convert SQLx record to Payment domain model
impl PaymentRepositorySqlx {
    fn convert_sqlx_to_payment(&self, record: sqlx::postgres::PgRow) -> Result<Payment, AppError> {
        // This is a placeholder - you'll need to implement the proper conversion
        // based on your actual SQLx query result structure
        unimplemented!("convert_sqlx_to_payment needs to be implemented")
    }
}