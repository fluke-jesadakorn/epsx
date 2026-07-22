use axum::http::Method;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPolicy {
    Public,
    CredentialExchange,
    Authenticated,
    Permission(&'static str),
    InternalOnly,
    Blocked,
}

/// Resolve the gateway boundary from the HTTP method and the canonical path.
/// The final arm is intentionally deny-by-default: mounting a new wildcard
/// proxy cannot accidentally make a downstream route reachable.
pub fn classify(method: &Method, path: &str) -> AccessPolicy {
    let Some(segments) = normalized_segments(path) else {
        return AccessPolicy::Blocked;
    };

    match (method, segments.as_slice()) {
        // Gateway liveness only. Axum's GET route also services HEAD.
        (&Method::GET | &Method::HEAD, ["health"]) => AccessPolicy::Public,

        // Identity credential exchange. Refresh is authenticated by the
        // refresh credential itself and must not require a valid access JWT.
        (&Method::POST, ["api", "v1", "identity", "auth", "challenge"])
        | (&Method::POST, ["api", "v1", "identity", "auth", "siwe"])
        | (&Method::POST, ["api", "v1", "identity", "auth", "refresh"]) => {
            AccessPolicy::CredentialExchange
        }
        (&Method::GET, ["api", "v1", "identity", "auth", "me"]) => AccessPolicy::Authenticated,
        (&Method::GET, ["api", "v1", "identity", "users"])
        | (&Method::GET, ["api", "v1", "identity", "users", _]) => {
            AccessPolicy::Permission("admin:users:read")
        }
        (&Method::POST, ["api", "v1", "identity", "users"]) => {
            AccessPolicy::Permission("admin:users:create")
        }
        (&Method::PUT, ["api", "v1", "identity", "users", _]) => {
            AccessPolicy::Permission("admin:users:update")
        }
        (&Method::DELETE, ["api", "v1", "identity", "users", _]) => {
            AccessPolicy::Permission("admin:users:delete")
        }

        // Wallet. Owner comparisons are still required in the wallet service;
        // this gateway slice supplies only the verified principal boundary.
        (&Method::GET, ["api", "v1", "wallet", "balance", _, _])
        | (&Method::POST, ["api", "v1", "wallet", "verify-message"])
        | (&Method::POST, ["api", "v1", "wallet", "estimate-gas"]) => AccessPolicy::Public,
        (&Method::GET | &Method::POST, ["api", "v1", "wallet", "accounts"])
        | (&Method::GET, ["api", "v1", "wallet", "accounts", _])
        | (&Method::POST, ["api", "v1", "wallet", "send"])
        | (&Method::POST, ["api", "v1", "wallet", "sign-message"]) => AccessPolicy::Authenticated,

        // Subscription. Plan and vault reads are the locked public surface.
        (&Method::GET, ["api", "v1", "subscription", "plans"])
        | (&Method::GET, ["api", "v1", "subscription", "plans", _])
        | (&Method::GET, ["api", "v1", "subscription", "vault", _]) => AccessPolicy::Public,
        (&Method::POST, ["api", "v1", "subscription", "plans"]) => {
            AccessPolicy::Permission("admin:plans:manage")
        }
        (&Method::GET | &Method::POST, ["api", "v1", "subscription", "subscriptions"])
        | (&Method::GET, ["api", "v1", "subscription", "subscriptions", _])
        | (&Method::POST, ["api", "v1", "subscription", "subscriptions", _, "cancel"]) => {
            AccessPolicy::Authenticated
        }

        // Content public rendering and read models.
        (&Method::GET, ["api", "v1", "content", "pages", _, "render"])
        | (&Method::GET, ["api", "v1", "content", "themes"])
        | (&Method::GET, ["api", "v1", "content", "themes", _])
        | (&Method::GET, ["api", "v1", "content", "blocks"])
        | (&Method::GET, ["api", "v1", "content", "blocks", _])
        | (&Method::GET, ["api", "v1", "content", "navigation"])
        | (&Method::GET, ["api", "v1", "content", "site"])
        | (&Method::GET, ["api", "v1", "content", "news"])
        | (&Method::GET, ["api", "v1", "content", "news", _])
        | (&Method::GET, ["api", "v1", "content", "plans"])
        | (&Method::GET, ["api", "v1", "content", "rankings"])
        | (&Method::GET, ["api", "v1", "content", "portfolio", _]) => AccessPolicy::Public,
        (&Method::GET | &Method::POST, ["api", "v1", "content", "pages"])
        | (&Method::GET | &Method::PUT, ["api", "v1", "content", "pages", _])
        | (&Method::POST, ["api", "v1", "content", "pages", _, "publish"])
        | (&Method::POST, ["api", "v1", "content", "themes"])
        | (&Method::PUT, ["api", "v1", "content", "themes", _])
        | (&Method::POST, ["api", "v1", "content", "edit", "start"])
        | (&Method::POST, ["api", "v1", "content", "edit", "commit"])
        | (&Method::GET, ["api", "v1", "content", "edit", "sessions"]) => {
            AccessPolicy::Permission("admin:content:manage")
        }

        // Public compatibility aliases are GET-only.
        (&Method::GET, ["api", "v1", "news"])
        | (&Method::GET, ["api", "v1", "news", _])
        | (&Method::GET, ["api", "v1", "portfolio", _])
        | (&Method::GET, ["api", "v1", "plans"])
        | (&Method::GET, ["api", "v1", "rankings"]) => AccessPolicy::Public,

        // Notifications. Owner filtering remains a downstream requirement.
        (&Method::GET | &Method::POST, ["api", "v1", "notification", "templates"])
        | (&Method::GET | &Method::DELETE, ["api", "v1", "notification", "templates", _])
        | (&Method::POST, ["api", "v1", "notification", "send"]) => {
            AccessPolicy::Permission("admin:notifications:manage")
        }
        (&Method::GET, ["api", "v1", "notification", "list"])
        | (&Method::GET, ["api", "v1", "notification", "unread-count"])
        | (&Method::POST, ["api", "v1", "notification", "mark-all-read"])
        | (&Method::POST, ["api", "v1", "notification", "clear-all"])
        | (&Method::GET | &Method::DELETE, ["api", "v1", "notification", _])
        | (&Method::POST, ["api", "v1", "notification", _, "read"])
        | (&Method::POST, ["api", "v1", "notification", _, "unread"]) => {
            AccessPolicy::Authenticated
        }

        // Analytics. Prometheus surfaces remain private to trusted internal
        // networking until an internal service identity is implemented.
        (&Method::POST, ["api", "v1", "analytics", "track"]) => AccessPolicy::Authenticated,
        (&Method::GET, ["api", "v1", "analytics", "metrics", "prometheus"])
        | (&Method::GET, ["api", "v1", "analytics", "prometheus", "metrics"]) => {
            AccessPolicy::InternalOnly
        }
        (&Method::GET, ["api", "v1", "analytics", "events"])
        | (&Method::GET, ["api", "v1", "analytics", "metrics", _])
        | (&Method::GET, ["api", "v1", "analytics", "revenue"]) => {
            AccessPolicy::Permission("admin:analytics:view")
        }

        // Indexer reads are public; sync is narrowed to the intended POST
        // operator mutation despite the candidate service's `any` mount.
        (&Method::GET, ["api", "v1", "indexer", "status", _])
        | (&Method::GET, ["api", "v1", "indexer", "block", _, _])
        | (&Method::GET, ["api", "v1", "indexer", "tx", _, _])
        | (&Method::GET, ["api", "v1", "indexer", "transfers", _, _]) => AccessPolicy::Public,
        (&Method::POST, ["api", "v1", "indexer", "sync"]) => {
            AccessPolicy::Permission("admin:indexer:manage")
        }

        // Account payment history is the only Pay route exposed by this
        // gateway slice. The Pay service remains the owner authority: this
        // boundary authenticates the caller but does not compare wallets.
        (&Method::GET, ["api", "v1", "pay", "history", wallet]) if safe_wallet_segment(wallet) => {
            AccessPolicy::Authenticated
        }

        // All other Pay and legacy payment shapes remain deny-by-default.
        _ => AccessPolicy::Blocked,
    }
}

pub(super) const MAX_WALLET_SEGMENT_BYTES: usize = 128;

fn safe_wallet_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= MAX_WALLET_SEGMENT_BYTES
        && !segment.starts_with("force-")
        && !matches!(
            segment,
            "health"
                | "pay"
                | "admin"
                | "intents"
                | "escrows"
                | "links"
                | "history"
                | "webhooks"
                | "on-chain"
                | "sync"
                | "confirm"
                | "cancel"
                | "release"
                | "refund"
                | "dispute"
                | "resolve"
                | "confirm-deposit"
                | "redeem"
                | "force-cancel"
                | "force-release"
                | "force-refund"
        )
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn normalized_segments(path: &str) -> Option<Vec<&str>> {
    if !path.starts_with('/')
        || path.len() > 2048
        || path.contains('%')
        || path.contains("//")
        || path.ends_with('/')
    {
        return None;
    }
    let segments: Vec<_> = path[1..].split('/').collect();
    if segments.is_empty()
        || segments
            .iter()
            .any(|segment| segment.is_empty() || matches!(*segment, "." | ".."))
    {
        return None;
    }
    Some(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_policy_table_is_method_and_path_exact() {
        let cases = [
            (Method::GET, "/health", AccessPolicy::Public),
            (
                Method::POST,
                "/api/v1/identity/auth/challenge",
                AccessPolicy::CredentialExchange,
            ),
            (
                Method::POST,
                "/api/v1/identity/auth/siwe",
                AccessPolicy::CredentialExchange,
            ),
            (
                Method::POST,
                "/api/v1/identity/auth/refresh",
                AccessPolicy::CredentialExchange,
            ),
            (
                Method::GET,
                "/api/v1/identity/auth/me",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/identity/users/123",
                AccessPolicy::Permission("admin:users:read"),
            ),
            (
                Method::POST,
                "/api/v1/wallet/accounts",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/wallet/balance/56/0xabc",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/subscription/plans",
                AccessPolicy::Permission("admin:plans:manage"),
            ),
            (
                Method::GET,
                "/api/v1/subscription/subscriptions/123",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/content/pages/home/render",
                AccessPolicy::Public,
            ),
            (
                Method::PUT,
                "/api/v1/content/pages/home",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (Method::GET, "/api/v1/news/story", AccessPolicy::Public),
            (
                Method::POST,
                "/api/v1/notification/abc/read",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/notification/send",
                AccessPolicy::Permission("admin:notifications:manage"),
            ),
            (
                Method::GET,
                "/api/v1/analytics/events",
                AccessPolicy::Permission("admin:analytics:view"),
            ),
            (
                Method::GET,
                "/api/v1/analytics/metrics/prometheus",
                AccessPolicy::InternalOnly,
            ),
            (
                Method::GET,
                "/api/v1/indexer/status/56",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/indexer/sync",
                AccessPolicy::Permission("admin:indexer:manage"),
            ),
            (
                Method::GET,
                "/api/v1/pay/history/0x1111111111111111111111111111111111111111",
                AccessPolicy::Authenticated,
            ),
        ];

        for (method, path, expected) in cases {
            assert_eq!(classify(&method, path), expected, "{method} {path}");
        }
    }

    #[test]
    fn every_reachable_locked_service_route_has_an_explicit_policy() {
        let cases = [
            // identity (10 reachable non-health routes)
            (
                Method::POST,
                "/api/v1/identity/auth/challenge",
                AccessPolicy::CredentialExchange,
            ),
            (
                Method::POST,
                "/api/v1/identity/auth/siwe",
                AccessPolicy::CredentialExchange,
            ),
            (
                Method::POST,
                "/api/v1/identity/auth/refresh",
                AccessPolicy::CredentialExchange,
            ),
            (
                Method::GET,
                "/api/v1/identity/auth/me",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/identity/users",
                AccessPolicy::Permission("admin:users:read"),
            ),
            (
                Method::POST,
                "/api/v1/identity/users",
                AccessPolicy::Permission("admin:users:create"),
            ),
            (
                Method::GET,
                "/api/v1/identity/users/123",
                AccessPolicy::Permission("admin:users:read"),
            ),
            (
                Method::PUT,
                "/api/v1/identity/users/123",
                AccessPolicy::Permission("admin:users:update"),
            ),
            (
                Method::DELETE,
                "/api/v1/identity/users/123",
                AccessPolicy::Permission("admin:users:delete"),
            ),
            // wallet (8 routes)
            (
                Method::POST,
                "/api/v1/wallet/accounts",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/wallet/accounts",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/wallet/accounts/0xabc",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/wallet/balance/56/0xabc",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/wallet/send",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/wallet/sign-message",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/wallet/verify-message",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/wallet/estimate-gas",
                AccessPolicy::Public,
            ),
            // subscription (8 routes)
            (
                Method::POST,
                "/api/v1/subscription/plans",
                AccessPolicy::Permission("admin:plans:manage"),
            ),
            (
                Method::GET,
                "/api/v1/subscription/plans",
                AccessPolicy::Public,
            ),
            (
                Method::GET,
                "/api/v1/subscription/plans/basic",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/subscription/subscriptions",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/subscription/subscriptions",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/subscription/subscriptions/123",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/subscription/subscriptions/123/cancel",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/subscription/vault/56",
                AccessPolicy::Public,
            ),
            // content (22 routes)
            (
                Method::GET,
                "/api/v1/content/pages/home",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::PUT,
                "/api/v1/content/pages/home",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::POST,
                "/api/v1/content/pages",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::GET,
                "/api/v1/content/pages",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::POST,
                "/api/v1/content/pages/123/publish",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::GET,
                "/api/v1/content/pages/home/render",
                AccessPolicy::Public,
            ),
            (Method::GET, "/api/v1/content/themes", AccessPolicy::Public),
            (
                Method::POST,
                "/api/v1/content/themes",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::GET,
                "/api/v1/content/themes/123",
                AccessPolicy::Public,
            ),
            (
                Method::PUT,
                "/api/v1/content/themes/123",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (Method::GET, "/api/v1/content/blocks", AccessPolicy::Public),
            (
                Method::GET,
                "/api/v1/content/blocks/hero",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/content/edit/start",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::POST,
                "/api/v1/content/edit/commit",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::GET,
                "/api/v1/content/edit/sessions",
                AccessPolicy::Permission("admin:content:manage"),
            ),
            (
                Method::GET,
                "/api/v1/content/navigation",
                AccessPolicy::Public,
            ),
            (Method::GET, "/api/v1/content/site", AccessPolicy::Public),
            (Method::GET, "/api/v1/content/news", AccessPolicy::Public),
            (
                Method::GET,
                "/api/v1/content/news/story",
                AccessPolicy::Public,
            ),
            (Method::GET, "/api/v1/content/plans", AccessPolicy::Public),
            (
                Method::GET,
                "/api/v1/content/rankings",
                AccessPolicy::Public,
            ),
            (
                Method::GET,
                "/api/v1/content/portfolio/0xabc",
                AccessPolicy::Public,
            ),
            // public gateway aliases (5 routes)
            (Method::GET, "/api/v1/news", AccessPolicy::Public),
            (Method::GET, "/api/v1/news/story", AccessPolicy::Public),
            (Method::GET, "/api/v1/portfolio/0xabc", AccessPolicy::Public),
            (Method::GET, "/api/v1/plans", AccessPolicy::Public),
            (Method::GET, "/api/v1/rankings", AccessPolicy::Public),
            // notification (13 routes)
            (
                Method::GET,
                "/api/v1/notification/templates",
                AccessPolicy::Permission("admin:notifications:manage"),
            ),
            (
                Method::POST,
                "/api/v1/notification/templates",
                AccessPolicy::Permission("admin:notifications:manage"),
            ),
            (
                Method::GET,
                "/api/v1/notification/templates/123",
                AccessPolicy::Permission("admin:notifications:manage"),
            ),
            (
                Method::DELETE,
                "/api/v1/notification/templates/123",
                AccessPolicy::Permission("admin:notifications:manage"),
            ),
            (
                Method::POST,
                "/api/v1/notification/send",
                AccessPolicy::Permission("admin:notifications:manage"),
            ),
            (
                Method::GET,
                "/api/v1/notification/list",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/notification/unread-count",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/notification/mark-all-read",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/notification/clear-all",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/notification/123/read",
                AccessPolicy::Authenticated,
            ),
            (
                Method::POST,
                "/api/v1/notification/123/unread",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/notification/123",
                AccessPolicy::Authenticated,
            ),
            (
                Method::DELETE,
                "/api/v1/notification/123",
                AccessPolicy::Authenticated,
            ),
            // analytics (6 routes)
            (
                Method::POST,
                "/api/v1/analytics/track",
                AccessPolicy::Authenticated,
            ),
            (
                Method::GET,
                "/api/v1/analytics/events",
                AccessPolicy::Permission("admin:analytics:view"),
            ),
            (
                Method::GET,
                "/api/v1/analytics/metrics/active-users",
                AccessPolicy::Permission("admin:analytics:view"),
            ),
            (
                Method::GET,
                "/api/v1/analytics/revenue",
                AccessPolicy::Permission("admin:analytics:view"),
            ),
            (
                Method::GET,
                "/api/v1/analytics/metrics/prometheus",
                AccessPolicy::InternalOnly,
            ),
            (
                Method::GET,
                "/api/v1/analytics/prometheus/metrics",
                AccessPolicy::InternalOnly,
            ),
            // indexer (5 routes)
            (
                Method::GET,
                "/api/v1/indexer/status/56",
                AccessPolicy::Public,
            ),
            (
                Method::GET,
                "/api/v1/indexer/block/56/100",
                AccessPolicy::Public,
            ),
            (
                Method::GET,
                "/api/v1/indexer/tx/56/0xabc",
                AccessPolicy::Public,
            ),
            (
                Method::GET,
                "/api/v1/indexer/transfers/56/0xabc",
                AccessPolicy::Public,
            ),
            (
                Method::POST,
                "/api/v1/indexer/sync",
                AccessPolicy::Permission("admin:indexer:manage"),
            ),
        ];

        for (method, path, expected) in cases {
            assert_eq!(classify(&method, path), expected, "{method} {path}");
        }
    }

    #[test]
    fn unknown_method_path_and_payment_drift_are_blocked() {
        let cases = [
            (Method::POST, "/api/v1/wallet/balance/56/0xabc"),
            (Method::GET, "/api/v1/identity/auth/demo"),
            (Method::POST, "/api/v1/identity/auth/demo"),
            (Method::GET, "/api/v1/payment/intents"),
            (Method::POST, "/api/v1/payment/intents"),
            (Method::GET, "/api/v1/pay/intents"),
            (Method::POST, "/api/v1/pay/intents"),
            (Method::GET, "/api/v1/news/too/many"),
            (Method::GET, "/api/v1/unknown"),
            (Method::GET, "/api/v1/indexer/sync"),
            (Method::PUT, "/api/v1/indexer/sync"),
            (Method::GET, "/api//v1/content/site"),
            (Method::GET, "/api/v1/content/%2e%2e/site"),
        ];
        for (method, path) in cases {
            assert_eq!(
                classify(&method, path),
                AccessPolicy::Blocked,
                "{method} {path}"
            );
        }
    }

    #[test]
    fn owner_history_policy_is_exact_and_rejects_unsafe_wallet_segments() {
        let wallet = "0x1111111111111111111111111111111111111111";
        assert_eq!(
            classify(&Method::GET, &format!("/api/v1/pay/history/{wallet}")),
            AccessPolicy::Authenticated
        );

        let oversized = "a".repeat(MAX_WALLET_SEGMENT_BYTES + 1);
        let blocked = [
            (Method::HEAD, format!("/api/v1/pay/history/{wallet}")),
            (Method::POST, format!("/api/v1/pay/history/{wallet}")),
            (Method::PUT, format!("/api/v1/pay/history/{wallet}")),
            (Method::DELETE, format!("/api/v1/pay/history/{wallet}")),
            (Method::GET, "/api/v1/pay/history".into()),
            (Method::GET, format!("/api/v1/pay/history/{wallet}/extra")),
            (Method::GET, format!("/api/v1/pay/history/{wallet}/")),
            (
                Method::GET,
                format!("/api/v1/pay/history/{wallet}?limit=10"),
            ),
            (Method::GET, "/api/v1/pay/history/0xabc%2Fextra".into()),
            (Method::GET, "/api/v1/pay/history/0xabc%252Fextra".into()),
            (Method::GET, "/api/v1/pay/history/0xabc\\extra".into()),
            (Method::GET, "/api/v1/pay/history/wallet:0xabc".into()),
            (Method::GET, "/api/v1/pay/history/history".into()),
            (Method::GET, "/api/v1/pay/history/force-release".into()),
            (Method::GET, format!("/api/v1/pay/history/{oversized}")),
            (Method::GET, format!("/api/v1/payment/history/{wallet}")),
        ];
        for (method, path) in blocked {
            assert_eq!(
                classify(&method, &path),
                AccessPolicy::Blocked,
                "{method} {path}"
            );
        }
    }
}
