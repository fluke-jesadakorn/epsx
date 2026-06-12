// Infrastructure Adapters
// Concrete implementations of domain repository ports

pub mod notification;
pub mod pubsub;
pub mod repositories;
pub mod services;

// wave10(track-c): cross-cutting kernel-level port adapters (ROADMAP §5
// R1 + R6). 1:1 wrappers around `UnifiedPermissionService`. See
// `permission::mod` for the per-adapter design notes.
pub mod permission;


// Re-export with explicit imports to avoid conflicts
// Repository adapters use SQLx for database operations
pub use services::{
    SecurityMonitoringServiceAdapter,
    TradingViewRestClient, TradingViewWebSocketService,
    TradingViewCache, tradingview_types,
    TradingViewWebSocketClient, WebSocketFrontendEPSData,
    // SendGridEmailService removed - Web3-first system doesn't use traditional email
};

pub use pubsub::{InMemoryPubsubAdapter, RedisPubsubAdapter};
