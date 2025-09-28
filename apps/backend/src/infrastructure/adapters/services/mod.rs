// Service Adapters
// Web3-first service implementations for blockchain integration and external services

// NEW - Web3-first service adapters (primary)
pub mod web3_permission_service_adapter;

// Core service adapters
pub mod security_monitoring_service_adapter;

pub mod email_service;
pub mod tradingview;
pub mod tradingview_websocket;
pub mod comprehensive_rate_limiting_service;
pub mod unified_admin_client_adapter;
pub mod granular_permissions_admin_client_adapter;
pub mod notification_service_adapter;
// pub mod payment_security_service; // Temporarily disabled
pub mod resilience_patterns;
pub mod trading_view_market_data_adapter;

// Re-export service adapters with explicit imports to avoid conflicts

// NEW - Web3-first service adapter exports (primary)
pub use web3_permission_service_adapter::{
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
pub use email_service::{ SendGridEmailService, SmtpEmailService };
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
pub use comprehensive_rate_limiting_service::{
  ComprehensiveRateLimitingService,
  RateLimitTier,
  RateLimitClientId,
  RateLimitViolation,
  RateLimitResult,
};
pub use unified_admin_client_adapter::{ UnifiedAdminClientAdapter, AdminUser };
pub use granular_permissions_admin_client_adapter::{
  GranularPermissionsAdminClientAdapter,
  GranularPermission,
};
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
pub use trading_view_market_data_adapter::TradingViewMarketDataAdapter;
