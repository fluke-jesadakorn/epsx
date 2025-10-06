// Get Wallet List Query Handler
// CQRS handler for retrieving wallet list with filtering and pagination

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::application::wallet_management::queries::admin_models::{
    GetWalletListQuery, GetWalletListResponse, PaginationDto, WalletSummaryDto,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{error, info};

pub struct GetWalletListQueryHandler {
    db_pool: Arc<PgPool>,
}

impl GetWalletListQueryHandler {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl QueryHandler<GetWalletListQuery> for GetWalletListQueryHandler {
    async fn handle(
        &self,
        query: GetWalletListQuery,
    ) -> ApplicationResult<GetWalletListResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Extract parameters with defaults
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(50).min(1000); // Cap at 1000
        let offset = (page - 1) * limit;

        // 3. Build dynamic query conditions
        let mut conditions = Vec::new();
        let mut condition_values: Vec<String> = Vec::new();

        // Search filter (wallet address ILIKE)
        if let Some(ref search) = query.search {
            conditions.push(format!("wu.wallet_address ILIKE ${}", conditions.len() + 1));
            condition_values.push(format!("%{}%", search));
        }

        // Status filter (is_active)
        if let Some(ref status) = query.status {
            let is_active = status == "active";
            conditions.push(format!("wu.is_active = ${}", conditions.len() + 1));
            condition_values.push(is_active.to_string());
        }

        // Date range filters
        if let Some(ref date_from) = query.date_from {
            if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(date_from) {
                conditions.push(format!("wu.created_at >= ${}", conditions.len() + 1));
                condition_values.push(parsed_date.to_rfc3339());
            }
        }

        if let Some(ref date_to) = query.date_to {
            if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(date_to) {
                conditions.push(format!("wu.created_at <= ${}", conditions.len() + 1));
                condition_values.push(parsed_date.to_rfc3339());
            }
        }

        // Build WHERE clause
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 4. Build ORDER BY clause
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("DESC");

        // Validate sort_by column to prevent SQL injection
        let valid_sort_columns =
            ["created_at", "wallet_address", "last_auth_at", "is_active"];
        let safe_sort_by = if valid_sort_columns.contains(&sort_by) {
            sort_by
        } else {
            "created_at"
        };

        // 5. Build main query
        let main_query = format!(
            r#"
            SELECT
                wu.wallet_address,
                wu.is_active,
                wu.created_at,
                wu.last_auth_at,
                COALESCE(perms.count, 0) as permissions_count,
                0 as groups_count,
                wu.created_at as last_activity
            FROM wallet_users wu
            LEFT JOIN (
                SELECT wallet_address, COUNT(*) as count
                FROM wallet_group_memberships
                WHERE is_active = true
                GROUP BY wallet_address
            ) perms ON wu.wallet_address = perms.wallet_address
            {}
            ORDER BY wu.{} {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, safe_sort_by, sort_order, limit, offset
        );

        // 6. Execute main query
        let mut sql_query = sqlx::query(&main_query);
        for value in &condition_values {
            sql_query = sql_query.bind(value);
        }

        let rows = sql_query
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("❌ Failed to fetch wallet list: {}", e);
                ApplicationError::infrastructure(format!("Failed to fetch wallets: {}", e))
            })?;

        // 7. Build count query
        let count_query = format!("SELECT COUNT(*) as total FROM wallet_users wu {}", where_clause);

        let mut count_sql_query = sqlx::query(&count_query);
        for value in &condition_values {
            count_sql_query = count_sql_query.bind(value);
        }

        let count_row = count_sql_query
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("❌ Failed to count wallets: {}", e);
                ApplicationError::infrastructure(format!("Failed to count wallets: {}", e))
            })?;

        let total: i64 = count_row.try_get("total").unwrap_or(0);

        // 8. Convert rows to DTOs
        let wallets: Vec<WalletSummaryDto> = rows
            .into_iter()
            .map(|row| WalletSummaryDto {
                wallet_address: row.try_get("wallet_address").unwrap_or_default(),
                is_active: row.try_get("is_active").unwrap_or(false),
                created_at: row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now()),
                last_auth_at: row.try_get("last_auth_at").ok(),
                permissions_count: row.try_get("permissions_count").unwrap_or(0),
                groups_count: row.try_get("groups_count").unwrap_or(0),
                last_activity: row.try_get("last_activity").ok(),
            })
            .collect();

        // 9. Build pagination
        let total_pages = ((total as f64) / (limit as f64)).ceil() as i32;
        let pagination = PaginationDto {
            page,
            limit,
            total: total as i32,
            total_pages,
            has_next_page: page < total_pages,
            has_previous_page: page > 1,
        };

        info!(
            "✅ Successfully retrieved {} wallets (page {}/{})",
            wallets.len(),
            page,
            total_pages
        );

        Ok(GetWalletListResponse {
            success: true,
            wallets,
            pagination,
        })
    }
}
