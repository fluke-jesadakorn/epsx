// Module-based API routing system - CLEANED UP
// Removed placeholder implementations: stock_ranking, portfolio_analysis, market_data
// These were all stub implementations returning "implementation pending" messages

use axum::{
    middleware::from_fn_with_state,
    Router,
};

use crate::web::auth::AppState;

// Simple stub for removed module middleware
#[derive(Debug, Clone)]
pub enum AccessLevel {
    Basic,
    Premium,
    Admin,
}

// Module handlers and routes removed - were all placeholder implementations

// ========================================
// MODULE ROUTER CREATION
// ========================================

/// Create the main modules router
pub fn create_modules_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        // Module-based routing system cleaned up
        // Removed placeholder implementations for stock-ranking, portfolio-analysis, and market-data
        // These were all stub implementations that returned "implementation pending" messages
        
        
        // Apply simple auth middleware (replaces complex casbin/module permissions)
        .route_layer(from_fn_with_state(
            app_state,
            crate::web::middleware::stateless_auth_middleware,
        ))
}

/// Create admin module management router - CLEANED UP
/// Removed placeholder implementations
pub fn create_admin_modules_router(__app_state: AppState) -> Router<AppState> {
    Router::new()
        // Module admin functionality removed - was placeholder implementation
}

// ========================================
// MODULE ROUTER CLEANUP
// ========================================

// All individual module routers (stock_ranking, portfolio_analysis, market_data) 
// have been removed as they were placeholder implementations returning 
// "implementation pending" messages


// ========================================
// UTILITY FUNCTIONS
// ========================================

/// Apply access level requirement to a router (placeholder during migration)
pub fn require_access_level(router: Router<AppState>, _level: AccessLevel, __app_state: AppState) -> Router<AppState> {
    // TODO: Implement with Casbin during migration
    router
}

/// Create module-specific middleware stack (placeholder during migration)
pub fn create_module_middleware_stack(_module_name: &str, _min_level: AccessLevel, __app_state: AppState) -> Router<AppState> {
    Router::new()
        // TODO: Fix middleware trait bounds
        // .route_layer(from_fn_with_state(
        //     app_state,
        //     module_auth_middleware,
        // ))
}

// ========================================
// ROUTE GROUPS BY ACCESS LEVEL
// ========================================

// Route access level definitions removed
// These were for placeholder modules that have been cleaned up
// (stock-ranking, portfolio-analysis, market-data were all stub implementations)