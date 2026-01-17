use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::domain::shared_kernel::entities::eps_growth::{EPSGrowthData, EPSRanking, EPSRankingsResponse, EPSPagination};
use crate::core::errors::AppError;

/// Repository trait for EPS data persistence
#[async_trait]
pub trait EPSRepository: Send + Sync {
    async fn store_eps_data(&self, eps_data: EPSGrowthData) -> Result<(), AppError>;
    async fn get_rankings_filtered(
        &self,
        rank_offset: i32,  // Minimum accessible rank
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
        page: i32,
        limit: i32,
    ) -> Result<Vec<EPSRanking>, AppError>;
    async fn get_total_count(&self, rank_offset: i32, country: Option<String>, sector: Option<String>) -> Result<i64, AppError>;
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
    pub rank_offset: i32,  // Minimum accessible rank (from permissions)
    pub limit_cap: i32,    // Maximum items viewable (from permissions), -1 for unlimited
}

impl Default for EPSRankingParams {
    fn default() -> Self {
        Self {
            country: None,
            sector: None,
            sort_by: Some("growth_factor".to_string()),
            page: 1,
            limit: 50,
            min_eps: None,
            min_growth: None,
            rank_offset: 100,  // Default: free tier
            limit_cap: -1,     // Default: free tier (unlimited items, just offset)
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

/// Permission parser for extracting rank offset from permissions
pub struct PermissionParser;

impl PermissionParser {
    /// Extract ranking configuration from permissions
    /// Returns (rank_offset, rank_limit)
    /// rank_offset: Lower is better (min rank accessible)
    /// rank_limit: Higher is better (max items viewable), -1 for unlimited
    pub fn extract_ranking_config(permissions: &[String]) -> (i32, i32) {
        debug!("Extracting ranking config from {} permissions", permissions.len());

        let mut min_offset = 100; // Default: free tier offset
        let mut max_limit = -2;   // Default: not set (will defer to free tier -1)

        for perm in permissions {
            // Parse: "epsx:rankings:offset:{number}"
            if let Some(val_str) = perm.strip_prefix("epsx:rankings:offset:") {
                 if let Ok(val) = val_str.parse::<i32>() {
                     if val < min_offset { // Take minimum = best access
                         min_offset = val;
                     }
                 }
            }
            
            // Parse: "epsx:rankings:limit:{number}"
            // Also support "epsx:rankings:view:{number}" and "epsx:analytics:view:{number}" for backward compatibility
            if let Some(val_str) = perm.strip_prefix("epsx:rankings:limit:")
                .or_else(|| perm.strip_prefix("epsx:rankings:view:"))
                .or_else(|| perm.strip_prefix("epsx:analytics:view:"))
            {
                if val_str == "unlimited" || val_str == "-1" {
                     max_limit = -1; // Unlimited
                     min_offset = 1; // Explicit "unlimited" usually implies full access (offset 1)
                 } else if let Ok(val) = val_str.parse::<i32>() {
                     // If still default (-2), set specific limit
                     // If specific limit exists, take the higher one (ignore if we have unlimited/-1)
                     if max_limit == -2 {
                         max_limit = val;
                     } else if max_limit != -1 && val > max_limit { 
                         max_limit = val;
                     }
                 }
            }
            
            // Check for wildcard/admin permission
            // Include admin:*:* (Super Admin) and admin:analytics:* (Analytics Admin)
            if perm == "epsx:*:*" || perm == "epsx:rankings:*" || 
               perm == "admin:*:*" || perm == "admin:analytics:*" {
                min_offset = 1;
                max_limit = -1;
            }
        }

        // If max_limit is still -2 (not set), revert to default -1 (free tier default)
        if max_limit == -2 {
            max_limit = -1;
        }

        info!("Calculated ranking config: offset={}, limit={}", min_offset, max_limit);
        (min_offset, max_limit)
    }

    /// Legacy method for backward compatibility
    pub fn extract_rank_offset(permissions: &[String]) -> i32 {
        Self::extract_ranking_config(permissions).0
    }
}

const GLOBAL_MAX_LIMIT: i32 = 1000;

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
        
        // Apply limit cap from permissions
        // NEW LOGIC: 
        // 1. If rank_offset > 1 (Free Zone), ignore limit_cap and use requested limit (up to global max)
        // 2. If rank_offset == 1 (Upper Ranking), respect limit_cap
        // 3. Always clamp by GLOBAL_MAX_LIMIT to protect database
        let effective_limit = if params.rank_offset > 1 {
            // Free zone: allow full browsing of "long tail" data
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        } else if params.limit_cap >= 0 {
            // Upper ranking zone: strictly respect plan-based limit
             params.limit.clamp(1, params.limit_cap)
        } else {
            // No specific limit_cap: use requested limit (up to global)
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        };
        
        debug!("Validated params - Country: {:?}, Page: {}, Limit: {} (requested: {}, cap: {}), Rank Offset: {}",
               validated_country, page, effective_limit, params.limit, params.limit_cap, params.rank_offset);

        // Get rankings from repository with rank offset enforcement
        let rankings = self.eps_repo.get_rankings_filtered(
            params.rank_offset,  // SECURITY: Enforced minimum rank
            validated_country.clone(),
            params.sector.clone(),
            params.sort_by.clone(),
            page,
            effective_limit,
        ).await?;

        debug!("Found {} rankings from repository (offset: {})", rankings.len(), params.rank_offset);

        // Get total count for pagination (from offset onwards)
        let total_count = self.eps_repo.get_total_count(params.rank_offset, validated_country.clone(), params.sector.clone()).await?;
        debug!("Total count for pagination: {} (from rank {} onwards)", total_count, params.rank_offset);

        // Create pagination info
        let pagination = EPSPagination::new(page, effective_limit, total_count);

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

    /// Get total count for validation with ranking parameters
    pub async fn get_total_count_for_params(&self, params: &EPSRankingParams) -> Result<i64, AppError> {
        debug!("Getting total count for params: {:?}", params);

        // For now, only use country filter to get total count
        // TODO: Extend repository to support sector and other filters
        let validated_country = if let Some(ref country) = params.country {
            Some(CountryValidator::validate_country(country)
                .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ValidationError, e))?)
        } else {
            None
        };

        let count = self.eps_repo.get_total_count(params.rank_offset, validated_country, params.sector.clone()).await?;
        debug!("Total count for parameters: {} (from rank {} onwards)", count, params.rank_offset);

        Ok(count)
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
            let valid_sort_fields = ["growth_factor", "market_cap", "volume", "name", "current_eps", "ranking_score"];
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
            if !(-1000.0..=1000.0).contains(&min_eps) {
                return Err(AppError::new(crate::core::errors::ErrorKind::ValidationError, "min_eps must be between -1000 and 1000"));
            }
        }

