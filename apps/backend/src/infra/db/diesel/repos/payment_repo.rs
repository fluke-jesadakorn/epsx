use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use rust_decimal::Decimal;

use crate::app::ports::repositories::{PayRepo, RepoError, PaymentStats};
use crate::dom::entities::Payment;
use crate::dom::values::{PayId, UserId, PayStatus};
use crate::infra::db::diesel::{
    DbPool,
    schema::payments,
    models::{DieselPayment, NewDieselPayment},
};

pub struct DieselPaymentRepo {
    pool: Arc<DbPool>,
}

impl DieselPaymentRepo {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PayRepo for DieselPaymentRepo {
    async fn get(&self, _id: &PayId) -> Result<Option<Payment>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let diesel_payment = payments::table
            .filter(payments::id.eq(uuid))
            .first::<DieselPayment>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        match diesel_payment {
            Some(diesel_payment) => {
                let payment = diesel_payment.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselPayment: {:?}", e)))?;
                Ok(Some(payment))
            }
            None => Ok(None)
        }
    }
    
    async fn save(&self, payment: &Payment) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let new_payment: NewDieselPayment = payment.try_into()
            .map_err(|e| RepoError::SerializationError(format!("Failed to convert Payment: {:?}", e)))?;
        
        diesel::insert_into(payments::table)
            .values(&new_payment)
            .on_conflict(payments::id)
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Payment>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let user_uuid = Uuid::parse_str(&uid.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let diesel_payments = payments::table
            .filter(payments::user_id.eq(user_uuid))
            .order(payments::created_at.desc())
            .load::<DieselPayment>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let payments: Result<Vec<Payment>, RepoError> = diesel_payments
            .into_iter()
            .map(|diesel_payment| {
                diesel_payment.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselPayment: {:?}", e)))
            })
            .collect();
        
        payments
    }
    
    async fn find_by_status(&self, _status: &PayStatus) -> Result<Vec<Payment>, RepoError> {
        // Stub implementation
        Ok(vec![])
    }
    
    async fn find_by_date_range(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<Vec<Payment>, RepoError> {
        // Stub implementation
        Ok(vec![])
    }
    
    async fn total_revenue(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<Decimal, RepoError> {
        // Stub implementation
        Ok(Decimal::new(0, 0))
    }
    
    async fn payment_stats(&self) -> Result<PaymentStats, RepoError> {
        // Stub implementation
        Ok(PaymentStats {
            total_payments: 0,
            total_revenue: Decimal::new(0, 0),
            pending_payments: 0,
            completed_payments: 0,
            failed_payments: 0,
        })
    }
}