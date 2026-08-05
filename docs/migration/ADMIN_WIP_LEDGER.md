# Admin migration WIP ledger

This ledger records the admin-related hunks that were present in the dirty
checkout when the clean integration worktree was created. The source checkout
was not modified. `adopted` means the hunk was copied into the integration
worktree for validation; `deferred` means it remains only in the dirty
checkout until an explicit, evidence-backed integration decision.

| WIP path | Hunk scope | Disposition | Evidence / reason |
| --- | --- | --- | --- |
| `apps/admin/Cargo.toml` | `url` dependency addition | adopted | Required by strict form decoding in `apps/admin/src/main.rs`. |
| `apps/admin/src/auth.rs` | local/design bypass UI identity and tests | adopted | UI-only preview identity; no bearer token or upstream authorization. |
| `apps/admin/src/main.rs` | bounded notification form, notification routes/metrics/template handlers, route allowlist, tests | adopted | Root-owned central wiring; must pass admin authorization and SSR tests before merge. |
| `apps/admin/src/notification_admin_adapter.rs` | strict notification query/projection/load outcome adapters, metrics, read/delete actions, and tests | adopted | Route-specific BFF behavior; no private payload projection; IDs and filters are bounded before service calls. |
| `apps/admin/src/session_auth.rs` | session-clear response header handling | adopted | Root-owned auth/session behavior; covered by auth-session gate. |
| `apps/admin/src/ssr.rs` | notification loader outcome, private HTML, query dispatch, tests | adopted | Root-owned SSR wiring; no sample data and no cacheable authenticated HTML. |
| `apps/admin/src/styles/index.css` | admin state/form styles | adopted | Presentation-only support for truthful page states. |
| `shared/rust/bff/src/dev_bypass.rs` | local design-bypass identity helper | adopted | Shared local-only UI identity; it cannot create a bearer token. |
| `shared/rust/client/src/lib.rs` | status-preserving POST helper and test | adopted | Required to distinguish durable notification enqueue responses. |
| `shared/rust/dioxus_ui/src/pages/admin_pages.rs` | admin module registry/dispatcher and tests | adopted | Central route wiring; source inventory remains authoritative. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/access_denied.rs` | denial rendering | adopted | Preserves fail-closed denial state. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/access_denied_panel.rs` | denial panel presentation | adopted | Presentation-only; no permission decisions. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/analytics.rs` | analytics unavailable/ready state presentation | adopted | No client-side analytics fallback. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/auth_redirect.rs` | auth redirect presentation | adopted | Fixed same-origin route only. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/dashboard.rs` | dashboard data-state presentation and tests | adopted | No fabricated operational records; data remains backend-owned. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/developer_portal.rs` | developer portal state presentation | adopted | No client-side key/permission logic. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/notifications.rs` | notification DTO validation, filtered list/metrics/read-delete/create state presentation and tests | adopted | Strict bounded DTOs and truthful backend-authorized mutation states. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/notifications_redirect.rs` | notifications redirect presentation | adopted | Fixed same-origin redirect only. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/settings.rs` | settings state presentation | adopted | No local settings authority. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/unauthorized.rs` | denial rendering | adopted | Preserves fail-closed denial state. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/wallet_access.rs` | unavailable wallet access presentation | adopted | No frontend permission/plan decision. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/wallet_redirect.rs` | wallet redirect presentation | adopted | Fixed same-origin redirect only. |
| `apps/admin/build.rs` | formatting-only newline/line-wrap changes | deferred | No behavioral value; avoid unrelated generated churn. |
| `apps/admin/public/dist/tailwind.css` | generated stylesheet replacement | deferred | Regenerate only after source CSS and route UI validation. |
| `apps/admin/src/analytics_admin_adapter.rs` | strict event-analytics read adapter | adopted | Uses the backend analytics projection with bounded decoding and explicit forbidden/unavailable/malformed states; it does not substitute for legacy market-ranking authority. |
| `apps/admin/src/commerce_adapter.rs` | wallet, credit, subscription, plan, and payment-link read adapters | adopted | Service-owned reads are bounded, deny-unknown decoded, parameter-validated, classified fail-closed, and projected without ownership, metadata, correlation, or mutation evidence. |
| `apps/admin/src/developer_portal_adapter.rs` | redacted developer inventory/stats adapter | adopted | Reads only backend summaries; API-key secrets and management controls never enter PageContext. |
| `apps/admin/src/settings_adapter.rs` | allowlisted settings read adapter | adopted | Read-only settings projection; no local defaults or manage authority. |
| `apps/admin/src/ssr.rs` | analytics, developer, settings, and commerce SSR loaders | adopted | Wires service-owned reads to typed page state; user-access and wallet mutations remain explicit unavailable surfaces. |
| `apps/backend/src/web/admin/developer_portal_handlers.rs` | redacted API-key list response | adopted | List responses expose summaries only; creation remains the only secret-bearing lifecycle point. |
| `apps/backend/src/web/admin/routes.rs` | exact admin audience and split read/manage permissions | adopted | Admin handlers reject API-key/non-admin audiences and keep read permissions separate from mutation permissions. |
| `apps/backend/migrations/core/20260727100000_remove_persisted_api_key_secret` | remove legacy persisted API-key secret column | adopted | Structural, guarded migration; creation-only secrets are no longer stored and the down migration cannot recover values. |
| `services/subscription/src/admin.rs` | bounded global access read | adopted | Supports the admin access inventory with bounded limit/offset and optional wallet filtering; mutations remain separately authorized. |
| `shared/rust/dioxus_ui/src/pages/admin_pages/{analytics,developer_portal,settings,wallet_credits,wallet_access,wallet_plans,wallet_wallets,payments}.rs` | typed redacted commerce/developer/settings/analytics page projections | adopted | Explicit ready/empty/forbidden/unavailable/malformed states, strict decoders, no sample operational data, and no mutation affordances on read-only surfaces. |
| `scripts/migration/test-admin-live-data.sh` | stale hard-coded test counts and wallet evidence names | adopted | Updated to the current focused counts 161/16/8/1/5/6 and the active commerce adapter evidence; full self-test runs with the shared target directory. |
| `scripts/migration/verify-frontend-live-data.ts` | one evidence anchor | adopted | Matches the bounded loaded-page unread label in the adopted notification UI. |

The remaining dirty-checkout changes outside this table are not part of the
admin migration baseline and remain untouched.
