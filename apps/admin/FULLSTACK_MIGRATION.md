# Admin Fullstack migration coverage

All native Admin UI routes now use Dioxus Fullstack. This ledger separates implemented coverage from final release verification.

## Implemented foundation

- Native BFF construction lives in `src/lib.rs`; `bff-admin` keeps its name, environment, verified sessions, REST routes, and middleware.
- `/_server/admin/analytics` has a typed query/result and an app-specific provider. It verifies the admin session and requests backend authorization before fetching rankings. Frontend and Pay endpoints are excluded by the shared registry.
- `HydratedAdminAnalytics` uses an SSR future and Dioxus state, query navigation, pending/error/retry handling, and response ordering. It shares the existing ranking presentation and Dioxus form callbacks.
- The Dioxus build entry serves all UI routes listed below through FullstackState with `/wasm` and `/assets`; it uses a Dioxus-owned shell and canonical CSS. Native `bff-admin` now uses the Fullstack application and verifies packaged assets; the legacy SSR module is test-only.
- Audit uses a typed category/cursor query and redacted list, preserving backend authorization and cursor validation. The ready-state presentation is shared with the old SSR adapter; category and pagination links use Dioxus events in the hydrated tree.

## Route and interaction inventory

The native router, admin dispatcher, and standalone renderers are authoritative; the old Dioxus enum does not cover them all. The dispatcher also accepts a single `/admin` prefix.

| Route family | Existing behavior to preserve | Hydrated status |
| --- | --- | --- |
| `/`, `/index`, `/dashboard` | Dashboard user status and analytics; independent dependency failures | Typed data and original body extracted; root/index/dashboard typed; root session selects dashboard or Dioxus sign-in |
| `/analytics` | Rankings, filters, sorting, page size and page navigation | Typed provider and reactive component implemented |
| `/audit-log` | Category/cursor filters and results | Typed provider, shared presentation, Dioxus navigation, Fullstack SSR entry; artifact/runtime browser validation passed |
| `/chat`, `/chat/:id` | List/detail, reply, assign, close/reopen | Typed query/read/commands and original body; signed-in browser reply/status/assign/read passed |
| `/developer-portal`, `/developer-portal/api-keys/create` | Key listing, create, one-time secret, revoke, expiration | Typed provider/UI; actual browser create/secret-once/expiry/revoke passed |
| `/media` | Bucket selection, upload, delete | Typed provider/UI integrated; parent browser upload/delete/filters passed |
| `/news`, `/news/create`, `/news/:id/edit` | Pagination/status, editor, image upload, lifecycle, delete | Parent-owned typed provider/UI; final browser acceptance in progress |
| `/notifications` | Redirect to manage | Preserve HTTP redirect |
| `/notifications/manage`, `/notifications/create` | Pagination, metrics, send, mark-read, delete, idempotency | Typed query/provider/events; signed-in browser read/delete/send/reset/repeated-send passed |
| `/payments` | Tabs, payer/status filters, pagination, access and links | Typed query/read/cancel/link create/disable; signed-in browser filters/cancel/link create/disable/access passed |
| `/settings` | Settings display/update/reset | Typed read/update/reset, SSR idempotency identities, pending/result/retry and tabs implemented |
| `/policies` | Intentionally unavailable route | Preserve original HTTP 404; no policy UI/backend existed |
| `/auth`, `/unauthorized`, `/access-denied` | Auth and denied states, HTTP redirects | Pay agent-owned SIWE/session/typed logout integrated on /auth; root/denied/notfound typed wrappers implemented |
| `/wallet-management` | Redirect | Preserve HTTP redirect |
| `/wallet-management/wallets`, `/wallet-management/:address`, `/wallet-management/wallets/:address/disable` | List, detail, disable confirmation | Pay-agent typed provider/UI; wallet six-group browser audit passed |
| `/wallet-management/credits`, `/wallet-management/access` | Credit/access state | Pay-agent typed provider/UI; wallet six-group browser audit passed |
| `/wallet-management/access/plans`, `/wallet-management/access/plans/:planId` | Plans, editor | Pay-agent typed provider/UI; wallet six-group browser audit passed |
| `/plans`, `/plans/:id` | Standalone native plan catalog/editor | Typed read/save, original form fields, native authorization/origin checks and pending/result states implemented |
| `/payments/epsx`, `/payments/epsx/:id` | Standalone native order listing/detail | Typed scoped provider and reactive refresh/offset/detail navigation implemented; browser validation passed |
| `/pay/escrows`, `/pay/escrows/:id` | Standalone native escrow management | Pay agent-owned typed provider/UI integrated; browser validation passed |
| `/pay/merchant-escrows`, `/pay/merchant-escrows/:id` | Standalone merchant controls, resolve/confirm | Pay agent-owned typed provider/UI integrated; browser validation passed |

