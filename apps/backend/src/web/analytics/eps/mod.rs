// EPS Analytics Module - Focused Domain Separation
// Breaks down 1,865-line God Object into focused modules with clear domain boundaries

// Public modules - each handles a specific domain
pub mod types;         // Data Transfer Objects and API structures
pub mod rankings;      // Core EPS rankings business logic
pub mod metadata;      // Country and sector data management
pub mod cache;         // Cache management and caching logic
pub mod enhancement;   // WebSocket data enhancement
pub mod transform;     // Data transformation and formatting
pub mod errors;        // EPS-specific error handling

// Internal modules for transform decomposition
mod quarterly;         // Quarterly data generation logic
mod price;             // Price growth calculations
mod date_metrics;      // Date utilities and metrics
mod estimate;          // Next quarter estimation
mod system;            // System mode and config utilities

// Re-export key types for easy access
pub use types::*;
pub use rankings::{get_eps_rankings, convert_screening_result_to_eps_ranking, is_valid_eps_for_ranking};
pub use metadata::{get_available_countries, get_all_valid_countries, get_sectors_by_country};
pub use cache::{get_unified_analytics_rankings_cached, get_cache_stats, force_cache_refresh};
pub use enhancement::enhance_with_websocket_data;
pub use transform::{transform_ranking_to_unified_format, transform_unified_to_card_format};
pub use errors::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Test that all modules are accessible
        let _dto_test = types::EPSRankingQueryParams {
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
        use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;

        // Create proper EPSRanking using the correct constructor
        let mut ranking = EPSRanking::default();
        ranking.symbol = "AAPL".to_string();
        ranking.name = "Apple Inc".to_string();
        ranking.country = "america".to_string();
        ranking.sector = "Technology".to_string();
        ranking.exchange = "NASDAQ".to_string();
        ranking.current_eps = Some(1.5);
        ranking.growth_factor = Some(10.0);
        ranking.price_current = Some(150.0);
        ranking.market_cap = Some(2500000000);
        ranking.volume = Some(50000000);
        ranking.ranking_position = Some(1);

        let unified = transform::transform_ranking_to_unified_format(ranking, 1);
        assert_eq!(unified.symbol, "AAPL");

        let card = transform::transform_unified_to_card_format(&unified);
        assert_eq!(card.symbol, "AAPL");
        assert_eq!(card.rank, 1);
    }
}