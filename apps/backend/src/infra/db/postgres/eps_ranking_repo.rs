use async_trait::async_trait;
use sqlx::{PgPool, Row, QueryBuilder, Postgres};
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use rust_decimal::Decimal;

use crate::core::errors::{ErrorKind, ErrorContextBuilder};

use crate::core::errors::AppError;
use crate::dom::entities::eps_growth::{EPSGrowthData, EPSRanking};
use crate::dom::services::eps_ranking_service::EPSRepository;

/// PostgreSQL implementation of EPS Repository
pub struct PostgresEPSRepository {
    pool: Arc<PgPool>,
}

impl PostgresEPSRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Build ORDER BY clause based on sort parameter
    fn build_order_clause(sort_by: &Option<String>) -> String {
        match sort_by.as_deref() {
            Some("qoq_growth") => "qoq_growth_rate DESC NULLS LAST",
            Some("market_cap") => "market_cap DESC NULLS LAST",
            Some("volume") => "volume DESC NULLS LAST", 
            Some("name") => "name ASC",
            Some("current_eps") => "current_eps DESC NULLS LAST",
            Some("ranking_score") => "ranking_score DESC NULLS LAST",
            Some("symbol") => "symbol ASC",
            _ => "qoq_growth_rate DESC NULLS LAST" // Default sorting
        }.to_string()
    }

    /// Build WHERE clause conditions
    fn build_where_conditions<'a>(
        query: &mut QueryBuilder<'a, Postgres>, 
        country: &'a Option<String>,
        sector: &'a Option<String>,
        min_eps: &'a Option<f64>,
        min_growth: &'a Option<f64>,
    ) {
        if let Some(ref country) = country {
            debug!("Adding country filter: {}", country);
            query.push(" AND country = ");
            query.push_bind(country);
        }

        if let Some(ref sector) = sector {
            debug!("Adding sector filter: {}", sector);
            query.push(" AND sector = ");
            query.push_bind(sector);
        }

        if let Some(min_eps) = min_eps {
            debug!("Adding min EPS filter: {}", min_eps);
            query.push(" AND current_eps >= ");
            query.push_bind(min_eps);
        }

        if let Some(min_growth) = min_growth {
            debug!("Adding min growth filter: {}", min_growth);
            query.push(" AND qoq_growth_rate >= ");
            query.push_bind(min_growth);
        }

        // Always filter out stocks with missing essential data
        query.push(" AND current_eps IS NOT NULL");
        query.push(" AND qoq_growth_rate IS NOT NULL");
        query.push(" AND price_current IS NOT NULL");
    }
}

