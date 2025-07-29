// IAM use cases - business logic for IAM operations

use std::sync::Arc;
use crate::dom::entities::iam::{
    IamRole, IamPolicy, UserPermissionOverride, PolicyDocument,
    PolicyStatement, ActionSet, ResourceSet, Effect, PackageTier, Permission,
    RoleId, PolicyId, IamError,
};
use crate::dom::values::UserId;
use crate::dom::services::policy_engine::{PolicyEngine, EvaluationContext};
use crate::app::ports::repositories::{UserRepo, IamRepo};

/// IAM use cases for role, policy, and permission management
pub struct IamUC {
    user_repo: Arc<dyn UserRepo>,
    iam_repo: Arc<dyn IamRepo>,
    policy_engine: Arc<tokio::sync::Mutex<PolicyEngine>>,
}

/// Request to create a new IAM role
#[derive(Debug, serde::Deserialize)]
pub struct CreateRoleReq {
    pub name: String,
    pub description: Option<String>,
    pub package_tier: String,
    pub policies: Vec<String>,
    pub inline_permissions: Vec<CreatePermissionReq>,
    pub assignable: bool,
}

/// Request to create a permission
#[derive(Debug, serde::Deserialize)]
pub struct CreatePermissionReq {
    pub action: String,
    pub resource: String,
    pub conditions: Option<std::collections::HashMap<String, String>>,
}

/// Request to update an IAM role
#[derive(Debug, serde::Deserialize)]
pub struct UpdateRoleReq {
    pub name: Option<String>,
    pub description: Option<String>,
    pub assignable: Option<bool>,
    pub add_policies: Option<Vec<String>>,
    pub remove_policies: Option<Vec<String>>,
    pub add_permissions: Option<Vec<CreatePermissionReq>>,
}

/// Request to create a policy
#[derive(Debug, serde::Deserialize)]
pub struct CreatePolicyReq {
    pub name: String,
    pub description: Option<String>,
    pub policy_document: PolicyDocumentReq,
}

/// Policy document in request format
#[derive(Debug, serde::Deserialize)]
pub struct PolicyDocumentReq {
    pub statements: Vec<PolicyStatementReq>,
}

/// Policy statement in request format
#[derive(Debug, serde::Deserialize)]
pub struct PolicyStatementReq {
    pub effect: String, // "Allow" or "Deny"
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub conditions: Option<std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>>,
}

/// Request to evaluate user permissions
#[derive(Debug, serde::Deserialize)]
pub struct EvaluatePermissionReq {
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub context_vars: Option<std::collections::HashMap<String, String>>,
}

/// Response for permission evaluation
#[derive(Debug, serde::Serialize)]
pub struct EvaluatePermissionRes {
    pub allowed: bool,
    pub reasons: Vec<String>,
    pub matching_policies: Vec<String>,
    pub package_tier_access: bool,
    pub explicit_permissions: Vec<String>,
}

/// Request to set user permission overrides
#[derive(Debug, serde::Deserialize)]
pub struct SetUserOverrideReq {
    pub user_id: String,
    pub permission_grants: Vec<CreatePermissionReq>,
    pub permission_denials: Vec<CreatePermissionReq>,
    pub additional_roles: Vec<String>,
    pub reason: Option<String>,
}

/// Response for IAM role operations
#[derive(Debug, serde::Serialize)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub package_tier: String,
    pub policies: Vec<String>,
    pub inline_permissions: Vec<PermissionResponse>,
    pub assignable: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Response for IAM policy operations
#[derive(Debug, serde::Serialize)]
pub struct PolicyResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub policy_document: PolicyDocumentResponse,
    pub version: String,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Policy document in response format
#[derive(Debug, serde::Serialize)]
pub struct PolicyDocumentResponse {
    pub version: String,
    pub statements: Vec<PolicyStatementResponse>,
}

/// Policy statement in response format
#[derive(Debug, serde::Serialize)]
pub struct PolicyStatementResponse {
    pub effect: String,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
}

