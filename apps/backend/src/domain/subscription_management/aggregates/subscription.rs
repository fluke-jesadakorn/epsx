use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use crate::domain::subscription_management::{SubscriptionId, PlanId};
use crate::domain::wallet_management::WalletAddress;

/// Subscription status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    Expired,
    PendingRenewal,
}

/// Subscription Aggregate Root
/// Represents an active subscription of a wallet to a plan
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Subscription {
    id: SubscriptionId,
    wallet_address: WalletAddress,
    plan_id: PlanId,
    status: SubscriptionStatus,
    started_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    auto_renew: bool,
    payment_method_id: Option<String>,
    metadata: serde_json::Value,
    base: AggregateBase,
}

pub struct CreateSubscriptionParams {
    pub wallet_address: WalletAddress,
    pub plan_id: PlanId,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_renew: bool,
    pub payment_method_id: Option<String>,
}

impl Subscription {
    /// Create a new subscription
    pub fn create(params: CreateSubscriptionParams) -> Self {
        let now = Utc::now();

        Self {
            id: SubscriptionId::new(),
            wallet_address: params.wallet_address,
            plan_id: params.plan_id,
            status: SubscriptionStatus::Active,
            started_at: now,
            expires_at: params.expires_at,
            cancelled_at: None,
            auto_renew: params.auto_renew,
            payment_method_id: params.payment_method_id,
            metadata: serde_json::json!({}),
            base: AggregateBase::new(),
        }
    }

    /// Cancel subscription
    pub fn cancel(&mut self) -> AppResult<()> {
        if self.status == SubscriptionStatus::Cancelled {
            return Err(AppError::validation_error("Subscription is already cancelled"));
        }

        self.status = SubscriptionStatus::Cancelled;
        self.cancelled_at = Some(Utc::now());
        self.auto_renew = false;
        self.base.touch();
        self.base.increment_version();

        Ok(())
    }

    /// Renew subscription
    pub fn renew(&mut self, new_expires_at: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
        if self.status == SubscriptionStatus::Cancelled {
            return Err(AppError::validation_error("Cannot renew cancelled subscription"));
        }

        self.expires_at = Some(new_expires_at);
        self.status = SubscriptionStatus::Active;
        self.base.touch();
        self.base.increment_version();

        Ok(())
    }

    /// Mark subscription as expired
    pub fn expire(&mut self) -> AppResult<()> {
        self.status = SubscriptionStatus::Expired;
        self.auto_renew = false;
        self.base.touch();
        self.base.increment_version();

        Ok(())
    }

    /// Check if subscription is currently active
    pub fn is_active(&self) -> bool {
        if self.status != SubscriptionStatus::Active {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() < expires_at
        } else {
            true // Lifetime subscription
        }
    }

    // Getters
    pub fn id(&self) -> &SubscriptionId {
        &self.id
    }

    pub fn wallet_address(&self) -> &WalletAddress {
        &self.wallet_address
    }

    pub fn plan_id(&self) -> &PlanId {
        &self.plan_id
    }

    pub fn status(&self) -> &SubscriptionStatus {
        &self.status
    }

    pub fn started_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.started_at
    }

    pub fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.expires_at
    }

    pub fn auto_renew(&self) -> bool {
        self.auto_renew
    }
}

impl AggregateRoot for Subscription {
    type Id = SubscriptionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version
    }

    fn increment_version(&mut self) {
        self.base.increment_version();
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        self.base.uncommitted_events()
    }

    fn mark_events_as_committed(&mut self) {
        self.base.mark_events_as_committed();
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch();
    }
}