#[async_trait]
impl EPSRepository for PostgresEPSRepository {
    async fn store_eps_data(&self, eps_data: EPSGrowthData) -> Result<(), AppError> {
        debug!("Storing EPS data for symbol: {}", eps_data.symbol);

        let query = sqlx::query!(
            r#"
            INSERT INTO eps_growth_analytics (
                symbol, name, country, sector, exchange, 
                current_eps, qoq_growth_rate, price_current, 
                market_cap, volume, ranking_score
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (symbol) 
            DO UPDATE SET
                name = EXCLUDED.name,
                country = EXCLUDED.country,
                sector = EXCLUDED.sector,
                exchange = EXCLUDED.exchange,
                current_eps = EXCLUDED.current_eps,
                qoq_growth_rate = EXCLUDED.qoq_growth_rate,
                price_current = EXCLUDED.price_current,
                market_cap = EXCLUDED.market_cap,
                volume = EXCLUDED.volume,
                ranking_score = EXCLUDED.ranking_score,
                updated_at = NOW()
            "#,
            eps_data.symbol,
            eps_data.name,
            eps_data.country,
            eps_data.sector,
            eps_data.exchange,
            eps_data.current_eps.map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
            eps_data.qoq_growth.map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
            eps_data.price_current.map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
            eps_data.market_cap,
            eps_data.volume,
            eps_data.ranking_score.map(|v| Decimal::from_f64_retain(v).unwrap_or_default())
        );

        match query.execute(&*self.pool).await {
            Ok(result) => {
                debug!("Successfully stored EPS data for {} (rows affected: {})", 
                       eps_data.symbol, result.rows_affected());
                Ok(())
            }
            Err(e) => {
                error!("Failed to store EPS data for {}: {:?}", eps_data.symbol, e);
                Err(crate::database_error!("store_eps_data", e))
            }
        }
    }

    async fn get_rankings_filtered(
        &self,
        country: Option<String>,
        sort_by: Option<String>,
        page: i32,
        limit: i32,
    ) -> Result<Vec<EPSRanking>, AppError> {
        debug!("Getting EPS rankings - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
               country, sort_by, page, limit);

        let mut query = QueryBuilder::new(
            r#"
            SELECT 
                symbol, name, country, sector, exchange,
                current_eps, qoq_growth_rate, price_current, 
                market_cap, volume,
                ROW_NUMBER() OVER (ORDER BY "# 
        );
        
        // Add dynamic ORDER BY clause
        let order_clause = Self::build_order_clause(&sort_by);
        query.push(&order_clause);
        query.push(") as ranking_position FROM eps_growth_analytics WHERE 1=1");

        // Add WHERE conditions
        Self::build_where_conditions(&mut query, &country, &None, &None, &None);

        // Add ORDER BY for final result
        query.push(" ORDER BY ").push(&order_clause);

        // Add pagination
        let offset = (page - 1) * limit;
        debug!("Pagination - Offset: {}, Limit: {}", offset, limit);
        query.push(" OFFSET ").push_bind(offset);
        query.push(" LIMIT ").push_bind(limit);

        let built_query = query.build();
        debug!("Executing SQL query for EPS rankings");

        match built_query.fetch_all(&*self.pool).await {
            Ok(rows) => {
                let mut rankings = Vec::new();
                
                for row in rows {
                    let ranking = EPSRanking {
                        symbol: row.try_get("symbol").unwrap_or_default(),
                        name: row.try_get("name").unwrap_or_default(),
                        country: row.try_get("country").unwrap_or_default(),
                        sector: row.try_get("sector").unwrap_or_default(),
                        exchange: row.try_get("exchange").unwrap_or_default(),
                        current_eps: row.try_get("current_eps").ok(),
                        qoq_growth: row.try_get("qoq_growth_rate").ok(),
                        price_current: row.try_get("price_current").ok(),
                        market_cap: row.try_get("market_cap").ok(),
                        volume: row.try_get("volume").ok(),
                        ranking_position: row.try_get("ranking_position").ok(),
                    };
                    rankings.push(ranking);
                }

                debug!("Successfully retrieved {} EPS rankings", rankings.len());
                Ok(rankings)
            }
            Err(e) => {
                error!("Failed to get EPS rankings: {:?}", e);
                Err(crate::database_error!("get_eps_rankings", e))
            }
        }
    }

    async fn get_total_count(&self, country: Option<String>) -> Result<i64, AppError> {
        debug!("Getting total count for country: {:?}", country);

        let mut query = QueryBuilder::new(
            "SELECT COUNT(*) as total FROM eps_growth_analytics WHERE 1=1"
        );

        // Add country filter if provided
        if let Some(ref country) = country {
            debug!("Adding country filter to count query: {}", country);
            query.push(" AND country = ");
            query.push_bind(country);
        }

        // Filter out records with missing essential data
        query.push(" AND current_eps IS NOT NULL");
        query.push(" AND qoq_growth_rate IS NOT NULL");
        query.push(" AND price_current IS NOT NULL");

        let built_query = query.build();
        debug!("Executing count query");

        match built_query.fetch_one(&*self.pool).await {
            Ok(row) => {
                let total: i64 = row.try_get("total").unwrap_or(0);
                debug!("Total count: {}", total);
                Ok(total)
            }
            Err(e) => {
                error!("Failed to get total count: {:?}", e);
                Err(crate::database_error!("get_eps_rankings_count", e))
            }
        }
    }

    async fn batch_store_eps_data(&self, eps_data_list: Vec<EPSGrowthData>) -> Result<usize, AppError> {
        info!("Batch storing {} EPS data entries", eps_data_list.len());

        if eps_data_list.is_empty() {
            warn!("Empty EPS data list provided for batch storage");
            return Ok(0);
        }

        let mut stored_count = 0;
        let batch_size = 100; // Process in batches to avoid memory issues

        for chunk in eps_data_list.chunks(batch_size) {
            debug!("Processing batch of {} entries", chunk.len());

            // Start transaction for this batch
            let mut tx = self.pool.begin().await
                .map_err(|e| crate::database_error!("start_transaction", e))?;

            for eps_data in chunk {
                let result = sqlx::query!(
                    r#"
                    INSERT INTO eps_growth_analytics (
                        symbol, name, country, sector, exchange,
                        current_eps, qoq_growth_rate, price_current,
                        market_cap, volume, ranking_score
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    ON CONFLICT (symbol)
                    DO UPDATE SET
                        name = EXCLUDED.name,
                        country = EXCLUDED.country,
                        sector = EXCLUDED.sector,
                        exchange = EXCLUDED.exchange,
                        current_eps = EXCLUDED.current_eps,
                        qoq_growth_rate = EXCLUDED.qoq_growth_rate,
                        price_current = EXCLUDED.price_current,
                        market_cap = EXCLUDED.market_cap,
                        volume = EXCLUDED.volume,
                        ranking_score = EXCLUDED.ranking_score,
                        updated_at = NOW()
                    "#,
                    eps_data.symbol,
                    eps_data.name,
                    eps_data.country,
                    eps_data.sector,
                    eps_data.exchange,
                    eps_data.current_eps.map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
                    eps_data.qoq_growth.map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
                    eps_data.price_current.map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
                    eps_data.market_cap,
                    eps_data.volume,
                    eps_data.ranking_score.map(|v| Decimal::from_f64_retain(v).unwrap_or_default())
                ).execute(&mut *tx).await;

                match result {
                    Ok(_) => {
                        stored_count += 1;
                        debug!("Stored EPS data for symbol: {}", eps_data.symbol);
                    }
                    Err(e) => {
                        warn!("Failed to store EPS data for {}: {:?}", eps_data.symbol, e);
                        // Continue with other records in the batch
                    }
                }
            }

            // Commit this batch
            tx.commit().await
                .map_err(|e| crate::database_error!("commit_batch", e))?;

            debug!("Committed batch, stored {} entries so far", stored_count);
        }

        info!("Batch storage completed - stored {} out of {} entries", stored_count, eps_data_list.len());
        Ok(stored_count)
    }

    async fn get_countries(&self) -> Result<Vec<String>, AppError> {
        debug!("Getting distinct countries from EPS data");

        let query = sqlx::query!(
            r#"
            SELECT DISTINCT country 
            FROM eps_growth_analytics 
            WHERE country IS NOT NULL 
              AND current_eps IS NOT NULL
              AND qoq_growth_rate IS NOT NULL
            ORDER BY country ASC
            "#
        );

        match query.fetch_all(&*self.pool).await {
            Ok(rows) => {
                let countries: Vec<String> = rows
                    .into_iter()
                    .filter_map(|row| Some(row.country))
                    .collect();

                debug!("Found {} distinct countries", countries.len());
                Ok(countries)
            }
            Err(e) => {
                error!("Failed to get countries: {:?}", e);
                Err(crate::database_error!("get_countries", e))
            }
        }
    }

    async fn get_sectors_by_country(&self, country: Option<String>) -> Result<Vec<String>, AppError> {
        debug!("Getting sectors for country: {:?}", country);

        let mut query = QueryBuilder::new(
            r#"
            SELECT DISTINCT sector 
            FROM eps_growth_analytics 
            WHERE sector IS NOT NULL 
              AND current_eps IS NOT NULL
              AND qoq_growth_rate IS NOT NULL
            "#
        );

        if let Some(ref country) = country {
            debug!("Adding country filter for sectors: {}", country);
            query.push(" AND country = ");
            query.push_bind(country);
        }

        query.push(" ORDER BY sector ASC");

        let built_query = query.build();

        match built_query.fetch_all(&*self.pool).await {
            Ok(rows) => {
                let sectors: Vec<String> = rows
                    .into_iter()
                    .filter_map(|row| row.try_get("sector").ok())
                    .collect();

                debug!("Found {} sectors", sectors.len());
                Ok(sectors)
            }
            Err(e) => {
                error!("Failed to get sectors: {:?}", e);
                Err(crate::database_error!("get_sectors", e))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_order_clause() {
        assert_eq!(
            PostgresEPSRepository::build_order_clause(&Some("qoq_growth".to_string())),
            "qoq_growth_rate DESC NULLS LAST"
        );
        
        assert_eq!(
            PostgresEPSRepository::build_order_clause(&Some("name".to_string())),
            "name ASC"
        );
        
        assert_eq!(
            PostgresEPSRepository::build_order_clause(&None),
            "qoq_growth_rate DESC NULLS LAST"
        );
    }
}