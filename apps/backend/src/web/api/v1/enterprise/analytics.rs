// Enterprise Analytics API
// Advanced analytics endpoints for enterprise customers

use axum::{
    extract::{Query, State, Path},
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::infrastructure::container::DomainContainer;
use crate::web::middleware::web3_enterprise_auth::{Web3AuthenticatedUser, EnterpriseTier};
use crate::core::errors::AppError;

type ApiResult<T> = Result<T, AppError>;

/// Create analytics routes for enterprise API
pub fn create_analytics_routes() -> Router<Arc<DomainContainer>> {
    Router::new()
        .route("/rankings", get(get_enterprise_rankings))
        .route("/performance", get(get_performance_analytics))
        .route("/market-data", get(get_market_data))
        .route("/portfolio/:wallet_address", get(get_portfolio_analytics))
        .route("/custom-metrics", post(create_custom_metrics))
        .route("/alerts", get(get_analytics_alerts))
        .route("/alerts", post(create_analytics_alert))
        .route("/reports", get(get_analytics_reports))
        .route("/exports/:format", get(export_analytics_data))
}

/// Request parameters for enterprise rankings
#[derive(Debug, Deserialize)]
pub struct RankingsQuery {
    pub symbols: Option<String>,
    pub sectors: Option<String>,
    pub countries: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub timeframe: Option<String>,
    pub sort_by: Option<String>,
    pub include_historical: Option<bool>,
}

/// Enterprise analytics rankings with enhanced data
pub async fn get_enterprise_rankings(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Query(params): Query<RankingsQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check tier access
    if !user.enterprise_tier.has_feature_access("basic_analytics") {
        return Err(AppError::forbidden("Insufficient tier for analytics access"));
    }

    let limit = params.limit.unwrap_or(100).min(1000); // Max 1000 for enterprise
    let offset = params.offset.unwrap_or(0);
    let timeframe = params.timeframe.as_deref().unwrap_or("1d");
    
    // Enhanced data for enterprise tiers
    let include_advanced_metrics = user.enterprise_tier.has_feature_access("real_time_data");
    let include_custom_indicators = user.enterprise_tier.has_feature_access("custom_integration");

    let response = serde_json::json!({
        "data": {
            "rankings": generate_mock_rankings(limit, offset, &user.enterprise_tier),
            "metadata": {
                "total_count": 5000,
                "returned_count": limit,
                "offset": offset,
                "timeframe": timeframe,
                "last_updated": Utc::now(),
                "data_quality": if include_advanced_metrics { "premium" } else { "standard" }
            },
            "enhanced_metrics": if include_advanced_metrics {
                Some(serde_json::json!({
                    "volatility_analysis": true,
                    "correlation_matrix": true,
                    "risk_metrics": true,
                    "sentiment_analysis": true
                }))
            } else { None },
            "custom_indicators": if include_custom_indicators {
                Some(serde_json::json!({
                    "custom_formulas": [],
                    "user_defined_metrics": [],
                    "personalized_scores": []
                }))
            } else { None }
        },
        "tier_info": {
            "current_tier": format!("{:?}", user.enterprise_tier),
            "data_retention_days": user.enterprise_tier.data_retention_days(),
            "rate_limit_remaining": user.enterprise_tier.rate_limit_per_minute(),
            "features_enabled": get_tier_features(&user.enterprise_tier)
        },
        "timestamp": Utc::now()
    });

    Ok(Json(response))
}

/// Performance analytics for enterprise customers
#[derive(Debug, Deserialize)]
pub struct PerformanceQuery {
    pub wallet_addresses: Option<String>, // Comma-separated
    pub timeframe: Option<String>,
    pub include_breakdown: Option<bool>,
    pub benchmark: Option<String>,
}

pub async fn get_performance_analytics(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Query(params): Query<PerformanceQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check advanced analytics access
    if !user.enterprise_tier.has_feature_access("real_time_data") {
        return Err(AppError::forbidden("Performance analytics requires Business tier or higher"));
    }

    let wallet_addresses = params.wallet_addresses
        .as_deref()
        .unwrap_or(&user.wallet_address)
        .split(',')
        .map(|s| s.trim())
        .collect::<Vec<_>>();

    // Limit wallet addresses based on tier
    let max_wallets = match user.enterprise_tier {
        EnterpriseTier::Business => 10,
        EnterpriseTier::Enterprise => 100,
        EnterpriseTier::Whale => 1000,
        _ => 1,
    };

    if wallet_addresses.len() > max_wallets {
        return Err(AppError::bad_request(&format!(
            "Too many wallet addresses. Maximum for your tier: {}", max_wallets
        )));
    }

    let response = serde_json::json!({
        "performance_data": {
            "wallets": wallet_addresses.iter().map(|addr| {
                serde_json::json!({
                    "wallet_address": addr,
                    "total_value_usd": 150000.0 + (addr.len() as f64 * 1000.0),
                    "24h_change_percent": 2.5,
                    "7d_change_percent": -1.2,
                    "30d_change_percent": 15.8,
                    "portfolio_breakdown": {
                        "defi_protocols": 45.0,
                        "nft_collections": 25.0,
                        "tokens": 30.0
                    },
                    "risk_metrics": {
                        "volatility": 0.35,
                        "sharpe_ratio": 1.2,
                        "max_drawdown": -12.5
                    }
                })
            }).collect::<Vec<_>>(),
            "aggregated_metrics": {
                "total_portfolio_value": wallet_addresses.len() as f64 * 150000.0,
                "best_performer": wallet_addresses.first(),
                "worst_performer": wallet_addresses.last(),
                "correlation_to_market": 0.75
            }
        },
        "tier_benefits": {
            "max_wallets_tracked": max_wallets,
            "real_time_updates": user.enterprise_tier.has_feature_access("real_time_data"),
            "custom_benchmarks": user.enterprise_tier.has_feature_access("custom_integration")
        },
        "timestamp": Utc::now()
    });

    Ok(Json(response))
}

/// Market data with enterprise enhancements
#[derive(Debug, Deserialize)]
pub struct MarketDataQuery {
    pub symbols: Option<String>,
    pub include_orderbook: Option<bool>,
    pub depth: Option<u32>,
    pub include_trades: Option<bool>,
}

pub async fn get_market_data(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Query(params): Query<MarketDataQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let symbols = params.symbols.as_deref().unwrap_or("BTC,ETH,BNB");
    let include_orderbook = params.include_orderbook.unwrap_or(false) && 
        user.enterprise_tier.has_feature_access("real_time_data");
    let include_trades = params.include_trades.unwrap_or(false) &&
        user.enterprise_tier.has_feature_access("real_time_data");

    let response = serde_json::json!({
        "market_data": symbols.split(',').map(|symbol| {
            let symbol = symbol.trim();
            serde_json::json!({
                "symbol": symbol,
                "price_usd": match symbol {
                    "BTC" => 65000.0,
                    "ETH" => 3200.0,
                    "BNB" => 580.0,
                    _ => 100.0
                },
                "24h_change_percent": 1.5,
                "volume_24h": 1000000000.0,
                "market_cap": 1000000000000.0,
                "orderbook": if include_orderbook {
                    Some(serde_json::json!({
                        "bids": [[64900.0, 0.5], [64800.0, 1.2]],
                        "asks": [[65100.0, 0.3], [65200.0, 0.8]],
                        "depth": params.depth.unwrap_or(10)
                    }))
                } else { None },
                "recent_trades": if include_trades {
                    Some(serde_json::json!([
                        {"price": 65000.0, "quantity": 0.1, "timestamp": Utc::now()},
                        {"price": 64999.0, "quantity": 0.2, "timestamp": Utc::now()}
                    ]))
                } else { None }
            })
        }).collect::<Vec<_>>(),
        "enterprise_features": {
            "real_time_updates": user.enterprise_tier.has_feature_access("real_time_data"),
            "market_depth_available": include_orderbook,
            "trade_history_available": include_trades,
            "custom_symbols_supported": user.enterprise_tier.has_feature_access("custom_integration")
        },
        "timestamp": Utc::now()
    });

    Ok(Json(response))
}

/// Portfolio analytics for specific wallet
pub async fn get_portfolio_analytics(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Path(wallet_address): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate wallet address access
    if wallet_address != user.wallet_address && 
       !user.enterprise_tier.has_feature_access("custom_integration") {
        return Err(AppError::forbidden("Cross-wallet analytics requires Enterprise tier"));
    }

    let response = serde_json::json!({
        "wallet_address": wallet_address,
        "portfolio_summary": {
            "total_value_usd": 250000.0,
            "token_count": 45,
            "nft_count": 12,
            "defi_positions": 8,
            "last_activity": Utc::now()
        },
        "asset_allocation": {
            "tokens": {
                "value_usd": 175000.0,
                "percentage": 70.0,
                "top_holdings": [
                    {"symbol": "ETH", "value_usd": 100000.0, "percentage": 40.0},
                    {"symbol": "USDC", "value_usd": 50000.0, "percentage": 20.0}
                ]
            },
            "nfts": {
                "value_usd": 50000.0,
                "percentage": 20.0,
                "collections": 5
            },
            "defi": {
                "value_usd": 25000.0,
                "percentage": 10.0,
                "protocols": ["Uniswap", "Compound", "AAVE"]
            }
        },
        "risk_analysis": if user.enterprise_tier.has_feature_access("real_time_data") {
            Some(serde_json::json!({
                "portfolio_beta": 1.2,
                "concentration_risk": "medium",
                "liquidity_score": 0.85,
                "correlation_breakdown": {
                    "btc_correlation": 0.7,
                    "market_correlation": 0.8
                }
            }))
        } else { None },
        "timestamp": Utc::now()
    });

    Ok(Json(response))
}

/// Custom metrics creation for enterprise customers
#[derive(Debug, Deserialize)]
pub struct CustomMetricsRequest {
    pub name: String,
    pub description: Option<String>,
    pub formula: String,
    pub data_sources: Vec<String>,
    pub update_frequency: String,
}

pub async fn create_custom_metrics(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<CustomMetricsRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Require Enterprise tier for custom metrics
    if !user.enterprise_tier.has_feature_access("custom_integration") {
        return Err(AppError::forbidden("Custom metrics require Enterprise tier or higher"));
    }

    let response = serde_json::json!({
        "metric_id": uuid::Uuid::new_v4(),
        "name": request.name,
        "description": request.description,
        "formula": request.formula,
        "status": "created",
        "data_sources": request.data_sources,
        "update_frequency": request.update_frequency,
        "created_by": user.wallet_address,
        "created_at": Utc::now(),
        "estimated_processing_time": "5-10 minutes"
    });

    Ok(Json(response))
}

/// Get analytics alerts
pub async fn get_analytics_alerts(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    if !user.enterprise_tier.has_feature_access("real_time_data") {
        return Err(AppError::forbidden("Analytics alerts require Business tier or higher"));
    }

    let response = serde_json::json!({
        "alerts": [
            {
                "id": uuid::Uuid::new_v4(),
                "type": "price_threshold",
                "symbol": "ETH",
                "condition": "price > 3500",
                "status": "active",
                "created_at": Utc::now()
            }
        ],
        "total_alerts": 1,
        "active_alerts": 1,
        "max_alerts_for_tier": match user.enterprise_tier {
            EnterpriseTier::Business => 50,
            EnterpriseTier::Enterprise => 500,
            EnterpriseTier::Whale => u32::MAX,
            _ => 10,
        }
    });

    Ok(Json(response))
}

/// Create analytics alert
#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub alert_type: String,
    pub symbol: Option<String>,
    pub condition: String,
    pub webhook_url: Option<String>,
}

