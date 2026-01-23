//! Payment Context Repository Adapter (Infrastructure Layer)
//! PostgreSQL implementation for payment context persistence using Diesel
//!
//! NOTE: This requires the `payment_contexts` table to exist. Run migrations first:
//! ```bash
//! cd apps/backend
//! diesel migration run --migration-dir diesel_migrations_payments
//! ```

use crate::prelude::*;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ============================================================================
// DATABASE MODELS
// ============================================================================

/// Database model for payment_contexts table
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::payments::payment_contexts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PaymentContextDb {
    pub id: Uuid,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub currency: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    pub is_active: bool,
    pub created_by: String,
    pub metadata: serde_json::Value,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New payment context for insert
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::payments::payment_contexts)]
pub struct NewPaymentContextDb {
    pub id: Uuid,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub currency: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    pub is_active: bool,
    pub created_by: String,
    pub metadata: serde_json::Value,
}

/// Changeset for updating payment context
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schemas::payments::payment_contexts)]
pub struct UpdatePaymentContextDb {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub amount: Option<BigDecimal>,
    pub currency: Option<String>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub max_uses: Option<Option<i32>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// SEARCH CRITERIA
// ============================================================================

/// Search criteria for payment contexts
#[derive(Debug, Clone, Default)]
pub struct PaymentContextSearchCriteria {
    pub context_type: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// REPOSITORY ADAPTER
// ============================================================================

/// PostgreSQL payment context repository adapter
#[derive(Clone)]
pub struct PaymentContextRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl PaymentContextRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }

    /// Save a new payment context
    pub async fn save(&self, context: NewPaymentContextDb) -> AppResult<PaymentContextDb> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        info!("Saving payment context: {} ({})", context.name, context.slug);

