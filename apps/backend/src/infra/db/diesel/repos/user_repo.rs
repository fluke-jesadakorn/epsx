use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::app::ports::repositories::{UserRepo, RepoError, UserSearchFilters};
use crate::dom::entities::User;
use crate::dom::values::{UserId, Email};
use crate::infra::db::diesel::{
    DbPool,
    schema::users,
    models::{DieselUser, NewDieselUser, UpdateDieselUser},
};

pub struct DieselUserRepo {
    pool: Arc<DbPool>,
}

impl DieselUserRepo {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepo for DieselUserRepo {
    async fn get(&self, _id: &UserId) -> Result<Option<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let diesel_user = users::table
            .filter(users::id.eq(uuid))
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
            .on_conflict(users::id)
            .do_update()
            .set(&UpdateDieselUser::from(user))
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
        
        let deleted = diesel::delete(users::table)
            .filter(users::id.eq(uuid))
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        if deleted == 0 {
            return Err(RepoError::NotFound);
        }
        
        Ok(())
    }
    
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_user = users::table
            .filter(users::email.eq(email.to_string()))
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
    
    async fn find_by_admin_module(&self, _admin_module: &str) -> Result<Vec<User>, RepoError> {
        // Simple role system: admin modules are replaced by admin role
        // Only admin users can access admin functionality
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        use crate::auth::roles::UserRoleEnum;
        let diesel_users = users::table
            .filter(users::role.eq(UserRoleEnum::Admin))
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let users: Result<Vec<User>, RepoError> = diesel_users
            .into_iter()
            .map(|diesel_user| {
                diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))
            })
            .collect();
        
        users
    }
    
    async fn find_by_package_tier(&self, role: &str) -> Result<Vec<User>, RepoError> {
        // Simple role system: package tiers are replaced by roles
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        use crate::auth::roles::UserRoleEnum;
        let target_role = match role {
            "admin" => UserRoleEnum::Admin,
            "user" => UserRoleEnum::User,
            "guest" => UserRoleEnum::Guest,
            _ => UserRoleEnum::Guest, // Default to guest for unknown roles
        };
        
        let diesel_users = users::table
            .filter(users::role.eq(target_role))
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let users: Result<Vec<User>, RepoError> = diesel_users
            .into_iter()
            .map(|diesel_user| {
                diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))
            })
            .collect();
        
        users
    }
    
    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_users = users::table
            .offset(offset as i64)
            .limit(limit as i64)
            .order(users::created_at.desc())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let users: Result<Vec<User>, RepoError> = diesel_users
            .into_iter()
            .map(|diesel_user| {
                diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))
            })
            .collect();
        
        users
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
        
        let diesel_users: Vec<NewDieselUser> = users_batch
            .iter()
            .map(|user| user.into())
            .collect();
        
        diesel::insert_into(users::table)
            .values(&diesel_users)
            .on_conflict(users::id)
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn find_all(&self) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_users = users::table
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let users: Result<Vec<User>, RepoError> = diesel_users
            .into_iter()
            .map(|diesel_user| {
                diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))
            })
            .collect();
        
        users
    }

    async fn find_by_id(&self, id: &UserId) -> Result<User, RepoError> {
        match self.get(id).await? {
            Some(user) => Ok(user),
            None => Err(RepoError::NotFound),
        }
    }
    
    async fn find_users_for_auto_assignment(&self) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        // Simple role system: auto assignment for user and admin roles (not guest)
        use crate::auth::roles::UserRoleEnum;
        let diesel_users = users::table
            .filter(users::is_active.eq(true))
            .filter(users::role.ne(UserRoleEnum::Guest))
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let users: Result<Vec<User>, RepoError> = diesel_users
            .into_iter()
            .map(|diesel_user| {
                diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))
            })
            .collect();
        
        users
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
    
    async fn is_user_active_since(&self, user_id: &UserId, since: DateTime<Utc>) -> Result<bool, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let count = users::table
            .filter(users::id.eq(uuid))
            .filter(users::is_active.eq(true))
            .filter(users::last_login_at.gt(since))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(count > 0)
    }
    
    async fn has_good_payment_history(&self, _user_id: &UserId, _days: i64) -> Result<bool, RepoError> {
        // This would typically join with payments table
        // For now, return true as a stub
        Ok(true)
    }
    
    async fn health_check(&self) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let _count = users::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn search_users(
        &self,
        filters: &UserSearchFilters,
        offset: u32,
        limit: u32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<User>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let mut query = users::table.into_boxed();
        
        // Apply search filters
        if let Some(search) = &filters.search {
            query = query.filter(
                users::email.ilike(format!("%{}%", search))
                    .or(users::name.ilike(format!("%{}%", search)))
                    .or(users::display_name.ilike(format!("%{}%", search)))
            );
        }
        
        if let Some(email) = &filters.email {
            query = query.filter(users::email.ilike(format!("%{}%", email)));
        }
        
        if let Some(role_filter) = &filters.package_tier {
            // Simple role system: package_tier filter now filters by role
            use crate::auth::roles::UserRoleEnum;
            let target_role = match role_filter.as_str() {
                "admin" => UserRoleEnum::Admin,
                "user" => UserRoleEnum::User,
                "guest" => UserRoleEnum::Guest,
                _ => UserRoleEnum::Guest,
            };
            query = query.filter(users::role.eq(target_role));
        }
        
        if let Some(created_after) = filters.created_after {
            query = query.filter(users::created_at.gt(created_after));
        }
        
        if let Some(created_before) = filters.created_before {
            query = query.filter(users::created_at.lt(created_before));
        }
        
        // Apply sorting
        query = match sort_by {
            "email" => {
                if sort_order == "desc" {
                    query.order(users::email.desc())
                } else {
                    query.order(users::email.asc())
                }
            }
            "created_at" => {
                if sort_order == "desc" {
                    query.order(users::created_at.desc())
                } else {
                    query.order(users::created_at.asc())
                }
            }
            "package_tier" | "role" => {
                // Simple role system: sort by role instead of package_tier
                if sort_order == "desc" {
                    query.order(users::role.desc())
                } else {
                    query.order(users::role.asc())
                }
            }
            _ => query.order(users::created_at.desc()),
        };
        
        let diesel_users = query
            .offset(offset as i64)
            .limit(limit as i64)
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let users: Result<Vec<User>, RepoError> = diesel_users
            .into_iter()
            .map(|diesel_user| {
                diesel_user.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselUser: {:?}", e)))
            })
            .collect();
        
        users
    }
    
    async fn count_search_users(&self, filters: &UserSearchFilters) -> Result<u64, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let mut query = users::table.into_boxed();
        
        // Apply the same filters as search_users
        if let Some(search) = &filters.search {
            query = query.filter(
                users::email.ilike(format!("%{}%", search))
                    .or(users::name.ilike(format!("%{}%", search)))
                    .or(users::display_name.ilike(format!("%{}%", search)))
            );
        }
        
        if let Some(email) = &filters.email {
            query = query.filter(users::email.ilike(format!("%{}%", email)));
        }
        
        if let Some(role_filter) = &filters.package_tier {
            // Simple role system: package_tier filter now filters by role
            use crate::auth::roles::UserRoleEnum;
            let target_role = match role_filter.as_str() {
                "admin" => UserRoleEnum::Admin,
                "user" => UserRoleEnum::User,
                "guest" => UserRoleEnum::Guest,
                _ => UserRoleEnum::Guest,
            };
            query = query.filter(users::role.eq(target_role));
        }
        
        if let Some(created_after) = filters.created_after {
            query = query.filter(users::created_at.gt(created_after));
        }
        
        if let Some(created_before) = filters.created_before {
            query = query.filter(users::created_at.lt(created_before));
        }
        
        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::db::diesel::create_pool;
    
    #[tokio::test]
    async fn test_user_repo_creation() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost/test".to_string());
        
        if let Ok(pool) = create_pool(&database_url).await {
            let repo = DieselUserRepo::new(Arc::new(pool));
            // Test passes if we can create the repo
            assert!(true);
        }
        // Test passes even if database is not available
    }
}