# Admin dev/production parity audit — 2026-08-21

Reference production deployment: commit `5f0925b` (`admin.epsx.io`).

Audited development surface: `http://localhost:3001` on branch
`migration/dioxus-microservices` at `e38b88c9`, including the current dirty
worktree. This audit is intentionally local-only; it does not authorize or
perform a production deployment.

## Cross-cutting findings

1. **The generated browser runtime was missing from the running dev surface.**
   The page emitted `/runtime/epsx_browser_runtime_bootstrap.js?rev=2`, but the
   bootstrap, JS module, and WASM all returned `404`. That disabled wallet
   sign-in, logout, sidebar disclosure, theme switching, header menus, copy
   actions, and other progressive interactions. `cargo xtask browser-runtime
   build` restored all three assets to `200` locally.
2. **Authenticated pages use two different app shells.** Dashboard, Analytics,
   and Settings render page-owned `AdminShell`; most other pages use the
   BFF-owned `AuthLayout -> MainLayout`. Production has one `AuthLayout ->
   MainLayout` composition for every normal route. The dev split changes outer
   classes, height/overflow, header inputs, footer placement, notification
   state, and page padding.
3. **Signed-out routes can stack multiple authentication compositions.** The
   dispatcher emits `AuthPageOverlay` (and often `SkeletonPage`), while the BFF
   can add another generic `AuthGate` over a guest `MainLayout`. The result is
   route-dependent and differs from the single production auth overlay.
4. **The header is not fed the production layout data.** Production seeds the
   notification bell from the server layout. The Rust shell currently passes
   no initial notifications/unread count, so an authenticated page can show a
   different badge/menu from production even when its body matches.
5. **Many route bodies are truthful backend projections, but not ports of the
   production components.** This avoids fabricated data, which is correct, but
   it also replaces production layouts and controls rather than rendering the
   same UI with explicit unavailable/empty states.
6. **The production wallet parent layout was missing.** Every deployed
   `/wallet-management/**` page is nested under `Wallet Management Hub` plus a
   five-card dashboard. Development previously jumped straight into a
   route-specific heading, so Wallets, Access, and Credits all diverged before
   their own content began.
7. **Mobile header actions can overflow the viewport.** At 390x844 the sidebar
   correctly leaves the frame, but long breadcrumbs plus Theme and Wallet
   controls can clip the right edge. The production header also lacks a
   dedicated mobile navigation trigger, so this needs an explicit product
   decision rather than an unverified frontend-only menu.

## Remediation verified during this audit

- Browser runtime bootstrap, JavaScript, and WASM now return `200` locally.
- Normal routes use one BFF-owned `AuthLayout -> MainLayout` shell and one
  route-aware authentication composition. Dashboard, Analytics, and Settings
  no longer own a second shell.
- Dashboard uses the production `Operational Modules` hierarchy and 220px
  bento row contract. Settings uses the production heading and
  Reset/Synchronized control bar.
- Wallets, Access, and Credits now render under the production-shaped `Wallet
  Management Hub` and five-card dashboard. Only the three wallet-owned counts
  display values; subscription-owned metrics remain explicitly unavailable
  until the subscription service exposes an authoritative summary.
- The Wallets toolbar now preserves a closed URL query and forwards Search,
  Active/Disabled, 10/25/50 rows, and page offset to the wallet backend.
  Platform and alternative sort controls remain visible but disabled because
  the extracted wallet API does not own those fields yet.
- Payments now uses the production heading/subtitle without a duplicate body
  tab bar. `User Access` reads the existing backend
  `/api/admin/plans/user-access/list` projection and renders the production
  desktop-table/mobile-card hierarchy with refresh, paging, and wallet-detail
  navigation. Backend denial and unavailable responses remain distinct.
- Payments keeps the production action bar, four-card summary, filters, and
  inventory hierarchy visible in unavailable states. Unsupported revenue
  aggregates and CSV export remain explicitly unavailable rather than being
  derived from a bounded page. Payment Links now keeps its New Link/Refresh,
  filter, and inventory composition across ready, empty, denied, malformed,
  and unavailable states.
- Analytics now uses the route that is actually deployed in production: EPS
  growth ranking cards, rankings-access banner, Country/Sector filters, and
  pagination. The admin BFF reads the strict analytics ranking/filter contracts
  from the analytics service; unavailable and malformed responses keep that
  same composition, including the four-card ranking grid, without sample stocks.
  The previous operational-metrics dashboard came from an unused production
  component and is no longer mounted at `/analytics`.
- Audit Log now matches the production header/filter/table hierarchy. Category
  and refresh controls are server-functional; search/date/export remain visible
  but disabled until the analytics service exposes redacted contracts. Rows use
  native expandable details containing only the safe summary projection.
- Settings now renders the production Nodes/Signals/Vault/Optics panels from a
  closed `tab` query. The BFF projects only the eight allowlisted non-secret
  values, and the native controls submit typed text, boolean, and numeric values
  back through the backend-authorized mutation endpoint while preserving the
  active tab.
