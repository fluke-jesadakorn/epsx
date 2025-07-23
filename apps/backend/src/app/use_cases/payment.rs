// Payment use cases

use std::sync::Arc;

use crate::dom::entities::Payment;
use crate::dom::values::Currency;
use crate::dom::events::PaymentCompletedEvent;
use crate::app::ports::{PayRepo, UserRepo, PayGw, EventDispatcher};
use crate::app::dtos::{CreatePayReq, CreatePayRes, GetPayReq, GetPayRes, ConfirmPayReq, ConfirmPayRes, ListPayReq, ListPayRes, PaymentDto};

pub struct PayUC {
    pay_repo: Arc<dyn PayRepo>,
    user_repo: Arc<dyn UserRepo>,
    pay_gw: Arc<dyn PayGw>,
    event_dispatcher: Arc<dyn EventDispatcher>,
}

impl PayUC {
    pub fn new(
        pay_repo: Arc<dyn PayRepo>,
        user_repo: Arc<dyn UserRepo>,
        pay_gw: Arc<dyn PayGw>,
        event_dispatcher: Arc<dyn EventDispatcher>,
    ) -> Self {
        Self {
            pay_repo,
            user_repo,
            pay_gw,
            event_dispatcher,
        }
    }
    
    pub async fn create_payment(&self, req: CreatePayReq) -> Result<CreatePayRes, PayUseCaseError> {
        req.validate().map_err(|e| PayUseCaseError::ValidationError(e.to_string()))?;
        
        let currency = req.curr.parse::<Currency>()
            .map_err(|_| PayUseCaseError::InvalidCurrency(req.curr.clone()))?;
        
        // Verify user exists
        let _user = self.user_repo.get(&req.usr_id).await
            .map_err(|e| PayUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| PayUseCaseError::UserNotFound(req.usr_id.to_string()))?;
        
        // Create payment
        let mut payment = Payment::new(req.usr_id.clone(), req.amt, currency.clone());
        
        // Generate payment address
        let pay_addr = self.pay_gw.create_payment_address(&currency, &req.usr_id).await
            .map_err(|e| PayUseCaseError::PaymentGatewayError(e.to_string()))?;
        
        payment.set_addr(pay_addr.address.clone());
        
        // Save payment
        self.pay_repo.save(&payment).await
            .map_err(|e| PayUseCaseError::RepositoryError(e.to_string()))?;
        
        Ok(CreatePayRes {
            pay: PaymentDto::from_entity(&payment),
            addr: Some(pay_addr.address),
            qr_code: pay_addr.qr_code_url,
        })
    }
    
    pub async fn get_payment(&self, req: GetPayReq) -> Result<GetPayRes, PayUseCaseError> {
        let payment = self.pay_repo.get(&req.pay_id).await
            .map_err(|e| PayUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| PayUseCaseError::PaymentNotFound(req.pay_id.to_string()))?;
        
        // Check authorization
        if payment.uid() != &req.usr_id {
            return Err(PayUseCaseError::PermissionDenied);
        }
        
        Ok(GetPayRes {
            pay: PaymentDto::from_entity(&payment),
        })
    }
    
    pub async fn confirm_payment(&self, req: ConfirmPayReq) -> Result<ConfirmPayRes, PayUseCaseError> {
        let mut payment = self.pay_repo.get(&req.pay_id).await
            .map_err(|e| PayUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| PayUseCaseError::PaymentNotFound(req.pay_id.to_string()))?;
        
        // Confirm payment
        payment.confirm(req.tx_hash.clone())
            .map_err(|e| PayUseCaseError::DomainError(e.to_string()))?;
        
        // Complete if enough confirmations
        if req.confirmations >= 3 {
            payment.complete()
                .map_err(|e| PayUseCaseError::DomainError(e.to_string()))?;
            
            // Emit completion event
            let event = PaymentCompletedEvent::new(
                payment.id().clone(),
                payment.uid().clone(),
                payment.amt()
            );
            
            self.event_dispatcher.dispatch(Box::new(event)).await
                .map_err(|e| PayUseCaseError::EventDispatchFailed(e.to_string()))?;
        }
        
        // Save changes
        self.pay_repo.save(&payment).await
            .map_err(|e| PayUseCaseError::RepositoryError(e.to_string()))?;
        
        Ok(ConfirmPayRes {
            pay: PaymentDto::from_entity(&payment),
            usr_notified: payment.is_completed(),
        })
    }
    
    pub async fn list_payments(&self, req: ListPayReq) -> Result<ListPayRes, PayUseCaseError> {
        req.validate().map_err(|e| PayUseCaseError::ValidationError(e.to_string()))?;
        
        let payments = self.pay_repo.find_by_user(&req.usr_id).await
            .map_err(|e| PayUseCaseError::RepositoryError(e.to_string()))?;
        
        // Apply status filter if provided
        let filtered_payments = if let Some(status_filter) = &req.status_filter {
            let status = status_filter.parse()
                .map_err(|_| PayUseCaseError::InvalidStatus(status_filter.clone()))?;
            payments.into_iter()
                .filter(|p| p.stat() == &status)
                .collect()
        } else {
            payments
        };
        
        // Apply pagination
        let start = req.offset as usize;
        let end = std::cmp::min(start + req.limit as usize, filtered_payments.len());
        let paginated = if start < filtered_payments.len() {
            filtered_payments[start..end].to_vec()
        } else {
            vec![]
        };
        
        Ok(ListPayRes {
            payments: paginated.iter().map(PaymentDto::from_entity).collect(),
            total: filtered_payments.len() as u64,
            offset: req.offset,
            limit: req.limit,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PayUseCaseError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),
    
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Payment not found: {0}")]
    PaymentNotFound(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Domain error: {0}")]
    DomainError(String),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Payment gateway error: {0}")]
    PaymentGatewayError(String),
    
    #[error("Event dispatch failed: {0}")]
    EventDispatchFailed(String),
}