// kernel extraction wave9 — moved from apps/backend/src/domain/shared_kernel/value_objects/mod.rs
// Shared Value Objects - Common types used across bounded contexts
//
// wave10(track-c): added `ranking_offset` (ROADMAP §5 R6). The audit said the
// value object already existed in apps/backend/src/domain/market_analytics/
// value_objects/; at HEAD it did not. The wrapper lives here so it can be
// shared with the future epsx-identity binary.

pub mod user_id;
pub mod session_id;
pub mod email;
pub mod user_limits;
pub mod common_types;
pub mod identifiers;
pub mod payments;
pub mod quarterly_eps_data;
pub mod symbol;
pub mod market;
pub mod ranking_offset;

// Re-export commonly used value objects
pub use user_id::UserId;
pub use session_id::{ SessionId, SessId };
pub use email::Email;
pub use user_limits::{ ResolvedUserLimits, UserDynamicLimit };
pub use common_types::*;
pub use identifiers::*;
pub use payments::{ Currency, Network };
pub use quarterly_eps_data::QuarterlyEPSData;
pub use symbol::Symbol;
pub use market::Market;
pub use ranking_offset::RankingOffset;
