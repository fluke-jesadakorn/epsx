use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::app::ports::repositories::{UserRepository, RepoError, UserSearchFilters};
use crate::dom::entities::User;
use crate::dom::values::{UserId, Email};
use crate::infra::db::diesel::{
    DbPool,
    schema::users,
    models::{DieselUser, NewDieselUser},
};

pub struct DieselUserRepository {
    pool: Arc<DbPool>,
}

impl DieselUserRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for DieselUserRepository {
    async fn get(&self, _id: &UserId) -> Result<Option<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let diesel_user = users::table
            .filter(users::id.eq(uuid))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        match diesel_user {
            Some(diesel_user) => {
                let user = diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
                Ok(Some(user))
            }
            None => Ok(None)
        }
    }

    async fn save(&self, user: &User) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let new_user: NewDieselUser = user.into();
        
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    async fn delete(&self, _id: &UserId) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        diesel::delete(users::table.filter(users::id.eq(uuid)))
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_user = users::table
            .filter(users::email.eq(email.to_string()))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        match diesel_user {
            Some(diesel_user) => {
                let user = diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
                Ok(Some(user))
            }
            None => Ok(None)
        }
    }

    async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_user = users::table
            .filter(users::firebase_uid.eq(firebase_uid))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        match diesel_user {
            Some(diesel_user) => {
                let user = diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
                Ok(Some(user))
            }
            None => Ok(None)
        }
    }

    async fn find_by_package_tier(&self, package_tier: &str) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_users = users::table
            .filter(users::package_tier.eq(package_tier))
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut result = Vec::new();
        for diesel_user in diesel_users {
            let user = diesel_user.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
            result.push(user);
        }
        
        Ok(result)
    }

    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_users = users::table
            .offset(offset as i64)
            .limit(limit as i64)
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut result = Vec::new();
        for diesel_user in diesel_users {
            let user = diesel_user.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
            result.push(user);
        }
        
        Ok(result)
    }

    async fn count(&self) -> Result<u64, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let count = users::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(count as u64)
    }

    async fn save_batch(&self, users_batch: &[User]) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let new_users: Vec<NewDieselUser> = users_batch.iter().map(|u| u.into()).collect();
        
        diesel::insert_into(users::table)
            .values(&new_users)
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_users = users::table
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut result = Vec::new();
        for diesel_user in diesel_users {
            let user = diesel_user.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
            result.push(user);
        }
        
        Ok(result)
    }

    async fn find_by_id(&self, id: &UserId) -> Result<User, RepoError> {
        match self.get(id).await? {
            Some(user) => Ok(user),
            None => Err(RepoError::NotFound)
        }
    }

    async fn find_users_for_auto_assignment(&self) -> Result<Vec<User>, RepoError> {
        // Simple implementation - return active users with basic permissions
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_users = users::table
            .filter(users::is_active.eq(Some(true)))
            .limit(100)
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut result = Vec::new();
        for diesel_user in diesel_users {
            let user = diesel_user.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
            result.push(user);
        }
        
        Ok(result)
    }

    async fn count_total_users(&self) -> Result<i64, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let count = users::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(count)
    }

    async fn is_user_active_since(&self, _user_id: &UserId, _since: DateTime<Utc>) -> Result<bool, RepoError> {
        // Simple implementation
        Ok(true)
    }

    async fn has_good_payment_history(&self, _user_id: &UserId, _days: i64) -> Result<bool, RepoError> {
        // Simple implementation
        Ok(true)
    }

    async fn health_check(&self) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        // Simple health check
        users::table
            .limit(1)
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    async fn search_users(
        &self,
        filters: &UserSearchFilters,
        offset: u32,
        limit: u32,
        _sort_by: &str,
        _sort_order: &str
    ) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let mut query = users::table.into_boxed();
        
        if let Some(email) = &filters.email {
            query = query.filter(users::email.ilike(format!("%{}%", email)));
        }
        
        if let Some(package_tier) = &filters.package_tier {
            query = query.filter(users::package_tier.eq(package_tier));
        }
        
        let diesel_users = query
            .offset(offset as i64)
            .limit(limit as i64)
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut result = Vec::new();
        for diesel_user in diesel_users {
            let user = diesel_user.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))?;
            result.push(user);
        }
        
        Ok(result)
    }

    async fn count_search_users(&self, filters: &UserSearchFilters) -> Result<u64, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let mut query = users::table.into_boxed();
        
        if let Some(email) = &filters.email {
            query = query.filter(users::email.ilike(format!("%{}%", email)));
        }
        
        if let Some(package_tier) = &filters.package_tier {
            query = query.filter(users::package_tier.eq(package_tier));
        }
        
        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(count as u64)
    }
}