/// Permission in response format
#[derive(Debug, serde::Serialize)]
pub struct PermissionResponse {
    pub action: String,
    pub resource: String,
    pub conditions: Option<std::collections::HashMap<String, String>>,
}

impl IamUC {
    pub fn new(
        user_repo: Arc<dyn UserRepo>,
        iam_repo: Arc<dyn IamRepo>,
        policy_engine: Arc<tokio::sync::Mutex<PolicyEngine>>,
    ) -> Self {
        Self {
            user_repo,
            iam_repo,
            policy_engine,
        }
    }

    /// Create a new IAM role
    pub async fn create_role(
        &self,
        req: CreateRoleReq,
        created_by: UserId,
    ) -> Result<RoleResponse, IamError> {
        // Parse package tier
        let package_tier: PackageTier = req.package_tier.parse()?;

        // Create role entity
        let mut role = IamRole::new(req.name, package_tier, created_by);

        // Add description if provided
        if let Some(_description) = req.description {
            // TODO: Add set_description method to IamRole
        }

        // Add policies
        for policy_id_str in req.policies {
            let policy_id = PolicyId::new(policy_id_str);
            role.add_policy(policy_id);
        }

        // Add inline permissions
        for perm_req in req.inline_permissions {
            let permission = if let Some(conditions) = perm_req.conditions {
                Permission::with_conditions(perm_req.action, perm_req.resource, conditions)
            } else {
                Permission::new(perm_req.action, perm_req.resource)
            };
            role.add_permission(permission);
        }

        // Set assignable
        role.set_assignable(req.assignable);

        // Save to repository
        let saved_role = self.iam_repo.create_role(role).await?;

        Ok(self.role_to_response(&saved_role))
    }

    /// Update an existing IAM role
    pub async fn update_role(
        &self,
        role_id: String,
        req: UpdateRoleReq,
    ) -> Result<RoleResponse, IamError> {
        let role_id = RoleId::new(role_id);
        let mut role = self.iam_repo.get_role(&role_id).await?;

        // Update name if provided
        if let Some(_name) = req.name {
            // TODO: Add set_name method to IamRole
        }

        // Update assignable if provided
        if let Some(assignable) = req.assignable {
            role.set_assignable(assignable);
        }

        // Add policies
        if let Some(policy_ids) = req.add_policies {
            for policy_id_str in policy_ids {
                let policy_id = PolicyId::new(policy_id_str);
                role.add_policy(policy_id);
            }
        }

        // Remove policies
        if let Some(policy_ids) = req.remove_policies {
            for policy_id_str in policy_ids {
                let policy_id = PolicyId::new(policy_id_str);
                role.remove_policy(&policy_id);
            }
        }

        // Add permissions
        if let Some(permissions) = req.add_permissions {
            for perm_req in permissions {
                let permission = if let Some(conditions) = perm_req.conditions {
                    Permission::with_conditions(perm_req.action, perm_req.resource, conditions)
                } else {
                    Permission::new(perm_req.action, perm_req.resource)
                };
                role.add_permission(permission);
            }
        }

        // Save updated role
        let updated_role = self.iam_repo.update_role(role).await?;

        Ok(self.role_to_response(&updated_role))
    }

    /// Get all IAM roles
    pub async fn list_roles(&self) -> Result<Vec<RoleResponse>, IamError> {
        let roles = self.iam_repo.list_roles().await?;
        Ok(roles.into_iter().map(|r| self.role_to_response(&r)).collect())
    }

    /// Get an IAM role by ID
    pub async fn get_role(&self, role_id: String) -> Result<RoleResponse, IamError> {
        let role_id = RoleId::new(role_id);
        let role = self.iam_repo.get_role(&role_id).await?;
        Ok(self.role_to_response(&role))
    }

    /// Delete an IAM role
    pub async fn delete_role(&self, role_id: String) -> Result<(), IamError> {
        let role_id = RoleId::new(role_id);
        self.iam_repo.delete_role(&role_id).await
    }

