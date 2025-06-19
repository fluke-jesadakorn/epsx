use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub permissions: Vec<String>,
    pub description: String,
    pub priority: i32,
    pub is_default: bool,
    pub metadata: RoleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleMetadata {
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[allow(dead_code)]
impl Role {
    pub fn new(name: String, permissions: Vec<String>, description: String) -> Self {
        let now = DateTime::now();
        Self {
            id: ObjectId::new(),
            name,
            permissions,
            description,
            priority: 0,
            is_default: false,
            metadata: RoleMetadata {
                created_at: now,
                updated_at: now,
            },
        }
    }

    pub fn default_roles() -> Vec<Self> {
        vec![
            Self::new(
                "free".to_string(),
                vec!["read:basic".to_string()],
                "Free tier user with basic access".to_string(),
            ),
            Self::new(
                "personal".to_string(),
                vec![
                    "read:basic".to_string(),
                    "read:advanced".to_string(),
                    "write:basic".to_string(),
                ],
                "Personal plan subscriber".to_string(),
            ),
            Self::new(
                "company".to_string(),
                vec![
                    "read:basic".to_string(),
                    "read:advanced".to_string(),
                    "write:basic".to_string(),
                    "write:advanced".to_string(),
                ],
                "Company plan subscriber".to_string(),
            ),
            Self::new(
                "api".to_string(),
                vec![
                    "read:basic".to_string(),
                    "read:advanced".to_string(),
                    "write:basic".to_string(),
                    "write:advanced".to_string(),
                    "api:access".to_string(),
                ],
                "API plan subscriber".to_string(),
            ),
        ]
    }
}
