// IAM Repository implementation - placeholder for now

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::app::ports::repositories::IamRepo;
use crate::dom::entities::iam::{
    IamRole, IamPolicy, IamGroup, UserPermissionOverride, 
    RoleId, PolicyId, GroupId, IamError,
};
use crate::dom::values::UserId;

/// In-memory IAM repository implementation (temporary)
pub struct IamRepoImpl {
    roles: Mutex<HashMap<String, IamRole>>,
    policies: Mutex<HashMap<String, IamPolicy>>,
    groups: Mutex<HashMap<String, IamGroup>>,
    user_roles: Mutex<HashMap<String, Vec<String>>>, // user_id -> role_ids
    user_overrides: Mutex<HashMap<String, UserPermissionOverride>>, // user_id -> overrides
}

impl IamRepoImpl {
    pub fn new() -> Self {
        Self {
            roles: Mutex::new(HashMap::new()),
            policies: Mutex::new(HashMap::new()),
            groups: Mutex::new(HashMap::new()),
            user_roles: Mutex::new(HashMap::new()),
            user_overrides: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl IamRepo for IamRepoImpl {
    // Role operations
    async fn create_role(&self, role: IamRole) -> Result<IamRole, IamError> {
        let mut roles = self.roles.lock().unwrap();
        let role_id = role.id().value().to_string();
        roles.insert(role_id, role.clone());
        Ok(role)
    }

    async fn get_role(&self, id: &RoleId) -> Result<IamRole, IamError> {
        let roles = self.roles.lock().unwrap();
        roles.get(id.value())
            .cloned()
            .ok_or_else(|| IamError::RoleNotFound(id.value().to_string()))
    }

    async fn update_role(&self, role: IamRole) -> Result<IamRole, IamError> {
        let mut roles = self.roles.lock().unwrap();
        let role_id = role.id().value().to_string();
        roles.insert(role_id, role.clone());
        Ok(role)
    }

    async fn delete_role(&self, id: &RoleId) -> Result<(), IamError> {
        let mut roles = self.roles.lock().unwrap();
        roles.remove(id.value())
            .ok_or_else(|| IamError::RoleNotFound(id.value().to_string()))?;
        Ok(())
    }

    async fn list_roles(&self) -> Result<Vec<IamRole>, IamError> {
        let roles = self.roles.lock().unwrap();
        Ok(roles.values().cloned().collect())
    }

    // Policy operations
    async fn create_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError> {
        let mut policies = self.policies.lock().unwrap();
        let policy_id = policy.id().value().to_string();
        policies.insert(policy_id, policy.clone());
        Ok(policy)
    }

    async fn get_policy(&self, id: &PolicyId) -> Result<IamPolicy, IamError> {
        let policies = self.policies.lock().unwrap();
        policies.get(id.value())
            .cloned()
            .ok_or_else(|| IamError::PolicyNotFound(id.value().to_string()))
    }

    async fn update_policy(&self, policy: IamPolicy) -> Result<IamPolicy, IamError> {
        let mut policies = self.policies.lock().unwrap();
        let policy_id = policy.id().value().to_string();
        policies.insert(policy_id, policy.clone());
        Ok(policy)
    }

    async fn delete_policy(&self, id: &PolicyId) -> Result<(), IamError> {
        let mut policies = self.policies.lock().unwrap();
        policies.remove(id.value())
            .ok_or_else(|| IamError::PolicyNotFound(id.value().to_string()))?;
        Ok(())
    }

    async fn list_policies(&self) -> Result<Vec<IamPolicy>, IamError> {
        let policies = self.policies.lock().unwrap();
        Ok(policies.values().cloned().collect())
    }

    // Group operations  
    async fn create_group(&self, group: IamGroup) -> Result<IamGroup, IamError> {
        let mut groups = self.groups.lock().unwrap();
        let group_id = group.id().value().to_string();
        groups.insert(group_id, group.clone());
        Ok(group)
    }

    async fn get_group(&self, id: &GroupId) -> Result<IamGroup, IamError> {
        let groups = self.groups.lock().unwrap();
        groups.get(id.value())
            .cloned()
            .ok_or_else(|| IamError::GroupNotFound(id.value().to_string()))
    }

    async fn update_group(&self, group: IamGroup) -> Result<IamGroup, IamError> {
        let mut groups = self.groups.lock().unwrap();
        let group_id = group.id().value().to_string();
        groups.insert(group_id, group.clone());
        Ok(group)
    }

    async fn delete_group(&self, id: &GroupId) -> Result<(), IamError> {
        let mut groups = self.groups.lock().unwrap();
        groups.remove(id.value())
            .ok_or_else(|| IamError::GroupNotFound(id.value().to_string()))?;
        Ok(())
    }

    async fn list_groups(&self) -> Result<Vec<IamGroup>, IamError> {
        let groups = self.groups.lock().unwrap();
        Ok(groups.values().cloned().collect())
    }

    // User-role relationships
    async fn get_user_roles(&self, user_id: &UserId) -> Result<Vec<IamRole>, IamError> {
        let user_roles = self.user_roles.lock().unwrap();
        let roles = self.roles.lock().unwrap();
        
        let empty_vec = Vec::new();
        let role_ids = user_roles.get(&user_id.to_string())
            .unwrap_or(&empty_vec);
        
        let mut user_role_objects = Vec::new();
        for role_id in role_ids {
            if let Some(role) = roles.get(role_id) {
                user_role_objects.push(role.clone());
            }
        }
        
        Ok(user_role_objects)
    }

    async fn assign_role_to_user(&self, user_id: &UserId, role_id: &RoleId) -> Result<(), IamError> {
        let mut user_roles = self.user_roles.lock().unwrap();
        let user_id_str = user_id.to_string();
        let role_id_str = role_id.value().to_string();
        
        let roles = user_roles.entry(user_id_str).or_insert_with(Vec::new);
        if !roles.contains(&role_id_str) {
            roles.push(role_id_str);
        }
        
        Ok(())
    }

    async fn remove_role_from_user(&self, user_id: &UserId, role_id: &RoleId) -> Result<(), IamError> {
        let mut user_roles = self.user_roles.lock().unwrap();
        let user_id_str = user_id.to_string();
        let role_id_str = role_id.value().to_string();
        
        if let Some(roles) = user_roles.get_mut(&user_id_str) {
            roles.retain(|r| r != &role_id_str);
        }
        
        Ok(())
    }

    // User permission overrides
    async fn get_user_overrides(&self, user_id: &UserId) -> Result<UserPermissionOverride, IamError> {
        let user_overrides = self.user_overrides.lock().unwrap();
        user_overrides.get(&user_id.to_string())
            .cloned()
            .ok_or_else(|| IamError::PolicyEvaluationFailed("No user overrides found".to_string()))
    }

    async fn set_user_overrides(&self, overrides: UserPermissionOverride) -> Result<(), IamError> {
        let mut user_overrides = self.user_overrides.lock().unwrap();
        let user_id = overrides.user_id.to_string();
        user_overrides.insert(user_id, overrides);
        Ok(())
    }

    async fn delete_user_overrides(&self, user_id: &UserId) -> Result<(), IamError> {
        let mut user_overrides = self.user_overrides.lock().unwrap();
        user_overrides.remove(&user_id.to_string());
        Ok(())
    }
}