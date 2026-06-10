// Service Adapters
// Web3-first service implementations for blockchain integration and external services

pub mod permission_adapter;
pub mod security_monitoring_service_adapter;
pub mod tradingview;
pub mod tradingview_websocket;
pub mod notification_service_adapter;
pub mod resilience_patterns;

pub use permission_adapter::{
    Web3PermissionServiceAdapter,
    BlockchainConfig,
    NftOwnershipResult,
    TokenBalanceResult,
    DaoMembershipResult,
};

pub use security_monitoring_service_adapter::{
  SecurityMonitoringServiceAdapter,
};
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
pub use notification_service_adapter::NotificationServiceAdapter;
pub use resilience_patterns::{
  CircuitBreaker,
  CircuitBreakerState,
  RetryPolicy,
  RateLimiter,
};
