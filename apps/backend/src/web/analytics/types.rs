// Analytics API Types
// Shared types for analytics endpoints

use serde::Deserialize;

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub permissions: Vec<String>,
}

/// Analytics query parameters
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub granularity: Option<String>,
}
