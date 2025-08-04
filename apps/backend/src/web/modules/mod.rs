// Module-based API routing system
// New IAM v2 architecture with module-scoped endpoints

pub mod handlers;
pub mod routes;
pub mod stock_ranking;
pub mod portfolio_analysis;
pub mod market_data;
pub mod trading_signals;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put, delete},
    Router,
};

use crate::web::{
    auth::AppState,
    middleware::{
        auth_middleware,
        module_auth_middleware::{module_auth_middleware, require_module_access, AccessLevel},
        module_permission_middleware::module_permission_middleware,
    },
};

use handlers::*;
use routes::*;

// ========================================
// MODULE ROUTER CREATION
// ========================================

/// Create the main modules router
pub fn create_modules_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        // Module discovery and management
        .route("/modules", get(list_available_modules))
        .route("/modules/:module_id", get(get_module_details))
        .route("/modules/:module_name/access", get(check_module_access))
        .route("/modules/:module_name/quota", get(get_module_quota_status))
        
        // Stock ranking module
        .nest("/stock-ranking", create_stock_ranking_router())
        
        // Portfolio analysis module  
        .nest("/portfolio-analysis", create_portfolio_analysis_router())
        
        // Market data module
        .nest("/market-data", create_market_data_router())
        
        // Trading signals module
        .nest("/trading-signals", create_trading_signals_router())
        
        // Apply module permission middleware (includes auth + quota checking)
        .route_layer(from_fn_with_state(
            app_state,
            module_permission_middleware,
        ))
}

/// Create admin module management router
pub fn create_admin_modules_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        // Module definition management
        .route("/modules", post(create_module))
        .route("/modules", get(list_all_modules))
        .route("/modules/:module_id", get(get_module_admin_details))
        .route("/modules/:module_id", put(update_module))
        .route("/modules/:module_id", delete(delete_module))
        .route("/modules/:module_id/status", put(update_module_status))
        
        // User module assignments
        .route("/users/:user_id/modules", get(get_user_modules))
        .route("/users/:user_id/modules", post(assign_user_modules))
        .route("/users/:user_id/modules/:module_id", put(update_user_module_access))
        .route("/users/:user_id/modules/:module_id", delete(revoke_user_module_access))
        
        // Bulk operations
        .route("/users/bulk/assign-modules", post(bulk_assign_modules))
        .route("/users/bulk/revoke-modules", post(bulk_revoke_modules))
        .route("/modules/:module_id/users", get(list_module_users))
        
        // API key management
        .route("/api-keys", post(create_api_key))
        .route("/api-keys", get(list_api_keys))
        .route("/api-keys/:key_id", get(get_api_key_details))
        .route("/api-keys/:key_id", put(update_api_key))
        .route("/api-keys/:key_id", delete(revoke_api_key))
        .route("/api-keys/:key_id/modules", put(update_api_key_modules))
        .route("/api-keys/:key_id/usage", get(get_api_key_usage))
        
        // Analytics and reporting
        .route("/analytics/modules", get(get_module_analytics))
        .route("/analytics/users", get(get_user_analytics))
        .route("/analytics/api-keys", get(get_api_key_analytics))
        .route("/analytics/usage", get(get_usage_analytics))
        
        // Audit and compliance
        .route("/audit/assignments", get(get_assignment_audit_logs))
        .route("/audit/api-usage", get(get_api_usage_audit_logs))
        .route("/audit/quota-violations", get(get_quota_violation_logs))
        
        // Apply authentication middleware (admin access required)
        .route_layer(from_fn_with_state(
            app_state,
            auth_middleware,
        ))
}

// ========================================
// INDIVIDUAL MODULE ROUTERS
// ========================================

/// Stock ranking module router
fn create_stock_ranking_router() -> Router<AppState> {
    Router::new()
        // Basic rankings (Bronze+)
        .route("/rankings", get(stock_ranking::get_basic_rankings))
        .route("/rankings/top-performers", get(stock_ranking::get_top_performers))
        .route("/rankings/by-sector", get(stock_ranking::get_rankings_by_sector))
        
        // EPS analysis (Bronze+)
        .route("/eps/growth", get(stock_ranking::get_eps_growth_rankings))
        .route("/eps/analysis/:symbol", get(stock_ranking::get_eps_analysis))
        
        // Enhanced features (Silver+)
        .route("/rankings/ai-insights", get(stock_ranking::get_ai_insights))
        .route("/rankings/pattern-analysis", get(stock_ranking::get_pattern_analysis))
        .route("/alerts", get(stock_ranking::get_alerts))
        .route("/alerts", post(stock_ranking::create_alert))
        
        // Custom algorithms (Gold+) 
        .route("/rankings/custom", post(stock_ranking::create_custom_ranking))
        .route("/algorithms", get(stock_ranking::list_algorithms))
        .route("/algorithms/:algo_id", get(stock_ranking::get_algorithm_details))
        
        // Real-time features (Silver+)
        .route("/rankings/live", get(stock_ranking::get_live_rankings))
        .route("/feed/connect", get(stock_ranking::connect_live_feed))
        
        // Export functionality (Bronze+, quota limited)
        .route("/export/csv", post(stock_ranking::export_csv))
        .route("/export/excel", post(stock_ranking::export_excel))
        .route("/export/pdf", post(stock_ranking::export_pdf))
        
        // Historical data (Silver+)
        .route("/historical/:symbol", get(stock_ranking::get_historical_rankings))
        // TODO: Fix handler signatures for Axum compatibility
        // .route("/historical/comparison", post(stock_ranking::compare_historical_performance))
        
        // Enterprise features (Platinum+)
        // TODO: Fix handler signatures for Axum compatibility
        // .route("/models/custom", post(stock_ranking::create_custom_model))
        // .route("/models/:model_id/backtest", post(stock_ranking::backtest_model))
        // .route("/bulk/analyze", post(stock_ranking::bulk_analyze_stocks))
}

