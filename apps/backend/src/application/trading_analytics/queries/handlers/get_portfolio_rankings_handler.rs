use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    GetPortfolioRankingsQuery, GetPortfolioRankingsResponse,
};
use crate::application::trading_analytics::dtos::{
    CardDashboardResponse, SymbolCardData, EPSPaginationResponse,
    CardDashboardMetadata, QuarterlyPerformanceData, EPSQuarterlyData,
};
use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;

/// Query handler for getting portfolio rankings (positive-growth stocks only)
pub struct GetPortfolioRankingsQueryHandler {
    tradingview_service: Arc<TradingViewApiService>,
}

impl GetPortfolioRankingsQueryHandler {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self { tradingview_service }
    }

    /// Transform EPSRanking to SymbolCardData
    fn transform_to_card(&self, ranking: EPSRanking, rank: i32) -> SymbolCardData {
        let current_eps = ranking.current_eps.unwrap_or(0.0);
        let growth_factor = ranking.growth_factor.unwrap_or(0.0);
        let price_current = ranking.price_current.unwrap_or(0.0);

        // Build quarterly performance from ranking data
        let quarterly_performance = vec![
            QuarterlyPerformanceData {
                quarter: "Q1".to_string(),
                date: chrono::Utc::now().to_string(),
                price: price_current,
                eps: current_eps,
                eps_growth: growth_factor,
                price_growth: 0.0,
                announcement_date: None,
                announcement_timestamp: None,
                is_estimated: false,
            },
        ];

        SymbolCardData {
            rank,
            symbol: ranking.symbol.clone(),
            latest_date: chrono::Utc::now().to_string(),
            value: current_eps,
            active_status: "active".to_string(),
            quarterly_performance,
            next_quarter_estimate: None,
            eps_quarterly: Some(EPSQuarterlyData {
                eps_q_minus_2: None,
                eps_q_minus_1: None,
                eps_q_current: Some(current_eps),
                eps_q_next_estimate: None,
                eps_q_minus_2_date: None,
                eps_q_minus_1_date: None,
                eps_q_current_date: Some(chrono::Utc::now().to_string()),
                eps_q_next_estimate_date: None,
                qoq_growth_current: Some(growth_factor),
                yoy_growth_current: Some(growth_factor),
                trend_direction: Some(if growth_factor > 0.0 {
                    "up".to_string()
                } else {
                    "down".to_string()
                }),
                avg_growth_rate: Some(growth_factor),
                consistency_score: Some("high".to_string()),
            }),
            next_earnings_date: ranking.next_earnings_date.and_then(|s| s.parse::<i64>().ok()),
            last_earnings_date: ranking.last_earnings_date.and_then(|s| s.parse::<i64>().ok()),
            next_earnings_date_formatted: None,
            days_until_next_earnings: None,
            progress_percentage: None,
        }
    }
}

#[async_trait]
impl QueryHandler<GetPortfolioRankingsQuery> for GetPortfolioRankingsQueryHandler {
    async fn handle(
        &self,
        query: GetPortfolioRankingsQuery,
    ) -> ApplicationResult<GetPortfolioRankingsResponse> {
        let start = std::time::Instant::now();

        // Extract parameters with defaults
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(10);
        let skip = (page - 1) * limit;
        let min_growth = query.min_growth.unwrap_or(0.0); // Default to 0.0 for positive growth

        // Fetch rankings from TradingView
        let (screening_results, _total_count) = self
            .tradingview_service
            .fetch_eps_growth_ranking(
                Some(skip),
                Some(limit),
                query.country,
                query.sector,
                query.sort_by,
            )
            .await
            .map_err(|e| ApplicationError::external_service("TradingView", e.to_string()))?;

        // Convert to EPSRanking and filter for positive growth
        let rankings: Vec<EPSRanking> = screening_results
            .into_iter()
            .map(|result| {
                // Convert StockScreeningResult to EPSRanking
                EPSRanking {
                    symbol: result.symbol.clone(),
                    name: result.name.clone(),
                    country: "US".to_string(), // Default country
                    sector: result.sector.clone().unwrap_or_default(),
                    exchange: "NASDAQ".to_string(), // Default exchange
                    current_eps: result.current_eps,
                    growth_factor: result.qoq_growth_current.or(result.yoy_growth_current),
                    price_current: Some(result.price),
                    market_cap: result.market_cap.map(|v| v as i64),
                    volume: Some(result.volume as i64),
                    ranking_position: None,
                    quarterly_data: None,
                    next_earnings_date: result.next_earnings_date.map(|v| v.to_string()),
                    last_earnings_date: result.last_earnings_date.map(|v| v.to_string()),
                }
            })
            .filter(|r| {
                r.growth_factor.unwrap_or(0.0) >= min_growth // Filter for positive growth
            })
            .collect();

        // Calculate filtered pagination
        let filtered_count = rankings.len() as i64;
        let total_pages = ((filtered_count as f64) / (limit as f64)).ceil() as i32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        // Transform to card data
        let card_data: Vec<SymbolCardData> = rankings
            .into_iter()
            .enumerate()
            .map(|(index, ranking)| self.transform_to_card(ranking, skip + index as i32 + 1))
            .collect();

        // Build metadata
        let metadata = CardDashboardMetadata {
            available_countries: vec![
                "US".to_string(),
                "UK".to_string(),
                "JP".to_string(),
                "DE".to_string(),
                "FR".to_string(),
            ],
            available_sectors: vec![
                "Technology".to_string(),
                "Healthcare".to_string(),
                "Finance".to_string(),
                "Energy".to_string(),
                "Consumer".to_string(),
            ],
            request_timestamp: chrono::Utc::now(),
            data_source: "tradingview_portfolio".to_string(),
        };

        // Build response
        let rankings = CardDashboardResponse {
            success: true,
            data: card_data,
            pagination: EPSPaginationResponse {
                page,
                limit,
                total: filtered_count,
                total_pages,
                has_next,
                has_prev,
            },
            metadata,
            message: Some(format!(
                "Fetched {} positive-growth stocks",
                filtered_count
            )),
            processing_time_ms: start.elapsed().as_millis() as u64,
        };

        Ok(GetPortfolioRankingsResponse { rankings })
    }
}