pub async fn create_analytics_alert(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Json(request): Json<CreateAlertRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if !user.enterprise_tier.has_feature_access("real_time_data") {
        return Err(AppError::forbidden("Analytics alerts require Business tier or higher"));
    }

    let response = serde_json::json!({
        "alert_id": uuid::Uuid::new_v4(),
        "type": request.alert_type,
        "symbol": request.symbol,
        "condition": request.condition,
        "webhook_url": request.webhook_url,
        "status": "active",
        "created_by": user.wallet_address,
        "created_at": Utc::now()
    });

    Ok(Json(response))
}

/// Get analytics reports
pub async fn get_analytics_reports(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
) -> ApiResult<Json<serde_json::Value>> {
    let response = serde_json::json!({
        "reports": [
            {
                "id": uuid::Uuid::new_v4(),
                "name": "Monthly Performance Report",
                "type": "performance",
                "period": "monthly",
                "status": "ready",
                "generated_at": Utc::now()
            }
        ],
        "available_report_types": get_available_reports(&user.enterprise_tier)
    });

    Ok(Json(response))
}

/// Export analytics data in various formats
pub async fn export_analytics_data(
    State(_container): State<Arc<DomainContainer>>,
    user: Web3AuthenticatedUser,
    Path(format): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if !user.enterprise_tier.has_feature_access("custom_integration") {
        return Err(AppError::forbidden("Data export requires Enterprise tier or higher"));
    }

    let supported_formats = vec!["json", "csv", "excel", "pdf"];
    if !supported_formats.contains(&format.as_str()) {
        return Err(AppError::bad_request("Unsupported export format"));
    }

    let response = serde_json::json!({
        "export_id": uuid::Uuid::new_v4(),
        "format": format,
        "status": "processing",
        "estimated_completion": Utc::now() + chrono::Duration::minutes(5),
        "download_url": format!("/api/v1/enterprise/analytics/downloads/{}", uuid::Uuid::new_v4()),
        "expires_at": Utc::now() + chrono::Duration::hours(24)
    });

    Ok(Json(response))
}

