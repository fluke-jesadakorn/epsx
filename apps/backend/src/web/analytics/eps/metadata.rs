// Country and Sector Data Management
// Focused module handling country/sector metadata for EPS analytics

use axum::{
    extract::{Query, Extension},
    response::Json,
};
use std::sync::Arc;
use tracing::{debug, info};

use crate::core::errors::AppError;
use crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService;
use super::types::*;

/// GET /api/analytics/eps-rankings/countries
/// Returns list of available countries for TradingView API
#[utoipa::path(
    get,
    path = "/api/analytics/eps-rankings/countries",
    tag = "analytics",
    responses(
        (status = 200, description = "Successfully retrieved available countries", body = CountriesResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_available_countries(
    Extension(_service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<CountriesResponse>, AppError> {
    debug!("Getting available countries for TradingView API");

    let countries = get_available_countries_with_labels();
    debug!("Found {} countries for TradingView API", countries.len());

    let response = CountriesResponse {
        count: countries.len(),
        countries,
    };

    info!("Returning {} available countries with display names", response.count);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/countries/all
/// Returns complete list of valid countries for TradingView API
#[utoipa::path(
    get,
    path = "/api/analytics/eps-rankings/countries/all",
    tag = "analytics",
    responses(
        (status = 200, description = "Successfully retrieved all valid countries", body = CountriesResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_valid_countries() -> Result<Json<CountriesResponse>, AppError> {
    debug!("Getting all valid countries for TradingView API");

    let countries = get_available_countries_with_labels();
    debug!("Found {} valid countries for TradingView API", countries.len());

    let response = CountriesResponse {
        count: countries.len(),
        countries,
    };

    info!("Returning {} valid countries with display names", response.count);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/sectors?country=america
/// Returns list of available sectors, optionally filtered by country
#[utoipa::path(
    get,
    path = "/api/analytics/eps-rankings/sectors",
    tag = "analytics",
    responses(
        (status = 200, description = "Successfully retrieved available sectors", body = SectorsResponse),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("country" = Option<String>, Query, description = "Filter sectors by country code (e.g., 'america', 'uk')")
    )
)]
pub async fn get_sectors_by_country(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<SectorsResponse>, AppError> {
    debug!("Getting sectors for country: {:?}", params.country);

    let sectors = service.get_sectors_by_country(params.country.clone()).await
        .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ExternalServiceError, e.to_string()))?;
    debug!("Found {} sectors", sectors.len());

    let response = SectorsResponse {
        count: sectors.len(),
        sectors,
        country: params.country,
    };

    info!("Returning {} sectors for country {:?}", response.count, response.country);
    Ok(Json(response))
}

/// GET /api/analytics/filters
/// Returns combined filter options for the frontend
#[utoipa::path(
    get,
    path = "/api/analytics/filters",
    tag = "analytics",
    responses(
        (status = 200, description = "Successfully retrieved filter options", body = FiltersResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_filter_options(
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<FiltersResponse>, AppError> {
    debug!("Getting combined filter options for frontend");

    let countries = get_available_countries_with_labels();
    let sectors = service.get_sectors_by_country(None).await
        .unwrap_or_else(|_| get_available_sectors_static());
    let exchanges = vec![
        "NASDAQ".to_string(),
        "NYSE".to_string(),
        "LSE".to_string(),
        "TSX".to_string(),
        "ASX".to_string(),
        "HKEX".to_string(),
        "TSE".to_string(),
        "EURONEXT".to_string(),
    ];
    let stock_types = vec![
        "common".to_string(),
        "preferred".to_string(),
        "reit".to_string(),
        "etf".to_string(),
    ];

    let response = FiltersResponse {
        countries,
        sectors,
        exchanges,
        stock_types,
    };

    info!("Returning combined filter options with {} countries, {} sectors, {} exchanges, {} stock types", 
        response.countries.len(), response.sectors.len(), response.exchanges.len(), response.stock_types.len());
    Ok(Json(response))
}

/// Get static list of available countries with proper display names
pub fn get_available_countries_with_labels() -> Vec<CountryData> {
    vec![
        CountryData { value: "america".to_string(), label: "United States".to_string() },
        CountryData { value: "argentina".to_string(), label: "Argentina".to_string() },
        CountryData { value: "australia".to_string(), label: "Australia".to_string() },
        CountryData { value: "austria".to_string(), label: "Austria".to_string() },
        CountryData { value: "bahrain".to_string(), label: "Bahrain".to_string() },
        CountryData { value: "bangladesh".to_string(), label: "Bangladesh".to_string() },
        CountryData { value: "belgium".to_string(), label: "Belgium".to_string() },
        CountryData { value: "brazil".to_string(), label: "Brazil".to_string() },
        CountryData { value: "canada".to_string(), label: "Canada".to_string() },
        CountryData { value: "chile".to_string(), label: "Chile".to_string() },
        CountryData { value: "china".to_string(), label: "China".to_string() },
        CountryData { value: "colombia".to_string(), label: "Colombia".to_string() },
        CountryData { value: "cyprus".to_string(), label: "Cyprus".to_string() },
        CountryData { value: "czech".to_string(), label: "Czech Republic".to_string() },
        CountryData { value: "denmark".to_string(), label: "Denmark".to_string() },
        CountryData { value: "egypt".to_string(), label: "Egypt".to_string() },
        CountryData { value: "estonia".to_string(), label: "Estonia".to_string() },
        CountryData { value: "finland".to_string(), label: "Finland".to_string() },
        CountryData { value: "france".to_string(), label: "France".to_string() },
        CountryData { value: "germany".to_string(), label: "Germany".to_string() },
        CountryData { value: "greece".to_string(), label: "Greece".to_string() },
        CountryData { value: "hongkong".to_string(), label: "Hong Kong".to_string() },
        CountryData { value: "hungary".to_string(), label: "Hungary".to_string() },
        CountryData { value: "iceland".to_string(), label: "Iceland".to_string() },
        CountryData { value: "india".to_string(), label: "India".to_string() },
        CountryData { value: "indonesia".to_string(), label: "Indonesia".to_string() },
        CountryData { value: "ireland".to_string(), label: "Ireland".to_string() },
        CountryData { value: "israel".to_string(), label: "Israel".to_string() },
        CountryData { value: "italy".to_string(), label: "Italy".to_string() },
        CountryData { value: "japan".to_string(), label: "Japan".to_string() },
        CountryData { value: "kenya".to_string(), label: "Kenya".to_string() },
        CountryData { value: "kuwait".to_string(), label: "Kuwait".to_string() },
        CountryData { value: "latvia".to_string(), label: "Latvia".to_string() },
        CountryData { value: "lithuania".to_string(), label: "Lithuania".to_string() },
        CountryData { value: "luxembourg".to_string(), label: "Luxembourg".to_string() },
        CountryData { value: "malaysia".to_string(), label: "Malaysia".to_string() },
        CountryData { value: "mexico".to_string(), label: "Mexico".to_string() },
        CountryData { value: "morocco".to_string(), label: "Morocco".to_string() },
        CountryData { value: "netherlands".to_string(), label: "Netherlands".to_string() },
        CountryData { value: "newzealand".to_string(), label: "New Zealand".to_string() },
        CountryData { value: "nigeria".to_string(), label: "Nigeria".to_string() },
        CountryData { value: "norway".to_string(), label: "Norway".to_string() },
        CountryData { value: "pakistan".to_string(), label: "Pakistan".to_string() },
        CountryData { value: "peru".to_string(), label: "Peru".to_string() },
        CountryData { value: "philippines".to_string(), label: "Philippines".to_string() },
        CountryData { value: "poland".to_string(), label: "Poland".to_string() },
        CountryData { value: "portugal".to_string(), label: "Portugal".to_string() },
        CountryData { value: "qatar".to_string(), label: "Qatar".to_string() },
        CountryData { value: "romania".to_string(), label: "Romania".to_string() },
        CountryData { value: "russia".to_string(), label: "Russia".to_string() },
        CountryData { value: "ksa".to_string(), label: "Saudi Arabia".to_string() },
        CountryData { value: "serbia".to_string(), label: "Serbia".to_string() },
        CountryData { value: "singapore".to_string(), label: "Singapore".to_string() },
        CountryData { value: "slovakia".to_string(), label: "Slovakia".to_string() },
        CountryData { value: "rsa".to_string(), label: "South Africa".to_string() },
        CountryData { value: "korea".to_string(), label: "South Korea".to_string() },
        CountryData { value: "spain".to_string(), label: "Spain".to_string() },
        CountryData { value: "srilanka".to_string(), label: "Sri Lanka".to_string() },
        CountryData { value: "sweden".to_string(), label: "Sweden".to_string() },
        CountryData { value: "switzerland".to_string(), label: "Switzerland".to_string() },
        CountryData { value: "taiwan".to_string(), label: "Taiwan".to_string() },
        CountryData { value: "thailand".to_string(), label: "Thailand".to_string() },
        CountryData { value: "tunisia".to_string(), label: "Tunisia".to_string() },
        CountryData { value: "turkey".to_string(), label: "Turkey".to_string() },
        CountryData { value: "uae".to_string(), label: "United Arab Emirates".to_string() },
        CountryData { value: "uk".to_string(), label: "United Kingdom".to_string() },
        CountryData { value: "venezuela".to_string(), label: "Venezuela".to_string() },
        CountryData { value: "vietnam".to_string(), label: "Vietnam".to_string() },
    ]
}

/// Get static list of available countries (for backward compatibility)
pub fn get_available_countries_static() -> Vec<String> {
    get_available_countries_with_labels().into_iter().map(|c| c.value).collect()
}

/// Get static list of available sectors (matches TradingView sector.tr values)
pub fn get_available_sectors_static() -> Vec<String> {
    vec![
        "Technology".to_string(),
        "Healthcare".to_string(),
        "Financial Services".to_string(),
        "Consumer Cyclical".to_string(),
        "Consumer Defensive".to_string(),
        "Industrials".to_string(),
        "Energy".to_string(),
        "Utilities".to_string(),
        "Real Estate".to_string(),
        "Basic Materials".to_string(),
        "Communication Services".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countries_response() {
        let countries = get_available_countries_with_labels();
        assert!(!countries.is_empty());
        
        // Check US is included
        let us = countries.iter().find(|c| c.value == "america");
        assert!(us.is_some());
        assert_eq!(us.unwrap().label, "United States");
    }

    #[test]
    fn test_sectors_static() {
        let sectors = get_available_sectors_static();
        assert!(!sectors.is_empty());
        assert!(sectors.contains(&"Technology".to_string()));
        assert!(sectors.contains(&"Healthcare".to_string()));
    }
}