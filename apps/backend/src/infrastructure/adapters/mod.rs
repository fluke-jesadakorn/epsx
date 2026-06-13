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

// wave11(track-c): `EventPublisherPort` in-process adapter (ROADMAP §5
// R7). The in-process impl is a no-op stub that logs at
// `tracing::info!` and optionally forwards to the legacy
// `DomainEventBus` via `tokio::spawn`. See
// `events::in_process_event_publisher` for the design notes.
pub mod events;


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
pub use events::InProcessEventPublisher;
