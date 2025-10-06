use crate::prelude::*;
use crate::domain::subscription_management::{Subscription, SubscriptionId, PlanId};
use crate::domain::wallet_management::WalletAddress;

/// Search criteria for subscriptions
#[derive(Debug, Clone, Default)]
pub struct SubscriptionSearchCriteria {
    pub wallet_address: Option<WalletAddress>,
    pub plan_id: Option<PlanId>,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Repository port for subscription operations
#[async_trait]
pub trait SubscriptionRepositoryPort: Send + Sync {
    /// Find subscription by ID
    async fn find_by_id(&self, id: &SubscriptionId) -> AppResult<Option<Subscription>>;

    /// Find subscriptions for a wallet
    async fn find_by_wallet(&self, wallet: &WalletAddress) -> AppResult<Vec<Subscription>>;

    /// Find active subscription for a wallet and plan
    async fn find_active_subscription(
        &self,
        wallet: &WalletAddress,
        plan_id: &PlanId,
    ) -> AppResult<Option<Subscription>>;

    /// List all subscriptions with filtering
    async fn find_all(&self, criteria: SubscriptionSearchCriteria) -> AppResult<Vec<Subscription>>;

    /// Save (create or update) a subscription
    async fn save(&self, subscription: &Subscription) -> AppResult<()>;

    /// Delete a subscription
    async fn delete(&self, id: &SubscriptionId) -> AppResult<()>;

    /// Count subscriptions matching criteria
    async fn count(&self, criteria: SubscriptionSearchCriteria) -> AppResult<i64>;
}
