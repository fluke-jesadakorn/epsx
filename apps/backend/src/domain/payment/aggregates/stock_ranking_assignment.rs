//! `StockRankingAssignment` value object.
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 this type
//! lived in `application::market_analytics::queries::models::get_stock_ranking_assignments`
//! even though it represents a row in the `stock_ranking_assignments`
//! table — a payments-owned table (per
//! `apps/backend/diesel_payments.toml:7` filter and the
//! `migrations/payments/00000000000001_consolidated_payments_v3` table list).
//!
//! Track B moves the type into the payments domain (its actual
//! owner) and adds the read method on `SubscriptionRepositoryPort`.
//! The market-analytics query object becomes a thin facade that
//! calls the port; the SQL lives in
//! `infrastructure::adapters::repositories::payment::subscription_repository_adapter`.
//!
//! The shape of the type is unchanged from the pre-wave-11
//! market_analytics version; the only thing that changes is the
//! module path. All field names, JSON serde representations, and
//! behavior are preserved for backward compatibility with any
//! downstream consumer of the query response.
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 4
//!     (the "strongest outward leak" — market_analytics reading
//!     `stock_ranking_assignments`)
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Domain DTO for a single row in the `stock_ranking_assignments`
/// table.
///
/// `assignment_id` is the table's primary key (a UUID-as-string in
/// the schema, see
/// `migrations/payments/00000000000001_consolidated_payments_v3/up.sql`).
/// `wallet_address` is the EVM address of the wallet that holds the
/// assignment. `package_id` / `package_name` identify the
/// stock-ranking tier (the legacy "package" wording is preserved
/// from the original product). `rank_access_level` is the
/// numeric tier (3, 25, 50, 100, …). `assignment_source` is a free
/// string (e.g. "admin", "system_activation", "purchase"). The
/// `days_remaining` is computed by the adapter, not stored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StockRankingAssignment {
    pub assignment_id: String,
    pub wallet_address: String,
    pub package_id: String,
    pub package_name: String,
    pub rank_access_level: i32,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub assignment_source: String,
    pub auto_renew: bool,
    pub days_remaining: Option<i32>,
}

impl StockRankingAssignment {
    /// Compute `days_remaining` from `expires_at` and a `now`
    /// reference. Returns `None` for never-expiring rows (i.e.
    /// `expires_at = None`). Negative values are clamped to zero
    /// for downstream rendering.
    pub fn compute_days_remaining(
        expires_at: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Option<i32> {
        // `num_days()` returns `i64`; the `Deref` / `i32` cast
        // is the contract — the admin UI only ever shows a
        // small number (today, the max is 365). Out-of-range
        // values are clamped at the production clamp site (see
        // `web/admin/payment_link_handlers.rs` and the
        // `Subscription::is_expired` helper); here we just
        // truncate.
        expires_at.map(|exp| (exp - now).num_days().max(0).min(i32::MAX as i64) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_remaining_none_when_no_expiry() {
        let now = Utc::now();
        let days = StockRankingAssignment::compute_days_remaining(None, now);
        assert_eq!(days, None);
    }

    #[test]
    fn days_remaining_positive_when_in_future() {
        let now = Utc::now();
        let exp = now + chrono::Duration::days(7);
        let days = StockRankingAssignment::compute_days_remaining(Some(exp), now);
        assert_eq!(days, Some(7));
    }

    #[test]
    fn days_remaining_clamped_to_zero_for_past() {
        let now = Utc::now();
        let exp = now - chrono::Duration::days(3);
        let days = StockRankingAssignment::compute_days_remaining(Some(exp), now);
        assert_eq!(days, Some(0));
    }

    #[test]
    fn dto_serde_round_trip() {
        let now = Utc::now();
        let sra = StockRankingAssignment {
            assignment_id: "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d".to_string(),
            wallet_address: "0xabc".to_string(),
            package_id: "pkg_pro".to_string(),
            package_name: "Pro Tier".to_string(),
            rank_access_level: 100,
            assigned_at: now,
            expires_at: Some(now + chrono::Duration::days(30)),
            is_active: true,
            assignment_source: "admin".to_string(),
            auto_renew: false,
            days_remaining: Some(30),
        };
        let json = serde_json::to_string(&sra).unwrap();
        let parsed: StockRankingAssignment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sra);
    }
}
