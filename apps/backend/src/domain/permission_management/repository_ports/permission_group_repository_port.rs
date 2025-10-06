use crate::prelude::*;
use crate::domain::permission_management::{PermissionGroup, GroupId, GroupSlug};

/// Search criteria for permission groups
#[derive(Debug, Clone, Default)]
pub struct GroupSearchCriteria {
    pub group_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Permission group statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStatistics {
    pub total_groups: i64,
    pub active_groups: i64,
    pub promoted_groups: i64,
    pub total_members: i64,
}

/// Repository port for permission group operations
#[async_trait]
pub trait PermissionGroupRepositoryPort: Send + Sync {
    /// Find group by ID
    async fn find_by_id(&self, id: &GroupId) -> AppResult<Option<PermissionGroup>>;

    /// Find group by slug
    async fn find_by_slug(&self, slug: &GroupSlug) -> AppResult<Option<PermissionGroup>>;

    /// List all groups with optional filtering
    async fn find_all(&self, criteria: GroupSearchCriteria) -> AppResult<Vec<PermissionGroup>>;

    /// Save (create or update) a permission group
    async fn save(&self, group: &PermissionGroup) -> AppResult<()>;

    /// Delete a permission group
    async fn delete(&self, id: &GroupId) -> AppResult<()>;

    /// Count groups matching criteria
    async fn count(&self, criteria: GroupSearchCriteria) -> AppResult<i64>;

    /// Get group statistics
    async fn get_statistics(&self) -> AppResult<GroupStatistics>;

    /// Check if slug exists
    async fn slug_exists(&self, slug: &GroupSlug) -> AppResult<bool>;
}
