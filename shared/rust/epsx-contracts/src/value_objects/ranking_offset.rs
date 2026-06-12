//! `RankingOffset` value object (Wave 10 — Track C, ROADMAP §5 R6).
//!
//! Represents a wallet's plan-tier ranking offset — a `0..=200`
//! range where `0` is the most privileged (top ranks) and
//! `FREE_PLAN_RANKING_OFFSET` (currently `100`) is the floor for
//! unauthenticated / Free Plan users. The audit's plan said the
//! value object already exists in
//! `apps/backend/src/domain/market_analytics/value_objects/`, but
//! it didn't (the underlying function returns `i32`). This file
//! is the new home.
//!
//! The wrapper adds:
//!   - Range validation (`0..=1000` — generous upper bound; the
//!     prod maximum is `100`).
//!   - `Serialize` / `Deserialize` for HTTP transport.
//!   - `Display` + `Default` + `From<i32>` for ergonomic conversion
//!     from the existing `i32` call sites.
//!
//! Outbound dependents that will need to update their imports:
//!   - `apps/backend/src/web/analytics/eps/rankings.rs`
//!     (`calculate_ranking_config_from_permissions`)
//!   - `apps/backend/src/web/analytics/eps/cache.rs`
//!   - `apps/backend/src/auth/unified_permission_service.rs`
//!     (`get_wallet_ranking_offset` return type)
//!   - `apps/backend/src/infrastructure/adapters/permission/
//!      in_process_ranking_offset_adapter.rs` (the adapter this
//!      wave adds)

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::constants;
use crate::value_object::{ValueObject, ValueObjectError};

/// Hard upper bound on the ranking offset. The product's
/// largest non-vip tier is `100`; we leave headroom up to `1000`
/// for future premium tiers. Beyond that, treat the value as a
/// configuration error.
pub const RANKING_OFFSET_MAX: i32 = 1000;

/// Plan-tier ranking offset value object.
///
/// Lower is better. `0` means the wallet can see the top of the
/// ranking list; `100` is the Free Plan floor; > `100` is a
/// "buried deep in the list" floor for anonymous users with no
/// JWT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RankingOffset(i32);

impl Default for RankingOffset {
    /// Default to the free-plan offset so callers that don't
    /// know the wallet's tier fall through to a safe, documented
    /// value.
    fn default() -> Self {
        Self(constants::FREE_PLAN_RANKING_OFFSET)
    }
}

impl RankingOffset {
    /// Construct a `RankingOffset` from a raw `i32`. Returns
    /// `Err` if the value is out of range.
    pub fn new(value: i32) -> Result<Self, ValueObjectError> {
        Self(value).validate()?;
        Ok(Self(value))
    }

    /// Construct a `RankingOffset` from a raw `i32` without
    /// validation. Intended for the in-process adapter where the
    /// value has just been clamped by the underlying SQL query
    /// (the query returns `min(perm_offset)` across all active
    /// plans, so negative values would only appear if the seed
    /// data is corrupt — the audit treats that as a separate bug).
    pub fn new_unchecked(value: i32) -> Self {
        Self(value)
    }

    /// Construct the Free-Plan default explicitly. Equivalent
    /// to `Self::default()` but reads better at call sites that
    /// want to be explicit ("no plan, free tier").
    pub fn free_plan() -> Self {
        Self(constants::FREE_PLAN_RANKING_OFFSET)
    }

    /// Raw `i32` accessor.
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl ValueObject for RankingOffset {
    type Error = ValueObjectError;

    fn validate(&self) -> Result<(), Self::Error> {
        if self.0 < 0 {
            return Err(ValueObjectError::OutOfRange(format!(
                "Ranking offset cannot be negative (got {})",
                self.0
            )));
        }
        if self.0 > RANKING_OFFSET_MAX {
            return Err(ValueObjectError::OutOfRange(format!(
                "Ranking offset {} exceeds maximum {}",
                self.0, RANKING_OFFSET_MAX
            )));
        }
        Ok(())
    }
}

impl fmt::Display for RankingOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for RankingOffset {
    /// Lossy-validated conversion. Falls back to the free-plan
    /// default on out-of-range inputs so existing `i32` call
    /// sites that do `let offset: RankingOffset = raw.into();`
    /// keep working even if the underlying query returns
    /// something unexpected. **Prefer `RankingOffset::new` for
    /// new code.**
    fn from(value: i32) -> Self {
        if value < 0 || value > RANKING_OFFSET_MAX {
            tracing::debug!(
                "RankingOffset::from({}) out of range; falling back to free-plan offset",
                value
            );
            Self::default()
        } else {
            Self(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_validates_in_range() {
        assert!(RankingOffset::new(0).is_ok());
        assert!(RankingOffset::new(100).is_ok());
        assert!(RankingOffset::new(1000).is_ok());
    }

    #[test]
    fn new_rejects_out_of_range() {
        assert!(RankingOffset::new(-1).is_err());
        assert!(RankingOffset::new(1001).is_err());
        assert!(RankingOffset::new(i32::MIN).is_err());
        assert!(RankingOffset::new(i32::MAX).is_err());
    }

    #[test]
    fn from_i32_clamps_invalid_values() {
        // Out-of-range inputs fall back to the free-plan default.
        let neg: RankingOffset = (-5).into();
        assert_eq!(neg.value(), constants::FREE_PLAN_RANKING_OFFSET);
        let too_big: RankingOffset = 5000.into();
        assert_eq!(too_big.value(), constants::FREE_PLAN_RANKING_OFFSET);
    }

    #[test]
    fn default_is_free_plan() {
        assert_eq!(
            RankingOffset::default().value(),
            constants::FREE_PLAN_RANKING_OFFSET
        );
    }

    #[test]
    fn value_object_validate() {
        assert!(RankingOffset::new(0).unwrap().is_valid());
        // Out-of-range construction fails; new_unchecked lets us
        // exercise the validate() method itself.
        assert!(!RankingOffset::new_unchecked(-1).is_valid());
        assert!(!RankingOffset::new_unchecked(2000).is_valid());
        assert!(RankingOffset::new_unchecked(50).is_valid());
    }

    #[test]
    fn serde_round_trip() {
        let original = RankingOffset::new(42).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: RankingOffset = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }
}