        let result = diesel::insert_into(payment_contexts::table)
            .values(&context)
            .returning(PaymentContextDb::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save payment context: {}", e);
                AppError::database_error(format!("Failed to save payment context: {}", e))
            })?;

        info!("Successfully saved payment context: {}", result.id);
        Ok(result)
    }

    /// Find payment context by ID
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PaymentContextDb>> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        debug!("Finding payment context by ID: {}", id);

        let result = payment_contexts::table
            .filter(payment_contexts::id.eq(id))
            .first::<PaymentContextDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find payment context by ID {}: {}", id, e);
                AppError::database_error(format!("Failed to find payment context: {}", e))
            })?;

        Ok(result)
    }

    /// Find payment context by slug
    pub async fn find_by_slug(&self, slug: &str) -> AppResult<Option<PaymentContextDb>> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        debug!("Finding payment context by slug: {}", slug);

        let result = payment_contexts::table
            .filter(payment_contexts::slug.eq(slug))
            .first::<PaymentContextDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find payment context by slug {}: {}", slug, e);
                AppError::database_error(format!("Failed to find payment context: {}", e))
            })?;

        Ok(result)
    }

    /// Find all payment contexts with criteria
    pub async fn find_all(
        &self,
        criteria: PaymentContextSearchCriteria,
    ) -> AppResult<Vec<PaymentContextDb>> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        debug!("Finding all payment contexts with criteria: {:?}", criteria);

        let mut query = payment_contexts::table.into_boxed();

        if let Some(ref context_type) = criteria.context_type {
            query = query.filter(payment_contexts::context_type.eq(context_type));
        }

        if let Some(is_active) = criteria.is_active {
            query = query.filter(payment_contexts::is_active.eq(is_active));
        }

        if let Some(ref created_by) = criteria.created_by {
            query = query.filter(payment_contexts::created_by.eq(created_by));
        }

        if let Some(limit) = criteria.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = criteria.offset {
            query = query.offset(offset);
        }

        query = query.order(payment_contexts::created_at.desc());

        let results = query.load::<PaymentContextDb>(&mut conn).await.map_err(|e| {
            error!("Failed to find payment contexts: {}", e);
            AppError::database_error(format!("Failed to find payment contexts: {}", e))
        })?;

        info!("Found {} payment contexts", results.len());
        Ok(results)
    }

    /// Update a payment context
    pub async fn update(
        &self,
        id: Uuid,
        changeset: UpdatePaymentContextDb,
    ) -> AppResult<PaymentContextDb> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        info!("Updating payment context: {}", id);

        let result = diesel::update(payment_contexts::table.filter(payment_contexts::id.eq(id)))
            .set(&changeset)
            .returning(PaymentContextDb::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update payment context: {}", e);
                AppError::database_error(format!("Failed to update payment context: {}", e))
            })?;

        info!("Successfully updated payment context: {}", id);
        Ok(result)
    }

    /// Soft delete (deactivate) a payment context
    pub async fn soft_delete(&self, id: Uuid) -> AppResult<()> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        warn!("Soft deleting payment context: {}", id);

        diesel::update(payment_contexts::table.filter(payment_contexts::id.eq(id)))
            .set((
                payment_contexts::is_active.eq(false),
                payment_contexts::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete payment context: {}", e);
                AppError::database_error(format!("Failed to delete payment context: {}", e))
            })?;

        info!("Successfully soft deleted payment context: {}", id);
        Ok(())
    }

    /// Increment usage count
    pub async fn increment_usage(&self, id: Uuid) -> AppResult<PaymentContextDb> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        info!("Incrementing usage for payment context: {}", id);

        let result = diesel::update(payment_contexts::table.filter(payment_contexts::id.eq(id)))
            .set((
                payment_contexts::current_uses.eq(payment_contexts::current_uses + 1),
                payment_contexts::updated_at.eq(Utc::now()),
            ))
            .returning(PaymentContextDb::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to increment usage: {}", e);
                AppError::database_error(format!("Failed to increment usage: {}", e))
            })?;

        info!(
            "Usage incremented to {} for payment context: {}",
            result.current_uses, id
        );
        Ok(result)
    }

    /// Count payment contexts matching criteria
    pub async fn count(&self, criteria: PaymentContextSearchCriteria) -> AppResult<i64> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        let mut query = payment_contexts::table.into_boxed();

        if let Some(ref context_type) = criteria.context_type {
            query = query.filter(payment_contexts::context_type.eq(context_type));
        }

        if let Some(is_active) = criteria.is_active {
            query = query.filter(payment_contexts::is_active.eq(is_active));
        }

        if let Some(ref created_by) = criteria.created_by {
            query = query.filter(payment_contexts::created_by.eq(created_by));
        }

        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count payment contexts: {}", e);
                AppError::database_error(format!("Failed to count payment contexts: {}", e))
            })?;

        Ok(count)
    }

    /// Find expired but still active payment contexts
    pub async fn find_expired(&self) -> AppResult<Vec<PaymentContextDb>> {
        use crate::schemas::payments::payment_contexts;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now();

        let results = payment_contexts::table
            .filter(payment_contexts::is_active.eq(true))
            .filter(payment_contexts::expires_at.lt(now))
            .load::<PaymentContextDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find expired payment contexts: {}", e);
                AppError::database_error(format!("Failed to find expired contexts: {}", e))
            })?;

        info!("Found {} expired payment contexts", results.len());
        Ok(results)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if a payment context is usable
pub fn is_context_usable(context: &PaymentContextDb) -> bool {
    if !context.is_active {
        return false;
    }

    // Check expiration
    if let Some(expires_at) = context.expires_at {
        if Utc::now() > expires_at {
            return false;
        }
    }

    // Check usage limits
    if let Some(max_uses) = context.max_uses {
        if context.current_uses >= max_uses {
            return false;
        }
    }

    true
}

/// Compute link hash for smart contract verification
pub fn compute_link_hash(slug: &str) -> String {
    // Simple hash for now - proper keccak256 would need sha3 crate
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    slug.hash(&mut hasher);
    format!("0x{:064x}", hasher.finish())
}
