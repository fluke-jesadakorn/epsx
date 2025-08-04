// Domain layer - Core business entities, value objects, and domain services
// This layer contains the core business logic and is independent of external concerns

pub mod entities;
pub mod values; 
pub mod services;
pub mod events;
pub mod error;
pub mod ports;

#[allow(ambiguous_glob_reexports)]
pub use entities::*;
#[allow(ambiguous_glob_reexports)]
pub use values::*;
#[allow(ambiguous_glob_reexports)]
pub use services::*;
#[allow(ambiguous_glob_reexports)]
pub use events::*;
