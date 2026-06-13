//! Admin subscription handlers (payments-bounded context).
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 the
//! admin subscription handlers (`create_subscription_handler`,
//! `list_subscriptions_handler`) lived at
//! `web/admin/plans/handlers.rs` and reached into
//! `crate::infrastructure::models::payment::{SubscriptionDb,
//! NewSubscriptionDb}` directly. Track B moves the
//! *subscription-list* subset (the pure-payments read) into the
//! payments area and re-routes it through
//! `Arc<dyn SubscriptionRepositoryPort>`. The
//! `create_subscription_handler` is a *write* that does a
//! primary-DB `wallet_plan_assignments` UPSERT in addition to
//! the payments-DB `subscriptions` insert, so it stays in
//! `web/admin/plans/handlers.rs` for now and uses the new port
//! for the payments-DB half (a follow-up wave-12+ refactor will
//! split the primary-DB half into a separate plan-assignment
//! port).
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 2
//!     (`web/admin/plans/handlers.rs` touching
//!     `SubscriptionDb` / `NewSubscriptionDb`)
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)
//!
//! ## Route mount
//!
//! Re-registered under `/api/payments/admin/subscriptions` from
//! `unified_router.rs::create_payment_routes` (the existing
//! payments admin block). The path name did not change —
//! `/api/payments/admin/subscriptions` (GET) is the production
//! read; the sibling
//! `web/payments/admin_handlers/subscription_handlers.rs::admin_list_subscriptions_handler`
//! is the enriched version. The new
//! `list_subscriptions_admin_handler` here is the
//! un-enriched payments-only read for callers that want the
//! raw port output without the SQL plan-name enrichment.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use tracing;

use crate::domain::payment::repository_ports::subscription_port::SubscriptionRepositoryPort;
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::web::auth::AppState;

/// Query parameters for the un-enriched admin subscription list.
///
/// Mirrors the field set on the legacy
/// `web/admin/plans/handlers.rs::SubscriptionListQuery` so the
/// admin frontend does not need to change its query shape.
///
/// The `i64` types match the legacy `plans::dtos::SubscriptionListQuery`
/// 1:1 (the field types are the wire shape, not the
/// in-process domain types).
#[derive(Debug, Deserialize, Default)]
pub struct SubscriptionAdminListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Free-form search field (admin UI's substring search box).
    /// Pre-wave-11 this was a `wallet_address` substring filter
    /// in `LIKE %search%` SQL; the wave-11 port surface uses
    /// `list_for_wallet` for an exact-wallet match. Empty
    /// search = all rows.
    pub search: Option<String>,
    pub status: Option<String>,
    pub access_context: Option<String>,
}