    /// Create a new IAM policy
    pub async fn create_policy(
        &self,
        req: CreatePolicyReq,
        created_by: UserId,
    ) -> Result<PolicyResponse, IamError> {
        // Convert request to domain policy document
        let policy_document = self.req_to_policy_document(req.policy_document)?;

        // Create policy entity
        let policy = IamPolicy::new(
            req.name,
            policy_document,
            crate::dom::entities::iam::PolicyType::Managed,
            created_by,
        );

        // Save to repository
        let saved_policy = self.iam_repo.create_policy(policy).await?;

        Ok(self.policy_to_response(&saved_policy))
    }

    /// Get all IAM policies
    pub async fn list_policies(&self) -> Result<Vec<PolicyResponse>, IamError> {
        let policies = self.iam_repo.list_policies().await?;
        Ok(policies.into_iter().map(|p| self.policy_to_response(&p)).collect())
    }

    /// Get an IAM policy by ID
    pub async fn get_policy(&self, policy_id: String) -> Result<PolicyResponse, IamError> {
        let policy_id = PolicyId::new(policy_id);
        let policy = self.iam_repo.get_policy(&policy_id).await?;
        Ok(self.policy_to_response(&policy))
    }

    /// Delete an IAM policy
    pub async fn delete_policy(&self, policy_id: String) -> Result<(), IamError> {
        let policy_id = PolicyId::new(policy_id);
        self.iam_repo.delete_policy(&policy_id).await
    }

    /// Evaluate user permission for an action and resource
    pub async fn evaluate_permission(
        &self,
        req: EvaluatePermissionReq,
    ) -> Result<EvaluatePermissionRes, IamError> {
        let user_id = UserId::new(req.user_id);

        // Get user details
        let user = self.user_repo.find_by_id(&user_id.clone()).await
            .map_err(|_| IamError::PolicyEvaluationFailed("User not found".to_string()))?;

        // Create evaluation context
        let mut context = EvaluationContext::new(user_id.clone(), req.action, req.resource);
        
        if let Some(context_vars) = req.context_vars {
            for (key, value) in context_vars {
                context = context.with_context_var(key, value);
            }
        }

        // Get user's package tier
        let package_tier = PackageTier::from_user_role(user.role());

        // Get user's roles
        let user_roles = self.iam_repo.get_user_roles(&user_id.clone()).await?;

        // Get user's permission overrides
        let user_overrides = self.iam_repo.get_user_overrides(&user_id).await.ok();

        // Get all policies for the user's roles
        let mut all_policies = Vec::new();
        for role in &user_roles {
            for policy_id in role.policies() {
                if let Ok(policy) = self.iam_repo.get_policy(policy_id).await {
                    all_policies.push(policy);
                }
            }
        }

        // Evaluate permission using policy engine
        let mut engine = self.policy_engine.lock().await;
        let decision = engine.evaluate_permission(
            &context,
            &package_tier,
            &user_roles,
            &user_overrides,
            &all_policies,
        )?;

        Ok(EvaluatePermissionRes {
            allowed: decision.is_allowed(),
            reasons: decision.reasons,
            matching_policies: decision.matching_policies,
            package_tier_access: decision.package_tier_access,
            explicit_permissions: decision.explicit_permissions,
        })
    }

