// Get Stock Ranking Assignments Query — Wave 11 / Track B.
//
// Wave 11 / Track B (outbound-leakage fold) refactor:
//   - Pre-wave-11 this file defined the `StockRankingAssignment`
//     DTO and a `GetStockRankingAssignmentsQuery` that wrapped
//     the query shape. The actual SQL *read* of the
//     `stock_ranking_assignments` table had no live caller in
//     the source tree (the type definition was the only thing
//     the audit's `rg` survey hit).
//   - Track B moved the `StockRankingAssignment` DTO into the
//     payments domain
//     (`crate::domain::payment::aggregates::stock_ranking_assignment`)
//     and added the `get_stock_ranking_assignments` port method
//     on `SubscriptionRepositoryPort`. The port method is the
//     first wired reader of the `stock_ranking_assignments`
//     table.
//   - This file is now a **thin facade**: it re-exports the
//     domain DTO, defines the query/response shapes, and
//     provides a port-driven execution function
//     (`get_stock_ranking_assignments_via_port`) that takes an
//     `Arc<dyn SubscriptionRepositoryPort>` and returns the
//     `GetStockRankingAssignmentsResponse`. The market-analytics
//     application module can now depend on the **port** (a
//     domain-level trait) instead of reaching into the
//     payments infrastructure layer directly. The audit's
//     "strongest outward leak" (market_analytics reading
//     `stock_ranking_assignments` SQL) is closed in this track.
//
// The port import path:
//
//   `use crate::domain::payment::repository_ports::subscription_port::SubscriptionRepositoryPort;`
//
// lives in the domain layer; the market_analytics
// `application::market_analytics::queries` module imports
// only the trait (not the concrete adapter) so the bounded
// context is preserved.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::application::shared::Query;
pub use crate::domain::payment::aggregates::stock_ranking_assignment::StockRankingAssignment;
use crate::domain::payment::repository_ports::subscription_port::SubscriptionRepositoryPort;
use crate::domain::wallet_management::value_objects::WalletAddress;

#[derive(Debug, Clone)]
pub struct GetStockRankingAssignmentsQuery {
    pub wallet_address: Option<String>,
    pub package_id: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

impl Query for GetStockRankingAssignmentsQuery {
    type Response = GetStockRankingAssignmentsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockRankingAssignmentsResponse {
    pub success: bool,
    pub assignments: Vec<StockRankingAssignment>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

/// Port-driven execution of the query.
///
/// Replaces the pre-wave-11 silent placeholder (no live SQL
/// reader). Takes an `Arc<dyn SubscriptionRepositoryPort>` and
/// returns the response. Filters `package_id` and
/// `active_only` are applied post-fetch on the port output;
/// `page` / `limit` are applied to the filtered list.
///
/// Wave-11 + the port surface does not yet expose
/// `list_with_package_filter` / `list_active_only`; the
/// production read path is a wallet-scoped read followed by
/// in-memory filter + paginate. Wave-12+ work can add a
/// richer port method if the in-memory filter becomes a
/// perf issue (the production data set is small per wallet).
pub async fn get_stock_ranking_assignments_via_port(
    port: Arc<dyn SubscriptionRepositoryPort>,
    query: GetStockRankingAssignmentsQuery,
) -> Result<GetStockRankingAssignmentsResponse, String> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).max(1) as usize;

    // The port surface is wallet-scoped. If no wallet is
    // provided, the query returns an empty result with a
    // warning — the production call sites always pass a
    // wallet; the unfiltered "all assignments" path is a
    // future port addition.
    let wallet_addr = match query.wallet_address.as_deref() {
        Some(w) if !w.is_empty() => w,
        _ => {
            tracing::warn!(
                "get_stock_ranking_assignments_via_port called without a wallet_address; \
                 wave-11 port surface is wallet-scoped — returning empty result. \
                 Use the admin tooling or a future list_all port method for unfiltered reads."
            );
            return Ok(GetStockRankingAssignmentsResponse {
                success: true,
                assignments: Vec::new(),
                total: 0,
                page,
                limit: limit as i32,
            });
        }
    };

