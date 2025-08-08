use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::dom::entities::eps_growth::{EPSGrowthData, EPSRanking, EPSRankingsResponse, EPSPagination};
use crate::core::errors::AppError;

/// Repository trait for EPS data persistence
#[async_trait]
pub trait EPSRepository: Send + Sync {
    async fn store_eps_data(&self, eps_data: EPSGrowthData) -> Result<(), AppError>;
    async fn get_rankings_filtered(
        &self,
        country: Option<String>,
        sort_by: Option<String>,
        page: i32,
        limit: i32,
    ) -> Result<Vec<EPSRanking>, AppError>;
    async fn get_total_count(&self, country: Option<String>) -> Result<i64, AppError>;
    async fn batch_store_eps_data(&self, eps_data_list: Vec<EPSGrowthData>) -> Result<usize, AppError>;
    async fn get_countries(&self) -> Result<Vec<String>, AppError>;
    async fn get_sectors_by_country(&self, country: Option<String>) -> Result<Vec<String>, AppError>;
}

/// EPS Rankings Service for business logic
pub struct EPSRankingService {
    eps_repo: Arc<dyn EPSRepository>,
}

/// EPS ranking parameters for filtering and sorting
#[derive(Debug, Clone)]
pub struct EPSRankingParams {
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub page: i32,
    pub limit: i32,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

impl Default for EPSRankingParams {
    fn default() -> Self {
        Self {
            country: None,
            sector: None,
            sort_by: Some("qoq_growth".to_string()),
            page: 1,
            limit: 50,
            min_eps: None,
            min_growth: None,
        }
    }
}

/// Country validation helper
pub struct CountryValidator;

impl CountryValidator {
    /// Valid countries from MarketCountry enum
    const VALID_COUNTRIES: &'static [&'static str] = &[
        "america", "argentina", "australia", "austria", "bahrain", "bangladesh", 
        "belgium", "brazil", "canada", "chile", "china", "colombia", "cyprus", 
        "czech", "denmark", "egypt", "estonia", "finland", "france", "germany", 
        "greece", "hongkong", "hungary", "iceland", "india", "indonesia", 
        "ireland", "israel", "italy", "japan", "kenya", "kuwait", "latvia", 
        "lithuania", "luxembourg", "malaysia", "mexico", "morocco", "netherlands", 
        "newzealand", "nigeria", "norway", "pakistan", "peru", "philippines", 
        "poland", "portugal", "qatar", "romania", "russia", "ksa", "serbia", 
        "singapore", "slovakia", "rsa", "korea", "spain", "srilanka", "sweden", 
        "switzerland", "taiwan", "thailand", "tunisia", "turkey", "uae", "uk", 
        "venezuela", "vietnam"
    ];

    pub fn validate_country(country: &str) -> Result<String, String> {
        debug!("Validating country filter: {}", country);
        
        let normalized = country.to_lowercase();
        
        if Self::VALID_COUNTRIES.contains(&normalized.as_str()) {
            debug!("Country validation passed: {}", country);
            Ok(normalized)
        } else {
            warn!("Invalid country filter provided: {}", country);
            Err(format!("Invalid country: {}", country))
        }
    }

    pub fn get_valid_countries() -> Vec<String> {
        Self::VALID_COUNTRIES.iter().map(|&s| s.to_string()).collect()
    }
}

impl EPSRankingService {
    pub fn new(eps_repo: Arc<dyn EPSRepository>) -> Self {
        Self { eps_repo }
    }

