// EPS Analytics Module - Focused Domain Separation
// Breaks down 1,865-line God Object into 7 focused modules with clear domain boundaries

// Public modules - each handles a specific domain
pub mod dto;           // Data Transfer Objects and API structures
pub mod rankings;      // Core EPS rankings business logic
pub mod metadata;      // Country and sector data management  
pub mod health;        // Health checks and debug endpoints
pub mod cache;         // Cache management and caching logic
pub mod enhancement;   // WebSocket data enhancement
pub mod transform;     // Data transformation and formatting
pub mod errors;        // EPS-specific error handling

// Re-export key types for easy access
pub use dto::*;
pub use rankings::{get_eps_rankings, convert_screening_result_to_eps_ranking, is_valid_eps_for_ranking};
pub use metadata::{get_available_countries, get_all_valid_countries, get_sectors_by_country};
pub use health::{eps_health_check, debug_eps_correction, debug_ranking_data, debug_websocket_eps, trigger_eps_sync};
pub use cache::{get_unified_analytics_rankings_cached, get_cache_stats, force_cache_refresh, cache_health_check};
pub use enhancement::enhance_with_websocket_data;
pub use transform::{transform_ranking_to_unified_format, transform_unified_to_card_format};
pub use errors::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Test that all modules are accessible
        let _dto_test = dto::EPSRankingQueryParams {
            page: Some(1),
            limit: Some(10),
            country: None,
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        // Test rankings validation
        assert!(rankings::is_valid_eps_for_ranking(1.0));
        assert!(!rankings::is_valid_eps_for_ranking(0.0));

        // Test metadata static methods
        let countries = metadata::get_available_countries_static();
        assert!(!countries.is_empty());

        let sectors = metadata::get_available_sectors_static();
        assert!(!sectors.is_empty());
    }

    #[test]
    fn test_transformation_functions() {
        use crate::dom::entities::eps_growth::EPSRanking;

        let ranking = EPSRanking {
            symbol: "AAPL".to_string(),
            name: "Apple Inc".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_eps: Some(1.5),
            qoq_growth: Some(10.0),
            price_current: Some(150.0),
            market_cap: Some(2500000000),
            volume: Some(50000000),
            ranking_position: Some(1),
            quarterly_data: None,
        };

        let unified = transform::transform_ranking_to_unified_format(ranking, 1);
        assert_eq!(unified.symbol, "AAPL");
        
        let card = transform::transform_unified_to_card_format(unified);
        assert_eq!(card.symbol, "AAPL");
        assert_eq!(card.rank, 1);
    }
}