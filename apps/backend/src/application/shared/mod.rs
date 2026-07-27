// Shared Application Layer Concerns
// Common patterns and abstractions used across all application services

pub mod command_bus;
pub mod error;
pub mod query_bus;
pub mod validation;

pub use command_bus::{Command, CommandHandler};
pub use error::{ApplicationError, ApplicationResult};
pub use query_bus::{PaginationParams, Query, QueryHandler, SortDirection, SortParams};
pub use validation::{ValidationError, ValidationUtils, Validator};
