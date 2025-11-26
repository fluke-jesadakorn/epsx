// TradingView EPS Repository Implementation
// Domain repository adapter for TradingView data source

use std::sync::Arc;
use async_trait::async_trait;

use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
use crate::domain::shared_kernel::services::eps_ranking_service::EPSRepository;
use crate::domain::shared_kernel::entities::eps_growth::{EPSGrowthData, EPSRanking};
use crate::core::errors::AppError;

/// TradingView-based EPS Repository implementation
#[derive(Clone)]
pub struct TradingViewEPSRepository {
    tradingview_service: Arc<TradingViewApiService>,
}

impl TradingViewEPSRepository {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self { tradingview_service }
    }

    /// Convert TradingView screening results to EPSRanking format
    fn convert_screening_to_eps_ranking(
        &self,
        result: crate::domain::shared_kernel::entities::market_data::StockScreeningResult,
        rank: i32,
    ) -> EPSRanking {
        let current_eps = result.current_eps.or_else(|| {
            if let Some(pe_ratio) = result.pe_ratio {
                Some(result.price / pe_ratio.max(1.0))
            } else {
                None
            }
        });

        let growth_factor = result.eps_growth_yoy.or(Some(result.change_percent));

        // Convert TradingView earnings timestamps to strings for EPSGrowthData
        let next_earnings_date_str = result.next_earnings_date.and_then(|ts| {
            chrono::DateTime::from_timestamp(ts as i64, 0)
                .map(|dt| {
                    let date_str = dt.format("%Y-%m-%d").to_string();
                    tracing::info!("🔄 [{}] next_earnings_date: {} -> {}",
                        result.symbol, ts, date_str);
                    date_str
                })
        });

        let last_earnings_date_str = result.last_earnings_date.and_then(|ts| {
            chrono::DateTime::from_timestamp(ts as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
        });

        if next_earnings_date_str.is_none() && last_earnings_date_str.is_none() {
            tracing::warn!("⚠️ [{}] NO earnings dates from TradingView - will use fallback", result.symbol);
        }

        EPSRanking::from_eps_data(
            EPSGrowthData {
                symbol: result.symbol,
                name: result.name,
                country: "US".to_string(),
                sector: result.sector.unwrap_or_else(|| "Unknown".to_string()),
                exchange: "NASDAQ".to_string(),
                current_eps,
                growth_factor,
                price_current: Some(result.price),
                market_cap: result.market_cap.map(|mc| mc as i64),
                volume: Some(result.volume as i64),
                ranking_score: Some(rank as f64),
                created_at: None,
                updated_at: None,
                next_earnings_date: next_earnings_date_str,
                last_earnings_date: last_earnings_date_str,
            },
            Some(rank)
        )
    }
}

#[async_trait]
impl EPSRepository for TradingViewEPSRepository {
    async fn store_eps_data(&self, _eps_data: EPSGrowthData) -> Result<(), AppError> {
        // TradingView is read-only, so storage is not supported
        Ok(())
    }

    async fn get_rankings_filtered(
        &self,
        rank_offset: i32,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
        page: i32,
        limit: i32,
    ) -> Result<Vec<EPSRanking>, AppError> {
        // Apply rank offset: skip to the user's accessible rank range
        let skip = rank_offset + (page - 1) * limit;

        let (screening_results, _total) = self.tradingview_service
            .fetch_eps_growth_ranking(
                Some(skip),
                Some(limit),
                country,
                sector,
                sort_by
            )
            .await
            .map_err(|e| AppError::new(
                crate::core::errors::ErrorKind::ExternalServiceError,
                format!("TradingView API error: {}", e)
            ))?;

        let rankings = screening_results
            .into_iter()
            .enumerate()
            .map(|(i, result)| {
                let rank = skip + i as i32 + 1;  // Actual rank includes offset
                self.convert_screening_to_eps_ranking(result, rank)
            })
            .collect();

        Ok(rankings)
    }

    async fn get_total_count(&self, rank_offset: i32, country: Option<String>, sector: Option<String>) -> Result<i64, AppError> {
        // For TradingView, we'll return a reasonable estimate since exact count isn't always available
        let (_results, total) = self.tradingview_service
            .fetch_eps_growth_ranking(
                Some(0),
                Some(1), // Just get first item to get total count
                country,
                sector,
                None
            )
            .await
            .map_err(|e| AppError::new(
                crate::core::errors::ErrorKind::ExternalServiceError,
                format!("TradingView API error: {}", e)
            ))?;

        // Return count of accessible ranks (total - offset)
        let accessible_count = (total - rank_offset).max(0) as i64;
        Ok(accessible_count)
    }

    async fn batch_store_eps_data(&self, _eps_data_list: Vec<EPSGrowthData>) -> Result<usize, AppError> {
        // TradingView is read-only, so batch storage is not supported
        Ok(0)
    }

    async fn get_countries(&self) -> Result<Vec<String>, AppError> {
        // Return supported countries for TradingView
        Ok(vec![
            "america".to_string(),
            "canada".to_string(),
            "germany".to_string(),
            "france".to_string(),
            "uk".to_string(),
            "japan".to_string(),
            "australia".to_string(),
        ])
    }

    async fn get_sectors_by_country(&self, _country: Option<String>) -> Result<Vec<String>, AppError> {
        // Return common sectors available in TradingView
        Ok(vec![
            "Technology".to_string(),
            "Healthcare".to_string(),
            "Financial Services".to_string(),
            "Consumer Cyclical".to_string(),
            "Industrials".to_string(),
            "Energy".to_string(),
            "Utilities".to_string(),
            "Real Estate".to_string(),
            "Materials".to_string(),
            "Consumer Defensive".to_string(),
            "Communication Services".to_string(),
        ])
    }
}
