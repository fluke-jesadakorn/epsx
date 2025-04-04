use axum::{ extract::Query, routing::get, Json, Router };
use serde::Deserialize;
use std::sync::Arc;

use super::{ service::StockService, models::TableDataMetrics, StockServiceError };

pub fn stock_router(stock_service: Arc<StockService>) -> Router {
    Router::new()
        .route("/screener", get(stock_screener))
        .route("/eps-growth-ranking", get(eps_growth_ranking))
        .with_state(stock_service)
}

#[derive(Debug, Deserialize)]
pub struct EpsGrowthRankingParams {
    limit: Option<i32>,
    skip: Option<i32>,
    sort_by: Option<String>,
}

// Handler for /v1/stock/screener
async fn stock_screener(axum::extract::State(
    stock_service,
): axum::extract::State<Arc<StockService>>) -> Result<
    Json<Vec<TableDataMetrics>>,
    StockServiceError
> {
    let data = stock_service.fetch_stock_screener_data().await?;
    Ok(Json(data))
}

// Handler for /v1/stock/eps-growth-ranking
async fn eps_growth_ranking(
    axum::extract::State(stock_service): axum::extract::State<Arc<StockService>>,
    Query(params): Query<EpsGrowthRankingParams>
) -> Result<Json<Vec<TableDataMetrics>>, StockServiceError> {
    let data = stock_service.fetch_eps_growth_ranking(
        params.limit,
        params.skip,
        params.sort_by
    ).await?;
    Ok(Json(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_stock_screener() {
        let config = Config::from_env().unwrap();
        let service = Arc::new(StockService::new(config));

        let result = stock_screener(axum::extract::State(service)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_eps_growth_ranking() {
        let config = Config::from_env().unwrap();
        let service = Arc::new(StockService::new(config));

        let params = EpsGrowthRankingParams {
            limit: Some(10),
            skip: Some(0),
            sort_by: Some("activityScore".to_string()),
        };

        let result = eps_growth_ranking(axum::extract::State(service), Query(params)).await;
        assert!(result.is_ok());
    }
}
