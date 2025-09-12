// Service Adapters  
// Implementations of service ports for external integrations

pub mod security_monitoring_service_adapter;
pub mod token_validation_service_adapter;
pub mod user_identity_service_adapter;
pub mod fcm_service;
pub mod email_service;
pub mod tradingview;
pub mod tradingview_websocket;
pub mod firebase;
pub mod oidc;

// Re-export service adapters with explicit imports to avoid conflicts
pub use security_monitoring_service_adapter::{SecurityMonitoringServiceAdapter};
pub use token_validation_service_adapter::{TokenValidationServiceAdapter};
pub use user_identity_service_adapter::{UserIdentityServiceAdapter};
pub use fcm_service::{FcmService, FcmTopicService, FcmNotification};
pub use email_service::{SendGridEmailService, SmtpEmailService};
pub use tradingview::{
    TradingViewRestClient, TradingViewWebSocketHandler as TradingViewWebSocketClient,
    TradingViewCache, types as tradingview_types
};
pub use tradingview_websocket::{
    TradingViewWebSocketService, FrontendEPSData as WebSocketFrontendEPSData
};
pub use firebase::{FirebaseAdmin, FirebaseUser, FirebaseError};
pub use oidc::{OIDCService, TokenValidationResult as OidcTokenValidationResult};