// Core shared kernel with cross-cutting concerns

pub mod errors;
pub mod error_recovery;
pub mod telemetry;
pub mod events;
pub mod types;
pub mod constants;
// OIDC client credential service removed - using pure Web3 authentication

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
// Database abstraction exports removed - unused layer (codebase uses SQLx directly)
pub use constants::*;
// OIDC client credential service exports removed - pure Web3 authentication