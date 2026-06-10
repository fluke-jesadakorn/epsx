use crate::primitives::icon::Icon;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
    pub email: Option<String>,
    pub tier: Option<String>,
    pub permissions: Vec<String>,
}

impl User {
    pub fn is_authed(&self) -> bool { !self.id.is_empty() }
    pub fn short_address(&self) -> String {
        if self.address.len() < 10 { return self.address.clone(); }
        format!("{}…{}", &self.address[..6], &self.address[self.address.len()-4..])
    }
    pub fn is_admin(&self) -> bool { self.roles.iter().any(|r| r == "admin" || r == "super_admin" || r == "Admin") }
    pub fn has_permission(&self, p: &str) -> bool { self.permissions.iter().any(|x| x == p) }
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: String::new(),
            address: String::new(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
        }
    }
}