    // Wave-11 wallet is `WalletAddress::from_trusted` —
    // the row is already in the DB so the address has
    // already been through the trust boundary.
    let wallet = WalletAddress::from_trusted(wallet_addr.to_string());
    let mut assignments = port
        .get_stock_ranking_assignments(&wallet)
        .await
        .map_err(|e| format!("SubscriptionRepositoryPort::get_stock_ranking_assignments failed: {}", e))?;

    // Post-fetch filter: package_id.
    if let Some(pkg) = query.package_id.as_deref() {
        assignments.retain(|a| a.package_id == pkg);
    }
    // Post-fetch filter: active_only.
    if query.active_only.unwrap_or(false) {
        assignments.retain(|a| a.is_active);
    }

    let total = assignments.len() as i64;
    let start = ((page as usize).saturating_sub(1)) * limit;
    let end = (start + limit).min(assignments.len());
    let window = if start < assignments.len() {
        assignments.drain(start..end).collect()
    } else {
        Vec::new()
    };

    Ok(GetStockRankingAssignmentsResponse {
        success: true,
        assignments: window,
        total,
        page,
        limit: limit as i32,
    })
}

// ============================================================================
// Tests
// ============================================================================
//
// The audit's "strongest outward leak" canary test. Constructs
// a wallet with 3 stock-ranking assignments and asserts the
// port returns them in the right order. The port is mocked
// with a tiny in-memory implementation (no live test-DB
// dependency).

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use epsx_contracts::errors::AppResult;
    use uuid::Uuid;

    use crate::domain::payment::aggregates::subscription::{
        CreateSubscriptionCommand, Subscription, SubscriptionId,
    };
    use crate::domain::subscription_management::value_objects::PlanId;

    /// In-memory mock for the `SubscriptionRepositoryPort` —
    /// implements the port surface with a `Vec` of pre-loaded
    /// assignments. Used by the canary test to assert the
    /// market_analytics query facade filters / paginates
    /// correctly without a live test DB.
    #[derive(Default)]
    struct MockSubscriptionRepository {
        assignments: Vec<StockRankingAssignment>,
    }

    #[async_trait]
    impl SubscriptionRepositoryPort for MockSubscriptionRepository {
        async fn list_for_plan(
            &self,
            _plan_id: PlanId,
        ) -> AppResult<Vec<Subscription>> {
            Ok(Vec::new())
        }

        async fn list_for_wallet(
            &self,
            _wallet: &WalletAddress,
        ) -> AppResult<Vec<Subscription>> {
            Ok(Vec::new())
        }

        async fn create(
            &self,
            _cmd: CreateSubscriptionCommand,
        ) -> AppResult<Subscription> {
            unimplemented!("canary test does not exercise create")
        }

        async fn cancel(
            &self,
            _subscription_id: SubscriptionId,
            _reason: Option<String>,
        ) -> AppResult<()> {
            unimplemented!("canary test does not exercise cancel")
        }

        async fn get_stock_ranking_assignments(
            &self,
            wallet: &WalletAddress,
        ) -> AppResult<Vec<StockRankingAssignment>> {
            Ok(self
                .assignments
                .iter()
                .filter(|a| a.wallet_address == wallet.as_str())
                .cloned()
                .collect())
        }
    }