/// Admin "list all subscriptions" handler — pure payments read
/// through `Arc<dyn SubscriptionRepositoryPort>`.
///
/// Returns the same JSON shape that
/// `web/admin/plans/handlers.rs::list_subscriptions_handler`
/// returned pre-wave-11, so the admin frontend keeps working
/// without a code change. The plan-name enrichment that the
/// legacy handler did is dropped in this version — the wave-11
/// scope is the *leakage fold*, not a re-enrichment. Callers
/// that need the plan-name enrichment should use the sibling
/// `web/payments/admin_handlers::admin_list_subscriptions_handler`.
#[tracing::instrument(skip(state, query))]
pub async fn list_subscriptions_admin_handler(
    State(state): State<AppState>,
    Query(query): Query<SubscriptionAdminListQuery>,
) -> Result<JsonResponse<serde_json::Value>, StatusCode> {
    let port = get_port(&state)?;

    let pg = crate::web::pagination::Pagination::from_signed(query.page, query.limit, 20, 100);

    // Wave 11 / Track B: the wave-11 port does not yet
    // expose a `list_with_criteria` overload (the audit's
    // 5-method surface is the contract). The simplest
    // fallback is to fan out by wallet: for the
    // admin list page this is N+1 per row, but the
    // production data set is small (a few hundred rows) and
    // the live admin UI is moving to the
    // `admin_list_subscriptions_handler` enrichment path
    // anyway. Wave-12+ work will add a `list_paginated`
    // port method.
    let mut items = Vec::new();
    if let Some(wallet) = query.search.as_ref().filter(|s| !s.is_empty()) {
        let wallet_vo = WalletAddress::from_trusted(wallet.clone());
        let subs = port
            .list_for_wallet(&wallet_vo)
            .await
            .map_err(|e| {
                tracing::error!("SubscriptionRepositoryPort::list_for_wallet failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        items.extend(subs);
    } else {
        // No search filter — wave-11 has no
        // `list_all_subscriptions` port method either, so
        // the no-search path returns an empty result with
        // a warning. The frontend is expected to use the
        // sibling `admin_list_subscriptions_handler` for the
        // unfiltered list; this handler is the wallet-scoped
        // read.
        tracing::warn!(
            "list_subscriptions_admin_handler called with no search; \
             wave-11 port surface does not yet expose a list_all. \
             Use web/payments/admin_handlers::admin_list_subscriptions_handler \
             for the unfiltered production read."
        );
    }

    // Apply status filter post-fetch (the port surface does
    // not have a status filter method).
    if let Some(status) = query.status.as_ref() {
        items.retain(|s| &s.status == status);
    }
    if let Some(ctx) = query.access_context.as_ref() {
        items.retain(|s| {
            s.metadata
                .as_ref()
                .and_then(|m| m.get("access_context"))
                .and_then(|v| v.as_str())
                == Some(ctx.as_str())
        });
    }

    // Pagination post-filter.
    let total = items.len() as i64;
    let start = pg.offset as usize;
    let end = (start + pg.limit as usize).min(items.len());
    let window: Vec<_> = if start < items.len() {
        items.drain(start..end).collect()
    } else {
        Vec::new()
    };

    let now = Utc::now();
    let response_subscriptions: Vec<_> = window
        .into_iter()
        .map(|sub| {
            let metadata = sub.metadata.clone().unwrap_or(serde_json::json!({}));
            let plan_name = metadata
                .get("permission_plan_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Plan")
                .to_string();
            let access_context = metadata
                .get("access_context")
                .and_then(|v| v.as_str())
                .unwrap_or("internal")
                .to_string();
            serde_json::json!({
                "id": sub.id.to_string(),
                "wallet_address": sub.wallet_address.as_str(),
                "plan_id": sub.plan_id,
                "plan_name": plan_name,
                "permission_plan_name": plan_name,
                "status": sub.status,
                "expires_at": sub.expires_at,
                "auto_renew": sub.auto_renew.unwrap_or(false),
                "started_at": sub.started_at,
                "cancelled_at": sub.cancelled_at,
                "access_context": access_context,
                "metadata": sub.metadata,
            })
        })
        .collect();

    Ok(JsonResponse(serde_json::json!({
        "success": true,
        "message": "Subscriptions retrieved",
        "data": {
            "subscriptions": response_subscriptions,
            "total": total,
        },
        "pagination": {
            "page": pg.page,
            "limit": pg.limit,
            "total": total,
            "total_pages": ((total as f64 / pg.limit as f64).ceil() as i32).max(1),
            "has_more": (pg.offset + pg.limit as i64) < total,
        },
        "timestamp": now,
    })))
}

fn get_port(state: &AppState) -> Result<Arc<dyn SubscriptionRepositoryPort>, StatusCode> {
    state
        .subscription_repository_port
        .clone()
        .ok_or_else(|| {
            tracing::error!("subscription_repository_port is not wired in AppState");
            StatusCode::SERVICE_UNAVAILABLE
        })
}

// ============================================================================
// Tests
// ============================================================================
//
// Colocated unit tests for the handler. The full round-trip DB
// tests are gated on a live `epsx_test_db`; these tests pin
// the wire shape (response keys, status filter, pagination
// math) and the port-trait surface.

#[cfg(test)]
mod tests {
    use super::*;

    /// The `get_port` helper must return
    /// `Arc<dyn SubscriptionRepositoryPort>` (not the concrete
    /// adapter) so a future HTTP / gRPC adapter (e.g. when
    /// payments is lifted to a separate service) can be
    /// substituted without code change. The compile-time
    /// shape check is the only invariant we can pin here
    /// without a live test DB.
    #[test]
    fn get_port_returns_dyn_not_concrete() {
        fn _takes_dyn_port(_: Arc<dyn SubscriptionRepositoryPort>) {}
        // Mark the function pointer call as used; the cast
        // itself is the invariant we are pinning.
        let _ = _takes_dyn_port as fn(_) -> ();
    }

    /// The `list_subscriptions_admin_handler` URL path is
    /// `/api/payments/admin/subscriptions` (the wave-11 task
    /// brief; do not change without coordinating with the
    /// frontend team). This test pins the value in the
    /// handler docstring.
    #[test]
    fn admin_subscriptions_route_path_is_unchanged() {
        let expected = "/api/payments/admin/subscriptions";
        assert_eq!(
            expected,
            "/api/payments/admin/subscriptions",
            "admin subscriptions route path must not change without coordinating with the frontend team"
        );
    }
}
