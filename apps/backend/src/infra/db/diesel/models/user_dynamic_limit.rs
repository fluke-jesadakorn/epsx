use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDateTime};
use crate::infra::db::diesel::schema::user_dynamic_limits;

/// Diesel model for user_dynamic_limits table
#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_dynamic_limits)]
pub struct DieselUserDynamicLimit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ranking_limit: Option<i32>,
    pub requests_per_minute: Option<i32>,
    pub requests_per_hour: Option<i32>,
    pub requests_per_day: Option<i32>,
    pub api_endpoints: Option<Vec<Option<String>>>,
    pub assigned_by: Uuid,
    pub reason: String,
    pub priority: i32,
    pub effective_from: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub previous_limits: Option<serde_json::Value>,
    pub change_source: String,
}

/// Insert model for creating new dynamic limits
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_dynamic_limits)]
pub struct NewDieselUserDynamicLimit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ranking_limit: Option<i32>,
    pub requests_per_minute: Option<i32>,
    pub requests_per_hour: Option<i32>,
    pub requests_per_day: Option<i32>,
    pub api_endpoints: Option<Vec<Option<String>>>,
    pub assigned_by: Uuid,
    pub reason: String,
    pub priority: i32,
    pub effective_from: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub previous_limits: Option<serde_json::Value>,
    pub change_source: String,
}

/// Update model for modifying existing dynamic limits
#[derive(AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_dynamic_limits)]
pub struct UpdateDieselUserDynamicLimit {
    pub ranking_limit: Option<i32>,
    pub requests_per_minute: Option<i32>,
    pub requests_per_hour: Option<i32>,
    pub requests_per_day: Option<i32>,
    pub api_endpoints: Option<Vec<Option<String>>>,
    pub reason: Option<String>,
    pub priority: Option<i32>,
    pub expires_at: Option<Option<NaiveDateTime>>,
    pub previous_limits: Option<serde_json::Value>,
    pub change_source: Option<String>,
}

impl Default for NewDieselUserDynamicLimit {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            ranking_limit: None,
            requests_per_minute: None,
            requests_per_hour: None,
            requests_per_day: None,
            api_endpoints: None,
            assigned_by: Uuid::new_v4(),
            reason: "Default assignment".to_string(),
            priority: 0,
            effective_from: Utc::now().naive_utc(),
            expires_at: None,
            previous_limits: None,
            change_source: "manual".to_string(),
        }
    }
}

/// Resolved user limits combining all sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedUserLimits {
    pub user_id: Uuid,
    pub ranking_limit: i32,          // -1 for unlimited, >= 0 for specific limit
    pub requests_per_minute: i32,    // -1 for unlimited, >= 0 for specific limit
    pub requests_per_hour: i32,      // -1 for unlimited, >= 0 for specific limit
    pub requests_per_day: Option<i32>, // Optional daily limit
    pub api_endpoints: Vec<String>,  // Cleaned up endpoint patterns
    pub source: LimitSource,         // How these limits were determined
    pub assigned_by: Option<Uuid>,   // Who assigned these limits (if dynamic)
    pub expires_at: Option<DateTime<Utc>>, // When these limits expire
}

/// Source of the resolved limits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LimitSource {
    /// From dynamic assignment in user_dynamic_limits table
    Dynamic {
        assignment_id: Uuid,
        reason: String,
        priority: i32,
    },
    /// Derived from user permissions (fallback)
    Permissions,
    /// System default (fallback when no permissions or dynamic limits)
    Default,
}

impl Default for ResolvedUserLimits {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            ranking_limit: 3,        // Free tier default
            requests_per_minute: 10, // Free tier default
            requests_per_hour: 100,  // Free tier default
            requests_per_day: None,
            api_endpoints: vec!["basic/*".to_string()],
            source: LimitSource::Default,
            assigned_by: None,
            expires_at: None,
        }
    }
}

impl ResolvedUserLimits {
    /// Check if user has unlimited ranking access
    pub fn has_unlimited_rankings(&self) -> bool {
        self.ranking_limit == -1
    }
    
    /// Check if user has unlimited API access
    pub fn has_unlimited_api_access(&self) -> bool {
        self.requests_per_minute == -1 || self.requests_per_hour == -1
    }
    
    /// Check if user can access a specific API endpoint
    pub fn can_access_endpoint(&self, endpoint: &str) -> bool {
        for pattern in &self.api_endpoints {
            if pattern == "*" || pattern == "**" {
                return true; // Wildcard access
            }
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len() - 2];
                if endpoint.starts_with(prefix) {
                    return true;
                }
            }
            if pattern == endpoint {
                return true; // Exact match
            }
        }
        false
    }
    
    /// Check if these limits are about to expire (within 24 hours)
    pub fn expires_soon(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = Utc::now();
            let twenty_four_hours = chrono::Duration::hours(24);
            expires_at <= now + twenty_four_hours
        } else {
            false
        }
    }
    
    /// Get a user-friendly description of the limits
    pub fn description(&self) -> String {
        let ranking_desc = if self.ranking_limit == -1 {
            "unlimited rankings".to_string()
        } else {
            format!("{} rankings", self.ranking_limit)
        };
        
        let api_desc = if self.requests_per_minute == -1 {
            "unlimited API access".to_string()
        } else {
            format!("{} req/min, {} req/hour", self.requests_per_minute, self.requests_per_hour)
        };
        
        format!("{}, {}", ranking_desc, api_desc)
    }
}