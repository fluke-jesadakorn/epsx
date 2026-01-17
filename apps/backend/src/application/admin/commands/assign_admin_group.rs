
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::domain::auth::ports::IdentityProviderPort;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct AssignAdminGroupCommand {
    pub wallet_address: String,
    pub group_name: String,
    pub custom_claims: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct AssignAdminGroupResponse {
    pub success: bool,
    pub message: String,
    pub wallet_address: String,
    pub assigned_group: String,
    pub custom_claims: HashMap<String, Value>,
}

pub struct AssignAdminGroupHandler {
    identity_provider: Arc<dyn IdentityProviderPort>,
}

impl AssignAdminGroupHandler {
    pub fn new(identity_provider: Arc<dyn IdentityProviderPort>) -> Self {
        Self { identity_provider }
    }

    pub async fn handle(
        &self,
        command: AssignAdminGroupCommand,
    ) -> Result<AssignAdminGroupResponse, anyhow::Error> {
        tracing::info!(
            "Handling Admin Assignment for user {} to group {}", 
            command.wallet_address, 
            command.group_name
        );

        // Define default admin claims
        let mut custom_claims = HashMap::new();
        custom_claims.insert("admin".to_string(), Value::Bool(true));
        custom_claims.insert("access_level".to_string(), Value::String("full".to_string()));
        
        let permissions = vec![
            "admin:*:*",
            "epsx:*:*",
            "system_admin",
            "module_management",
            "database_access",
            "developer_portal",
        ];
        
        custom_claims.insert(
            "permissions".to_string(), 
            Value::Array(permissions.into_iter().map(|p| Value::String(p.to_string())).collect())
        );

        // Merge additional claims
        if let Some(additional) = command.custom_claims {
            for (k, v) in additional {
                custom_claims.insert(k, v);
            }
        }

        // Use the port to set claims
        self.identity_provider.set_custom_claims(&command.wallet_address, &custom_claims).await?;

        Ok(AssignAdminGroupResponse {
            success: true,
            message: "Admin group assigned successfully".to_string(),
            wallet_address: command.wallet_address,
            assigned_group: command.group_name,
            custom_claims,
        })
    }
}
