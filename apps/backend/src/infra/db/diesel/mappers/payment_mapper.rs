use chrono::Utc;
use uuid::Uuid;


use crate::dom::entities::Payment;

use crate::dom::values::{PayId, UserId, PayStatus};

use crate::infra::db::diesel::models::{DieselPayment, NewDieselPayment, UpdateDieselPayment};

use crate::app::ports::repositories::RepoError;


impl TryFrom<DieselPayment> for Payment {
    type Error = RepoError;

    fn try_from(diesel_payment: DieselPayment) -> Result<Self, Self::Error> {
        let payment_id = PayId::from_str(&diesel_payment.id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid PayId: {}", e)))?;
        
        let user_id = UserId::from_str(&diesel_payment.user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UserId: {}", e)))?;
        
        let status = diesel_payment.status.parse::<PayStatus>()
            .map_err(|e| RepoError::InvalidData(format!("Invalid PayStatus: {}", e)))?;
        
        Ok(Payment::reconstruct(
            payment_id,
            user_id,
            diesel_payment.amount.into(),
            diesel_payment.currency.parse().map_err(|e| RepoError::InvalidData(format!("Invalid Currency: {}", e)))?,
            status,
            diesel_payment.payment_method,
            diesel_payment.transaction_id,
            diesel_payment.created_at,
            diesel_payment.updated_at,
        ))
    }
}

impl TryFrom<&Payment> for NewDieselPayment {
    type Error = RepoError;

    fn try_from(payment: &Payment) -> Result<Self, Self::Error> {
        let payment_uuid = Uuid::parse_str(&payment.id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid payment UUID: {}", e)))?;
        
        let user_uuid = Uuid::parse_str(&payment.user_id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid user UUID: {}", e)))?;
        
        Ok(NewDieselPayment {
            id: payment_uuid,
            user_id: user_uuid,
            amount: crate::infra::db::diesel::types::DieselDecimal(payment.amount()),
            currency: payment.currency().to_string(),
            status: payment.status().to_string(),
            payment_method: payment.payment_method().map(|s| s.to_string()),
            transaction_id: payment.transaction_id().map(|s| s.to_string()),
            description: payment.description().map(|s| s.to_string()),
            created_at: payment.created_at(),
            updated_at: payment.updated_at(),
        })
    }
}

impl From<&Payment> for UpdateDieselPayment {
    fn from(payment: &Payment) -> Self {
        UpdateDieselPayment {
            status: Some(payment.status().to_string()),
            payment_method: payment.payment_method().map(|s| s.to_string()),
            transaction_id: payment.transaction_id().map(|s| s.to_string()),
            updated_at: Utc::now(),
        }
    }
}