// Get Wallet List Query Handler
// CQRS handler for retrieving wallet list with filtering and pagination

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::infrastructure::database::diesel_connection_manager::TlsPool;
use crate::application::wallet_management::queries::admin_models::{
    GetWalletListQuery, GetWalletListResponse, PaginationDto, WalletSummaryDto,
};
use crate::application::wallet_management::wallet_management_repository::{
    WalletManagementRepository, WalletSearchCriteria,
};
use crate::web::pagination::Pagination;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info};

pub struct GetWalletListQueryHandler {
    db_pool: Arc<&'static TlsPool>,
}

impl GetWalletListQueryHandler {
    pub fn new(db_pool: Arc<&'static TlsPool>) -> Self {
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

        // 2. Extract parameters with pagination (default 50, max 1000)
        // Note: query.page and query.limit are Option<i32>, so we use from_signed
        let pg = Pagination::from_signed(query.page, query.limit, 50, 1000);

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
            limit: pg.limit as i32,
            offset: pg.offset as i32,
        };

        // 5. Fetch wallets using repository
        let wallets_result = repo
            .find_wallets_paginated(&criteria)
            .await
            .map_err(|e| {
                error!("Failed to fetch wallet list: {}", e);
                ApplicationError::infrastructure(e.to_string())
            })?;

        // 6. Count total using repository
        let total = repo
            .count_wallets(&criteria)
            .await
            .map_err(|e| {
                error!("Failed to count wallets: {}", e);
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
                plan_name: w.plan_name,
                last_activity: Some(w.created_at),
                metadata: w.wallet_metadata,
            })
            .collect();

        // 8. Build pagination
        let total_pages = pg.total_pages(total as u64) as i32;
        let pagination = PaginationDto {
            page: pg.page as i32,
            limit: pg.limit as i32,
            total: total as i32,
            total_pages,
            has_next_page: pg.has_next(total as u64),
            has_previous_page: pg.has_prev(),
        };

        info!(
            "Successfully retrieved {} wallets (page {}/{})",
            wallets.len(),
            pg.page,
            total_pages
        );

        Ok(GetWalletListResponse {
            success: true,
            wallets,
            pagination,
        })
    }
}
