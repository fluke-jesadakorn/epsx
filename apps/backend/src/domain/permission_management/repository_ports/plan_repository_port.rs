use crate::prelude::*;
use crate::domain::permission_management::{Plan, PlanId, PlanSlug};

/// Search criteria for plans
#[derive(Debug, Clone, Default)]
pub struct PlanSearchCriteria {
    pub plan_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Plan statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStatistics {
    pub total_plans: i64,
    pub active_plans: i64,
    pub promoted_plans: i64,
    pub total_members: i64,
}

/// Repository port for plan operations (new name)
/// Also exported as PermissionPlanRepositoryPort for backward compatibility
#[async_trait]
pub trait PlanRepositoryPort: Send + Sync {
    /// Find plan by ID
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>>;

    /// Find plan by slug
    async fn find_by_slug(&self, slug: &PlanSlug) -> AppResult<Option<Plan>>;

    /// List all plans with optional filtering
    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>>;

    /// Save (create or update) a plan
    async fn save(&self, plan: &Plan) -> AppResult<()>;

    /// Delete a plan
    async fn delete(&self, id: &PlanId) -> AppResult<()>;

    /// Count plans matching criteria
    async fn count(&self, criteria: PlanSearchCriteria) -> AppResult<i64>;

    /// Get plan statistics
    async fn get_statistics(&self) -> AppResult<PlanStatistics>;

    /// Check if slug exists
    async fn slug_exists(&self, slug: &PlanSlug) -> AppResult<bool>;
}

/// Backward compatibility: alias trait with same signature
pub trait PermissionPlanRepositoryPort: PlanRepositoryPort {}

/// Blanket implementation: any PlanRepositoryPort is also a PermissionPlanRepositoryPort
impl<T: PlanRepositoryPort> PermissionPlanRepositoryPort for T {}
