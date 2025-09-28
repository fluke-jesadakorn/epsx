use serde::{Serialize, Deserialize};
use crate::application::shared::{Query, ApplicationResult};

/// Query to get user permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserPermissionsQuery {
    pub wallet_address: String,
    pub include_expired: bool,
}

/// User permissions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserPermissionsResponse {
    pub wallet_address: String,
    pub permissions: Vec<String>,
    pub active_permissions: Vec<String>,
    pub expired_permissions: Vec<String>,
}

impl Query for GetUserPermissionsQuery {
    type Response = GetUserPermissionsResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // TODO: Implement validation
        Ok(())
    }
}