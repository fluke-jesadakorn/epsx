# Migration: Next.js → Dioxus 0.7 + Axum

**Branch:** `migration/dioxus-microservices`
**Status:** Restructure complete. Workspace builds green. Per-component port in progress.

## What landed

### Layout (restructure done, 2026-06-11)

The `migrated/` sub-monorepo prefix is gone. The Rust workspace is the
workspace root, and the Next.js TS apps are preserved as `apps-old/`.

```
.
├── Cargo.toml, Cargo.lock           # Rust workspace
├── apps/
│   ├── admin/                       # Dioxus+Axum admin BFF   (NEW)
│   ├── frontend/                    # Dioxus+Axum frontend BFF (NEW)
│   ├── pay/                         # Dioxus+Axum pay BFF
│   ├── preview/                     # Dioxus+Axum preview BFF
│   ├── backend/                     # Axum backend (restored)
│   └── contracts/                   # Foundry (restored)
├── apps-old/                        # Next.js TS apps (preserved as port source)
│   ├── frontend/                    # Next.js 14 (READ-ONLY source of truth for the port)
│   └── admin-frontend/              # Next.js 14 admin
├── services/                        # 9 Rust microservices
│   ├── analytics, content, gateway, identity, indexer,
│   ├── notification, payment, subscription, wallet
├── shared/
│   ├── rust/                        # 13 shared Rust crates
│   │   ├── kernel, events, config, crypto, auth, database,
│   │   ├── observability, web3, client, renderer, templates,
│   │   ├── dioxus_ui, bff
│   └── (TS Next.js shared code; untouched)
├── content/                         # Marketing MDX / page blocks
├── proto/                           # gRPC protos
├── infrastructure/                  # K8s, Docker, Cloudflare Tunnel
├── scripts/                         # Dev / test / deploy scripts
├── apps/contracts/                  # Foundry contracts
└── apps/backend/                    # Axum backend
```

### Commits on `migration/dioxus-microservices` (latest first)

| SHA | Title |
|---|---|
| `a670c0fd` | chore: add root Cargo.lock |
| `55d6304e` | chore(migrate-e): drop migrated/ folder |
| `1dc9e350` | fix(migrate-d): missed services/notification in the path update |
| `70c8a5dc` | chore(migrate-d): finish the move - Rust source + BFFs to root, old TS apps to apps-old/ |
| `8c307c7f` | chore(migrate-c): flatten migrated/ subtree back to workspace root |
| `d83d7a32` | chore(migrate-b): drop apps/backend (prior baseline) |

### Compile gate

```
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 54.82s

$ cargo build --workspace --bins
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 34.15s
```

24/24 workspace members compile. Warnings only, no errors. No `migrated/`
references anywhere in the tree.

### Smoke test

`bff-frontend` (port 13000), `bff-admin` (port 13001), `bff-pay`, `bff-preview`
all start and serve their route surface (44 routes total per
MIGRATION_PORT_TODO.md, all 200s as of 2026-06-10 baseline).

## Next: per-component port (Next.js → Dioxus+Axum)

The previous baseline already stubbed the surface (303 frontend .rs +
310 admin .rs + ~90 dioxus_ui .rs) as a 1:1 file mapping. The next
phase is to **fill in the bodies** with real Dioxus 0.7 `rsx!` translations
of the Next.js/React code in `apps-old/frontend` and `apps-old/admin-frontend`.

### Mapping strategy

- For each Next.js page in `apps-old/frontend/app/<path>/page.tsx`, find
  the corresponding stub in `apps/frontend/src/app/<path>.rs` and replace
  the placeholder body with the real Dioxus `#[component] fn` translation.
- For each shared component in `apps-old/frontend/components/<X>.tsx`,
  find the corresponding stub in `shared/rust/dioxus_ui/src/primitives/<x>.rs`
  and translate the JSX body.
- For each `app/actions/<X>.ts` (server action) or
  `app/api/<X>/route.ts` (API route), translate to
  `pub async fn handler(...)` and wire through `epsx_client::*`.

### Porting order (rough priority)

1. **Core primitives (27)** — Button, Card, Dialog, Input, etc. All
   pages depend on these. (Wave 1)
2. **Layout (6)** — Header (with 3 dropdowns), Footer, Sidebar, Wallet
   Connect modal. (Wave 2)
3. **Auth (6)** — Login form, SIWE flow, session helpers. Wire to
   `epsx_client::identity::*`. (Wave 3)
4. **Feedback (4) + Data (3)** — Toasts, modals, tables, charts. (Wave 4)
5. **23 frontend pages** — Home, Auth, Dashboard, Profile, Account,
   Analytics, Chat, Plans, Permissions, News, Notifications, Contact,
   About, Manual, Portfolio, Developer, Payment. (Wave 5)
6. **12 admin pages** — Dashboard, Users, Rankings, Plans, News, Media,
   Audit, Policies, Settings, Payments, Chat, wallet mgmt. (Wave 6)
7. **Service wiring** — replace `Err("stub")` returns in
   `api::*`/`auth::*`/`hooks::*` with real `epsx_client::*` calls
   (identity, wallet, payment, subscription, content, notification,
   analytics). (Wave 7, parallel with page ports)

### Hard rules

- `development` branch is **read-only**. It is the source of truth for
  what behaviour the Dioxus port must reproduce.
- `apps-old/frontend` and `apps-old/admin-frontend` are read-only
  references for the port. They get deleted once each page is ported
  and verified.
- The Rust workspace must stay green at every commit:
  `cargo check --workspace && cargo build --workspace --bins`.

## Open items carried over

- `#[server]` macro disabled (Dioxus 0.7.9's server feature pulls in
  broken liveview). Action functions are `pub async fn`; SSR layer
  needs manual wiring.
- Tailwind class strings in `rsx!` are static for now. Build-tool
  follow-up to enable Dioxus tailwind processing.
- `cva` / `class-variance-authority` replaced with simple `pub fn X(props)`;
  real variant logic deferred.

## Per-file Rust counts (target)

| Target | Rust files |
|---|---|
| `apps/frontend/src/**.rs` | 303 (placeholder bodies) |
| `apps/admin/src/**.rs` | 310 (placeholder bodies) |
| `shared/rust/dioxus_ui/src/**.rs` | ~90 (primitives + layout + pages stubs) |
| `shared/rust/bff/src/**.rs` | 4 (BFF middleware) |
| `services/*/src/**.rs` | ~9 services with handlers |
| `shared/rust/{kernel,events,...}/src/**.rs` | 13 shared crates |
