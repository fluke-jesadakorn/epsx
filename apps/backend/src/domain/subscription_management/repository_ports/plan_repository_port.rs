use crate::prelude::*;
use crate::domain::subscription_management::{Plan, PlanId};

/// Search criteria for plans
#[derive(Debug, Clone, Default)]
pub struct PlanSearchCriteria {
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Repository port for plan operations
#[async_trait]
pub trait PlanRepositoryPort: Send + Sync {
    /// Find plan by ID
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>>;

    /// List all plans with optional filtering
    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>>;

    /// Save (create or update) a plan
    async fn save(&self, plan: &Plan) -> AppResult<()>;

    /// Delete a plan
    async fn delete(&self, id: &PlanId) -> AppResult<()>;

    /// Count plans matching criteria
    async fn count(&self, criteria: PlanSearchCriteria) -> AppResult<i64>;

    /// Find active plans
    async fn find_active(&self) -> AppResult<Vec<Plan>>;

    /// Find promoted plans
    async fn find_promoted(&self) -> AppResult<Vec<Plan>>;
}
