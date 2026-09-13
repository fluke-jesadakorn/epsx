//! Payment Context Repository Adapter (Infrastructure Layer)
//! PostgreSQL implementation for payment context persistence using sqlx
//!
//! BIG-BANG: migrated to sqlx (real). All diesel DSL/derive replaced with raw SQL.

use crate::domain::payment::repository_ports::payment_context_port::PaymentContextRepositoryPort;
use crate::prelude::*;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// DATABASE MODELS
// ============================================================================

/// Database model for payment_contexts table (sqlx)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentContextDb {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub amount: Option<BigDecimal>,
    pub currency: Option<String>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub max_uses: Option<Option<i32>>,
    pub is_active: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Search criteria for listing payment contexts
#[derive(Debug, Clone, Default)]
pub struct PaymentContextSearchCriteria {
    pub context_type: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone)]
pub struct PaymentContextRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl PaymentContextRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    async fn save_impl(&self, context: NewPaymentContextDb) -> AppResult<PaymentContextDb> {
        sqlx::query_as::<_, PaymentContextDb>(
            r#"
            INSERT INTO payment_contexts (
                id, context_type, context_id, slug, name, description,
                amount, currency, expires_at, max_uses, current_uses,
                is_active, created_by, metadata, version, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14, 1, NOW(), NOW()
            )
            ON CONFLICT (id) DO UPDATE SET
                context_type = EXCLUDED.context_type,
                context_id = EXCLUDED.context_id,
                slug = EXCLUDED.slug,
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                amount = EXCLUDED.amount,
                currency = EXCLUDED.currency,
                expires_at = EXCLUDED.expires_at,
                max_uses = EXCLUDED.max_uses,
                current_uses = EXCLUDED.current_uses,
                is_active = EXCLUDED.is_active,
                created_by = EXCLUDED.created_by,
                metadata = EXCLUDED.metadata,
                version = payment_contexts.version + 1,
                updated_at = NOW()
            RETURNING id, context_type, context_id, slug, name, description,
                      amount, currency, expires_at, max_uses, current_uses,
                      is_active, created_by, metadata, version, created_at, updated_at
            "#,
        )
        .bind(context.id)
        .bind(&context.context_type)
        .bind(context.context_id)
        .bind(&context.slug)
        .bind(&context.name)
        .bind(&context.description)
        .bind(&context.amount)
        .bind(&context.currency)
        .bind(context.expires_at)
        .bind(context.max_uses)
        .bind(context.current_uses)
        .bind(context.is_active)
        .bind(&context.created_by)
        .bind(&context.metadata)
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("save payment_context: {}", e)))
    }

    async fn find_by_id_impl(&self, id: Uuid) -> AppResult<Option<PaymentContextDb>> {
        sqlx::query_as::<_, PaymentContextDb>(
            "SELECT id, context_type, context_id, slug, name, description, \
                    amount, currency, expires_at, max_uses, current_uses, \
                    is_active, created_by, metadata, version, created_at, updated_at \
             FROM payment_contexts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("find_by_id: {}", e)))
    }

    async fn find_by_slug_impl(&self, slug: &str) -> AppResult<Option<PaymentContextDb>> {
        sqlx::query_as::<_, PaymentContextDb>(
            "SELECT id, context_type, context_id, slug, name, description, \
                    amount, currency, expires_at, max_uses, current_uses, \
                    is_active, created_by, metadata, version, created_at, updated_at \
             FROM payment_contexts WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("find_by_slug: {}", e)))
    }

    async fn find_all_impl(
        &self,
        criteria: PaymentContextSearchCriteria,
    ) -> AppResult<Vec<PaymentContextDb>> {
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "SELECT id, context_type, context_id, slug, name, description, \
                    amount, currency, expires_at, max_uses, current_uses, \
                    is_active, created_by, metadata, version, created_at, updated_at \
             FROM payment_contexts WHERE 1=1",
        );
        if let Some(active) = criteria.is_active {
            qb.push(" AND is_active = ").push_bind(active);
        }
        if let Some(ctype) = criteria.context_type {
            qb.push(" AND context_type = ").push_bind(ctype);
        }
        if let Some(creator) = criteria.created_by {
            qb.push(" AND created_by = ").push_bind(creator);
        }
        qb.push(" ORDER BY created_at DESC");
        if let Some(limit) = criteria.limit {
            qb.push(" LIMIT ").push_bind(limit);
        }
        if let Some(offset) = criteria.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }
        qb.build_query_as::<PaymentContextDb>()
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("find_all: {}", e)))
    }

    async fn update_impl(
        &self,
        id: Uuid,
        changeset: UpdatePaymentContextDb,
    ) -> AppResult<PaymentContextDb> {
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
            sqlx::QueryBuilder::new("UPDATE payment_contexts SET updated_at = NOW()");
        if let Some(name) = changeset.name {
            qb.push(", name = ").push_bind(name);
        }
        if let Some(description) = changeset.description {
            qb.push(", description = ").push_bind(description);
        }
        if let Some(amount) = changeset.amount {
            qb.push(", amount = ").push_bind(amount);
        }
        if let Some(currency) = changeset.currency {
            qb.push(", currency = ").push_bind(currency);
        }
        if let Some(expires_at) = changeset.expires_at {
            qb.push(", expires_at = ").push_bind(expires_at);
        }
        if let Some(max_uses) = changeset.max_uses {
            qb.push(", max_uses = ").push_bind(max_uses);
        }
        if let Some(is_active) = changeset.is_active {
            qb.push(", is_active = ").push_bind(is_active);
        }
        if let Some(metadata) = changeset.metadata {
            qb.push(", metadata = ").push_bind(metadata);
        }
        qb.push(", version = version + 1 WHERE id = ").push_bind(id);
        qb.push(
            " RETURNING id, context_type, context_id, slug, name, description, \
                            amount, currency, expires_at, max_uses, current_uses, \
                            is_active, created_by, metadata, version, created_at, updated_at",
        );

        qb.build_query_as::<PaymentContextDb>()
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("update: {}", e)))
    }

    async fn soft_delete_impl(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(
            "UPDATE payment_contexts SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(self.db_pool.as_ref())
        .await
        .map(|_| ())
        .map_err(|e| AppError::database_error(format!("soft_delete: {}", e)))
    }

    async fn increment_usage_impl(&self, id: Uuid) -> AppResult<PaymentContextDb> {
        sqlx::query_as::<_, PaymentContextDb>(
            "UPDATE payment_contexts \
             SET current_uses = current_uses + 1, updated_at = NOW() \
             WHERE id = $1 \
             RETURNING id, context_type, context_id, slug, name, description, \
                       amount, currency, expires_at, max_uses, current_uses, \
                       is_active, created_by, metadata, version, created_at, updated_at",
        )
        .bind(id)
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("increment_usage: {}", e)))
    }

    async fn count_impl(&self, criteria: PaymentContextSearchCriteria) -> AppResult<i64> {
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM payment_contexts WHERE 1=1");
        if let Some(active) = criteria.is_active {
            qb.push(" AND is_active = ").push_bind(active);
        }
        if let Some(ctype) = criteria.context_type {
            qb.push(" AND context_type = ").push_bind(ctype);
        }
        if let Some(creator) = criteria.created_by {
            qb.push(" AND created_by = ").push_bind(creator);
        }
        let count: (i64,) = qb
            .build_query_as()
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("count: {}", e)))?;
        Ok(count.0)
    }

    async fn find_expired_impl(&self) -> AppResult<Vec<PaymentContextDb>> {
        sqlx::query_as::<_, PaymentContextDb>(
            "SELECT id, context_type, context_id, slug, name, description, \
                    amount, currency, expires_at, max_uses, current_uses, \
                    is_active, created_by, metadata, version, created_at, updated_at \
             FROM payment_contexts \
             WHERE is_active = TRUE AND expires_at < NOW()",
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("find_expired: {}", e)))
    }
}

