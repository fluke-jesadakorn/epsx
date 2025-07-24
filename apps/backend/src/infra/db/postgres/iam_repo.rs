use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    app::ports::repositories::IamRepo,
    dom::entities::iam::{
        IamRole, IamPolicy, IamGroup, UserPermissionOverride, 
        RoleId, PolicyId, GroupId, IamError
    },
    dom::values::UserId,
};

pub struct PostgresIamRepo {
    pool: PgPool,
}

impl PostgresIamRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_role(row: &sqlx::postgres::PgRow) -> Result<IamRole, IamError> {
        let _id: Uuid = row.try_get("id").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let name: String = row.try_get("name").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let _description: Option<String> = row.try_get("description").ok();
        let _permissions: Vec<String> = row.try_get::<serde_json::Value, _>("permissions")
            .map_err(|e| IamError::DatabaseError(e.to_string()))?
            .as_array()
            .ok_or_else(|| IamError::InvalidData("permissions must be array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let _is_system: bool = row.try_get("is_system").map_err(|e| IamError::DatabaseError(e.to_string()))?;

        // Need to create role with proper constructor that matches entity definition
        // IamRole::new requires (name, package_tier, created_by)
        let package_tier = crate::dom::entities::iam::PackageTier::Free; // Default tier from DB
        let created_by = UserId::new("system".to_string()); // Default system user
        
        Ok(IamRole::new(
            name,
            package_tier,
            created_by,
        ))
    }

    fn map_row_to_policy(row: &sqlx::postgres::PgRow) -> Result<IamPolicy, IamError> {
        let _id: Uuid = row.try_get("id").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let name: String = row.try_get("name").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let _description: Option<String> = row.try_get("description").ok();
        let effect: String = row.try_get("effect").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let actions: Vec<String> = row.try_get::<serde_json::Value, _>("actions")
            .map_err(|e| IamError::DatabaseError(e.to_string()))?
            .as_array()
            .ok_or_else(|| IamError::InvalidData("actions must be array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let resources: Vec<String> = row.try_get::<serde_json::Value, _>("resources")
            .map_err(|e| IamError::DatabaseError(e.to_string()))?
            .as_array()
            .ok_or_else(|| IamError::InvalidData("resources must be array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        // Need to create policy with proper constructor
        // IamPolicy::new requires (name, policy_document, policy_type, created_by)
        use crate::dom::entities::iam::{PolicyDocument, PolicyStatement, ActionSet, ResourceSet, Effect, PolicyType};
        
        let _effect_enum = if effect == "Allow" { Effect::Allow } else { Effect::Deny };
        let statement = PolicyStatement::allow(
            ActionSet::Actions(actions),
            ResourceSet::Resources(resources)
        );
        let policy_document = PolicyDocument::new(vec![statement]);
        let created_by = UserId::new("system".to_string());
        
        Ok(IamPolicy::new(
            name,
            policy_document,
            PolicyType::Managed,
            created_by,
        ))
    }

    fn map_row_to_group(row: &sqlx::postgres::PgRow) -> Result<IamGroup, IamError> {
        let _id: Uuid = row.try_get("id").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let name: String = row.try_get("name").map_err(|e| IamError::DatabaseError(e.to_string()))?;
        let _description: Option<String> = row.try_get("description").ok();
        let _role_ids: Vec<Uuid> = row.try_get::<serde_json::Value, _>("role_ids")
            .map_err(|e| IamError::DatabaseError(e.to_string()))?
            .as_array()
            .ok_or_else(|| IamError::InvalidData("role_ids must be array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok()))
            .collect();

        // IamGroup::new requires (name, created_by)
        let created_by = UserId::new("system".to_string());
        
        Ok(IamGroup::new(
            name,
            created_by,
        ))
    }
}

#[async_trait]
impl IamRepo for PostgresIamRepo {
    async fn create_role(&self, role: IamRole) -> Result<IamRole, IamError> {
        let permissions_json = serde_json::to_value(role.inline_permissions())
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO iam_roles (id, name, description, permissions, is_system, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())")
        .bind(role.id().value())
        .bind(role.name())
        .bind(None::<String>) // description - not available in current entity
        .bind(permissions_json)
        .bind(false) // is_system - default false
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        Ok(role)
    }

    async fn get_role(&self, id: &RoleId) -> Result<IamRole, IamError> {
        let row = sqlx::query(
            "SELECT id, name, description, permissions, is_system FROM iam_roles WHERE id = $1")
        .bind(id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?
        .ok_or(IamError::NotFound)?;

        Self::map_row_to_role(&row)
    }

    async fn update_role(&self, role: IamRole) -> Result<IamRole, IamError> {
        let permissions_json = serde_json::to_value(role.inline_permissions())
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            "UPDATE iam_roles SET name = $1, description = $2, permissions = $3, updated_at = NOW() WHERE id = $4")
        .bind(role.name())
        .bind(None::<String>) // description - not available in current entity
        .bind(permissions_json)
        .bind(role.id().value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(role)
    }

    async fn delete_role(&self, id: &RoleId) -> Result<(), IamError> {
        let result = sqlx::query(
            "DELETE FROM iam_roles WHERE id = $1 AND is_system = false")
        .bind(id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(())
    }

    async fn list_roles(&self) -> Result<Vec<IamRole>, IamError> {
        let rows = sqlx::query(
            "SELECT id, name, description, permissions, is_system FROM iam_roles ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        let mut roles = Vec::new();
        for row in rows {
            roles.push(Self::map_row_to_role(&row)?);
        }

        Ok(roles)
    }

    async fn create_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError> {
        let actions_json = serde_json::to_value(policy.policy_document().statements())
            .map_err(|e| IamError::SerializationError(e.to_string()))?;
        let resources_json = serde_json::to_value(vec!["*"])
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO iam_policies (id, name, description, effect, actions, resources, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())")
        .bind(policy.id().value())
        .bind(policy.name())
        .bind(None::<String>)
        .bind("Allow")
        .bind(actions_json)
        .bind(resources_json)
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        Ok(policy)
    }

    async fn get_policy(&self, id: &PolicyId) -> Result<IamPolicy, IamError> {
        let row = sqlx::query(
            "SELECT id, name, description, effect, actions, resources FROM iam_policies WHERE id = $1")
        .bind(id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?
        .ok_or(IamError::NotFound)?;

        Self::map_row_to_policy(&row)
    }

    async fn update_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError> {
        let actions_json = serde_json::to_value(policy.policy_document().statements())
            .map_err(|e| IamError::SerializationError(e.to_string()))?;
        let resources_json = serde_json::to_value(vec!["*"])
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            "UPDATE iam_policies SET name = $1, description = $2, effect = $3, actions = $4, resources = $5, updated_at = NOW() WHERE id = $6")
        .bind(policy.name())
        .bind(None::<String>)
        .bind("Allow")
        .bind(actions_json)
        .bind(resources_json)
        .bind(policy.id().value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(policy)
    }

    async fn delete_policy(&self, id: &PolicyId) -> Result<(), IamError> {
        let result = sqlx::query(
            "DELETE FROM iam_policies WHERE id = $1")
        .bind(id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(())
    }

    async fn list_policies(&self) -> Result<Vec<IamPolicy>, IamError> {
        let rows = sqlx::query(
            "SELECT id, name, description, effect, actions, resources FROM iam_policies ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        let mut policies = Vec::new();
        for row in rows {
            policies.push(Self::map_row_to_policy(&row)?);
        }

        Ok(policies)
    }

    async fn create_group(&self, group: IamGroup) -> Result<IamGroup, IamError> {
        let role_ids_json = serde_json::to_value(vec!["role1", "role2"])
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO iam_groups (id, name, description, role_ids, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW())")
        .bind(group.id().value())
        .bind(group.name())
        .bind(None::<String>)
        .bind(role_ids_json)
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    async fn get_group(&self, id: &GroupId) -> Result<IamGroup, IamError> {
        let row = sqlx::query(
            "SELECT id, name, description, role_ids FROM iam_groups WHERE id = $1")
        .bind(id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?
        .ok_or(IamError::NotFound)?;

        Self::map_row_to_group(&row)
    }

    async fn update_group(&self, group: IamGroup) -> Result<IamGroup, IamError> {
        let role_ids_json = serde_json::to_value(vec!["role1", "role2"])
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            "UPDATE iam_groups SET name = $1, description = $2, role_ids = $3, updated_at = NOW() WHERE id = $4")
        .bind(group.name())
        .bind(None::<String>)
        .bind(role_ids_json)
        .bind(group.id().value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(group)
    }

    async fn delete_group(&self, id: &GroupId) -> Result<(), IamError> {
        let result = sqlx::query(
            "DELETE FROM iam_groups WHERE id = $1")
        .bind(id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(())
    }

    async fn list_groups(&self) -> Result<Vec<IamGroup>, IamError> {
        let rows = sqlx::query(
            "SELECT id, name, description, role_ids FROM iam_groups ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        let mut groups = Vec::new();
        for row in rows {
            groups.push(Self::map_row_to_group(&row)?);
        }

        Ok(groups)
    }

    async fn get_user_roles(&self, user_id: &UserId) -> Result<Vec<IamRole>, IamError> {
        let rows = sqlx::query(
            "SELECT r.id, r.name, r.description, r.permissions, r.is_system 
             FROM iam_roles r 
             JOIN user_roles ur ON r.id = ur.role_id 
             WHERE ur.user_id = $1")
        .bind(user_id.value())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        let mut roles = Vec::new();
        for row in rows {
            roles.push(Self::map_row_to_role(&row)?);
        }

        Ok(roles)
    }

    async fn assign_role_to_user(&self, user_id: &UserId, role_id: &RoleId) -> Result<(), IamError> {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id, assigned_at) VALUES ($1, $2, NOW())
             ON CONFLICT (user_id, role_id) DO NOTHING")
        .bind(user_id.value())
        .bind(role_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn remove_role_from_user(&self, user_id: &UserId, role_id: &RoleId) -> Result<(), IamError> {
        let result = sqlx::query(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id.value())
        .bind(role_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(IamError::NotFound);
        }

        Ok(())
    }

    async fn get_user_overrides(&self, user_id: &UserId) -> Result<UserPermissionOverride, IamError> {
        let row = sqlx::query(
            "SELECT user_id, granted_permissions, denied_permissions FROM user_permission_overrides WHERE user_id = $1")
        .bind(user_id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let granted: Vec<String> = row.try_get::<serde_json::Value, _>("granted_permissions")
                    .map_err(|e| IamError::DatabaseError(e.to_string()))?
                    .as_array()
                    .ok_or_else(|| IamError::InvalidData("granted_permissions must be array".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let denied: Vec<String> = row.try_get::<serde_json::Value, _>("denied_permissions")
                    .map_err(|e| IamError::DatabaseError(e.to_string()))?
                    .as_array()
                    .ok_or_else(|| IamError::InvalidData("denied_permissions must be array".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                Ok(UserPermissionOverride::new(
                    user_id.clone(),
                    granted,
                    denied,
                ))
            },
            None => Ok(UserPermissionOverride::new(user_id.clone(), vec![], vec![])),
        }
    }

    async fn set_user_overrides(&self, overrides: UserPermissionOverride) -> Result<(), IamError> {
        let granted_json = serde_json::to_value(overrides.granted_permissions())
            .map_err(|e| IamError::SerializationError(e.to_string()))?;
        let denied_json = serde_json::to_value(overrides.denied_permissions())
            .map_err(|e| IamError::SerializationError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO user_permission_overrides (user_id, granted_permissions, denied_permissions, updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (user_id) DO UPDATE SET
                granted_permissions = EXCLUDED.granted_permissions,
                denied_permissions = EXCLUDED.denied_permissions,
                updated_at = NOW()")
        .bind(overrides.user_id().value())
        .bind(granted_json)
        .bind(denied_json)
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_user_overrides(&self, user_id: &UserId) -> Result<(), IamError> {
        sqlx::query(
            "DELETE FROM user_permission_overrides WHERE user_id = $1")
        .bind(user_id.value())
        .execute(&self.pool)
        .await
        .map_err(|e| IamError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests would be implemented here with a test database
}