/// Portfolio analysis module router  
fn create_portfolio_analysis_router() -> Router<AppState> {
    Router::new()
        // Basic portfolio operations (Bronze+)
        .route("/portfolios", get(portfolio_analysis::list_portfolios))
        .route("/portfolios", post(portfolio_analysis::create_portfolio))
        .route("/portfolios/:portfolio_id", get(portfolio_analysis::get_portfolio))
        .route("/portfolios/:portfolio_id", put(portfolio_analysis::update_portfolio))
        .route("/portfolios/:portfolio_id", delete(portfolio_analysis::delete_portfolio))
        
        // Performance tracking (Bronze+)
        .route("/portfolios/:portfolio_id/performance", get(portfolio_analysis::get_performance))
        .route("/portfolios/:portfolio_id/returns", get(portfolio_analysis::get_returns))
        
        // Risk analysis (Silver+)
        .route("/portfolios/:portfolio_id/risk", get(portfolio_analysis::get_risk_analysis))
        .route("/portfolios/:portfolio_id/var", get(portfolio_analysis::get_value_at_risk))
        .route("/portfolios/:portfolio_id/stress-test", post(portfolio_analysis::run_stress_test))
        
        // Benchmarking (Gold+)
        .route("/portfolios/:portfolio_id/benchmark", post(portfolio_analysis::compare_to_benchmark))
        .route("/benchmarks", get(portfolio_analysis::list_benchmarks))
        .route("/portfolios/:portfolio_id/attribution", get(portfolio_analysis::get_performance_attribution))
        
        // Alerts and monitoring (Silver+)
        .route("/portfolios/:portfolio_id/alerts", get(portfolio_analysis::get_portfolio_alerts))
        .route("/portfolios/:portfolio_id/alerts", post(portfolio_analysis::create_portfolio_alert))
        
        // Advanced analytics (Gold+)
        .route("/portfolios/:portfolio_id/optimization", post(portfolio_analysis::optimize_portfolio))
        .route("/portfolios/:portfolio_id/scenarios", post(portfolio_analysis::run_scenario_analysis))
        
        // Reporting (Silver+)
        .route("/portfolios/:portfolio_id/reports/summary", get(portfolio_analysis::generate_summary_report))
        .route("/portfolios/:portfolio_id/reports/detailed", get(portfolio_analysis::generate_detailed_report))
}

/// Market data module router
fn create_market_data_router() -> Router<AppState> {
    Router::new()
        // Basic quotes (Bronze+, delayed)
        .route("/quotes/:symbol", get(market_data::get_quote))
        .route("/quotes/batch", post(market_data::get_batch_quotes))
        
        // Real-time data (Silver+)
        .route("/quotes/:symbol/live", get(market_data::get_live_quote))
        .route("/stream/connect", get(market_data::connect_data_stream))
        
        // Historical data (Bronze+ limited, Silver+ extended)
        .route("/historical/:symbol", get(market_data::get_historical_data))
        .route("/historical/:symbol/intraday", get(market_data::get_intraday_data))
        .route("/historical/bulk", post(market_data::get_bulk_historical))
        
        // Technical indicators (Silver+)
        .route("/indicators/:symbol/sma", get(market_data::get_sma))
        .route("/indicators/:symbol/ema", get(market_data::get_ema))
        .route("/indicators/:symbol/rsi", get(market_data::get_rsi))
        .route("/indicators/:symbol/macd", get(market_data::get_macd))
        .route("/indicators/:symbol/bollinger", get(market_data::get_bollinger_bands))
        
        // Market alerts (Silver+)
        .route("/alerts", get(market_data::get_market_alerts))
        .route("/alerts", post(market_data::create_market_alert))  
        .route("/alerts/:alert_id", delete(market_data::delete_market_alert))
        
        // Market metadata
        .route("/symbols", get(market_data::list_symbols))
        .route("/symbols/search", get(market_data::search_symbols))
        .route("/exchanges", get(market_data::list_exchanges))
        .route("/sectors", get(market_data::list_sectors))
        
        // Premium data feeds (Gold+)
        .route("/level2/:symbol", get(market_data::get_level2_data))
        .route("/options/:symbol", get(market_data::get_options_data))
        .route("/futures/:symbol", get(market_data::get_futures_data))
        
        // International markets (Platinum+)
        .route("/international/:exchange/quotes", get(market_data::get_international_quotes))
        .route("/forex/:pair", get(market_data::get_forex_data))
        .route("/crypto/:symbol", get(market_data::get_crypto_data))
}

