// Service Adapters
// Web3-first service implementations for blockchain integration and external services

// NEW - Web3-first service adapters (primary)
pub mod permission_adapter;

// Core service adapters
pub mod security_monitoring_service_adapter;

// email_service removed - Web3-first system doesn't require traditional email functionality
pub mod tradingview;
pub mod tradingview_websocket;
pub mod websocket_earnings_service; // WebSocket earnings data enhancement
pub mod comprehensive_rate_limiting_service;
// pub mod unified_admin_client_adapter; // Removed - unused placeholder implementation
// pub mod granular_permissions_admin_client_adapter; // Removed - unused placeholder implementation
pub mod notification_service_adapter;
// pub mod payment_security_service; // File doesn't exist yet
pub mod resilience_patterns;
// pub mod trading_view_market_data_adapter; // REMOVED

// Re-export service adapters with explicit imports to avoid conflicts

// NEW - Web3-first service adapter exports (primary)
pub use permission_adapter::{
    Web3PermissionServiceAdapter,
    BlockchainConfig,
    NftOwnershipResult,
    TokenBalanceResult,
    DaoMembershipResult,
};

// Core service adapter exports
pub use security_monitoring_service_adapter::{
  SecurityMonitoringServiceAdapter,
};
// Email service exports removed - Web3-first system uses direct wallet notifications
pub use tradingview::{
  TradingViewRestClient,
  TradingViewWebSocketHandler as TradingViewWebSocketClient,
  TradingViewCache,
  types as tradingview_types,
};
pub use tradingview_websocket::{
  TradingViewWebSocketService,
  FrontendEPSData as WebSocketFrontendEPSData,
};
pub use websocket_earnings_service::WebSocketEarningsService;
pub use comprehensive_rate_limiting_service::{
  ComprehensiveRateLimitingService,
  RateLimitTier,
  RateLimitClientId,
  RateLimitViolation,
  RateLimitResult,
};
// Admin client adapters removed - unused placeholder implementations
pub use notification_service_adapter::{ NotificationServiceAdapter };
// Payment security service exports temporarily disabled
// pub use payment_security_service::{
//   SqlxPaymentSecurityService,
//   PaymentSecurityService,
//   PaymentSecurityConfig,
//   FraudAnalysis,
//   FraudFlag,
//   SecurityError,
// };
pub use resilience_patterns::{
  CircuitBreaker,
  CircuitBreakerState,
  RetryPolicy,
  RateLimiter,
};
// pub use trading_view_market_data_adapter::TradingViewMarketDataAdapter; // REMOVED