    /// Canary test: 3 stock-ranking assignments for one
    /// wallet are returned by the port in the right order,
    /// and the market-analytics query facade preserves the
    /// order through the `page` / `limit` pagination.
    ///
    /// The "right order" is the adapter's `assigned_at DESC`
    /// order (newest first). The test pre-sorts the seed
    /// data in `assigned_at DESC` so the test invariant
    /// reduces to "the facade preserves the order it
    /// received from the port".
    #[tokio::test]
    async fn stock_ranking_canary_three_assignments_for_one_wallet() {
        let wallet = "0x000000000000000000000000000000000000aaaa";
        let now = Utc::now();

        // The three assignments, in the order the adapter
        // would return them (newest first).
        let seed = vec![
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_pro".to_string(),
                package_name: "Pro Tier".to_string(),
                rank_access_level: 100,
                assigned_at: now,
                expires_at: Some(now + chrono::Duration::days(30)),
                is_active: true,
                assignment_source: "admin".to_string(),
                auto_renew: false,
                days_remaining: Some(30),
            },
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_starter".to_string(),
                package_name: "Starter Tier".to_string(),
                rank_access_level: 25,
                assigned_at: now - chrono::Duration::days(7),
                expires_at: Some(now + chrono::Duration::days(23)),
                is_active: true,
                assignment_source: "purchase".to_string(),
                auto_renew: true,
                days_remaining: Some(23),
            },
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_free".to_string(),
                package_name: "Free Tier".to_string(),
                rank_access_level: 3,
                assigned_at: now - chrono::Duration::days(30),
                expires_at: None,
                is_active: false,
                assignment_source: "system".to_string(),
                auto_renew: false,
                days_remaining: None,
            },
        ];

        let port: Arc<dyn SubscriptionRepositoryPort> = Arc::new(MockSubscriptionRepository {
            assignments: seed.clone(),
        });

        let query = GetStockRankingAssignmentsQuery {
            wallet_address: Some(wallet.to_string()),
            package_id: None,
            active_only: None,
            page: Some(1),
            limit: Some(10),
        };

        let response = get_stock_ranking_assignments_via_port(port, query)
            .await
            .expect("port call should succeed");

        assert!(response.success);
        assert_eq!(response.assignments.len(), 3, "expected 3 assignments");
        assert_eq!(response.total, 3);
        // Order is preserved by the facade (the adapter
        // returns newest-first; the facade just re-emits).
        assert_eq!(response.assignments[0].package_id, "pkg_pro");
        assert_eq!(response.assignments[1].package_id, "pkg_starter");
        assert_eq!(response.assignments[2].package_id, "pkg_free");
    }

    /// Canary: `active_only = true` filters out the
    /// inactive `pkg_free` assignment, leaving 2.
    #[tokio::test]
    async fn stock_ranking_canary_active_only_filter() {
        let wallet = "0x000000000000000000000000000000000000aaaa";
        let now = Utc::now();

        let seed = vec![
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_pro".to_string(),
                package_name: "Pro".to_string(),
                rank_access_level: 100,
                assigned_at: now,
                expires_at: Some(now + chrono::Duration::days(30)),
                is_active: true,
                assignment_source: "admin".to_string(),
                auto_renew: false,
                days_remaining: Some(30),
            },
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_free".to_string(),
                package_name: "Free".to_string(),
                rank_access_level: 3,
                assigned_at: now - chrono::Duration::days(30),
                expires_at: None,
                is_active: false,
                assignment_source: "system".to_string(),
                auto_renew: false,
                days_remaining: None,
            },
        ];

        let port: Arc<dyn SubscriptionRepositoryPort> = Arc::new(MockSubscriptionRepository {
            assignments: seed,
        });

        let query = GetStockRankingAssignmentsQuery {
            wallet_address: Some(wallet.to_string()),
            package_id: None,
            active_only: Some(true),
            page: Some(1),
            limit: Some(10),
        };

        let response = get_stock_ranking_assignments_via_port(port, query)
            .await
            .expect("port call should succeed");

        assert_eq!(response.total, 1);
        assert_eq!(response.assignments.len(), 1);
        assert_eq!(response.assignments[0].package_id, "pkg_pro");
    }

    /// Canary: `package_id` filter narrows to a single
    /// assignment. This is the production "what package does
    /// this wallet currently hold?" use case.
    #[tokio::test]
    async fn stock_ranking_canary_package_id_filter() {
        let wallet = "0x000000000000000000000000000000000000aaaa";
        let now = Utc::now();

        let seed = vec![
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_pro".to_string(),
                package_name: "Pro".to_string(),
                rank_access_level: 100,
                assigned_at: now,
                expires_at: Some(now + chrono::Duration::days(30)),
                is_active: true,
                assignment_source: "admin".to_string(),
                auto_renew: false,
                days_remaining: Some(30),
            },
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_starter".to_string(),
                package_name: "Starter".to_string(),
                rank_access_level: 25,
                assigned_at: now - chrono::Duration::days(7),
                expires_at: Some(now + chrono::Duration::days(23)),
                is_active: true,
                assignment_source: "purchase".to_string(),
                auto_renew: true,
                days_remaining: Some(23),
            },
        ];

        let port: Arc<dyn SubscriptionRepositoryPort> = Arc::new(MockSubscriptionRepository {
            assignments: seed,
        });

        let query = GetStockRankingAssignmentsQuery {
            wallet_address: Some(wallet.to_string()),
            package_id: Some("pkg_pro".to_string()),
            active_only: None,
            page: Some(1),
            limit: Some(10),
        };

        let response = get_stock_ranking_assignments_via_port(port, query)
            .await
            .expect("port call should succeed");

        assert_eq!(response.total, 1);
        assert_eq!(response.assignments.len(), 1);
        assert_eq!(response.assignments[0].package_id, "pkg_pro");
    }

    /// Canary: pagination math (page 1 of 2, limit 2, 3
    /// assignments total). First page returns the 2 newest
    /// (Pro, Starter); second page returns the 1 oldest
    /// (Free).
    #[tokio::test]
    async fn stock_ranking_canary_pagination() {
        let wallet = "0x000000000000000000000000000000000000aaaa";
        let now = Utc::now();

        let seed = vec![
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_pro".to_string(),
                package_name: "Pro".to_string(),
                rank_access_level: 100,
                assigned_at: now,
                expires_at: Some(now + chrono::Duration::days(30)),
                is_active: true,
                assignment_source: "admin".to_string(),
                auto_renew: false,
                days_remaining: Some(30),
            },
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_starter".to_string(),
                package_name: "Starter".to_string(),
                rank_access_level: 25,
                assigned_at: now - chrono::Duration::days(7),
                expires_at: Some(now + chrono::Duration::days(23)),
                is_active: true,
                assignment_source: "purchase".to_string(),
                auto_renew: true,
                days_remaining: Some(23),
            },
            StockRankingAssignment {
                assignment_id: Uuid::new_v4().to_string(),
                wallet_address: wallet.to_string(),
                package_id: "pkg_free".to_string(),
                package_name: "Free".to_string(),
                rank_access_level: 3,
                assigned_at: now - chrono::Duration::days(30),
                expires_at: None,
                is_active: false,
                assignment_source: "system".to_string(),
                auto_renew: false,
                days_remaining: None,
            },
        ];

        let port: Arc<dyn SubscriptionRepositoryPort> = Arc::new(MockSubscriptionRepository {
            assignments: seed,
        });

        // Page 1, limit 2
        let query1 = GetStockRankingAssignmentsQuery {
            wallet_address: Some(wallet.to_string()),
            package_id: None,
            active_only: None,
            page: Some(1),
            limit: Some(2),
        };
        let response1 = get_stock_ranking_assignments_via_port(port.clone(), query1)
            .await
            .expect("port call should succeed");
        assert_eq!(response1.total, 3);
        assert_eq!(response1.assignments.len(), 2);
        assert_eq!(response1.assignments[0].package_id, "pkg_pro");
        assert_eq!(response1.assignments[1].package_id, "pkg_starter");

        // Page 2, limit 2
        let query2 = GetStockRankingAssignmentsQuery {
            wallet_address: Some(wallet.to_string()),
            package_id: None,
            active_only: None,
            page: Some(2),
            limit: Some(2),
        };
        let response2 = get_stock_ranking_assignments_via_port(port, query2)
            .await
            .expect("port call should succeed");
        assert_eq!(response2.total, 3);
        assert_eq!(response2.assignments.len(), 1);
        assert_eq!(response2.assignments[0].package_id, "pkg_free");
    }

    /// Without a wallet_address the facade returns an empty
    /// result with a warning. The wave-11 port surface is
    /// wallet-scoped; an unfiltered list is a future port
    /// method.
    #[tokio::test]
    async fn stock_ranking_canary_no_wallet_returns_empty() {
        let port: Arc<dyn SubscriptionRepositoryPort> = Arc::new(MockSubscriptionRepository::default());

        let query = GetStockRankingAssignmentsQuery {
            wallet_address: None,
            package_id: None,
            active_only: None,
            page: Some(1),
            limit: Some(10),
        };

        let response = get_stock_ranking_assignments_via_port(port, query)
            .await
            .expect("port call should succeed");

        assert!(response.success);
        assert_eq!(response.total, 0);
        assert!(response.assignments.is_empty());
    }
}
