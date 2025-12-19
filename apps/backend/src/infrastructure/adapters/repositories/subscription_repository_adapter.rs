/// Subscription Repository Adapter (Infrastructure Layer)
/// PostgreSQL implementation for subscription persistence using Diesel
/// NOTE: This is a simplified adapter that works directly with database models
/// to avoid complexity with domain model type mismatches (PlanId i32 vs UUID)

use crate::prelude::*;
use tracing::{info, error, debug, warn};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::infrastructure::models::payment::{SubscriptionDb, NewSubscriptionDb};
use crate::schema::subscriptions;

/// Search criteria for subscriptions
#[derive(Debug, Clone, Default)]
pub struct SubscriptionSearchCriteria {
    pub wallet_address: Option<String>,
    pub plan_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// PostgreSQL subscription repository adapter
#[derive(Clone)]
pub struct SubscriptionRepositoryAdapter {
    db_pool: &'static Pool<AsyncPgConnection>,
}

impl SubscriptionRepositoryAdapter {
    pub fn new(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { db_pool }
    }

    /// Find subscription by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SubscriptionDb>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        debug!("Finding subscription by ID: {}", id);

        let result = subscriptions::table
            .filter(subscriptions::id.eq(id))
            .first::<SubscriptionDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find subscription by ID {}: {}", id, e);
                AppError::database_error(format!("Failed to find subscription: {}", e))
            })?;

        Ok(result)
    }

    /// Find subscriptions by wallet address
    pub async fn find_by_wallet(&self, wallet_address: &str) -> AppResult<Vec<SubscriptionDb>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        debug!("Finding subscriptions for wallet: {}", wallet_address);

        let results = subscriptions::table
            .filter(subscriptions::wallet_address.eq(wallet_address))
            .order(subscriptions::started_at.desc().nulls_last())
            .load::<SubscriptionDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find subscriptions for wallet {}: {}", wallet_address, e);
                AppError::database_error(format!("Failed to find subscriptions: {}", e))
            })?;

        info!("Found {} subscriptions for wallet {}", results.len(), wallet_address);
        Ok(results)
    }

    /// Find all subscriptions with criteria
    pub async fn find_all(&self, criteria: SubscriptionSearchCriteria) -> AppResult<Vec<SubscriptionDb>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        debug!("Finding all subscriptions with criteria: {:?}", criteria);

        let mut query = subscriptions::table.into_boxed();

        if let Some(ref wallet_addr) = criteria.wallet_address {
            query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
        }

        if let Some(ref plan_id) = criteria.plan_id {
            query = query.filter(subscriptions::plan_id.eq(plan_id));
        }

        if let Some(ref status) = criteria.status {
            query = query.filter(subscriptions::status.eq(status));
        }

        if let Some(limit) = criteria.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = criteria.offset {
            query = query.offset(offset);
        }

        query = query.order(subscriptions::started_at.desc().nulls_last());

        let results = query
            .load::<SubscriptionDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find subscriptions: {}", e);
                AppError::database_error(format!("Failed to find subscriptions: {}", e))
            })?;

        info!("Found {} subscriptions matching criteria", results.len());
        Ok(results)
    }

    /// Save a new subscription
    pub async fn save(&self, subscription: NewSubscriptionDb) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        info!(
            "Saving subscription for wallet {} on plan {}",
            subscription.wallet_address, subscription.plan_id
        );

        diesel::insert_into(subscriptions::table)
            .values(&subscription)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save subscription: {}", e);
                AppError::database_error(format!("Failed to save subscription: {}", e))
            })?;

        info!("Successfully saved subscription");
        Ok(())
    }

    /// Update subscription status
    pub async fn update_status(&self, id: Uuid, status: &str) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        info!("Updating subscription {} status to {}", id, status);

        diesel::update(subscriptions::table.filter(subscriptions::id.eq(id)))
            .set(subscriptions::status.eq(status))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update subscription status: {}", e);
                AppError::database_error(format!("Failed to update subscription: {}", e))
            })?;

        info!("Successfully updated subscription status");
        Ok(())
    }

    /// Cancel subscription
    pub async fn cancel(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        warn!("Cancelling subscription: {}", id);

        diesel::update(subscriptions::table.filter(subscriptions::id.eq(id)))
            .set((
                subscriptions::status.eq("cancelled"),
                subscriptions::cancelled_at.eq(Some(Utc::now())),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to cancel subscription: {}", e);
                AppError::database_error(format!("Failed to cancel subscription: {}", e))
            })?;

        info!("Successfully cancelled subscription");
        Ok(())
    }

    /// Delete subscription
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        warn!("Deleting subscription: {}", id);

        diesel::delete(subscriptions::table.filter(subscriptions::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete subscription: {}", e);
                AppError::database_error(format!("Failed to delete subscription: {}", e))
            })?;

        info!("Successfully deleted subscription");
        Ok(())
    }

    /// Count subscriptions
    pub async fn count(&self, criteria: SubscriptionSearchCriteria) -> AppResult<i64> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get database connection: {}", e)))?;

        let mut query = subscriptions::table.into_boxed();

        if let Some(ref wallet_addr) = criteria.wallet_address {
            query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
        }

        if let Some(ref plan_id) = criteria.plan_id {
            query = query.filter(subscriptions::plan_id.eq(plan_id));
        }

        if let Some(ref status) = criteria.status {
            query = query.filter(subscriptions::status.eq(status));
        }

        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count subscriptions: {}", e);
                AppError::database_error(format!("Failed to count subscriptions: {}", e))
            })?;

        info!("Counted {} subscriptions matching criteria", count);
        Ok(count)
    }
}
