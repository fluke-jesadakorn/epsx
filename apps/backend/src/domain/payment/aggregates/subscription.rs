//! `Subscription` aggregate and supporting value objects.
//!
//! Wave 11 — Track B (outbound-leakage fold). The pre-wave-11
//! "subscription" entity lived as the `SubscriptionDb` Diesel model
//! in `infrastructure::models::payment`. The new bounded-context
//! aggregate gives the rest of the app (the `SubscriptionRepositoryPort`
//! and the admin plans editor) a domain-shaped handle to the same
//! row without leaking the Diesel type across modules.
//!
//! Per the wave-11 task brief: "the 5 methods that
//! `subscription_repository_adapter.rs` already implements map 1:1
//! to the port methods; the `get_stock_ranking_assignments` method
//! is NEW — it moves the read from
//! `application/market_analytics/queries/models/get_stock_ranking_assignments.rs`
//! into the payments port."
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 3
//!     (`infrastructure/adapters/repositories/subscription_repository_adapter.rs`)
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)
//!
//! The aggregate is intentionally thin. The previous code treated
//! `SubscriptionDb` as a passive data carrier (no invariants, no
//! domain methods), and the admin plans editor already enforces the
//! real invariants in SQL (single-plan-active constraint, FK-less
//! `plan_id`). Lifting every SQL constraint into Rust would be a
//! parallel refactor; the scope of track B is the *leakage fold*,
//! not a domain re-modelling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::wallet_management::value_objects::WalletAddress;

/// Stable identifier for a subscription row.
///
/// Newtype around `Uuid` to match the `PaymentId` /
/// `PaymentContextId` pattern in `domain::payment`. The DB stores
/// `subscriptions.id UUID PRIMARY KEY` (see
/// `migrations/payments/00000000000001_consolidated_payments_v3/up.sql:73`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub Uuid);

impl SubscriptionId {
    /// Construct from a raw UUID. No validation beyond the type
    /// wrap; the underlying column is `UUID NOT NULL`.
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    /// Construct a fresh random ID. Used by the
    /// `CreateSubscriptionCommand` path when the caller does not
    /// have an ID yet (the `subscriptions` table does not have a
    /// default UUID, so the row is built with `id = Uuid::new_v4()`
    /// in the adapter).
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Borrow the inner UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SubscriptionId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

/// Domain-shaped `Subscription` aggregate.
///
/// Maps 1:1 to the `subscriptions` table row, with the
/// `id` field retyped to `SubscriptionId` and `wallet_address` retyped
/// to `WalletAddress`. The Diesel `SubscriptionDb` stays in
/// `infrastructure::models::payment` for use by the adapter and the
/// `web/payments/admin_handlers/subscription_handlers.rs` reader; the
/// domain shape is what flows across the port boundary.
///
/// Fields mirror the `SubscriptionDb` column set; the only field
/// that is *not* in the domain shape is `payment_id` (kept on
/// `SubscriptionDb` because it is a payments-internal FK). The
/// port-method `create` accepts a `CreateSubscriptionCommand` that
/// carries the payment id as `Option<Uuid>` so the
/// `web/admin/plans/handlers.rs` admin-assigned-subscription flow
/// (which sets `payment_id = None`) can keep working without
/// touching the payment flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub wallet_address: WalletAddress,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl Subscription {
    /// True iff `status == "cancelled"`. The adapter never sets
    /// this directly; the admin plans editor's "cancel" path goes
    /// through `cancel`.
    pub fn is_cancelled(&self) -> bool {
        self.status == "cancelled"
    }

    /// True iff the subscription has an expiry in the past.
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}

/// Command to create a new subscription.
///
/// Mirrors `NewSubscriptionDb` from `infrastructure::models::payment`
/// but uses domain types so the port signature is stable. The
/// adapter is responsible for translating the `WalletAddress` /
/// `Uuid` fields into the right column types.
#[derive(Debug, Clone)]
pub struct CreateSubscriptionCommand {
    pub wallet_address: WalletAddress,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

impl CreateSubscriptionCommand {
    /// Convenience constructor that matches the
    /// `web/admin/plans/handlers.rs::create_subscription_handler`
    /// admin-assigned subscription flow. Sets `status = "active"`,
    /// `started_at = now`, and `payment_id = None` (admin
    /// assignments are not tied to a payment row).
    ///
    /// `WalletAddress::from_trusted` is used here so the test
    /// fixture (e.g. `"0xabc"`) and the production caller (which
    /// has already validated the address from the request body)
    /// both succeed. Use `WalletAddress::new(s)` if you have a
    /// raw string from a request and want the
    /// `WalletAddressError::InvalidLength` / `InvalidFormat` check.
    pub fn admin_assign(
        wallet_address: WalletAddress,
        plan_id: Uuid,
        expires_at: DateTime<Utc>,
        auto_renew: bool,
    ) -> Self {
        Self {
            wallet_address,
            plan_id,
            payment_id: None,
            status: "active".to_string(),
            started_at: Some(Utc::now()),
            expires_at,
            auto_renew: Some(auto_renew),
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_id_round_trip() {
        let raw = Uuid::new_v4();
        let id = SubscriptionId::new(raw);
        assert_eq!(id.as_uuid(), raw);
        assert_eq!(id.to_string(), raw.to_string());
        let from_uuid: SubscriptionId = raw.into();
        assert_eq!(from_uuid.as_uuid(), raw);
    }

    #[test]
    fn subscription_is_cancelled() {
        let sub = Subscription {
            id: SubscriptionId::generate(),
            wallet_address: WalletAddress::from_trusted(
                "0x000000000000000000000000000000000000abc1".to_string(),
            ),
            plan_id: Uuid::new_v4(),
            payment_id: None,
            status: "cancelled".to_string(),
            started_at: None,
            expires_at: Utc::now() + chrono::Duration::days(7),
            cancelled_at: Some(Utc::now()),
            auto_renew: Some(false),
            metadata: None,
        };
        assert!(sub.is_cancelled());
    }

    #[test]
    fn admin_assign_sets_active_and_started_at() {
        let cmd = CreateSubscriptionCommand::admin_assign(
            WalletAddress::from_trusted(
                "0x000000000000000000000000000000000000abc2".to_string(),
            ),
            Uuid::new_v4(),
            Utc::now() + chrono::Duration::days(30),
            true,
        );
        assert_eq!(cmd.status, "active");
        assert!(cmd.started_at.is_some());
        assert!(cmd.payment_id.is_none());
        assert_eq!(cmd.auto_renew, Some(true));
    }
}