- Wallet Access, Plans, and Credits now keep the production hub and route-level
  workspace hierarchy in every backend state. Access and plan lifecycle forms
  remain backend-authorized; credit Overview/Grant/History navigation is
  present without inventing a history projection that the backend does not own.
- Wallet detail now stays under the shared hub and preserves the production
  Wallet Details, Identity & Access, Available Plans, Wallet Details, Active
  Subscription, and Assigned Plans hierarchy in failure states. Its BFF reads
  wallet-specific assignments and the plan catalog directly from their owning
  services; versioned assign/revoke and disable controls use existing audited
  backend mutations.
- Chat Support now renders the production stats, filter, inbox, conversation,
  assignment/status, and reply hierarchy. Counts and topics come from strict
  backend projections; inline streaming and attachment behavior still require
  dedicated contracts.
- Developer Portal now exposes the production Overview/API Keys/Docs/Usage
  workspaces and safe key lifecycle controls. Search, export, endpoint catalog,
  and usage timelines remain visibly disabled or unavailable where the backend
  has no typed projection.
- Notifications now uses the production Command Center, Overview/Send Signal
  navigation, action cards, Recent Broadcasts inventory, and Signal Generator
  composition. The targeted idempotent send path is functional; broadcast,
  classification, priority, action URL, asset URL, and analytics controls remain
  disabled until the backend owns matching idempotent contracts.
- News list/create/edit now use the deployed management cards and editor
  hierarchy. Server-authoritative create/edit/delete remain functional;
  transitions that require a full article projection stay visibly disabled on
  bounded list rows.
- Media keeps the deployed bucket toolbar, inventory grid, upload popover, and
  delete affordance across backend states. The live production route currently
  crashes with `S.map is not a function`; development renders a typed
  unavailable state instead of reproducing that runtime failure.
- Create API Key now uses the deployed Client/Contact/Description/Expiration/IP
  form. The BFF binds creation to the verified session wallet and normalizes the
  native date-time value to RFC3339, so no extra Wallet/Permissions controls
  are exposed solely to satisfy transport details.
- Wallet Disable and Plan Detail preserve their deployed duration/platform/
  reason/summary and editor-section hierarchy when reads fail. Ready mutations
  submit only backend-owned fields; plan update no longer asks the UI to
  round-trip the intentionally redacted merchant UUID.
- Authenticated desktop captures at 1440x1000 and mobile captures at 390x844
  completed without page errors. The generated WASM initializer still emits a
  duplicate deprecation warning, tracked separately from page parity.
- The source route inventory was rechecked against every deployed
  `apps/admin-frontend/app/**/page.tsx`. `/auth`, `/notifications`, and
  `/wallet-management` retain their fixed server redirects; `/access-denied`
  and `/unauthorized` use the deployed red-shield status composition. Denial
  query parameters remain intentionally non-authoritative in development.

## Route-by-route gap matrix