    /// Get EPS rankings with filtering and pagination
    pub async fn get_eps_rankings(
        &self,
        params: EPSRankingParams,
    ) -> Result<EPSRankingsResponse, AppError> {
        debug!("Getting EPS rankings with params: {:?}", params);
        
        // Validate country if provided
        let validated_country = if let Some(ref country) = params.country {
            Some(CountryValidator::validate_country(country)
                .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ValidationError, e))?)
        } else {
            None
        };

        // Validate pagination parameters
        let page = params.page.max(1);
        let limit = params.limit.clamp(1, 100); // Max 100 items per page

        debug!("Validated params - Country: {:?}, Page: {}, Limit: {}", 
               validated_country, page, limit);

        // Get rankings from repository
        let rankings = self.eps_repo.get_rankings_filtered(
            validated_country.clone(),
            params.sort_by.clone(),
            page,
            limit,
        ).await?;

        debug!("Found {} rankings from repository", rankings.len());

        // Get total count for pagination
        let total_count = self.eps_repo.get_total_count(validated_country.clone()).await?;
        debug!("Total count for pagination: {}", total_count);

        // Create pagination info
        let pagination = EPSPagination::new(page, limit, total_count);

        let response = EPSRankingsResponse {
            rankings,
            pagination,
        };

        info!("Returning {} EPS rankings for page {} (total: {})", 
              response.rankings.len(), page, total_count);

        Ok(response)
    }

    /// Store EPS data with validation
    pub async fn store_eps_data(&self, mut eps_data: EPSGrowthData) -> Result<(), AppError> {
        debug!("Storing EPS data for symbol: {}", eps_data.symbol);

        // Validate data before storing
        eps_data.validate()
            .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ValidationError, format!("EPS data validation failed: {}", e)))?;

        // Calculate ranking score
        eps_data.calculate_ranking_score();

        // Validate country
        let validated_country = CountryValidator::validate_country(&eps_data.country)
            .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ValidationError, e))?;
        eps_data.country = validated_country;

        debug!("Validated EPS data for {} with ranking score: {:?}", 
               eps_data.symbol, eps_data.ranking_score);

        self.eps_repo.store_eps_data(eps_data).await?;
        debug!("Successfully stored EPS data");
        
        Ok(())
    }

    /// Batch store EPS data with validation
    pub async fn batch_store_eps_data(&self, eps_data_list: Vec<EPSGrowthData>) -> Result<usize, AppError> {
        info!("Batch storing {} EPS data entries", eps_data_list.len());

        let mut validated_data = Vec::new();
        let mut validation_errors = 0;

        for mut eps_data in eps_data_list {
            // Validate each entry
            match eps_data.validate() {
                Ok(()) => {
                    // Calculate ranking score
                    eps_data.calculate_ranking_score();
                    
                    // Validate and normalize country
                    match CountryValidator::validate_country(&eps_data.country) {
                        Ok(validated_country) => {
                            eps_data.country = validated_country;
                            validated_data.push(eps_data);
                        }
                        Err(e) => {
                            warn!("Country validation failed for {}: {}", eps_data.symbol, e);
                            validation_errors += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("EPS data validation failed for {}: {}", eps_data.symbol, e);
                    validation_errors += 1;
                }
            }
        }

        if validation_errors > 0 {
            warn!("Validation errors occurred for {} entries", validation_errors);
        }

        debug!("Validated {} out of {} EPS entries", validated_data.len(), validated_data.len() + validation_errors);

        if validated_data.is_empty() {
            warn!("No valid EPS data to store after validation");
            return Ok(0);
        }

        let stored_count = self.eps_repo.batch_store_eps_data(validated_data).await?;
        info!("Successfully stored {} EPS data entries", stored_count);

        Ok(stored_count)
    }

    /// Get available countries
    pub async fn get_available_countries(&self) -> Result<Vec<String>, AppError> {
        debug!("Getting available countries from EPS data");
        
        let countries = self.eps_repo.get_countries().await?;
        debug!("Found {} countries in EPS data", countries.len());
        
        Ok(countries)
    }

    /// Get available sectors by country
    pub async fn get_sectors_by_country(&self, country: Option<String>) -> Result<Vec<String>, AppError> {
        debug!("Getting sectors for country: {:?}", country);

        // Validate country if provided
        let validated_country = if let Some(ref country) = country {
            Some(CountryValidator::validate_country(country)
                .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ValidationError, e))?)
        } else {
            None
        };

        let sectors = self.eps_repo.get_sectors_by_country(validated_country).await?;
        debug!("Found {} sectors", sectors.len());
        
        Ok(sectors)
    }

    /// Validate ranking parameters
    pub fn validate_ranking_params(&self, params: &EPSRankingParams) -> Result<(), AppError> {
        debug!("Validating ranking parameters: {:?}", params);

        // Validate country
        if let Some(ref country) = params.country {
            CountryValidator::validate_country(country)
                .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ValidationError, e))?;
        }

        // Validate sort field
        if let Some(ref sort_by) = params.sort_by {
            let valid_sort_fields = ["qoq_growth", "market_cap", "volume", "name", "current_eps", "ranking_score"];
            if !valid_sort_fields.contains(&sort_by.as_str()) {
                return Err(AppError::new(crate::core::errors::ErrorKind::ValidationError,
                    format!("Invalid sort field: {}. Valid options: {:?}", sort_by, valid_sort_fields)
                ));
            }
        }

        // Validate pagination
        if params.page < 1 {
            return Err(AppError::new(crate::core::errors::ErrorKind::ValidationError, "Page must be >= 1"));
        }

        if params.limit < 1 || params.limit > 100 {
            return Err(AppError::new(crate::core::errors::ErrorKind::ValidationError, "Limit must be between 1 and 100"));
        }

        // Validate numeric filters
        if let Some(min_eps) = params.min_eps {
            if min_eps < -1000.0 || min_eps > 1000.0 {
                return Err(AppError::new(crate::core::errors::ErrorKind::ValidationError, "min_eps must be between -1000 and 1000"));
            }
        }

        if let Some(min_growth) = params.min_growth {
            if min_growth < -500.0 || min_growth > 1000.0 {
                return Err(AppError::new(crate::core::errors::ErrorKind::ValidationError, "min_growth must be between -500 and 1000"));
            }
        }

        debug!("Ranking parameters validation passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_validation() {
        // Valid countries
        assert!(CountryValidator::validate_country("america").is_ok());
        assert!(CountryValidator::validate_country("AMERICA").is_ok());
        assert!(CountryValidator::validate_country("thailand").is_ok());
        
        // Invalid countries
        assert!(CountryValidator::validate_country("invalid").is_err());
        assert!(CountryValidator::validate_country("").is_err());
    }

    #[test]
    fn test_ranking_params_default() {
        let params = EPSRankingParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.limit, 50);
        assert_eq!(params.sort_by, Some("qoq_growth".to_string()));
    }

    #[test]
    fn test_eps_pagination() {
        let pagination = EPSPagination::new(1, 50, 1547);
        assert_eq!(pagination.total_pages, 31);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);

        let pagination = EPSPagination::new(31, 50, 1547);
        assert!(!pagination.has_next);
        assert!(pagination.has_prev);
    }
}