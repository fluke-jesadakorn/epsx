// Core shared kernel with cross-cutting concerns

pub mod errors;
pub mod telemetry;
pub mod events;
pub mod db;
pub mod plugins;
pub mod plugin_examples;

pub use errors::*;
// Re-export specific types to avoid conflicts
pub use telemetry::{LogContext, PerformanceStats, Alert, AlertSeverity, TelemetryConfig};
pub use events::{DomainEvent, EventEnvelope, StoredEvent, EventStream, Snapshot, EventHandler, Subscription, CircuitBreaker};
pub use db::{DatabaseConnection, DatabaseTransaction, QueryBuilder, QueryFilter, QuerySort, DatabaseHealth, ConnectionInfo};
pub use plugins::{Plugin, PluginManager, PluginRegistry, PluginMetadata, PluginConfig, PluginState, PluginFactory, TradingPlugin, DataProviderPlugin, NotificationPlugin};