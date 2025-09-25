// Service Adapters
// Implementations of service ports for external integrations

pub mod security_monitoring_service_adapter;
// pub mod token_validation_service_adapter; // Removed - legacy OIDC token validation
pub mod user_identity_service_adapter;
pub mod fcm_service;
pub mod email_service;
pub mod tradingview;
pub mod tradingview_websocket;
// pub mod oidc; // Removed - legacy OIDC authentication
// pub mod combined_rate_limiting_service; // Removed - depends on deleted rate_limiting_service
pub mod comprehensive_rate_limiting_service;
pub mod unified_admin_client_adapter;
pub mod granular_permissions_admin_client_adapter;
pub mod notification_service_adapter;
// pub mod blockchain_rpc_service; // Removed - lifetime issues
pub mod blockchain_service_adapter;
pub mod payment_security_service;
// pub mod smart_payment_verifier; // Removed - type issues
// pub mod payment_verification_logger; // Removed - type issues
pub mod resilience_patterns;
pub mod trading_view_market_data_adapter;

// Re-export service adapters with explicit imports to avoid conflicts
pub use security_monitoring_service_adapter::{
  SecurityMonitoringServiceAdapter,
};
// pub use token_validation_service_adapter::{ TokenValidationServiceAdapter }; // Removed - legacy OIDC
pub use user_identity_service_adapter::{ UserIdentityServiceAdapter };
pub use fcm_service::{ FcmService, FcmTopicService, FcmNotification };
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
// pub use oidc::{
//   OIDCService,
//   TokenValidationResult as OidcTokenValidationResult,
// }; // Removed - legacy OIDC authentication
// pub use combined_rate_limiting_service::CombinedRateLimitingService; // Removed - service deleted
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
// pub use blockchain_rpc_service::{
//   AlloyBlockchainRpcService,
//   BlockchainRpcService,
//   BlockchainConfig,
//   TransactionDetails,
//   TokenTransferDetails,
//   BlockchainError,
//   RpcEndpointResult,
//   MultiEndpointVerificationResult,
// }; // Removed - module deleted
pub use blockchain_service_adapter::BlockchainServiceAdapter;
pub use payment_security_service::{
  SqlxPaymentSecurityService,
  PaymentSecurityService,
  PaymentSecurityConfig,
  FraudAnalysis,
  FraudFlag,
  SecurityError,
};
// pub use smart_payment_verifier::{
//   SmartPaymentVerifier,
//   PaymentVerificationResult,
//   VerificationStatus,
//   BlockchainVerificationDetails,
//   SubscriptionActivationResult,
//   NextVerificationInfo,
//   PaymentVerificationError,
// }; // Removed - module deleted
// pub use payment_verification_logger::{
//   PaymentVerificationLogger,
//   PaymentVerificationEvent,
//   PaymentVerificationContext,
//   RpcEndpointMetrics,
//   BlockchainVerificationMetrics,
//   SubscriptionActivationDetails,
// }; // Removed - module deleted
pub use resilience_patterns::{
  CircuitBreaker,
  CircuitBreakerState,
  RetryPolicy,
  RateLimiter,
};
pub use trading_view_market_data_adapter::TradingViewMarketDataAdapter;
