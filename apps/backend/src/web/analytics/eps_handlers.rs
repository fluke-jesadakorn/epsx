// EPS Analytics Handlers - Refactored with Focused Module Architecture
// Originally 1,865 lines - now split into 7 focused modules with domain separation

// Re-export all handlers from focused modules for backward compatibility
pub use crate::web::analytics::eps::*;

// Re-export handler functions with their original names for routing compatibility
pub use rankings::get_eps_rankings;
pub use metadata::{get_available_countries, get_all_valid_countries, get_sectors_by_country};  
pub use health::{eps_health_check, debug_eps_correction, debug_ranking_data, debug_websocket_eps, trigger_eps_sync};
pub use cache::{get_unified_analytics_rankings_cached, get_cache_stats, force_cache_refresh, cache_health_check};

// Re-export key DTOs that are used in routes
pub use dto::{
    EPSRankingQueryParams, EPSRankingsApiResponse, EPSPaginationResponse,
    CountriesResponse, SectorsResponse, EPSHealthResponse,
    CardDashboardResponse, CacheStatsResponse, CacheRefreshResponse, CacheHealthResponse
};

// Test module for backward compatibility verification
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        // Test that all original exports are still available
        let _query_params = EPSRankingQueryParams {
            page: Some(1),
            limit: Some(10),
            country: None,
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        // Test helper functions are accessible
        assert!(rankings::is_valid_eps_for_ranking(1.0));
        assert!(!rankings::is_valid_eps_for_ranking(0.0));

        // Test metadata functions
        let countries = metadata::get_available_countries_static();
        assert!(!countries.is_empty());
    }

    #[test]
    fn test_focused_modules_integration() {
        // Test that focused modules work together correctly
        use crate::domain::trading_analytics::EPSRanking;

        let ranking = EPSRanking {
            symbol: "AAPL".to_string(),
            name: "Apple Inc".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_eps: Some(1.5),
            growth_factor: Some(10.0),
            price_current: Some(150.0),
            market_cap: Some(2500000000),
            volume: Some(50000000),
            ranking_position: Some(1),
            quarterly_data: None,
        };

        // Test data transformation pipeline
        let unified = crate::web::analytics::eps::transform::transform_ranking_to_unified_format(ranking, 1);
        assert_eq!(unified.symbol, "AAPL");
        assert_eq!(unified.ranking_position, 1);

        let card = crate::web::analytics::eps::transform::transform_unified_to_card_format(unified);
        assert_eq!(card.symbol, "AAPL");
        assert_eq!(card.rank, 1);
        assert_eq!(card.value, 150.0);
    }

    #[test]
    fn test_cache_key_generation() {
        let params = EPSRankingQueryParams {
            page: Some(1),
            limit: Some(10),
            country: Some("america".to_string()),
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        let cache_key = cache::generate_cache_key(&params);
        assert!(cache_key.starts_with("analytics:rankings:"));
        assert!(cache_key.len() > 20);
    }
}