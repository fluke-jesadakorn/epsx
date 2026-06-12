use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::domain::audit::repository::AuditLogRepository;
use crate::domain::shared_kernel::entities::audit::{AuditLogEntry, AuditQuery, AuditAction, ResourceType, AuditResult};
use crate::infrastructure::models::audit::{AuditLogDb, NewAuditLogDb};
use crate::infrastructure::database::diesel_connection_manager::{get_analytics_pool, get_payments_pool};
use crate::schemas::analytics::audit_logs;
use anyhow::{Result, Context};
use epsx_contracts::value_objects::UserId;
use chrono::{DateTime, Utc};
use diesel::sql_types::{Text, Nullable, Timestamptz, Jsonb, BigInt};

pub struct DieselAuditLogRepository;

impl Default for DieselAuditLogRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DieselAuditLogRepository {
    pub fn new() -> Self {
        Self
    }

    /// Query analytics DB with UNION ALL across 4 audit tables
    async fn query_analytics(
        query: &AuditQuery,
        category: Option<&str>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AuditLogEntry>, i64)> {
        let pool = get_analytics_pool().await?;
        let mut conn = pool.get().await
            .map_err(|e| anyhow::anyhow!("Analytics pool error: {:?}", e))?;

        // Build sub-selects based on category filter
        let mut unions: Vec<&str> = Vec::new();

        let audit_logs_sql = r#"
            SELECT id::text, wallet_address, action, resource_type, resource_id,
                   result, ip_address::text as ip_address, user_agent,
                   details, created_at, 'system' as category
            FROM audit_logs"#;

        let permission_sql = r#"
            SELECT id::text, wallet_address, event_type as action, 'Permission' as resource_type,
                   COALESCE(permission_string, group_name) as resource_id,
                   'Success' as result, ip_address::text as ip_address, user_agent,
                   jsonb_build_object(
                       'performed_by', performed_by,
                       'reason', reason,
                       'previous_state', previous_state,
                       'new_state', new_state
                   ) as details,
                   event_timestamp as created_at, 'permission' as category
            FROM permission_audit_log"#;

        let wallet_sql = r#"
            SELECT id::text, wallet_address, event_type as action, 'Wallet' as resource_type,
                   wallet_address as resource_id,
                   'Success' as result, NULL::text as ip_address, NULL::text as user_agent,
                   metadata as details, created_at, 'wallet' as category
            FROM wallet_activity_logs"#;

        let assignment_sql = r#"
            SELECT id::text, performed_by as wallet_address, action, 'Plan' as resource_type,
                   assignment_id::text as resource_id,
                   'Success' as result, NULL::text as ip_address, NULL::text as user_agent,
                   jsonb_build_object(
                       'old_value', old_value,
                       'new_value', new_value,
                       'reason', reason
                   ) as details,
                   performed_at as created_at, 'plan' as category
            FROM assignment_audit_log"#;

        let unified_base = r#"
            SELECT id::text, actor as wallet_address, action, resource_type, resource_id,
                   effect as result, ip_address::text as ip_address, user_agent,
                   COALESCE(
                       CASE WHEN before_state IS NOT NULL OR after_state IS NOT NULL
                            THEN jsonb_build_object('before', before_state, 'after', after_state)
                            ELSE NULL END,
                       metadata, '{}'::jsonb
                   ) as details,
                   created_at, category
            FROM unified_audit_log"#;

        // Whitelist category to prevent SQL injection — only known values are interpolated
        let valid_category = match category {
            Some("system") | Some("permission") | Some("wallet") | Some("plan") |
            Some("auth") | Some("developer") | Some("notification") | Some("payment") |
            Some("all") => category,
            Some(_) => None, // reject unknown categories
            None => None,
        };

        let unified_filtered = match valid_category {
            Some(cat) => format!("{} WHERE category = '{}'", unified_base, cat),
            None => unified_base.to_string(),
        };

        match valid_category {
            Some("system") => unions.push(audit_logs_sql),
            Some("permission") => unions.push(permission_sql),
            Some("wallet") => unions.push(wallet_sql),
            Some("plan") => unions.push(assignment_sql),
            Some("auth") | Some("developer") | Some("notification") => {}, // only in unified_audit_log
            _ => {
                unions.push(audit_logs_sql);
                unions.push(permission_sql);
                unions.push(wallet_sql);
                unions.push(assignment_sql);
            }
        }
        // Always include unified_audit_log (filtered by category when applicable)
        unions.push(&unified_filtered);

        let union_sql = unions.join("\n            UNION ALL\n");

        // Resolve search term from search param or wallet_address
        let bind_search: Option<String> = search
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| query.wallet_address.as_ref().map(|a| a.to_string()));
        let bind_from: Option<DateTime<Utc>> = query.from_date;
        let bind_to: Option<DateTime<Utc>> = query.to_date;

