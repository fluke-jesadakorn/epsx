use crate::prelude::*;

/// Plan display group for pricing page sections
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanGroup {
    Personal,
    Enterprise,
    Api,
    Custom,
}

impl PlanGroup {
    pub fn from_str(s: &str) -> AppResult<Self> {
        match s {
            "personal" => Ok(Self::Personal),
            "enterprise" => Ok(Self::Enterprise),
            "api" => Ok(Self::Api),
            "custom" => Ok(Self::Custom),
            _ => Err(AppError::validation_error(format!(
                "Invalid plan group: '{}'. Must be personal, enterprise, api, or custom",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Personal => "personal",
            Self::Enterprise => "enterprise",
            Self::Api => "api",
            Self::Custom => "custom",
        }
    }
}

impl std::fmt::Display for PlanGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for PlanGroup {
    fn default() -> Self {
        Self::Personal
    }
}