/// Helper functions

fn generate_mock_rankings(limit: u32, offset: u32, tier: &EnterpriseTier) -> Vec<serde_json::Value> {
    (0..limit).map(|i| {
        let rank = offset + i + 1;
        let include_advanced = tier.has_feature_access("real_time_data");
        
        let mut ranking = serde_json::json!({
            "rank": rank,
            "symbol": format!("STOCK{}", rank),
            "company": format!("Company {}", rank),
            "sector": "Technology",
            "eps_score": 85.5 - (rank as f64 * 0.1),
            "price_usd": 150.0 + (rank as f64 * 2.0),
            "market_cap": 1000000000.0 + (rank as f64 * 1000000.0)
        });

        if include_advanced {
            ranking.as_object_mut().unwrap().extend([
                ("volatility".to_string(), serde_json::json!(0.25 + (rank as f64 * 0.001))),
                ("beta".to_string(), serde_json::json!(1.0 + (rank as f64 * 0.01))),
                ("rsi".to_string(), serde_json::json!(50.0 + (rank as f64 % 40.0))),
                ("volume_profile".to_string(), serde_json::json!("high")),
            ]);
        }

        ranking
    }).collect()
}

fn get_tier_features(tier: &EnterpriseTier) -> Vec<String> {
    let mut features = vec![
        "basic_analytics".to_string(),
        "market_data".to_string(),
    ];

    if tier.has_feature_access("real_time_data") {
        features.extend(vec![
            "real_time_data".to_string(),
            "advanced_charts".to_string(),
            "custom_alerts".to_string(),
        ]);
    }

    if tier.has_feature_access("custom_integration") {
        features.extend(vec![
            "custom_integration".to_string(),
            "webhook_endpoints".to_string(),
            "data_export".to_string(),
        ]);
    }

    if tier.has_feature_access("unlimited_requests") {
        features.push("unlimited_requests".to_string());
    }

    features
}

fn get_available_reports(tier: &EnterpriseTier) -> Vec<String> {
    let mut reports = vec![
        "basic_performance".to_string(),
        "portfolio_summary".to_string(),
    ];

    if tier.has_feature_access("real_time_data") {
        reports.extend(vec![
            "advanced_performance".to_string(),
            "risk_analysis".to_string(),
            "market_correlation".to_string(),
        ]);
    }

    if tier.has_feature_access("custom_integration") {
        reports.extend(vec![
            "custom_reports".to_string(),
            "white_label_reports".to_string(),
            "compliance_reports".to_string(),
        ]);
    }

    reports
}