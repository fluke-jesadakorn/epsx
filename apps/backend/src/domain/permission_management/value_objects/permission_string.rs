use crate::prelude::*;

/// Permission string value object (platform:resource:action format)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionString(String);

impl PermissionString {
    pub fn new(permission: impl Into<String>) -> AppResult<Self> {
        let permission = permission.into();

        // Validate format: platform:resource:action (optionally :timestamp)
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() < 3 {
            return Err(AppError::validation_error(
                "Permission must follow format platform:resource:action"
            ));
        }

        // Validate each part is not empty
        if parts.iter().any(|p| p.is_empty()) {
            return Err(AppError::validation_error(
                "Permission parts cannot be empty"
            ));
        }

        Ok(Self(permission))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn value(&self) -> String {
        self.0.clone()
    }

    pub fn platform(&self) -> &str {
        self.0.split(':').next().unwrap_or("")
    }

    pub fn resource(&self) -> &str {
        self.0.split(':').nth(1).unwrap_or("")
    }

    pub fn action(&self) -> &str {
        self.0.split(':').nth(2).unwrap_or("")
    }
}

impl std::fmt::Display for PermissionString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