/// Trading signals module router
fn create_trading_signals_router() -> Router<AppState> {
    Router::new()
        // Basic signals (Silver+)
        .route("/signals", get(trading_signals::get_signals))
        .route("/signals/:symbol", get(trading_signals::get_symbol_signals))
        .route("/signals/generate", post(trading_signals::generate_signals))
        
        // Signal categories
        .route("/signals/technical", get(trading_signals::get_technical_signals))
        .route("/signals/fundamental", get(trading_signals::get_fundamental_signals))
        .route("/signals/sentiment", get(trading_signals::get_sentiment_signals))
        
        // AI-powered signals (Silver+)
        .route("/signals/ai", get(trading_signals::get_ai_signals))
        .route("/signals/ai/train", post(trading_signals::train_ai_model))
        
        // Strategy management (Gold+)
        .route("/strategies", get(trading_signals::list_strategies))
        .route("/strategies", post(trading_signals::create_strategy))
        .route("/strategies/:strategy_id", get(trading_signals::get_strategy))
        .route("/strategies/:strategy_id", put(trading_signals::update_strategy))
        .route("/strategies/:strategy_id", delete(trading_signals::delete_strategy))
        
        // Backtesting (Gold+)
        .route("/strategies/:strategy_id/backtest", post(trading_signals::backtest_strategy))
        .route("/backtests", get(trading_signals::list_backtests))
        .route("/backtests/:test_id", get(trading_signals::get_backtest_results))
        
        // Strategy optimization (Platinum+)
        .route("/strategies/:strategy_id/optimize", post(trading_signals::optimize_strategy))
        .route("/optimization/genetic", post(trading_signals::genetic_optimization))
        
        // Live trading integration (Platinum+)
        .route("/strategies/:strategy_id/deploy", post(trading_signals::deploy_strategy))
        .route("/strategies/:strategy_id/paper-trade", post(trading_signals::start_paper_trading))
        .route("/live/positions", get(trading_signals::get_live_positions))
        .route("/live/orders", get(trading_signals::get_live_orders))
        
        // Performance tracking
        .route("/strategies/:strategy_id/performance", get(trading_signals::get_strategy_performance))
        .route("/signals/performance", get(trading_signals::get_signal_performance))
        .route("/leaderboard", get(trading_signals::get_strategy_leaderboard))
}

// ========================================
// UTILITY FUNCTIONS
// ========================================

/// Apply access level requirement to a router
pub fn require_access_level(router: Router<AppState>, level: AccessLevel, app_state: AppState) -> Router<AppState> {
    router.route_layer(from_fn_with_state(
        app_state,
        require_module_access("", level), // Module name will be extracted from path
    ))
}

/// Create module-specific middleware stack
pub fn create_module_middleware_stack(module_name: &str, min_level: AccessLevel, app_state: AppState) -> Router<AppState> {
    Router::new()
        .route_layer(from_fn_with_state(
            app_state.clone(),
            require_module_access(module_name, min_level),
        ))
        .route_layer(from_fn_with_state(
            app_state,
            module_auth_middleware,
        ))
}

// ========================================
// ROUTE GROUPS BY ACCESS LEVEL
// ========================================

/// Routes that require Bronze level access or higher
pub fn bronze_routes() -> Vec<&'static str> {
    vec![
        "/stock-ranking/rankings",
        "/stock-ranking/eps/growth", 
        "/portfolio-analysis/portfolios",
        "/market-data/quotes",
        "/market-data/historical",
    ]
}

/// Routes that require Silver level access or higher  
pub fn silver_routes() -> Vec<&'static str> {
    vec![
        "/stock-ranking/rankings/ai-insights",
        "/stock-ranking/rankings/live",
        "/portfolio-analysis/risk",
        "/market-data/quotes/live",
        "/market-data/indicators",
        "/trading-signals/signals",
    ]
}

/// Routes that require Gold level access or higher
pub fn gold_routes() -> Vec<&'static str> {
    vec![
        "/stock-ranking/rankings/custom",
        "/stock-ranking/algorithms",
        "/portfolio-analysis/benchmark", 
        "/portfolio-analysis/optimization",
        "/market-data/level2",
        "/trading-signals/strategies",
        "/trading-signals/backtest",
    ]
}

/// Routes that require Platinum level access or higher
pub fn platinum_routes() -> Vec<&'static str> {
    vec![
        "/stock-ranking/models/custom",
        "/stock-ranking/bulk/analyze",
        "/market-data/international",
        "/trading-signals/optimize",
        "/trading-signals/deploy",
    ]
}