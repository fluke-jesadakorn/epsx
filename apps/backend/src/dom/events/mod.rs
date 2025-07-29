// Domain events for decoupled communication

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::dom::values::{UserId, Role, PayId};

pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> &Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn event_type(&self) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleChangedEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    old_role: Role,
    new_role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCompletedEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    payment_id: PayId,
    user_id: UserId,
    amount: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPriceAlertEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    symbol: String,
    price: rust_decimal::Decimal,
    threshold: rust_decimal::Decimal,
    user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecutedEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    symbol: String,
    quantity: i32,
    price: rust_decimal::Decimal,
    trade_type: TradeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    event_id: Uuid,
    occurred_at: DateTime<Utc>,
    user_id: UserId,
    deleted_by: UserId,
    reason: Option<String>,
}

impl UserRoleChangedEvent {
    pub fn new(user_id: UserId, old_role: Role, new_role: Role) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            old_role,
            new_role,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn old_role(&self) -> &Role { &self.old_role }
    pub fn new_role(&self) -> &Role { &self.new_role }
}

impl DomainEvent for UserRoleChangedEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "UserRoleChanged" }
}

impl PaymentCompletedEvent {
    pub fn new(payment_id: PayId, user_id: UserId, amount: rust_decimal::Decimal) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            payment_id,
            user_id,
            amount,
        }
    }
    
    pub fn payment_id(&self) -> &PayId { &self.payment_id }
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn amount(&self) -> rust_decimal::Decimal { self.amount }
}

impl DomainEvent for PaymentCompletedEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "PaymentCompleted" }
}

impl UserRegisteredEvent {
    pub fn new(user_id: UserId, email: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            email,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn email(&self) -> &str { &self.email }
}

impl DomainEvent for UserRegisteredEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "UserRegistered" }
}

impl StockPriceAlertEvent {
    pub fn new(symbol: String, price: rust_decimal::Decimal, threshold: rust_decimal::Decimal, user_id: UserId) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            symbol,
            price,
            threshold,
            user_id,
        }
    }
    
    pub fn symbol(&self) -> &str { &self.symbol }
    pub fn price(&self) -> rust_decimal::Decimal { self.price }
    pub fn threshold(&self) -> rust_decimal::Decimal { self.threshold }
    pub fn user_id(&self) -> &UserId { &self.user_id }
}

impl DomainEvent for StockPriceAlertEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "StockPriceAlert" }
}

impl TradeExecutedEvent {
    pub fn new(user_id: UserId, symbol: String, quantity: i32, price: rust_decimal::Decimal, trade_type: TradeType) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            symbol,
            quantity,
            price,
            trade_type,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn symbol(&self) -> &str { &self.symbol }
    pub fn quantity(&self) -> i32 { self.quantity }
    pub fn price(&self) -> rust_decimal::Decimal { self.price }
    pub fn trade_type(&self) -> &TradeType { &self.trade_type }
}

impl DomainEvent for TradeExecutedEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "TradeExecuted" }
}

impl UserDeletedEvent {
    pub fn new(user_id: UserId, deleted_by: UserId, reason: Option<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            user_id,
            deleted_by,
            reason,
        }
    }
    
    pub fn user_id(&self) -> &UserId { &self.user_id }
    pub fn deleted_by(&self) -> &UserId { &self.deleted_by }
    pub fn reason(&self) -> &Option<String> { &self.reason }
}

impl DomainEvent for UserDeletedEvent {
    fn event_id(&self) -> &Uuid { &self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "UserDeleted" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::{UserId, Role};
    
    #[test]
    fn should_create_user_role_changed_event() {
        let user_id = UserId::generate();
        let event = UserRoleChangedEvent::new(
            user_id.clone(),
            Role::User,
            Role::Premium
        );
        
        assert_eq!(event.user_id(), &user_id);
        assert_eq!(event.old_role(), &Role::User);
        assert_eq!(event.new_role(), &Role::Premium);
        assert_eq!(event.event_type(), "UserRoleChanged");
    }
    
    #[test]
    fn should_create_payment_completed_event() {
        let payment_id = PayId::generate();
        let user_id = UserId::generate();
        let amount = rust_decimal_macros::dec!(50.0);
        
        let event = PaymentCompletedEvent::new(
            payment_id.clone(),
            user_id.clone(),
            amount
        );
        
        assert_eq!(event.payment_id(), &payment_id);
        assert_eq!(event.user_id(), &user_id);
        assert_eq!(event.amount(), amount);
        assert_eq!(event.event_type(), "PaymentCompleted");
    }
}