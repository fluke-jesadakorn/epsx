// Admin Query Models for Wallet Management
// CQRS query models for admin wallet operations

mod get_wallet_list;
mod get_wallet_detail;
mod get_wallet_stats;

pub use get_wallet_list::{
    GetWalletListQuery, GetWalletListResponse, PaginationDto, WalletSummaryDto,
};
pub use get_wallet_detail::{
    GetWalletDetailQuery, GetWalletDetailResponse, WalletActivitySummaryDto, WalletDetailDto,
    WalletGroupDto, WalletPermissionDto,
};
pub use get_wallet_stats::{GetWalletStatsQuery, GetWalletStatsResponse, WalletStatsDto};