| Route | Severity | Current dev gap relative to production |
| --- | --- | --- |
| `/` | Medium | Uses the shared BFF shell plus the production `Operational Modules` hierarchy and 220px bento row. Health/latency/uptime and recent-wallet telemetry without typed backend projections remain explicitly unavailable. |
| `/dashboard` | Aligned | Production returns `404`; the Rust dispatcher intentionally keeps it not found. |
| `/wallet-management` | Medium | Redirect behavior must remain identical to the production wallet landing route and preserve history/query semantics. |
| `/wallet-management/wallets` | High | Parent hub, compact header, URL-persistent search/status/page-size controls, and backend pagination are now present. Platform/alternative sort remain disabled because the wallet service has no typed contract for them; responsive production row/card fields, metadata edit, and re-enable still need porting. |
| `/wallet-management/[address]` | High | Production-shaped hub/detail/access hierarchy and wallet-specific plan/assignment projections are present, with versioned assign/revoke and disable controls. Metadata editing and active-subscription detail remain disabled or unavailable until safe typed mutation/read projections are wired. |
| `/wallet-management/wallets/[address]/disable` | Medium | Dedicated duration/platform/reason/additional-action/summary composition is present in ready and failure states. The current wallet mutation is global and indefinite, so unsupported duration/platform/subscription/notification controls remain disabled while the versioned reason mutation stays functional. |
| `/wallet-management/access` | Medium | Production-shaped hub, access summary, assignment inventory, and lifecycle controls are present. Row-rich live-data validation and the production client-side drawer behavior remain. |
| `/wallet-management/access/plans` | Medium | Production-shaped plan inventory and backend-authorized create/edit lifecycle are present. Client-side drawers and live returned-row visual validation remain. |
| `/wallet-management/access/plans/[planId]` | Medium | The route keeps the hub, stats/action bars, and all deployed editor sections. Name/description/amount/currency/chain/interval/status update through the versioned backend contract; category, groups, limits, feature toggles, pricing-page features, permissions, and delete remain disabled because those fields/contracts are not exposed. Merchant ownership is preserved server-side and is no longer requested from the UI on update. |
| `/wallet-management/credits` | Medium | Overview, Grant Credits, and Credit History workspaces are present under the shared hub. History remains explicitly unavailable until a typed backend history projection exists. |
| `/payments?tab=payments` | Medium | The authoritative list, management toolbar, summary-card hierarchy, filters, and responsive inventory are present. Revenue/success/pending/today aggregates and CSV export remain unavailable until the payment backend exposes authoritative summary/export contracts. |
| `/payments?tab=user-access` | Medium | The authoritative production backend contract, responsive table/cards, refresh, pagination, and wallet navigation are now connected. A live signed-in capture with returned rows is still required; the isolated bypass audit correctly renders the backend-unavailable state. |
| `/payments?tab=payment-links` | Medium | Backend create/list/disable plus the production-shaped action, filter, and inventory hierarchy are present. Context/status filters are visible but disabled because the current redacted link endpoint does not expose those fields or a query contract; the create flow remains inline rather than modal. |
| `/chat` | Medium | Production stats, filters, inbox cards, paging, and detail split are present using strict conversation/stats/topics projections. Inline stream updates, attachments, and richer client-only filtering remain. |
| `/chat/[id]` | Medium | Conversation, assignment/status, read, and reply server forms now follow the production hierarchy. Streaming composer state and file upload remain unimplemented. |
| `/news` | Low | Deployed compact header, All/Draft/Published tabs, count, article cards, paging, edit, and versioned delete are present. Pin/publish list-row shortcuts remain disabled because the bounded summary omits the full article required by the transition mutation. |
| `/news/create`, `/news/[id]/edit` | Low | Deployed sticky action bar, cover image area, title/slug/summary/tags fields, markdown toolbar, long-form editor, and server-authoritative mutation feedback are present. Client-only rich editing shortcuts remain intentionally non-operative. |
| `/media` | Low | Deployed bucket toolbar, upload popover, grid cards, empty/failure inventory, and delete are present. Chat/Notifications buckets, search/view switching, object URL open/copy remain disabled because the media backend exposes only news/public keys and redacts object URLs. Live production currently crashes before rendering this browser. |
| `/analytics` | Low | The actual deployed EPS-ranking workspace is mounted: access banner, Country/Sector filters, responsive four-card ranking grid, and paging. Rankings/filter responses come from the analytics service; unavailable/malformed states preserve the grid without inventing stocks. Watchlist state and richer package naming remain unavailable until their owner-scoped contracts are connected. |
| `/audit-log` | Medium | Header, filter card, category/refresh controls, expandable redacted rows, and cursor paging are present. Search/date/export are intentionally disabled until the analytics backend owns safe query/export contracts; actor/target identity remains redacted. |
| `/developer-portal` | Medium | Production Overview/API Keys/Docs/Usage workspaces and key lifecycle controls are present. Search/export, endpoint catalog, and usage timeline remain disabled or unavailable without backend-owned contracts. |
| `/developer-portal/api-keys/create` | Low | Authenticated route uses the deployed form/shell and secret-once result. Creation is bound to the verified session wallet server-side; native date-time input is normalized before the strict backend mutation. Unauthenticated access remains fail-closed. |
| `/notifications/manage` | Medium | Command Center, workspace tabs, Synchronize/Analytics cards, filters, metrics, and Recent Broadcasts inventory now match the production hierarchy. Analytics remains disabled because the dispatcher exposes no route. |
| `/notifications/create` | Medium | Signal Generator and its targeting/classification/priority/message/action/asset hierarchy are present; the targeted idempotent send path is functional. Broadcast and fields absent from that backend contract remain visibly disabled. |
| `/notifications` | Aligned | Production and development both issue a fixed temporary redirect to `/notifications/manage`; request query values cannot choose or enter the target. |
| `/settings` | Medium | `Settings Nexus`, Reset/Synchronized controls, and Nodes/Signals/Vault/Optics panels are present. A closed query selects the active panel, and allowlisted values use typed backend forms. Production-style client dirty-state batching/confirmation remains to be matched; arbitrary backend configuration stays excluded. |
| `/policies` | Aligned | Production returns `404`; dev intentionally returns not found. |
| `/access-denied`, `/unauthorized` | Low | Both are shell-free and preserve the deployed red-shield, heading, reason, Error Details, and action hierarchy. Development intentionally refuses to present query-controlled route/permission/detail values as authoritative backend denial facts. |
| `/auth` | Aligned | The deployed route is a fixed redirect to `/`, not a wallet-method page. The BFF returns the same fixed temporary redirect and ignores query-controlled return targets; the wallet selector remains owned by the root auth gate. |

## Implementation order

1. Restore and continuously verify the generated browser runtime in local dev.
2. Collapse normal routes onto one BFF-owned production-shaped shell and one
   auth overlay.
3. Match high-traffic bodies first: Wallets, Payments, Dashboard, Analytics,
   Audit Log, and Settings.
4. Complete the remaining operational pages: Access/Plans/Credits, Chat,
   Developer Portal, Notifications, Media, and News.
5. Re-capture authenticated desktop and mobile states route by route, including
   empty, forbidden, unavailable, and mutation outcomes; only then consider a
   separately authorized production deploy.
