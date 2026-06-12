// Core shared kernel with cross-cutting concerns.
//
// kernel extraction wave9: the contents of this module are now thin
// re-export shims that pull from the new `epsx-contracts` workspace
// crate. The `types` module (which had 0 importers and only existed as
// a vestige of an earlier AppError shape) was deleted.

pub mod errors;
pub mod permissions;
pub mod telemetry;
pub mod constants;
// OIDC client credential service removed - using pure Web3 authentication

// Re-export specific items from errors to avoid conflicts
pub use errors::{ErrorKind, ErrorContext, AppError as ComplexAppError};
pub use telemetry::{LogContext, PerformanceStats, Alert, AlertSeverity, TelemetryConfig};
// Database abstraction exports removed - unused layer (codebase uses SQLx directly)
pub use constants::*;
// OIDC client credential service exports removed - pure Web3 authentication
