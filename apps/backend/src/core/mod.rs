// Core shared kernel with cross-cutting concerns

pub mod errors;
pub mod error_recovery;
pub mod telemetry;
pub mod events;
pub mod db;
pub mod permission_constants;
pub mod types;
pub mod client_credential_service;
pub mod iam_token_claims;
// pub mod plugins;
// pub mod plugin_examples;

// Using types::* for AppError (simplified version)
pub use types::*;
// Re-export specific items from errors to avoid conflicts
pub use errors::{ErrorKind, ErrorContext, AppError as ComplexAppError};
pub use error_recovery::{
    RecoveryConfig, RecoveryStrategy, RetryRecoveryStrategy, 
    CircuitBreakerRecovery, FallbackRecovery, RecoveryOrchestrator
};
// Re-export specific types to avoid conflicts
pub use telemetry::{LogContext, PerformanceStats, Alert, AlertSeverity, TelemetryConfig};
pub use events::{DomainEvent, EventEnvelope, StoredEvent, EventStream, Snapshot, EventHandler, Subscription, CircuitBreaker};
pub use db::{DatabaseConnection, DatabaseTransaction, QueryBuilder, QueryFilter, QuerySort, DatabaseHealth, ConnectionInfo};
pub use client_credential_service::{ClientCredentialService, ClientCredentials, ClientType};
pub use iam_token_claims::{AccessTokenClaims, IdTokenClaims, RefreshTokenClaims};
// pub use plugins::{Plugin, PluginManager, PluginRegistry, PluginMetadata, PluginConfig, PluginState, PluginFactory, TradingPlugin, DataProviderPlugin, NotificationPlugin};