        // Always reference all 3 filter params using IS NULL trick
        let where_clause = "WHERE ($1::text IS NULL OR wallet_address ILIKE '%' || $1 || '%') AND ($2::timestamptz IS NULL OR created_at >= $2) AND ($3::timestamptz IS NULL OR created_at <= $3)";

        // Count query
        let count_sql = format!(
            "SELECT COUNT(*)::bigint as cnt FROM ({union}) unified {where_clause}",
            union = union_sql,
            where_clause = where_clause
        );

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = BigInt)]
            cnt: i64,
        }

        let count_result = diesel::sql_query(&count_sql)
            .bind::<Nullable<Text>, _>(&bind_search)
            .bind::<Nullable<Timestamptz>, _>(&bind_from)
            .bind::<Nullable<Timestamptz>, _>(&bind_to)
            .get_result::<CountRow>(&mut conn)
            .await
            .context("Failed to count analytics audit logs")?;

        // Data query
        let data_sql = format!(
            "SELECT id, wallet_address, action, resource_type, resource_id, result, ip_address, user_agent, details, created_at, category FROM ({union}) unified {where_clause} ORDER BY created_at DESC LIMIT $4 OFFSET $5",
            union = union_sql,
            where_clause = where_clause
        );

        #[derive(QueryableByName, Debug)]
        struct UnifiedRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Nullable<Text>)]
            wallet_address: Option<String>,
            #[diesel(sql_type = Text)]
            action: String,
            #[diesel(sql_type = Text)]
            resource_type: String,
            #[diesel(sql_type = Nullable<Text>)]
            resource_id: Option<String>,
            #[diesel(sql_type = Text)]
            result: String,
            #[diesel(sql_type = Nullable<Text>)]
            ip_address: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            user_agent: Option<String>,
            #[diesel(sql_type = Nullable<Jsonb>)]
            details: Option<serde_json::Value>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = Text)]
            category: String,
        }

        let rows = diesel::sql_query(&data_sql)
            .bind::<Nullable<Text>, _>(&bind_search)
            .bind::<Nullable<Timestamptz>, _>(&bind_from)
            .bind::<Nullable<Timestamptz>, _>(&bind_to)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .get_results::<UnifiedRow>(&mut conn)
            .await
            .context("Failed to query analytics audit logs")?;

        let entries: Vec<AuditLogEntry> = rows.into_iter().map(|r| {
            AuditLogEntry {
                id: r.id,
                wallet_address: r.wallet_address.map(UserId::from_string_unchecked),
                action: parse_action(&r.action),
                resource_type: parse_resource_type(&r.resource_type),
                resource_id: r.resource_id,
                result: parse_result(&r.result),
                ip_address: r.ip_address,
                user_agent: r.user_agent,
                additional_data: r.details,
                timestamp: r.created_at,
                category: Some(r.category),
                action_raw: Some(r.action),
                resource_type_raw: Some(r.resource_type),
            }
        }).collect();

        Ok((entries, count_result.cnt))
    }

    /// Query payments DB for payment_audit_log
    async fn query_payments(
        query: &AuditQuery,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AuditLogEntry>, i64)> {
        let pool = get_payments_pool().await?;
        let mut conn = pool.get().await
            .map_err(|e| anyhow::anyhow!("Payments pool error: {:?}", e))?;

        let bind_search: Option<String> = search
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| query.wallet_address.as_ref().map(|a| a.to_string()));
        let bind_from: Option<DateTime<Utc>> = query.from_date;
        let bind_to: Option<DateTime<Utc>> = query.to_date;

        let where_clause = "WHERE ($1::text IS NULL OR COALESCE(performed_by, '') ILIKE '%' || $1 || '%') AND ($2::timestamptz IS NULL OR created_at >= $2) AND ($3::timestamptz IS NULL OR created_at <= $3)";

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = BigInt)]
            cnt: i64,
        }

        let count_sql = format!(
            "SELECT COUNT(*)::bigint as cnt FROM payment_audit_log {where_clause}"
        );

        let count_result = diesel::sql_query(&count_sql)
            .bind::<Nullable<Text>, _>(&bind_search)
            .bind::<Nullable<Timestamptz>, _>(&bind_from)
            .bind::<Nullable<Timestamptz>, _>(&bind_to)
            .get_result::<CountRow>(&mut conn)
            .await
            .context("Failed to count payment audit logs")?;

        #[derive(QueryableByName, Debug)]
        struct PaymentRow {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Nullable<Text>)]
            wallet_address: Option<String>,
            #[diesel(sql_type = Text)]
            action: String,
            #[diesel(sql_type = Nullable<Text>)]
            resource_id: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            old_status: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            new_status: Option<String>,
            #[diesel(sql_type = Nullable<Text>)]
            reason: Option<String>,
            #[diesel(sql_type = Nullable<Jsonb>)]
            metadata: Option<serde_json::Value>,
            #[diesel(sql_type = Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let data_sql = format!(
            "SELECT id::text, performed_by as wallet_address, action, payment_id::text as resource_id, old_status, new_status, reason, metadata, created_at FROM payment_audit_log {where_clause} ORDER BY created_at DESC LIMIT $4 OFFSET $5"
        );

        let rows = diesel::sql_query(&data_sql)
            .bind::<Nullable<Text>, _>(&bind_search)
            .bind::<Nullable<Timestamptz>, _>(&bind_from)
            .bind::<Nullable<Timestamptz>, _>(&bind_to)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .get_results::<PaymentRow>(&mut conn)
            .await
            .context("Failed to query payment audit logs")?;

        let entries: Vec<AuditLogEntry> = rows.into_iter().map(|r| {
            let mut details = r.metadata.unwrap_or(serde_json::json!({}));
            if let Some(obj) = details.as_object_mut() {
                if let Some(old) = &r.old_status { obj.insert("old_status".into(), serde_json::json!(old)); }
                if let Some(new) = &r.new_status { obj.insert("new_status".into(), serde_json::json!(new)); }
                if let Some(reason) = &r.reason { obj.insert("reason".into(), serde_json::json!(reason)); }
            }
            AuditLogEntry {
                id: r.id,
                wallet_address: r.wallet_address.map(UserId::from_string_unchecked),
                action: parse_action(&r.action),
                resource_type: ResourceType::Payment,
                resource_id: r.resource_id,
                result: AuditResult::Success,
                ip_address: None,
                user_agent: None,
                additional_data: Some(details),
                timestamp: r.created_at,
                category: Some("payment".to_string()),
                action_raw: Some(r.action),
                resource_type_raw: Some("payment".to_string()),
            }
        }).collect();

        Ok((entries, count_result.cnt))
    }

    /// Unified query across all audit sources
    pub async fn find_all_unified(query: &AuditQuery) -> Result<(Vec<AuditLogEntry>, i64)> {
        let category = query.category.as_deref();
        let search = query.search.as_deref();
        let limit = query.limit.unwrap_or(50) as i64;
        let offset = query.offset.unwrap_or(0) as i64;

        let need_analytics = !matches!(category, Some("payment"));
        let need_payments = matches!(category, None | Some("all") | Some("payment"));

        // When only one source is queried, push pagination to SQL directly
        let single_source = need_analytics != need_payments;

        let mut all_entries: Vec<AuditLogEntry> = Vec::new();
        let mut total: i64 = 0;

        if need_analytics {
            let (a_limit, a_offset) = if single_source {
                (limit, offset)
            } else {
                (limit + offset, 0)
            };
            match Self::query_analytics(query, category, search, a_limit, a_offset).await {
                Ok((entries, count)) => {
                    all_entries.extend(entries);
                    total += count;
                }
                Err(e) => {
                    tracing::warn!("Analytics audit query failed: {:?}", e);
                }
            }
        }

        if need_payments {
            let (p_limit, p_offset) = if single_source {
                (limit, offset)
            } else {
                (limit + offset, 0)
            };
            match Self::query_payments(query, search, p_limit, p_offset).await {
                Ok((entries, count)) => {
                    all_entries.extend(entries);
                    total += count;
                }
                Err(e) => {
                    tracing::warn!("Payments audit query failed: {:?}", e);
                }
            }
        }

        if single_source {
            // Already paginated in SQL
            Ok((all_entries, total))
        } else {
            // Sort by timestamp desc, then paginate in Rust
            all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            let result: Vec<AuditLogEntry> = all_entries
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect();
            Ok((result, total))
        }
    }
}

