use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult};
use crate::application::market_analytics::commands::{
    RefreshCacheCommand, RefreshCacheResponse,
};
use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;

/// Command handler for refreshing analytics cache
pub struct RefreshCacheCommandHandler {
    tradingview_service: Arc<TradingViewApiService>,
}

impl RefreshCacheCommandHandler {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self { tradingview_service }
    }
}

#[async_trait]
impl CommandHandler<RefreshCacheCommand> for RefreshCacheCommandHandler {
    async fn handle(
        &self,
        command: RefreshCacheCommand,
    ) -> ApplicationResult<RefreshCacheResponse> {
        let start = std::time::Instant::now();

        // Get current cache stats before refresh
        let stats_before = self.tradingview_service.get_cache_stats().await;
        let entries_before = stats_before.total_count;

        if command.force_full_refresh {
            // Clear entire cache
            self.tradingview_service.clear_cache().await;
        } else if let Some(_pattern) = &command.cache_key_pattern {
            // Clear specific pattern (not supported yet in TradingViewCache)
            // For now, just clear all if pattern is specified
            self.tradingview_service.clear_cache().await;
        }

        // Get cache stats after refresh
        let stats_after = self.tradingview_service.get_cache_stats().await;
        let entries_after = stats_after.total_count;
        let refreshed_entries = entries_before.saturating_sub(entries_after);

        let duration = start.elapsed();

        Ok(RefreshCacheResponse {
            success: true,
            refreshed_entries,
            duration_ms: duration.as_millis() as u64,
            message: format!(
                "Cache refreshed successfully. Cleared {} entries.",
                refreshed_entries
            ),
            timestamp: chrono::Utc::now(),
        })
    }
}