    /// Set user permission overrides
    pub async fn set_user_overrides(
        &self,
        req: SetUserOverrideReq,
        created_by: UserId,
    ) -> Result<(), IamError> {
        let user_id = UserId::new(req.user_id);

        // Get user to validate existence
        let user = self.user_repo.find_by_id(&user_id).await
            .map_err(|_| IamError::PolicyEvaluationFailed("User not found".to_string()))?;

        let package_tier = PackageTier::from_user_role(user.role());

        // Convert permissions
        let permission_grants = req.permission_grants.into_iter().map(|p| {
            if let Some(conditions) = p.conditions {
                Permission::with_conditions(p.action, p.resource, conditions)
            } else {
                Permission::new(p.action, p.resource)
            }
        }).collect();

        let permission_denials = req.permission_denials.into_iter().map(|p| {
            if let Some(conditions) = p.conditions {
                Permission::with_conditions(p.action, p.resource, conditions)
            } else {
                Permission::new(p.action, p.resource)
            }
        }).collect();

        let additional_roles = req.additional_roles.into_iter().map(RoleId::new).collect();

        // Create user permission override
        let override_entity = UserPermissionOverride {
            user_id,
            package_tier,
            additional_roles,
            permission_grants,
            permission_denials,
            expires_at: None, // TODO: Support expiration
            reason: req.reason,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
        };

        // Save to repository
        self.iam_repo.set_user_overrides(override_entity).await?;

        Ok(())
    }

    // Helper methods

    fn role_to_response(&self, role: &IamRole) -> RoleResponse {
        RoleResponse {
            id: role.id().value().to_string(),
            name: role.name().to_string(),
            description: None, // TODO: Add description getter
            package_tier: role.package_tier().to_string(),
            policies: role.policies().iter().map(|p| p.value().to_string()).collect(),
            inline_permissions: role.inline_permissions().iter().map(|p| PermissionResponse {
                action: p.action().to_string(),
                resource: p.resource().to_string(),
                conditions: p.conditions().cloned(),
            }).collect(),
            assignable: role.is_assignable(),
            created_at: "".to_string(), // TODO: Add created_at getter
            updated_at: "".to_string(), // TODO: Add updated_at getter
        }
    }

    fn policy_to_response(&self, policy: &IamPolicy) -> PolicyResponse {
        PolicyResponse {
            id: policy.id().value().to_string(),
            name: policy.name().to_string(),
            description: None, // TODO: Add description getter
            policy_document: PolicyDocumentResponse {
                version: policy.policy_document().version().to_string(),
                statements: policy.policy_document().statements().iter().map(|s| PolicyStatementResponse {
                    effect: match s.effect() {
                        Effect::Allow => "Allow".to_string(),
                        Effect::Deny => "Deny".to_string(),
                    },
                    actions: match s.actions() {
                        ActionSet::All => vec!["*".to_string()],
                        ActionSet::Actions(actions) => actions.clone(),
                        ActionSet::Patterns(patterns) => patterns.clone(),
                    },
                    resources: match s.resources() {
                        ResourceSet::All => vec!["*".to_string()],
                        ResourceSet::Resources(resources) => resources.clone(),
                        ResourceSet::Patterns(patterns) => patterns.clone(),
                    },
                }).collect(),
            },
            version: policy.version().to_string(),
            active: policy.is_active(),
            created_at: "".to_string(), // TODO: Add created_at getter
            updated_at: "".to_string(), // TODO: Add updated_at getter
        }
    }

    fn req_to_policy_document(&self, req: PolicyDocumentReq) -> Result<PolicyDocument, IamError> {
        let statements = req.statements.into_iter().map(|s| {
            let effect = match s.effect.as_str() {
                "Allow" => Effect::Allow,
                "Deny" => Effect::Deny,
                _ => return Err(IamError::InvalidPolicyDocument(format!("Invalid effect: {}", s.effect))),
            };

            let actions = if s.actions.contains(&"*".to_string()) {
                ActionSet::All
            } else {
                ActionSet::Actions(s.actions)
            };

            let resources = if s.resources.contains(&"*".to_string()) {
                ResourceSet::All
            } else {
                ResourceSet::Resources(s.resources)
            };

            let mut statement = PolicyStatement::allow(actions, resources);
            
            // Set effect (override the default Allow from the constructor)
            if effect == Effect::Deny {
                statement = PolicyStatement::deny(statement.actions().clone(), statement.resources().clone());
            }

            Ok(statement)
        }).collect::<Result<Vec<_>, IamError>>()?;

        Ok(PolicyDocument::new(statements))
    }
}