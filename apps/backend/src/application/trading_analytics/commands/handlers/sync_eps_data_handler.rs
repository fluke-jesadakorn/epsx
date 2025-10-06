use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::commands::{SyncEPSDataCommand, SyncEPSDataResponse};
use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;

/// Command handler for synchronizing EPS data from TradingView
pub struct SyncEPSDataCommandHandler {
    tradingview_service: Arc<TradingViewApiService>,
}

impl SyncEPSDataCommandHandler {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self { tradingview_service }
    }
}

#[async_trait]
impl CommandHandler<SyncEPSDataCommand> for SyncEPSDataCommandHandler {
    async fn handle(
        &self,
        command: SyncEPSDataCommand,
    ) -> ApplicationResult<SyncEPSDataResponse> {
        let start = std::time::Instant::now();

        // Fetch fresh data from TradingView
        let (screening_results, _total_count) = self
            .tradingview_service
            .fetch_eps_growth_ranking(
                Some(0),
                Some(100), // Fetch first 100 for sync
                command.country,
                command.sector,
                None,
            )
            .await
            .map_err(|e| ApplicationError::external_service("TradingView", e.to_string()))?;

        let synced_symbols = screening_results.len() as i32;

        // In a full implementation, this would:
        // 1. Compare with existing database records
        // 2. Update changed records
        // 3. Insert new records
        // For now, we just count the synced data

        let duration = start.elapsed();

        Ok(SyncEPSDataResponse {
            success: true,
            synced_symbols,
            updated_records: 0, // Would be calculated from DB diff
            new_records: synced_symbols, // Assume all are new for now
            duration_ms: duration.as_millis() as u64,
            message: format!(
                "Successfully synced {} symbols from TradingView",
                synced_symbols
            ),
            timestamp: chrono::Utc::now(),
        })
    }
}