fn parse_action(s: &str) -> AuditAction {
    match s {
        "Create" | "create" | "created" => AuditAction::Create,
        "Read" | "read" => AuditAction::Read,
        "Update" | "update" | "updated" | "status_change" => AuditAction::Update,
        "Delete" | "delete" | "deleted" | "revoked" => AuditAction::Delete,
        "Login" | "login" => AuditAction::Login,
        "Logout" | "logout" => AuditAction::Logout,
        "PermissionGranted" | "permission_granted" | "granted" => AuditAction::PermissionGranted,
        "PermissionRevoked" | "permission_revoked" => AuditAction::PermissionRevoked,
        "PaymentInitiated" | "payment_initiated" | "initiated" => AuditAction::PaymentInitiated,
        "PaymentCompleted" | "payment_completed" | "completed" | "confirmed" => AuditAction::PaymentCompleted,
        "Export" | "export" => AuditAction::Export,
        _ => AuditAction::Update,
    }
}

fn parse_resource_type(s: &str) -> ResourceType {
    match s {
        "User" | "user" => ResourceType::User,
        "Session" | "session" => ResourceType::Session,
        "Payment" | "payment" => ResourceType::Payment,
        "Notification" | "notification" => ResourceType::Notification,
        "Analytics" | "analytics" => ResourceType::Analytics,
        "Admin" | "admin" => ResourceType::Admin,
        "Permission" | "permission" => ResourceType::Admin,
        "Wallet" | "wallet" => ResourceType::User,
        "Plan" | "plan" => ResourceType::Payment,
        _ => ResourceType::Admin,
    }
}

