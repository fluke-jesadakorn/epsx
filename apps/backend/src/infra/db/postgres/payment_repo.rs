use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    app::ports::repositories::{PayRepo, PaymentStats, RepoError},
    dom::{
        entities::Payment,
        values::{PayId, UserId, Currency, PayStatus},
    },
};

pub struct PostgresPayRepo {
    pool: PgPool,
}

impl PostgresPayRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_payment(row: &sqlx::postgres::PgRow) -> Result<Payment, RepoError> {
        let id: Uuid = row.try_get("id").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let user_id: Uuid = row.try_get("user_id").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let amount: Decimal = row.try_get("amount").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let currency: String = row.try_get("currency").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let status: String = row.try_get("status").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let payment_address: Option<String> = row.try_get("stripe_payment_intent_id").ok(); // Reusing field for crypto address
        let tx_hash: Option<String> = row.try_get::<Option<serde_json::Value>, _>("metadata")
            .ok()
            .flatten()
            .and_then(|v| v.get("tx_hash").and_then(|tx| tx.as_str().map(|s| s.to_string())));
        let created_at: DateTime<Utc> = row.try_get("created_at").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at").map_err(|e| RepoError::QueryError(e.to_string()))?;

        let pay_id = PayId::from(id);
        let uid = UserId::from(user_id);
        let curr = Currency::from_str(&currency).map_err(|_| RepoError::InvalidData(format!("Invalid currency: {}", currency)))?;
        let stat = PayStatus::from_str(&status).map_err(|_| RepoError::InvalidData(format!("Invalid status: {}", status)))?;

        Ok(Payment::reconstruct(
            pay_id,
            uid,
            amount,
            curr,
            stat,
            payment_address,
            tx_hash,
            created_at,
            updated_at,
        ))
    }
}

#[async_trait]
impl PayRepo for PostgresPayRepo {
    async fn get(&self, id: &PayId) -> Result<Option<Payment>, RepoError> {
        let query = "SELECT * FROM payments WHERE id = $1";
        
        match sqlx::query(query)
            .bind(*id.value())
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(Some(Self::map_row_to_payment(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(RepoError::QueryError(e.to_string())),
        }
    }

    async fn save(&self, payment: &Payment) -> Result<(), RepoError> {
        let metadata = match payment.tx_hash() {
            Some(hash) => serde_json::json!({"tx_hash": hash}),
            None => serde_json::json!({}),
        };

        let query = r#"
            INSERT INTO payments (id, user_id, amount, currency, status, stripe_payment_intent_id, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                amount = EXCLUDED.amount,
                currency = EXCLUDED.currency,
                status = EXCLUDED.status,
                stripe_payment_intent_id = EXCLUDED.stripe_payment_intent_id,
                metadata = EXCLUDED.metadata,
                updated_at = EXCLUDED.updated_at
        "#;

        sqlx::query(query)
            .bind(*payment.id().value())
            .bind(*payment.uid().value())
            .bind(payment.amt())
            .bind(payment.curr().to_string())
            .bind(payment.stat().to_string())
            .bind(payment.addr())
            .bind(&metadata)
            .bind(payment.created_at())
            .bind(payment.updated_at())
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Payment>, RepoError> {
        let query = "SELECT * FROM payments WHERE user_id = $1 ORDER BY created_at DESC";
        
        let rows = sqlx::query(query)
            .bind(*uid.value())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut payments = Vec::new();
        for row in rows {
            payments.push(Self::map_row_to_payment(&row)?);
        }

        Ok(payments)
    }

    async fn find_by_status(&self, status: &PayStatus) -> Result<Vec<Payment>, RepoError> {
        let query = "SELECT * FROM payments WHERE status = $1 ORDER BY created_at DESC";
        
        let rows = sqlx::query(query)
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut payments = Vec::new();
        for row in rows {
            payments.push(Self::map_row_to_payment(&row)?);
        }

        Ok(payments)
    }

    async fn find_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Payment>, RepoError> {
        let query = "SELECT * FROM payments WHERE created_at >= $1 AND created_at <= $2 ORDER BY created_at DESC";
        
        let rows = sqlx::query(query)
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut payments = Vec::new();
        for row in rows {
            payments.push(Self::map_row_to_payment(&row)?);
        }

        Ok(payments)
    }

    async fn total_revenue(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Decimal, RepoError> {
        let query = r#"
            SELECT COALESCE(SUM(amount), 0) as total
            FROM payments 
            WHERE status = 'completed' 
            AND created_at >= $1 
            AND created_at <= $2
        "#;
        
        let row = sqlx::query(query)
            .bind(start)
            .bind(end)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let total: Decimal = row.try_get("total").map_err(|e| RepoError::QueryError(e.to_string()))?;
        Ok(total)
    }

    async fn payment_stats(&self) -> Result<PaymentStats, RepoError> {
        let query = r#"
            SELECT 
                COUNT(*) as total_payments,
                COALESCE(SUM(CASE WHEN status = 'completed' THEN amount ELSE 0 END), 0) as total_revenue,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_payments,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_payments,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_payments
            FROM payments
        "#;
        
        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let total_payments: i64 = row.try_get("total_payments").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let total_revenue: Decimal = row.try_get("total_revenue").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let pending_payments: i64 = row.try_get("pending_payments").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let completed_payments: i64 = row.try_get("completed_payments").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let failed_payments: i64 = row.try_get("failed_payments").map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(PaymentStats {
            total_payments: total_payments as u64,
            total_revenue,
            pending_payments: pending_payments as u64,
            completed_payments: completed_payments as u64,
            failed_payments: failed_payments as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/epsx_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    #[tokio::test]
    async fn test_save_and_get_payment() {
        let pool = setup_test_pool().await;
        let repo = PostgresPayRepo::new(pool);

        let uid = UserId::generate();
        let payment = Payment::new(uid, dec!(50.0), Currency::USDT);
        let payment_id = payment.id().clone();

        // Save payment
        repo.save(&payment).await.unwrap();

        // Get payment
        let retrieved = repo.get(&payment_id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_payment = retrieved.unwrap();
        assert_eq!(retrieved_payment.id(), &payment_id);
        assert_eq!(retrieved_payment.amt(), dec!(50.0));
        assert_eq!(retrieved_payment.curr(), &Currency::USDT);
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let pool = setup_test_pool().await;
        let repo = PostgresPayRepo::new(pool);

        let uid = UserId::generate();
        let payment1 = Payment::new(uid.clone(), dec!(50.0), Currency::USDT);
        let payment2 = Payment::new(uid.clone(), dec!(100.0), Currency::USDT);

        repo.save(&payment1).await.unwrap();
        repo.save(&payment2).await.unwrap();

        let payments = repo.find_by_user(&uid).await.unwrap();
        assert_eq!(payments.len(), 2);
    }
}