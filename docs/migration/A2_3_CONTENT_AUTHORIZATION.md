# A2.3b content service authorization

This bounded slice makes the content service consume `epsx-service-auth`
directly. It installs an exact method/path authorization boundary before the
existing database and filesystem handlers without changing runtime DDL, file
watching, synchronization, rendering, or other content-domain behavior.

## Enforced boundary

- `GET`/`HEAD /health` is anonymously reachable.
- The published-page render, theme, block-schema, navigation, site, news,
  plans, rankings, and public-portfolio GET shapes are explicit anonymous
  allowlists. Dynamic routes accept exactly the documented number of non-empty
  path segments; encoded or additional segments are rejected.
- Page editor/list/create/update/publish and theme create/update routes require
  a verified token with the exact admin audience and a canonical grant matching
  `admin:content:manage`. The shared backend grammar therefore accepts the
  literal grant and its valid resource/domain wildcards, but never arbitrary
  wildcard placement.
- Inbound identity, wallet, role, scope, and permission headers are stripped.
  Public requests also have `Authorization` removed before their handler.
- Unknown paths and unapproved methods return 404 before database or filesystem
  handlers.

The edit start, edit commit, and edit-session list routes deliberately return
404 before their handlers even for authorized administrators. Their legacy
model stores a caller-selected UUID `user_id`, while the canonical subject is a
wallet address. Enabling those handlers with only a permission check would
preserve identity forgery and omit session-ownership enforcement. They remain
blocked until a server-derived editor identity mapping and ownership contract
exist.

## Runtime configuration and verification

Content startup builds its verifier before connecting to PostgreSQL or reading
content files. The JWKS client disables redirects and has five-second connect,
fifteen-second total, and bounded idle-pool timeouts. `EPSX_ENV=production`
requires HTTPS, non-local issuer and JWKS URLs through the shared verifier.

```bash
cargo test -p epsx-content --no-fail-fast --locked
cargo check -p epsx-content --all-targets --locked
./scripts/migration/verify-service-authorization.sh
./scripts/migration/verify-contract-fixtures.sh
./scripts/migration/verify-permission-grammar.sh
./scripts/migration/test-permission-grammar.sh
git diff --check
```

Hermetic router tests use a fake token verifier and downstream handler, with no
PostgreSQL or filesystem access. They cover the complete anonymous allowlist,
bearer and spoof-header stripping, wrong audience, missing permissions,
canonical literal and wildcard admin grants, strict dynamic arity, blocked edit
routes, unknown paths, unapproved methods, and production URL rejection. Denial
cases assert downstream was never called.

## Residual blockers

- The public render handler does not yet prove `status = 'published'`; the
  boundary alone cannot prevent draft rendering.
- Public theme, block, navigation, site, news, plans, rankings, and portfolio
  data still need A10 domain-level review, publication rules, and backend-owned
  entitlement/filtering semantics. These routes remain `partial`.
- Editor UUID mapping and edit-session ownership are unresolved, so all edit
  session routes remain `blocked`.
- Content validation, mutation workflow correctness, file-derived data
  integrity, caching, and database behavior remain outside this authorization
  slice.

The service-authorization fixture retains `productionReadinessClaim: false`;
runtime production readiness is not proven.
