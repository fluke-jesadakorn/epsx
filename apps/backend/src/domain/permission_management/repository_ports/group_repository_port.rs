use crate::prelude::*;
use crate::domain::permission_management::{Group, GroupId, GroupSlug};

/// Search criteria for groups
#[derive(Debug, Clone, Default)]
pub struct GroupSearchCriteria {
    pub group_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Group statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStatistics {
    pub total_groups: i64,
    pub active_groups: i64,
    pub promoted_groups: i64,
    pub total_members: i64,
}

/// Repository port for group operations (new name)
/// Also exported as PermissionGroupRepositoryPort for backward compatibility
#[async_trait]
pub trait GroupRepositoryPort: Send + Sync {
    /// Find group by ID
    async fn find_by_id(&self, id: &GroupId) -> AppResult<Option<Group>>;

    /// Find group by slug
    async fn find_by_slug(&self, slug: &GroupSlug) -> AppResult<Option<Group>>;

    /// List all groups with optional filtering
    async fn find_all(&self, criteria: GroupSearchCriteria) -> AppResult<Vec<Group>>;

    /// Save (create or update) a group
    async fn save(&self, group: &Group) -> AppResult<()>;

    /// Delete a group
    async fn delete(&self, id: &GroupId) -> AppResult<()>;

    /// Count groups matching criteria
    async fn count(&self, criteria: GroupSearchCriteria) -> AppResult<i64>;

    /// Get group statistics
    async fn get_statistics(&self) -> AppResult<GroupStatistics>;

    /// Check if slug exists
    async fn slug_exists(&self, slug: &GroupSlug) -> AppResult<bool>;
}

/// Backward compatibility: alias trait with same signature
pub trait PermissionGroupRepositoryPort: GroupRepositoryPort {}

/// Blanket implementation: any GroupRepositoryPort is also a PermissionGroupRepositoryPort
impl<T: GroupRepositoryPort> PermissionGroupRepositoryPort for T {}