## Release acceptance

Matching server/WASM release assets were packaged and tested locally for all three BFFs. The Admin package serves SSR, hashed assets and correct unknown-route status, with foreign-app server functions unmounted. Refresh-cookie recovery, route aliases and malformed-route tests passed. Full evidence is recorded in `docs/migration/DIOXUS_FULLSTACK.md` and `target/dioxus-migration/package/report.json`. No production deployment was performed.

## Validation performed

- Admin scoped provider tests and real FullstackState audit SSR test passed (4 tests); the SSR request resolves its provider once and includes the original redacted row and cursor link.
- All Admin page tests passed (145 tests), including exact typed-versus-legacy Audit markup and Settings toggle successful controls.
- Seven scoped provider/Fullstack SSR tests passed before Catalog/Orders expansion, including Settings origin/auth rejection and SSR identity serialization without mutation. Catalog/Orders coverage passed in the final 191-test Admin suite.
- Dioxus native + WASM builds passed. Correct command: `dx build --package epsx-admin --bin dx-admin --platform web --no-default-features`. Do not add `--features web`, which enables web on the server as well and drops server-future hydration data.
- Actual localhost browser found and then verified the fix for server/client feature pollution. Account popover, category and Settings tab navigation now update via Dioxus without document reload; typed unauthenticated response retains UI state. Upstream was deliberately an unused localhost port, so no production data or mutations were involved.
- Signed-in browser proofs now cover the operational families below; Pay-agent reports cover auth, escrow and wallets. Native entry has been switched locally and final release packaging and startup checks passed.

- Signed-in ephemeral RSA/JWKS fixture verified Settings text and boolean mutations: exactly one backend write per action, fresh idempotency key after success, one SSR read and one reread per mutation. Account menu stays expanded through saves; console errors empty.
- Signed-in Catalog edit/save verified; browser direct-load testing caught and corrected select defaults. A narrowly escaped textarea adapter avoids Dioxus 0.7 raw-text hydration comments; injection-shaped literal text and browser hydration verification passed for this exception.
- `apps/admin/fixtures/fullstack_browser.py` runs isolated mock services and ephemeral signed cookies, with no production credentials or network dependencies. Use a unique copied executable when macOS caches a replaced bundle inode and terminates it with SIGKILL.

## Final cutover changes

All native UI fallback requests use FullstackState. A single `/admin` prefix has typed Router aliases preserving the URL, with a generated alias audit. Canonical notification and wallet redirects remain HTTP redirects. REST, auth API, escrow proxy, and health contracts are retained. The native router no longer registers the custom browser runtime. Legacy page renderers are compiled only for compatibility tests.

Admin shell events, sidebar child links, header actions, and breadcrumb navigation use Dioxus callbacks. Root theme state persists across route changes; its only browser adapter reads/writes the existing theme preference in localStorage. No production routes or deployment were changed.

- 27 scoped provider tests passed, including new Notifications/Payments authentication and cross-origin mutation rejection.
- Actual Developer browser: create yields a secret only in the mutation result, inventory omits it; expiration updates twice with distinct successful-command keys; revoke removes lifecycle controls; no console errors.
- Actual Settings browser: two identical successful text commands produce two writes with different idempotency keys; open account popover remains open throughout.
- Browser fixture snapshots both binary and public assets, allowing later builds without corrupting the running proof.
- Eleven native routing tests passed, covering malformed/unknown 404s, typed recovery serialization, canonical auth-clear cookies, redirects, protected REST shapes and method behavior. Prefix aliases, denied 403 and unknown 404 also passed Fullstack integration tests.
- Operational browser: Notification mark-read/delete/send, same-route create reset and identical repeated send passed; two sends use distinct successful-command keys. Payment limit filtering/cancellation, link creation/disable, and User Access passed against synthetic local data. Account menu state survives mutations and light theme survives route navigation; console errors were empty.
- Pay-agent recovery browser audit passed after the corrected Router key. Final native release package acceptance passed.
