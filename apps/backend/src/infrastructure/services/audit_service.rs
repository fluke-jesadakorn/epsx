use crate::infrastructure::models::audit::NewUnifiedAuditDb;
use crate::prelude::TlsPool;
use crate::schemas::infra_logs::unified_audit_log;
use axum::http::HeaderMap;
use diesel_async::RunQueryDsl;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// Service for writing to unified_audit_log (fire-and-forget or sync)
#[derive(Clone)]
pub struct AuditService {
    pool: Arc<&'static TlsPool>,
}

impl AuditService {
    pub fn new(pool: Arc<&'static TlsPool>) -> Self {
        Self { pool }
    }

    /// Fire-and-forget: spawns a tokio task to insert the audit row
    pub fn log(&self, ctx: AuditCtx, entry: AuditEntry) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::insert(&pool, &ctx, &entry).await {
                tracing::warn!("audit log failed: {:?}", e);
            }
        });
    }

    /// Synchronous insert (awaitable)
    pub async fn log_sync(&self, ctx: &AuditCtx, entry: &AuditEntry) -> anyhow::Result<()> {
        Self::insert(&self.pool, ctx, entry).await
    }

    async fn insert(
        pool: &Arc<&'static TlsPool>,
        ctx: &AuditCtx,
        entry: &AuditEntry,
    ) -> anyhow::Result<()> {
        let mut conn = pool
            .get()
            .await
            .map_err(|e| anyhow::anyhow!("audit pool error: {:?}", e))?;

        let row = NewUnifiedAuditDb {
            actor: ctx.actor.clone(),
            actor_type: ctx.actor_type.clone(),
            resource_type: entry.resource_type.clone(),
            resource_id: entry.resource_id.clone(),
            action: entry.action.clone(),
            effect: entry.effect.clone(),
            before_state: entry.before_state.clone(),
            after_state: entry.after_state.clone(),
            ip_address: ctx.ip_address.clone(),
            user_agent: ctx.user_agent.clone(),
            metadata: entry.metadata.clone(),
            category: entry.category.clone(),
        };

        diesel::insert_into(unified_audit_log::table)
            .values(&row)
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}

/// WHO + request context, extracted from headers
#[derive(Debug, Clone)]
pub struct AuditCtx {
    pub actor: Option<String>,
    pub actor_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditCtx {
    /// Build from headers + authenticated wallet address from JWT
    pub fn from_wallet(wallet: &str, headers: &HeaderMap) -> Self {
        let ip_address = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            });

        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            actor: Some(wallet.to_string()),
            actor_type: "admin".to_string(),
            ip_address,
            user_agent,
        }
    }

    /// Fallback: try x-wallet-address header (deprecated, prefer from_wallet)
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let actor = headers
            .get("x-wallet-address")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self::from_wallet(actor.as_deref().unwrap_or("unknown"), headers)
    }

    pub fn system() -> Self {
        Self {
            actor: Some("system".to_string()),
            actor_type: "system".to_string(),
            ip_address: None,
            user_agent: None,
        }
    }
}

/// WHAT / ACTION / EFFECT / CHANGE - builder pattern
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub effect: String,
    pub before_state: Option<JsonValue>,
    pub after_state: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub category: String,
}

impl AuditEntry {
    pub fn new(resource_type: &str, action: &str, category: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: None,
            action: action.to_string(),
            effect: "success".to_string(),
            before_state: None,
            after_state: None,
            metadata: None,
            category: category.to_string(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.resource_id = Some(id.to_string());
        self
    }

    pub fn effect(mut self, effect: &str) -> Self {
        self.effect = effect.to_string();
        self
    }

    pub fn before(mut self, state: JsonValue) -> Self {
        self.before_state = Some(state);
        self
    }

    pub fn after(mut self, state: JsonValue) -> Self {
        self.after_state = Some(state);
        self
    }

    pub fn meta(mut self, meta: JsonValue) -> Self {
        self.metadata = Some(meta);
        self
    }
}
