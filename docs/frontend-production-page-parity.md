# Frontend production pages and Developer API

Local update, 8 September 2026. Production reference: <https://epsx.io/>.

The How it works page and its links have been removed. `/manual` returns a 307 to `/analytics` for old bookmarks. The shared compatibility renderer also opens Explore. The Home introduction, rankings, plans and news remain.

## Navigation and pages

The public header now has Market, Developer and Company menus, matching the production information architecture. All three groups appear in mobile navigation. Workspace navigation exposes API keys, API usage and API documentation directly, alongside the existing product pages. Its middle section scrolls on short screens so every destination remains reachable. Account subsections remain available through Account.

| Group | Routes carried forward | Local signed-out behavior |
| --- | --- | --- |
| Market | `/`, `/analytics`, `/portfolio`, `/plans`, `/news` | Render; account-dependent controls retain sign-in states |
| Developer | `/developer`, `/developer/usage`, `/developer/docs` | Keys/usage require a verified account; documentation is readable |
| Company | `/about`, `/contact`, `/chat` | About/contact retain the existing production-compatible sign-in redirects; Support offers sign-in |
| Account | `/account`, `/dashboard`, `/profile`, `/permissions`, `/account/credits`, `/notifications`, `/payment` | Existing session gates and owner-scoped loaders |
| Detail pages | `/news/:slug`, `/portfolio/:address`, `/payment/:type/:id`, `/chat/:id`, `/chat/history` | News loads by slug; address redirects to Saved companies; private records require a session |
| Legal/auth/system | `/privacy`, `/terms`, `/auth`, `/offline`, `/access-denied`, unknown routes | Existing content, recovery and 404 behavior |
| Aliases | `/index`, `/pricing`, `/manual` | Home, Plans redirect, Explore redirect respectively |

The route implementations already existed before this update. This change restores discoverability across the frontend and fills specific Developer detail gaps; it is not a claim that every historical backend feature has been reimplemented.

## Developer details restored

- API key cards display creation, expiration and last-use timestamps returned by the backend.
- API usage includes per-key lifetime request counts, separately labeled from the selected 7/30/90-day reporting period. The request bars now have measurable heights and retain zero values.
- API documentation includes an operation index, quick start, cURL/server JavaScript/Python examples, authentication guidance, published parameters, request-body metadata and response descriptions. Examples select a registered read operation and contain environment-variable placeholders.
- The OpenAPI link and existing Try It action use the existing frontend BFF. The backend registry remains the source of operations, scopes and API-key callability. This update does not grant API-key access to session-only operations.

| Production API references | Current implementation |
| --- | --- |
| `/api/developer-portal/my-keys` and key revocation | Backend owner-scoped list/create/revoke; BFF `/api/v1/developer/keys` and `/keys/:id/revoke` |
| `/api/developer-portal/my-plans`, `/stats`, `/usage-history`, `/top-endpoints` | Retained backend routes; the new pages consume the consolidated `/api/developer-portal/overview` through BFF overview/usage adapters |
| Analytics rankings, countries, sectors, filters | Existing backend analytics routes and validated frontend projections |
| Account/profile/access, credit and payment history | Existing owner-scoped account and payment adapters |
| Watchlist/groups, news, notifications, chat | Existing BFF adapters, runtime actions and server-rendered pages |

## Evidence and limits

`e2e/enterprise/prod_inventory.py` fetched 26 known production routes and 61 script URLs published by those responses. Captures and endpoint references are in `target/prod-page-parity/production/inventory.json`. No credentials or production mutations were used. References found in a shared JavaScript bundle include old/admin/library entries: their presence does not prove a currently callable public API.

The local HTTP audit covers 33 URL cases, including detail paths, aliases, sign-in redirects and a 404. `target/prod-page-parity/local-routes.json` records status, location, a single main landmark, the current frontend stylesheet, absence of guide links and absence of fixture labels. The preview uses the existing real local backend on port 8080.

API smoke results are in `target/prod-page-parity/api-checks.json`. The backend currently publishes nine operations in its Developer OpenAPI registry. Anonymous protected reads return 401. Live wallet signing, real customer key creation/revocation, payments and authenticated production records were not exercised; the existing UI/BFF test suite covers owner validation and mutation contracts.

Validation results are recorded in `target/prod-page-parity/verification.json`. Production deployment, tunnel configuration and production processes were not changed.
