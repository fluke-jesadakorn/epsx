use diesel::prelude::*;
use uuid::Uuid;
use diesel_async::{RunQueryDsl, AsyncPgConnection};

use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDateTime};
use std::result::Result;
use crate::app::ports::repositories::RepoError;
use crate::infra::db::diesel::models::{
    DieselUserDynamicLimit, NewDieselUserDynamicLimit, UpdateDieselUserDynamicLimit
};
use crate::infra::db::diesel::schema::user_dynamic_limits;

/// Repository for managing user dynamic limits
#[derive(Debug, Clone)]
pub struct UserDynamicLimitRepository {
    // This will be injected with the connection pool
}

impl UserDynamicLimitRepository {
    pub fn new() -> Self {
        Self {}
    }

    /// Get active dynamic limits for a user (effective and not expired)
    pub async fn get_active_limits_for_user(
        &self,
        conn: &mut AsyncPgConnection,
        user_id: &Uuid,
    ) -> Result<Vec<DieselUserDynamicLimit>, RepoError> {
        let now = Utc::now().naive_utc();
        
        let limits = user_dynamic_limits::table
            .filter(user_dynamic_limits::user_id.eq(user_id))
            .filter(user_dynamic_limits::effective_from.le(now))
            .filter(
                user_dynamic_limits::expires_at.is_null()
                    .or(user_dynamic_limits::expires_at.gt(now))
            )
            .order_by((
                user_dynamic_limits::priority.desc(),
                user_dynamic_limits::created_at.desc()
            ))
            .load::<DieselUserDynamicLimit>(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(limits)
    }

    /// Get the highest priority active limit for a user
    pub async fn get_effective_limit_for_user(
        &self,
        conn: &mut AsyncPgConnection,
        user_id: &Uuid,
    ) -> Result<Option<DieselUserDynamicLimit>, RepoError> {
        let limits = self.get_active_limits_for_user(conn, user_id).await?;
        Ok(limits.into_iter().next()) // Already ordered by priority desc
    }

    /// Create a new dynamic limit assignment
    pub async fn create_dynamic_limit(
        &self,
        conn: &mut AsyncPgConnection,
        new_limit: NewDieselUserDynamicLimit,
    ) -> Result<DieselUserDynamicLimit, RepoError> {
        let created_limit = diesel::insert_into(user_dynamic_limits::table)
            .values(&new_limit)
            .get_result(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(created_limit)
    }

    /// Update an existing dynamic limit
    pub async fn update_dynamic_limit(
        &self,
        conn: &mut AsyncPgConnection,
        limit_id: &Uuid,
        update: UpdateDieselUserDynamicLimit,
    ) -> Result<DieselUserDynamicLimit, RepoError> {
        let updated_limit = diesel::update(user_dynamic_limits::table.find(limit_id))
            .set(&update)
            .get_result(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(updated_limit)
    }

    /// Delete a dynamic limit assignment
    pub async fn delete_dynamic_limit(
        &self,
        conn: &mut AsyncPgConnection,
        limit_id: &Uuid,
    ) -> Result<bool, RepoError> {
        let deleted_count = diesel::delete(user_dynamic_limits::table.find(limit_id))
            .execute(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(deleted_count > 0)
    }

    /// Get all limits for a user (including expired ones) for audit purposes
    pub async fn get_all_limits_for_user(
        &self,
        conn: &mut AsyncPgConnection,
        user_id: &Uuid,
    ) -> Result<Vec<DieselUserDynamicLimit>, RepoError> {
        let limits = user_dynamic_limits::table
            .filter(user_dynamic_limits::user_id.eq(user_id))
            .order_by((
                user_dynamic_limits::created_at.desc(),
                user_dynamic_limits::priority.desc()
            ))
            .load::<DieselUserDynamicLimit>(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(limits)
    }

    /// Get limits assigned by a specific admin
    pub async fn get_limits_assigned_by(
        &self,
        conn: &mut AsyncPgConnection,
        assigned_by: &Uuid,
    ) -> Result<Vec<DieselUserDynamicLimit>, RepoError> {
        let limits = user_dynamic_limits::table
            .filter(user_dynamic_limits::assigned_by.eq(assigned_by))
            .order_by(user_dynamic_limits::created_at.desc())
            .load::<DieselUserDynamicLimit>(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(limits)
    }

    /// Get limits that are expiring soon (within specified hours)
    pub async fn get_expiring_limits(
        &self,
        conn: &mut AsyncPgConnection,
        within_hours: i64,
    ) -> Result<Vec<DieselUserDynamicLimit>, RepoError> {
        let now = Utc::now().naive_utc();
        let expiry_threshold = now + chrono::Duration::hours(within_hours);
        
        let limits = user_dynamic_limits::table
            .filter(user_dynamic_limits::expires_at.is_not_null())
            .filter(user_dynamic_limits::expires_at.le(expiry_threshold))
            .filter(user_dynamic_limits::expires_at.gt(now)) // Still active
            .order_by(user_dynamic_limits::expires_at.asc())
            .load::<DieselUserDynamicLimit>(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(limits)
    }

    /// Bulk create multiple dynamic limits (for bulk assignment)
    pub async fn create_bulk_dynamic_limits(
        &self,
        conn: &mut AsyncPgConnection,
        new_limits: Vec<NewDieselUserDynamicLimit>,
    ) -> Result<Vec<DieselUserDynamicLimit>, RepoError> {
        let created_limits = diesel::insert_into(user_dynamic_limits::table)
            .values(&new_limits)
            .get_results(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(created_limits)
    }

    /// Get usage statistics for limits (count by different criteria)
    pub async fn get_limit_statistics(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> Result<HashMap<String, i64>, RepoError> {
        // This would require more complex queries, implementing basic version
        let total_active = user_dynamic_limits::table
            .filter(
                user_dynamic_limits::expires_at.is_null()
                    .or(user_dynamic_limits::expires_at.gt(Utc::now().naive_utc()))
            )
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let total_expired = user_dynamic_limits::table
            .filter(
                user_dynamic_limits::expires_at.is_not_null()
                    .and(user_dynamic_limits::expires_at.le(Utc::now().naive_utc()))
            )
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut stats = HashMap::new();
        stats.insert("total_active".to_string(), total_active);
        stats.insert("total_expired".to_string(), total_expired);
        stats.insert("total_all".to_string(), total_active + total_expired);

        Ok(stats)
    }
}

impl Default for UserDynamicLimitRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating new dynamic limit assignments with validation
#[derive(Debug, Clone)]
pub struct DynamicLimitAssignmentBuilder {
    user_id: Uuid,
    assigned_by: Uuid,
    reason: String,
    ranking_limit: Option<i32>,
    requests_per_minute: Option<i32>,
    requests_per_hour: Option<i32>,
    requests_per_day: Option<i32>,
    api_endpoints: Option<Vec<String>>,
    priority: i32,
    expires_at: Option<NaiveDateTime>,
    change_source: String,
}

impl DynamicLimitAssignmentBuilder {
    pub fn new(user_id: Uuid, assigned_by: Uuid, reason: String) -> Self {
        Self {
            user_id,
            assigned_by,
            reason,
            ranking_limit: None,
            requests_per_minute: None,
            requests_per_hour: None,
            requests_per_day: None,
            api_endpoints: None,
            priority: 0,
            expires_at: None,
            change_source: "manual".to_string(),
        }
    }

    pub fn ranking_limit(mut self, limit: i32) -> Self {
        self.ranking_limit = Some(limit);
        self
    }

    pub fn api_limits(mut self, per_minute: i32, per_hour: i32) -> Self {
        self.requests_per_minute = Some(per_minute);
        self.requests_per_hour = Some(per_hour);
        self
    }

    pub fn daily_limit(mut self, per_day: i32) -> Self {
        self.requests_per_day = Some(per_day);
        self
    }

    pub fn api_endpoints(mut self, endpoints: Vec<String>) -> Self {
        self.api_endpoints = Some(endpoints);
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at.naive_utc());
        self
    }

    pub fn change_source(mut self, source: String) -> Self {
        self.change_source = source;
        self
    }

    pub fn build(self) -> Result<NewDieselUserDynamicLimit, String> {
        // Validation
        if self.ranking_limit.is_none() 
            && self.requests_per_minute.is_none() 
            && self.requests_per_hour.is_none() 
            && self.api_endpoints.is_none() {
            return Err("At least one limit must be specified".to_string());
        }

        if let Some(limit) = self.ranking_limit {
            if limit < -1 {
                return Err("Ranking limit must be >= -1".to_string());
            }
        }

        if let Some(limit) = self.requests_per_minute {
            if limit < -1 {
                return Err("Requests per minute must be >= -1".to_string());
            }
        }

        if let Some(limit) = self.requests_per_hour {
            if limit < -1 {
                return Err("Requests per hour must be >= -1".to_string());
            }
        }

        let api_endpoints = self.api_endpoints
            .map(|endpoints| endpoints.into_iter().map(Some).collect());

        Ok(NewDieselUserDynamicLimit {
            id: Uuid::new_v4(),
            user_id: self.user_id,
            ranking_limit: self.ranking_limit,
            requests_per_minute: self.requests_per_minute,
            requests_per_hour: self.requests_per_hour,
            requests_per_day: self.requests_per_day,
            api_endpoints,
            assigned_by: self.assigned_by,
            reason: self.reason,
            priority: self.priority,
            effective_from: Utc::now().naive_utc(),
            expires_at: self.expires_at,
            previous_limits: None, // Will be filled by business logic
            change_source: self.change_source,
        })
    }
}