#[async_trait]
impl PaymentContextRepositoryPort for PaymentContextRepositoryAdapter {
    async fn save(&self, context: NewPaymentContextDb) -> AppResult<PaymentContextDb> {
        self.save_impl(context).await
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PaymentContextDb>> {
        self.find_by_id_impl(id).await
    }

    async fn find_by_slug(&self, slug: &str) -> AppResult<Option<PaymentContextDb>> {
        self.find_by_slug_impl(slug).await
    }

    async fn find_all(
        &self,
        criteria: PaymentContextSearchCriteria,
    ) -> AppResult<Vec<PaymentContextDb>> {
        self.find_all_impl(criteria).await
    }

    async fn update(
        &self,
        id: Uuid,
        changeset: UpdatePaymentContextDb,
    ) -> AppResult<PaymentContextDb> {
        self.update_impl(id, changeset).await
    }

    async fn soft_delete(&self, id: Uuid) -> AppResult<()> {
        self.soft_delete_impl(id).await
    }

    async fn increment_usage(&self, id: Uuid) -> AppResult<PaymentContextDb> {
        self.increment_usage_impl(id).await
    }

    async fn count(&self, criteria: PaymentContextSearchCriteria) -> AppResult<i64> {
        self.count_impl(criteria).await
    }

    async fn find_expired(&self) -> AppResult<Vec<PaymentContextDb>> {
        self.find_expired_impl().await
    }
}

#[allow(dead_code)]
pub fn is_context_usable(context: &PaymentContextDb) -> bool {
    if !context.is_active {
        return false;
    }
    if let Some(max) = context.max_uses {
        if context.current_uses >= max {
            return false;
        }
    }
    if let Some(expires) = context.expires_at {
        if expires < Utc::now() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usable_when_active_with_remaining_uses() {
        let now = Utc::now();
        let c = PaymentContextDb {
            id: Uuid::new_v4(),
            context_type: "subscription".to_string(),
            context_id: None,
            slug: "s".to_string(),
            name: "n".to_string(),
            description: None,
            amount: BigDecimal::from(0),
            currency: "USD".to_string(),
            expires_at: Some(now + chrono::Duration::days(7)),
            max_uses: Some(5),
            current_uses: 1,
            is_active: true,
            created_by: "system".to_string(),
            metadata: serde_json::json!({}),
            version: 1,
            created_at: now,
            updated_at: now,
        };
        assert!(is_context_usable(&c));
    }

    #[test]
    fn unusable_when_expired() {
        let now = Utc::now();
        let c = PaymentContextDb {
            id: Uuid::new_v4(),
            context_type: "subscription".to_string(),
            context_id: None,
            slug: "s".to_string(),
            name: "n".to_string(),
            description: None,
            amount: BigDecimal::from(0),
            currency: "USD".to_string(),
            expires_at: Some(now - chrono::Duration::days(1)),
            max_uses: None,
            current_uses: 0,
            is_active: true,
            created_by: "system".to_string(),
            metadata: serde_json::json!({}),
            version: 1,
            created_at: now,
            updated_at: now,
        };
        assert!(!is_context_usable(&c));
    }
}