fn parse_result(s: &str) -> AuditResult {
    match s {
        "Success" | "success" => AuditResult::Success,
        "Failed" | "failed" => AuditResult::Failed,
        "Denied" | "denied" => AuditResult::Denied,
        _ => AuditResult::Success,
    }
}

// Helper to map DB model to Domain entity
impl From<AuditLogDb> for AuditLogEntry {
    fn from(db: AuditLogDb) -> Self {
        AuditLogEntry {
            id: db.id.to_string(),
            wallet_address: db.wallet_address.map(UserId::from_string_unchecked),
            action: parse_action(&db.action),
            resource_type: parse_resource_type(&db.resource_type),
            resource_id: db.resource_id,
            result: parse_result(&db.result),
            ip_address: db.ip_address,
            user_agent: db.user_agent,
            additional_data: db.details,
            timestamp: db.created_at,
            category: Some("system".to_string()),
            action_raw: None,
            resource_type_raw: None,
        }
    }
}

#[async_trait]
impl AuditLogRepository for DieselAuditLogRepository {
    async fn save(&self, entry: AuditLogEntry) -> Result<AuditLogEntry> {
        let pool = get_analytics_pool().await?;
        let mut conn = pool.get().await.map_err(|e| anyhow::anyhow!("Failed to get DB connection: {:?}", e))?;

        let new_log = NewAuditLogDb {
            wallet_address: entry.wallet_address.map(|w| w.to_string()),
            action: format!("{:?}", entry.action),
            resource_type: format!("{:?}", entry.resource_type),
            resource_id: entry.resource_id.clone(),
            result: format!("{:?}", entry.result),
            ip_address: entry.ip_address.clone(),
            user_agent: entry.user_agent.clone(),
            details: entry.additional_data.clone(),
        };

        let inserted: AuditLogDb = diesel::insert_into(audit_logs::table)
            .values(&new_log)
            .get_result(&mut conn)
            .await
            .context("Failed to insert audit log")?;

        Ok(inserted.into())
    }

    async fn find_all(&self, query: AuditQuery) -> Result<(Vec<AuditLogEntry>, i64)> {
        // Delegate to unified query
        Self::find_all_unified(&query).await
    }
}
