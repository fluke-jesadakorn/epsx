use axum::{
    extract::{Query, Extension},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn, error};

use crate::core::errors::AppError;
use crate::dom::entities::eps_growth::{EPSRankingsResponse, EPSRanking};
use crate::dom::services::eps_ranking_service::{EPSRankingService, EPSRankingParams, CountryValidator};

/// Query parameters for EPS rankings endpoint
#[derive(Debug, Deserialize)]
pub struct EPSRankingQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

/// API response structure matching frontend pattern
#[derive(Debug, Serialize)]
pub struct EPSRankingsApiResponse {
    pub data: Vec<EPSRanking>,
    pub pagination: EPSPaginationResponse,
}

/// Pagination response structure
#[derive(Debug, Serialize)]
pub struct EPSPaginationResponse {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

/// Countries list response
#[derive(Debug, Serialize)]
pub struct CountriesResponse {
    pub countries: Vec<String>,
    pub count: usize,
}

/// Sectors list response
#[derive(Debug, Serialize)]
pub struct SectorsResponse {
    pub sectors: Vec<String>,
    pub count: usize,
    pub country: Option<String>,
}

/// Health check response for EPS service
#[derive(Debug, Serialize)]
pub struct EPSHealthResponse {
    pub status: String,
    pub message: String,
    pub available_countries: usize,
}

impl From<EPSRankingsResponse> for EPSRankingsApiResponse {
    fn from(response: EPSRankingsResponse) -> Self {
        Self {
            data: response.rankings,
            pagination: EPSPaginationResponse {
                page: response.pagination.page,
                limit: response.pagination.limit,
                total: response.pagination.total,
                total_pages: response.pagination.total_pages,
                has_next: response.pagination.has_next,
                has_prev: response.pagination.has_prev,
            },
        }
    }
}

/// GET /api/analytics/eps-rankings
/// Returns top EPS growth stocks with filtering and pagination
pub async fn get_eps_rankings(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<EPSRankingsApiResponse>, AppError> {
    debug!("EPS Rankings API called with params: {:?}", params);
    
    // Convert query params to service params with defaults
    let service_params = EPSRankingParams {
        country: params.country.clone(),
        sector: params.sector.clone(),
        sort_by: params.sort_by.clone().or(Some("qoq_growth".to_string())),
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(50),
        min_eps: params.min_eps,
        min_growth: params.min_growth,
    };

    debug!("Converted to service params: {:?}", service_params);

    // Validate parameters
    service.validate_ranking_params(&service_params)?;

    // Log request details for debugging
    info!("Processing EPS rankings request - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
          service_params.country, service_params.sort_by, service_params.page, service_params.limit);

    // Get rankings from service
    let start_time = std::time::Instant::now();
    let result = service.get_eps_rankings(service_params).await?;
    let duration = start_time.elapsed();

    // Log performance metrics
    debug!("EPS rankings query completed in {:?}", duration);
    info!("Returning {} EPS rankings for page {} (total: {})", 
          result.rankings.len(), result.pagination.page, result.pagination.total);

    // Convert to API response format
    let api_response = EPSRankingsApiResponse::from(result);

    Ok(Json(api_response))
}

/// GET /api/analytics/eps-rankings/countries
/// Returns list of available countries in EPS data
pub async fn get_available_countries(
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<CountriesResponse>, AppError> {
    debug!("Getting available countries from EPS data");

    let countries = service.get_available_countries().await?;
    debug!("Found {} countries in EPS data", countries.len());

    let response = CountriesResponse {
        count: countries.len(),
        countries,
    };

    info!("Returning {} available countries", response.count);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/countries/all
/// Returns complete list of valid countries from MarketCountry enum
pub async fn get_all_valid_countries() -> Result<Json<CountriesResponse>, AppError> {
    debug!("Getting all valid countries from MarketCountry enum");

    let countries = CountryValidator::get_valid_countries();
    debug!("Found {} valid countries", countries.len());

    let response = CountriesResponse {
        count: countries.len(),
        countries,
    };

    info!("Returning {} valid countries", response.count);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/sectors?country=america
/// Returns list of available sectors, optionally filtered by country
pub async fn get_sectors_by_country(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<SectorsResponse>, AppError> {
    debug!("Getting sectors for country: {:?}", params.country);

    let sectors = service.get_sectors_by_country(params.country.clone()).await?;
    debug!("Found {} sectors", sectors.len());

    let response = SectorsResponse {
        count: sectors.len(),
        sectors,
        country: params.country,
    };

    info!("Returning {} sectors for country {:?}", response.count, response.country);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/health
/// Health check endpoint for EPS analytics service
pub async fn eps_health_check(
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<EPSHealthResponse>, AppError> {
    debug!("EPS service health check requested");

    // Try to get available countries as a health indicator
    match service.get_available_countries().await {
        Ok(countries) => {
            let response = EPSHealthResponse {
                status: "healthy".to_string(),
                message: "EPS analytics service is operational".to_string(),
                available_countries: countries.len(),
            };
            info!("EPS service health check passed - {} countries available", countries.len());
            Ok(Json(response))
        }
        Err(e) => {
            error!("EPS service health check failed: {:?}", e);
            let response = EPSHealthResponse {
                status: "unhealthy".to_string(),
                message: format!("EPS analytics service error: {}", e),
                available_countries: 0,
            };
            Ok(Json(response))
        }
    }
}

/// Error handling for EPS-specific errors
impl From<AppError> for (StatusCode, Json<serde_json::Value>) {
    fn from(error: AppError) -> Self {
        use crate::core::errors::ErrorKind;
        
        match error.kind {
            ErrorKind::ValidationError => {
                warn!("Validation error in EPS API: {}", error.message);
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "validation_error",
                        "message": error.message,
                        "code": "EPS_VALIDATION_FAILED"
                    }))
                )
            }
            ErrorKind::DatabaseError => {
                error!("Database error in EPS API: {}", error.message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "database_error",
                        "message": "Internal server error",
                        "code": "EPS_DATABASE_ERROR"
                    }))
                )
            }
            _ => {
                error!("Unexpected error in EPS API: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "internal_error",
                        "message": "An unexpected error occurred",
                        "code": "EPS_INTERNAL_ERROR"
                    }))
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eps_ranking_query_params() {
        // Test default values
        let params = EPSRankingQueryParams {
            page: None,
            limit: None,
            country: None,
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        let service_params = EPSRankingParams {
            country: params.country,
            sector: params.sector,
            sort_by: params.sort_by.or(Some("qoq_growth".to_string())),
            page: params.page.unwrap_or(1),
            limit: params.limit.unwrap_or(50),
            min_eps: params.min_eps,
            min_growth: params.min_growth,
        };

        assert_eq!(service_params.page, 1);
        assert_eq!(service_params.limit, 50);
        assert_eq!(service_params.sort_by, Some("qoq_growth".to_string()));
    }

    #[test]
    fn test_pagination_response_serialization() {
        let pagination = EPSPaginationResponse {
            page: 1,
            limit: 50,
            total: 1000,
            total_pages: 20,
            has_next: true,
            has_prev: false,
        };

        let json = serde_json::to_string(&pagination).unwrap();
        assert!(json.contains("totalPages"));
        assert!(json.contains("hasNext"));
        assert!(json.contains("hasPrev"));
    }

    #[test]
    fn test_countries_response() {
        let response = CountriesResponse {
            countries: vec!["america".to_string(), "thailand".to_string()],
            count: 2,
        };

        assert_eq!(response.count, 2);
        assert!(response.countries.contains(&"america".to_string()));
    }
}