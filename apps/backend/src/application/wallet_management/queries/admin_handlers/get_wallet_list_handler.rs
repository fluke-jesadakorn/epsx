// Get Wallet List Query Handler
// CQRS handler for retrieving wallet list with filtering and pagination

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::application::wallet_management::queries::admin_models::{
    GetWalletListQuery, GetWalletListResponse, PaginationDto, WalletSummaryDto,
};
use crate::application::wallet_management::wallet_management_repository::{
    WalletManagementRepository, WalletSearchCriteria,
};
use async_trait::async_trait;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use std::sync::Arc;
use tracing::{error, info};

pub struct GetWalletListQueryHandler {
    db_pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl GetWalletListQueryHandler {
    pub fn new(db_pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
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

        // 3. Initialize repository
        let repo = WalletManagementRepository::new(self.db_pool.clone());

        // 4. Build search criteria from query
        let criteria = WalletSearchCriteria {
            search: query.search,
            status: query.status,
            date_from: query.date_from,
            date_to: query.date_to,
            sort_by: query.sort_by,
            sort_order: query.sort_order,
            exclude_plan_id: query.exclude_plan_id,
            limit,
            offset,
        };

        // 5. Fetch wallets using repository
        let wallets_result = repo
            .find_wallets_paginated(&criteria)
            .await
            .map_err(|e| {
                error!("❌ Failed to fetch wallet list: {}", e);
                ApplicationError::infrastructure(e.to_string())
            })?;

        // 6. Count total using repository
        let total = repo
            .count_wallets(&criteria)
            .await
            .map_err(|e| {
                error!("❌ Failed to count wallets: {}", e);
                ApplicationError::infrastructure(e.to_string())
            })?;

        // 7. Convert repository results to DTOs
        let wallets: Vec<WalletSummaryDto> = wallets_result
            .into_iter()
            .map(|w| WalletSummaryDto {
                wallet_address: w.wallet_address,
                is_active: w.is_active,
                created_at: w.created_at,
                last_auth_at: w.last_auth_at,
                permissions_count: w.permissions_count,
                plans_count: w.plans_count,
                last_activity: Some(w.created_at),
                metadata: w.wallet_metadata,
            })
            .collect();

        // 8. Build pagination
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
