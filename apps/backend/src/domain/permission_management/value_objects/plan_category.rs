use crate::prelude::*;

/// Plan category controlling stacking rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanCategory {
    Base,
    Addon,
    System,
    Exclusive,
}

impl PlanCategory {
    pub fn from_str(s: &str) -> AppResult<Self> {
        match s {
            "base" => Ok(Self::Base),
            "addon" => Ok(Self::Addon),
            "system" => Ok(Self::System),
            "exclusive" => Ok(Self::Exclusive),
            _ => Err(AppError::validation_error(format!(
                "Invalid plan category: '{}'. Must be base, addon, system, or exclusive",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Base => "base",
            Self::Addon => "addon",
            Self::System => "system",
            Self::Exclusive => "exclusive",
        }
    }

    /// Whether multiple plans of this category can stack on a wallet
    pub fn allows_stacking(&self) -> bool {
        matches!(self, Self::Addon | Self::System)
    }

    /// Maximum plans of this category per wallet
    pub fn max_per_wallet(&self) -> Option<usize> {
        match self {
            Self::Base => Some(1),
            Self::Addon => None, // unlimited
            Self::System => None,
            Self::Exclusive => Some(3),
        }
    }
}

impl std::fmt::Display for PlanCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for PlanCategory {
    fn default() -> Self {
        Self::Base
    }
}
