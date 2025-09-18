// Infrastructure Adapters
// Concrete implementations of domain repository ports

pub mod repositories;
pub mod services;
pub mod cache;

// Re-export with explicit imports to avoid conflicts
// TODO: Re-export repository adapters as they are migrated to SQLx
// diesel as repositories_diesel,  // Removed during SQLx migration
// Add other repository modules as needed
pub use services::{
    SecurityMonitoringServiceAdapter, TokenValidationServiceAdapter, 
    UserIdentityServiceAdapter, FcmService, FcmTopicService, FcmNotification,
    SendGridEmailService, TradingViewRestClient, TradingViewWebSocketService, 
    TradingViewCache, tradingview_types,
    TradingViewWebSocketClient, WebSocketFrontendEPSData,
    FirebaseAdmin, FirebaseUser, FirebaseError,
    OIDCService, OidcTokenValidationResult
};
pub use cache::{
    RealTimeCacheAdapter
};