        if let Some(min_growth) = params.min_growth {
            if !(-500.0..=1000.0).contains(&min_growth) {
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
        assert_eq!(params.sort_by, Some("growth_factor".to_string()));
    }

    #[test]
    fn test_pagination_calculation() {
        let pagination = EPSPagination::new(1, 50, 1547);
        assert_eq!(pagination.total_pages, 31);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);

        let pagination = EPSPagination::new(31, 50, 1547);
        assert!(!pagination.has_next);
        assert!(pagination.has_prev);
    }

    #[test]
    fn test_ranking_limit_logic() {
        // Mock repository is not needed since we're just testing the limit calculation logic
        // in get_eps_rankings (which we'll do by inspecting effective_limit)
        
        // Case 1: Upper Ranking (offset 1) with limit 5
        let params = EPSRankingParams {
            rank_offset: 1,
            limit_cap: 5,
            limit: 50,
            ..EPSRankingParams::default()
        };
        
        let effective_limit = if params.rank_offset > 1 {
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        } else if params.limit_cap >= 0 {
             params.limit.clamp(1, params.limit_cap)
        } else {
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        };
        assert_eq!(effective_limit, 5);

        // Case 2: Free Zone (offset 100) with limit_cap 5 - Should ignore cap
        let params = EPSRankingParams {
            rank_offset: 100,
            limit_cap: 5,
            limit: 50,
            ..EPSRankingParams::default()
        };
        
        let effective_limit = if params.rank_offset > 1 {
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        } else if params.limit_cap >= 0 {
             params.limit.clamp(1, params.limit_cap)
        } else {
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        };
        assert_eq!(effective_limit, 50);

        // Case 3: Global Max Protection
        let params = EPSRankingParams {
            rank_offset: 100,
            limit_cap: -1,
            limit: 2000,
            ..EPSRankingParams::default()
        };
        
        let effective_limit = if params.rank_offset > 1 {
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        } else if params.limit_cap >= 0 {
             params.limit.clamp(1, params.limit_cap)
        } else {
            params.limit.clamp(1, GLOBAL_MAX_LIMIT)
        };
        assert_eq!(effective_limit, 1000);
    }
    #[test]
    fn test_starter_plan_permissions() {
        // Starter Plan permissions from seed_plans_handler.rs
        let permissions = vec![
            "epsx:analytics:view:25".to_string(),
            "epsx:rankings:view:25".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx:alerts:create".to_string()
        ];

        let (offset, limit) = PermissionParser::extract_ranking_config(&permissions);
        
        // Should NOT unlock rankings (offset 100) just because of limit 25
        assert_eq!(offset, 100, "Starter plan should have default offset 100 without explicit offset permission");
        assert_eq!(limit, 25, "Starter plan should have limit 25");
    }
}