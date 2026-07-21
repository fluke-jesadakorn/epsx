# A7.0 frontend live-data and interaction audit

Status: **integrity PASS target; production readiness STOP**. This is an evidence and execution contract only. It does not authorize frontend/runtime changes, backend policy duplication, production access, or deployment. The machine-readable source of truth is [`contracts/frontend-live-data.json`](contracts/frontend-live-data.json).

## Baseline and audit result

The source baseline is pinned to `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. The route set is the checked 28-route frontend application in [`contracts/routes.json`](contracts/routes.json); the gate rejects missing, additional, duplicated, or reassigned routes.

The current result is deliberately not a completion claim:

- **3 aligned:** `/about`, `/access-denied`, and `/manual`, after focused Rust and real localhost Playwright proof for content/escaping, semantics, responsive layout, and keyboard navigation.
- **7 partial:** `/contact`, `/developer/docs`, `/news`, `/news/:slug`, `/offline`, `/privacy`, `/terms`.
- **18 blocked:** every other route in the contract.

The B7.1 proof closes `/access-denied` only. `/offline` now has a public accessible retry button and proves an already-rendered page survives an offline transition and reloads after reconnection, but fresh disconnected/service-worker cache delivery is absent. `/privacy` and `/terms` have responsive, hierarchy, and keyboard proof, yet their Google Sign-in/OIDC legal copy is not approved for the canonical wallet/SIWE flow. Terms also targets `/api/public/subscribe`, for which no matching email-subscription handler exists.

The focused B1 `/about` proof removes the prior invented team biographies, company statistics, roadmap, values, and hiring claims. It accepts only the pinned hero, DataTech lifecycle, benefits, mission, and vision copy in the pinned order, carries the exact title/description/keywords through an escaped optional metadata field, and proves the middleware-required route through an ephemeral localhost RS256/JWKS session. The fixture binds only loopback, uses a five-minute token and temporary key, and stops both processes after the mobile/desktop run; it adds no authentication bypass.

The focused B5 `/manual` proof accepts the pinned 35-feature catalog verbatim, verifies every referenced WebP through the localhost BFF, and proves the responsive category index, route links, screenshot fallback/dialog, and focus behavior at mobile and desktop viewports. It adds no permission, plan, ranking, feature-flag, or subscription decisions to the frontend.

The most important distinction is loader presence versus a production-usable flow. `/analytics`, `/plans`, `/portfolio`, and dynamic payment have SSR fetch hooks whose `data_*` payload is not consumed by the page. Dashboard, news, and developer usage consume in-process values that can still be canned. Other pages deserialize live data but silently replace failures with samples, zeroes, or empty arrays. None of those cases is live-data readiness.

## Exact route anchors and observed loader path

Anchors are literal substrings. Source anchors are verified with `git show` at the pinned commit; target and loader anchors are verified against the current worktree. The JSON contract contains the second source/target anchor, payload classification, auth/owner boundary, all interaction categories, four async states, hydration status, dependencies, and route blocker text.

| Route | Pinned source anchor | Current target anchor | Observed loader | State |
|---|---|---|---|---|
| `/` | `apps/frontend/app/page.tsx :: export default async function HomePage` | `pages/home.rs :: pub fn render(ctx: &PageContext)` | none; fixed performers/plans/news | blocked |
| `/about` | `apps/frontend/app/about/page.tsx :: title: 'About Us - EPSX Analytics Platform'` | `pages/about.rs :: meta.title = TITLE.to_string();` | accepted pinned static content only; authenticated responsive/metadata/keyboard proof | aligned |
| `/access-denied` | `apps/frontend/app/access-denied/page.tsx :: export default function AccessDeniedPage` | `pages/access_denied.rs :: .query_param("reason")` | bounded/control-filtered decoded text, escaped output, safe links, mobile/desktop keyboard proof | aligned |
| `/account` | `apps/frontend/app/account/page.tsx :: export default async function AccountPage()` | `pages/account.rs :: ctx.params.get("data_account")` | `GET /api/v1/account`, fallback `/api/v1/auth/me` | blocked |
| `/account/credits` | `apps/frontend/app/account/credits/page.tsx :: export default function CreditsPage()` | `pages/account_credits.rs :: ctx.params.get("data_credits")` | `GET /api/v1/credits` | blocked |
| `/analytics` | `apps/frontend/app/analytics/page.tsx :: export default async function AnalyticsPage` | `pages/analytics.rs :: fn sample_rankings()` | `GET /api/v1/analytics/summary`, unused | blocked |
| `/auth` | `apps/frontend/app/auth/page.tsx :: <WalletConnectAuth` | `pages/auth_page.rs :: data_connect_wallet: Some(true)` | browser wallet auth bridge | blocked |
| `/chat` | `apps/frontend/app/chat/page.tsx :: export default async function ChatPage()` | `pages/chat.rs :: sample_conversations()` | none | blocked |
| `/chat/:id` | `apps/frontend/app/chat/[id]/page.tsx :: useChatSSE({ enabled:` | `pages/chat_conversation.rs :: sample_conversations()` | none | blocked |
| `/chat/history` | `apps/frontend/app/chat/history/page.tsx :: const [loading, setLoading] = useState(true)` | `pages/chat_history.rs :: sample_conversations()` | none | blocked |
| `/contact` | `apps/frontend/app/contact/page.tsx :: CopyEmailBtn` | `pages/contact.rs :: action: "/api/v1/contact"` | native `POST /api/v1/contact` | partial |
| `/dashboard` | `apps/frontend/app/dashboard/page.tsx :: getSessionFromWeb3()` | `pages/dashboard.rs :: DashboardMockStats::default_mock` | in-process `dashboard_data_internal` | blocked |
| `/developer` | `apps/frontend/app/developer/page.tsx :: APIKeyManager` | `pages/developer.rs :: ctx.params.get("data_developer")` | `GET /api/v1/developer` | blocked |
| `/developer/docs` | `apps/frontend/app/developer/docs/page.tsx :: <ApiDocs />` | `pages/developer.rs :: cached_endpoint_categories()` | `GET /api/v1/developer/docs`, unused | partial |
| `/developer/usage` | `apps/frontend/app/developer/usage/page.tsx :: DeveloperUsagePage` | `pages/developer.rs :: data_developer_usage` | in-process `developer_usage_value` | blocked |
| `/manual` | `apps/frontend/app/manual/page.tsx :: Complete guide to all platform features` | `pages/manual.rs :: const FEATURES:` | accepted 35-feature static catalog; responsive landmarks, links, screenshot fallback/dialog, and mobile/desktop keyboard proof | aligned |
| `/news` | `apps/frontend/app/news/page.tsx :: searchParams: Promise<{ page?: string }>` | `pages/news.rs :: .unwrap_or_else(default_posts)` | in-process `news_list_value` | partial |
| `/news/:slug` | `apps/frontend/app/news/[slug]/page.tsx :: generateMetadata` | `pages/news_detail.rs :: data_news_post` | in-process `news_post_value` | partial |
| `/notifications` | `apps/frontend/app/notifications/page.tsx :: page: parseInt(params.page ?? '1')` | `pages/notifications.rs :: data_notifications` | `GET /api/v1/notification/list` | blocked |
| `/offline` | `apps/frontend/app/offline/page.tsx :: window.location.reload()` | `pages/offline.rs :: data-offline-reload` | public CSP-compatible retry and reconnect proof; fresh disconnected cache delivery open | partial |
| `/payment` | `apps/frontend/app/payment/page.tsx :: PaymentPage` | `pages/payment.rs :: /api/v1/payments/confirm` | native confirm only | blocked |
| `/payment/:type/:id` | `apps/frontend/app/payment/[type]/[id]/page.tsx :: Fetch plans on server` | `pages/payment.rs :: pub fn render_dynamic` | `GET /api/v1/payment/{id}`, intent-only and unused | blocked |
| `/permissions` | `apps/frontend/app/permissions/page.tsx :: usePermissionsPage({ base })` | `pages/permissions.rs :: let features = vec![` | none | blocked |
| `/plans` | `apps/frontend/app/plans/page.tsx :: PlansPage` | `pages/plans.rs :: let plans = default_plans();` | three plan endpoints, unused | blocked |
| `/portfolio` | `apps/frontend/app/portfolio/page.tsx :: <RequireSignIn` | `pages/portfolio.rs :: const WATCHED_STOCKS:` | two owner portfolio endpoints, unused | blocked |
| `/privacy` | `apps/frontend/app/privacy/page.tsx :: 1. Information We Collect` | `pages/privacy.rs :: const LAST_UPDATED:` | responsive h1/h2 landmarks and keyboard proof pass; wallet/SIWE legal approval open | partial |
| `/profile` | `apps/frontend/app/profile/page.tsx :: <WalletProfileClient wallet={session.user} />` | `pages/profile.rs :: let mut connected = use_signal(|| true);` | none | blocked |
| `/terms` | `apps/frontend/app/terms/page.tsx :: fetch('/api/public/subscribe'` | `pages/terms.rs :: action: "/api/public/subscribe"` | native subscription POST | partial |

Target paths in the table are abbreviated for readability; the verifier locks the full path `shared/rust/dioxus_ui/src/pages/<file>`. Loader anchors are in `apps/frontend/src/ssr.rs` where applicable.

## State, interaction, ownership, and hydration findings

Every route record explicitly inventories:

- static/sample payloads and placeholder/skeleton-only surfaces;
- producer kind, backend routes/loaders, and whether the page actually consumes the payload;
- public/optional/required authentication and the resource owner key;
- forms, pagination, search, wallet, keyboard, and other controls, including “missing” as an explicit finding;
- loading, empty, error, and retry as `present`, `missing`, or `not-applicable`;
- browser hydration need and current implementation state.

The cross-cutting stop conditions are:

1. Sample or zero-value fallbacks commonly turn dependency failures into plausible-looking success pages.
2. SSR-only rendering strips ordinary Dioxus event closures unless a proven hydration or raw browser bridge exists. A `use_signal` or `onclick` in source is not interaction proof.
3. Authenticated views need a verified session and owner-scoped backend queries. Dynamic identifiers must not expose foreign resources; use the backend’s chosen non-leaking result.
4. Frontend forms must surface pending, validation, success, error, and retry states. Local vector/signal mutations are not durable mutations.
5. Pagination and search must be URL- or backend-contract driven so refresh, back/forward, keyboard submission, and SSR all preserve state.
6. Wallet and payment success must come from A1/A6 verified state, never a cookie, query amount, local step, or submitted transaction alone.

## Dependency boundary

- **A1:** session/wallet ownership, safe return targets, real signature flow, rotation, revocation, and logout. Auth, identity, chat, owner views, and checkout cannot close before it.
- **A4:** permission, plan eligibility, ranking offset, feature flags, and entitlement decisions. The frontend may render backend decisions but must not recreate them.
- **A5:** route prefix, body/envelope/status/error, cookie/bearer forwarding, correlation, timeout, and retry alignment. A page adapter is not stable until this contract is locked.
- **A6:** intent, wallet submission, chain receipt/finality, status polling, idempotency, escrow, and activation. `/plans` checkout and both payment routes remain stopped.

## Small executable batches

Close one batch at a time; do not mark a route aligned merely because another route in its batch passes.

1. **B1 public content:** `/`, `/about`, `/news`, `/news/:slug`. `/about` is aligned from its accepted pinned static source. Establish canonical content/pricing/ranking producers for the remaining routes, remove invented fallbacks, and prove URL search/pagination plus article not-found/error/retry.
2. **B2 identity:** `/auth`, `/account`, `/account/credits`, `/profile`. Finish A1, add owner adapters, payment/credit pagination, and durable profile/preference mutations.
3. **B3 insights:** `/analytics`, `/dashboard`, `/portfolio`, `/permissions`. Consume live payloads, implement backend query/pagination/export/watchlist contracts, and render A4 decisions without policy duplication.
4. **B4 communication:** `/chat`, `/chat/:id`, `/chat/history`, `/notifications`. Add owner-scoped list/detail/create/send/resolve/read/preference APIs, SSE reconnect, and keyboard/pagination/error behavior.
5. **B5 developer:** `/developer`, `/developer/docs`, `/developer/usage`, `/manual`. Add secret-once API-key mutations, owner usage/range pagination, and choose generated-versus-live documentation as one canonical source. `/manual` is aligned as the accepted static 35-feature catalog with responsive, keyboard, screenshot-control, fallback, and accessibility browser proof.
6. **B6 commerce/support:** `/plans`, `/payment`, `/payment/:type/:id`, `/contact`. Consume canonical plans, complete A6, forbid client-owned price/eligibility decisions, and prove contact validation/rate-limit/feedback.
7. **B7 policy/fallback:** `/access-denied`, `/offline`, `/privacy`, `/terms`. `/access-denied` is aligned. Finish service-worker/cache delivery for a fresh offline navigation, obtain product/legal approval for wallet/SIWE privacy and terms copy, and implement a real email-subscription endpoint plus pending/success/error/retry feedback.

For each route, add a focused Rust unit/adapter test and a browser fixture that covers the contract’s applicable state and interaction fields. After a batch is implemented, change only evidence-backed route fields and anchors; the readiness gate will continue to exit `3` until all 28 route statuses are `aligned` and their blocker arrays are empty.

## Acceptance commands

Audit integrity and deterministic report:

```sh
./scripts/migration/verify-frontend-live-data.sh --mode integrity
./scripts/migration/verify-frontend-live-data.sh --mode emit
./scripts/migration/test-frontend-live-data.sh
bunx playwright test e2e/frontend/policy-fallback-runtime.spec.ts --project=frontend --workers=1
bunx playwright test e2e/frontend/manual-runtime.spec.ts --project=frontend --workers=1
./scripts/migration/run-about-runtime-proof.sh
```

Readiness is expected to stop today:

```sh
./scripts/migration/verify-frontend-live-data.sh --mode readiness
# expected exit 3 while any route is partial or blocked
```

Run the repository gates after each executable batch:

```sh
./scripts/migration/verify-route-inventory.sh
./scripts/migration/verify-contract-fixtures.sh
cargo test -p epsx-frontend
cargo test -p epsx-dioxus-ui --lib
bun test:e2e --project=frontend
```

The final A7 acceptance command from the production-readiness plan remains:

```sh
./scripts/migration/verify-no-frontend-sample-data.sh
```

That final verifier does not exist in this audit slice and must be implemented only with the route batches, after intentional static content is distinguished from executable sample business data. Integrity PASS means the audit is internally consistent; it does not mean the frontend is production ready.
