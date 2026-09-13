# Cloudflare Ecosystem Migration — Free-Tier Research + Internal Prod Simulation

**Date:** 2026-08-30 (verified `developers.cloudflare.com` via `curl` at 16:00 ICT)
**Status:** Research complete, simulation scaffolding ready, zero prod mutation
**Tunnel:** `6bee9b58-eede-4b4c-815c-94c0ee38fe58` (`api.epsx.io` → `:30080`, `pay.epsx.io` → `:4752→:30085`, `minio.epsx.io` → `:9100`)
**Stack constraint:** Keep `Rust/Dioxus` (`dioxus 0.7`, `dioxus_ssr`, `axum`, `tokio`, `diesel`, `alloy`)
**Non-mutating rule:** No Tunnel/DNS change until explicit `deploy to prod`. Validation = `cargo build --profile production` + `docker build` + `kubectl apply -k --dry-run=server` + local curl to NodePorts `30000/30001/30080` + optional isolated `wrangler pages deploy` → `*.pages.dev` / `*.workers.dev` preview with teardown.

---

## 1. Executive Summary

`epsx` today runs on **Colima K8s** (`epsx` profile, NodePorts `30000` frontend, `30001` admin, `30080` backend, `30084/30085` pay) fronted by **Cloudflare Tunnel** (QUIC) plus **Vercel-hybrid drift** (`render.yaml` `epsx-backend` Singapore starter, `.vercelignore` still tracked). Prod DBs are bare-metal via `host.docker.internal` (`postgres-external`/`redis-external`/`minio-external` ExternalName → `192.168.5.1` `hostAliases`).

**Verdict:** Cloudflare Free ($0) suffices for preview, internal prod, and low-traffic prod. Graduate to **Workers Paid $5/mo** when durable traffic exceeds ~3 M req/mo, CPU >10 ms/invocation (SSR), or D1 >5 M rows read/day. Free has no commercial-use restriction and unlimited bandwidth vs Vercel Hobby's non-commercial limit.

**Recommended path:** **Stage A ($0) immediately** → **Stage B hybrid (Free, recommended Stage 1)** → **Stage C full Workers ($5, Stage 2)**. Heavy indexer (`alloy` WS) and >5 cron triggers stay on K8s or `Containers` (Paid).

---

## 2. Live Pricing Verified 2026-08-30

Source: `curl -s https://developers.cloudflare.com/workers/platform/pricing/index.md`, `/workers/platform/limits/index.md`, `/r2/pricing/index.md`, `/pages/platform/limits/index.md`, `/kv/platform/{pricing,limits}/index.md`, `/d1/platform/pricing/index.md`, `/queues/platform/pricing/index.md`, `/containers/platform/pricing/index.md`, `/workers/languages/rust/index.md`.

### 2.1 Workers

| Plan | Requests | Duration | CPU time | Memory | Subrequests | Workers | Cron triggers | Static assets |
|------|----------|----------|----------|--------|-------------|---------|---------------|---------------|
| **Free** | **100,000 / day** (hard fail `Error 1027`/`exceededCpu`, reset 00:00 UTC) | no charge | **10 ms / invocation** | 128 MB | 50 / req | 100 | 5 / account | 20,000 files, 25 MiB/file |
| **Paid Standard $5/mo** | **10 M incl / mo + $0.30 / M** | no charge/limit | **30 M CPU-ms incl / mo + $0.02 / M**, max **5 min** per HTTP req (default 30 s), **15 min** per Cron/Queue consumer | 128 MB | 10,000 / req | 500 | **250 / account** | 100,000 files, 25 MiB/file |

Notes:
- No egress/bandwidth charge on either plan.
- Requests to **static assets are free & unlimited** (do not count vs 10 M).
- Service Bindings (worker→worker) bill as **one request** + sum CPU (Standard only).
- 6 simultaneous outgoing connections / req, 64–128 env vars (5 KB each), 3 MB Free / 10 MB Paid worker size, 1 s startup.

**Fine print:** `Workers Paid` is separate from site plan (Free/Pro/Business/Enterprise). Enterprise billed per contract. Cache hits still bill if they hit a Worker.

### 2.2 Pages

| Feature | Free | Pro / Business | Note |
|---------|------|----------------|------|
| Builds | **1 at a time**, **500 / mo** (20 min timeout) | 5 / 20 concurrent, 5k/20k per mo | Per account |
| Files / site | **20,000** | 100,000 (set `PAGES_WRANGLER_MAJOR_VERSION=4`) |  |
| File size | **25 MiB** | 25 MiB | Larger → R2 public bucket |
| Custom domains / project | 100 | 250 / 500 / 500 (Enterprise) | Per project |
| Projects / account | **100** | 100 | Use Workers for Platforms for scale |
| Preview deployments | unlimited | unlimited | |
| `_headers` | 100 rules, 2,000 char / header | same | Use Functions for larger |
| `_redirects` | **2,000 static + 100 dynamic = 2,100 total** | same | Bulk Redirects beyond |
| Functions | billed as Workers (Standard) | same | KV/DO bindings count |

Unlimited requests & bandwidth on both plans for Pages static serving.

### 2.3 R2

| | Standard | Infrequent Access | Free tier / mo (Standard only) |
|---|----------|-------------------|--------------------------------|
| Storage | **$0.015 / GB-mo** | $0.01 / GB-mo (30-day min) | **10 GB-mo** |
| Class A ops | **$4.50 / M** | $9.00 / M | **1 M** |
| Class B ops | **$0.36 / M** | $0.90 / M | **10 M** |
| Retrieval | none | $0.01 / GB | — |
| Egress | **Free forever** | Free | Free |

Class A: `ListBuckets`, `ListObjects`, `PutObject`, `CopyObject`, `CompleteMultipartUpload`, `CreateMultipartUpload`, `UploadPart`, `PutBucket*` etc.
Class B: `HeadBucket`, `HeadObject`, `GetObject`, `UsageSummary`, `GetBucket*` etc.
Free: `DeleteObject`, `DeleteBucket`, `AbortMultipartUpload`.
Rounded up to next billing unit (1.1 GB→2 GB, 1,000,001 ops→2 M).

### 2.4 KV

| | Free / day | Paid / mo | Note |
|---|-----------|-----------|------|
| Keys read | **100,000 / day** | **10 M incl + $0.50 / M** | incl dashboard/wrangler |
| Keys written | **1,000 / day** | **1 M incl + $5.00 / M** | REST bulk: each key = 1 write |
| Keys deleted | **1,000 / day** | **1 M incl + $5.00 / M** | |
| List requests | **1,000 / day** | **1 M incl + $5.00 / M** | |
| Stored data | **1 GB / account+namespace** | **1 GB incl + $0.50 / GB-mo** | Unlimited storage paid |
| Limits | 512 B key, 1,024 B metadata, **25 MiB value**, 1 write/sec same key, 1,000 ops / invocation, 1,000 namespaces/account, 1 GB/namespace free, 30 s min `cacheTtl` | same | Daily reset 00:00 UTC, hard fail on Free |

No egress.

### 2.5 D1

| Metric | Free / day | Paid / mo |
|--------|-----------|-----------|
| **Rows read** | **5 M / day** | **25 B incl + $0.001 / M** |
| **Rows written** | **100 K / day** | **50 M incl + $1.00 / M** |
| **Storage** | **5 GB total** (account sum) | **5 GB incl + $0.75 / GB-mo** |
| Scale-to-zero | yes (no compute/hours charge when idle) | same |

Reads = rows scanned (indexed filters reduce scan); writes = `INSERT`/`UPDATE`/`DELETE` rows + extra write per index touched. Storage = tables+indexes across all DBs. Track via `meta` object / GraphQL Analytics / dashboard. 6 simultaneous D1 connections / Worker invocation. Daily Free reset 00:00 UTC; monthly Paid reset on subscription anniversary. Batch/query limits: see `d1/platform/limits`.

### 2.6 Queues

| | Free / day | Paid / mo |
|---|-----------|-----------|
| Standard ops | **10,000 / day** | **1 M incl + $0.40 / M** |
| Retention | 24 h (non-configurable) | **4 days default, up to 14 days** |

Ops = per **64 KB chunk** (≈100 B metadata): 65 KB & 127 KB each = 2 ops. 3 ops per message lifecycle (1 write + 1 read + 1 delete); retries = extra reads; DLQ write after max retries. No egress.

### 2.7 Durable Objects

Paid only: **$5 incl** 1 M requests + 400,000 GB-s; overage $0.15 / M req + $12.50 / M GB-s. GB-s = 128 MB normalized (WS hibernatable: 20:1 billing ratio for messages). Empty table/SQLite DB consumes storage.

### 2.8 Containers (Firecracker microVMs — Paid only)

| Resource | Included / mo (Paid $5) | Overage |
|----------|------------------------|---------|
| Memory | **25 GiB-hours** | $0.0000025 / GiB-s |
| CPU (active only) | **375 vCPU-min** | $0.000020 / vCPU-s |
| Disk | **200 GB-hours** | $0.00000007 / GB-s |
| Egress | **1 TB NA & Europe**, 500 GB Oceania/Korea/Taiwan/Rest | $0.025 / GB (NA/EU), $0.05 Oceania/Korea/TW, $0.04 elsewhere |

Instance types: `lite` (1/16 vCPU 256 MiB 2 GB) → `basic` (1/4 1 GiB 4 GB) → `standard-1` (1/2 4 GiB 8 GB) → `standard-2` (1 6 GiB 12 GB) → `standard-3` (2 8 GiB 16 GB) → `standard-4` (4 12 GiB 20 GB). Billed from request/manual start until sleep (scale-to-zero). Each container = Worker + Durable Object (both billed).

**Free plan:** N/A — Containers require Paid.

### 2.9 Other

- **Workflows:** Free 100k req/day 10 ms CPU (same as Workers Free); Paid inherits Standard.
- **Hyperdrive:** Included in Free & Paid (connection pooling; requires `nodejs_compat` ≥2024-09-23 for Postgres drivers); tune `max_connections ≤5` per Worker.
- **Turnstile:** Free unlimited (widget), no per-request billing; use for bot protection (no separate table needed).
- **WAF/CDN/Rules:** Unlimited on Free site plan (separate from Workers).

---

## 3. Cost Matrix for `epsx` (Workers req only, before R2/KV/D1)

Assumes avg **7 ms CPU** per request (SSR median via `dioxus_ssr::render_element` + `ServiceClient` proxy).

| Scenario | Workers req / mo | + R2 | Est. monthly (Free vs Paid) | Quota verdict |
|----------|-----------------|------|-----------------------------|---------------|
| **Quiet preview** | 10k | 5 GB | **$0 Free** | 10k ≪ 100k/day (~3 M/mo capacity), 5 GB <10 GB Free |
| **Internal prod** | 100k | 50 GB | **$0 Free** (edge) → $0.60 R2 if Standard | 100k ≪ 3 M/mo Free headroom; R2 50 GB→40 GB billable ×$0.015=$0.60 (shard to stay Free) |
| **Early prod** | 1 M | 50 GB | **$5.60 Paid** ($5 + $0.60 R2; Workers $0 extra: 1 M <10 M, CPU 7 M <30 M) | Free would be ~33k/day OK but CPU margin tight; recommend Paid for safety |
| **Growth (doc example)** | 15 M | — | **$8.00 Paid** ($5 + $1.50 req + $1.50 CPU) | Example 1 from docs: (15 M-10 M)/1M×$0.30 + (7 ms×15 M -30 M)/1M×$0.02 |
| **Static-heavy (80 % assets)** | 15 M (12 M static free) | — | **$5.00 Paid** | Static `fetch` free/unlimited on both plans — Example 2 |
| **Cron indexer** (720 req/mo × 3 min) | 720 | — | **$6.99 Paid** | Example 3: (180k×720 -30M)/1M×$0.02=$1.99 + $5 |

**Decision rule:** Stay **Free** for preview / internal prod / low-traffic prod (<3 M req/mo sustained, CPU p99 <10 ms, D1 <5 M reads/day, KV <100k reads/day). Graduate to **$5 Paid** when any of: durable >3 M req/mo, SSR p50 >10 ms, D1 >5 M reads/day or >100k writes/day, Queues >10k ops/day, cron >5 triggers, or indexer needs >10 ms.

**Vercel Hobby contrast:** 100 GB bandwidth, 1 M edge req/invocations, 4 CPU-hours, **non-commercial only** ($20/seat Pro for commercial) — Cloudflare Free has **no commercial restriction** and **unlimited bandwidth / static** + 100k/day = ~3 M/mo free.

---

## 4. Rust/Dioxus Compatibility Spike

| `epsx` piece | Current impl | Workers viability | Path |
|--------------|--------------|-------------------|------|
| **SSR rendering** (`dioxus_ssr`) | `dioxus_ssr::render_element` on `VirtualDom` (runtime-agnostic, rsx→string) in `apps/frontend|admin|pay/src/main.rs` + `shared/rust/dioxus_ui` + `epsx_templates::page_shell_with_body_class` | ✅ **Go** — `dioxus_ssr` is framework-only; deployable to `workerd`/WASM (no Node). Must compile WASM server target (`wasm32-unknown-unknown` + `wasm-bindgen` 0.2.123 + `wasm-opt` with `lto=true strip=true codegen-units=1`). | Keep `dioxus_ssr`; add `worker-build` + WASM server binary. |
| **BFF Axum routers** (`bff-frontend :3000`, `bff-admin :3001`, `bff-pay :3002`) | `axum` + `tower-http` + `ServiceClient` → gateway `epsx` monolith `:8080` | ✅ **Thin edge** via `workers-rs` + `axum-cloudflare-adapter` (maps `http` crate `0.0.21+` between Axum & `worker`). Full `tokio`/`hyper` not available on Free. | Stage B: keep K8s origin, proxy via Service Bindings; Stage C: `workers-rs` edge. |
| **Monolith `epsx` (`apps/backend` :8080)** | `axum` + `tokio` full + `diesel` + 4 `Diesel` DBs (`diesel.toml`, `diesel_analytics.toml`, `diesel_notifications.toml`, `diesel_payments.toml`) + migrations (`ALTER TABLE ADD/RENAME` + `IF EXISTS`) | ⚠️ **Hybrid** — `diesel`/`tokio-postgres` native TCP blocked in Workers sandbox. Options: (a) keep on K8s/Containers, (b) Hyperdrive with `nodejs_compat` ≥2024-09-23 + `postgres`/`neon-serverless`/`drizzle` HTTP driver, pool ≤5 per Worker. | Keep monolith on K8s prod; expose via Hyperdrive TCP→HTTP or Service Binding. Do not rewrite Diesel → `postgres` until Stage C. |
| **Analytics / payments subsets** | `services/analytics`, `services/pay`, `services/subscription` + `epsx-analytics` crates | ⚠️ D1/Durable SQLite candidate for Stage C | Migrate read-heavy analytics to D1 (indexed) with 5 M/day Free headroom. |
| **Indexer (`services/indexer`)** | `alloy` WebSocket (BSC `wss`) + long jobs (>15 m) | ❌ **Off Free** — WS + >10 ms + >15 min → Paid Containers or K8s | Paid `Containers` (Paid incl 25 GiB-hr +375 vCPU-min) or keep K8s `epsx-prod` `Deployment` (current). |
| **Cron / Queues** | `xtask` cron triggers (5 Free max) | ❌ Free capped at 5, Paid 250 | Stay on K8s jobs or Paid Workflows/Queues (1 M/mo $0.40/M). |
| **DB** | PG `epsx_prod` etc. on brew (`192.168.5.1:5432` via `hostAliases` + `postgres-external:5432` ExternalName), Redis `6379` pw `epsx`, MinIO `:9100` | ✅ R2 replaces MinIO (free op via Super Slurper/Sippy), D1/KV for session/flags, Hyperdrive for PG | R2 migration: `rclone`/`Super Slurper` free; KV session (Workers KV REST/API). |
| **Browser runtime** | `shared/rust/browser-runtime` (CDylib/rlib, `wasm-bindgen-futures`, `web-sys`) + `cargo xtask browser-runtime build` → `target/epsx-browser-runtime` + `apps/frontend/src/ssr.rs` FOUC design system | ✅ WASM already built | Reuse `xtask browser-runtime build` as Pages static asset source. |
| **Assets / CSS** | `cargo xtask assets verify` (frozen Tailwind), `public/dist/tailwind.css` | ✅ Pages static assets | Publish `dist` + `target/epsx-browser-runtime` as Worker static assets (free/unlimited fetch). |
| **Auth / session** | `epsx_bff::session` (`JwksVerifier` + `CookieEnvironment` + `typed_session::TypedBffSession`), `epsx-auth` RSA mounted via `epsx-backend-keys` secret (persistent `RSA_*` from `.env.prod`) | ⚠️ KV or Workflows + `workers-rs` `Env` | KV for session/flags (Free 100k reads/day); mount RSA via `wrangler secret`. |

**Blockers to stay on K8s (not Workers Free):** `tokio`/`hyper` full features, `diesel` native TCP, `alloy` WS indexer, long jobs >15 min, 128 MB/10 ms Free cap.

---

## 5. Staged Architecture (pick one to implement)

### A — Keep K8s + CF front (lowest risk, $0) ⭐ Immediate

Keep `overlays/prod` NodePorts + hostAliases + ExternalName; front with Cloudflare site (Free) + R2.

- **Front:** Cloudflare CDN/WAF/Rules/Turnstile (unlimited, Free) → Tunnel `6bee9b58-...` unchanged (no ingress mutation).
- **Storage:** MinIO `:9100` → R2 migration via **Super Slurper** / **Sippy** (free operation, gradual; dual-write then cutover). Keep `MINIO_PUBLIC_URL` → `R2 public bucket` + custom domain `static.epsx.io` (or `assets.epsx.io`).
- **Build:** `cargo xtask build --profile production` + `docker build -f apps/frontend|admin|backend|pay/Dockerfile -t epsx-*:prod` + `kubectl apply -k overlays/prod` (existing flow).
- **Cleanup:** Delete `render.yaml` + `.vercelignore` (Vercel drift), update `docs/01_deploy-guide.md` to remove `NEXT_PUBLIC_*` docker-compose args, point to `R2` instead of `host.docker.internal:9100` when ready.
- **Cost:** $0 (R2 within Free: 10 GB-mo +1 M A +10 M B).
- **When:** Now (no Workers needed).

### B — Hybrid Pages/Workers → K8s origin (recommended Stage 1, Free) ⭐ Recommended

`apps/frontend` / `admin` / `pay` BFF Axum → **thin Workers edge** + **Pages static assets** (unlimited), API proxied to K8s.

```
[Browser] ──(Pages static assets, unlimited)──> R2/Pages ─┐
          ──(SSR /api)──> Worker (`workers-rs` + `axum-cloudflare-adapter`) ──Service Binding──> K8s origin (`api.epsx.io` :30080 via Hyperdrive Tunnel)
                                                                   └─> External PG/Redis via Hyperdrive (nodejs_compat)
```

- **Frontend/admin/pay:** Publish `dist` + `target/epsx-browser-runtime` as **Pages** or **Worker static assets** (free/unlimited fetch, 20k files/site 25 MiB/file, `_headers` 100 rules, `_redirects` 2100).
- **Edge SSR:** `dioxus_ssr` inside Worker (WASM). Heavy SSR fallback → `Containers` (Paid) or K8s origin.
- **API:** Worker proxies `/api/*` → `api.epsx.io` origin or Hyperdrive to external Postgres (pool ≤5). Keep `ServiceClient` (adds `security_headers`, `JwksVerifier`).
- **Quotas:** Workers Free 100k req/day comfortably covers SSR+API for epsx traffic; beyond → $5 Paid.
- **Cost:** $0 Free until >3 M req/mo.

### C — Full Workers (Stage 2, $5)

R2 replaces MinIO, D1/Durable SQLite replaces analytics/payments subsets, KV for session/flags, Queues + Workflows for indexer.

- **Storage:** R2 (10 GB Free → $0.015/GB-mo), KV (100k reads Free → $0.50/M), D1 (5 M reads Free → $0.001/M reads, 100k writes Free → $1/M).
- **Async:** Queues (Free 10k/day → Paid 1 M/mo $0.40/M, 24 h→14 day retention) + **Workflows** for indexer/notifications (Free 100k req/day).
- **Heavy compute:** `services/indexer` (`alloy` WS) on **Containers** (Paid 25 GiB-hr +375 vCPU-min incl) or stay on K8s (`epsx-indexer` Deployment) + Workflows orchestration.
- **Cron:** 250 triggers on Paid (Free only 5).
- **Cost:** $5 base + overage (e.g., 1 M req/mo 7 ms avg → $0; 15 M → $8; see §3). R2 50 GB→$0.60, D1/Q/KV within incl for epsx scale.

---

## 6. Test Plan — Simulation Only (No Prod Deploy)

> ⚠️ **Critical:** Never deploy to prod unless user explicitly says `deploy to prod`. Simulation is local + isolated preview only.

### 6.1 Build sim (non-mutating)

```bash
# WASM runtime (already: wasm-bindgen 0.2.123, wasm32-unknown-unknown)
cargo xtask browser-runtime build
# Production profiles (BFF Axum + Dioxus SSR + monolith)
cargo xtask build --profile production
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask audit no-node --strict
cargo xtask assets verify
# Docker images (same as prod Dockerfiles: rust:slim-bookworm → debian:bookworm-slim, HEALTHCHECK /api/health)
docker build -f apps/frontend/Dockerfile -t epsx-frontend:sim .
docker build -f apps/admin/Dockerfile    -t epsx-admin:sim .
docker build -f apps/backend/Dockerfile  -t epsx-backend:sim .
docker build -f apps/pay/Dockerfile      -t epsx-pay-bff:sim .  # if Dockerfile exists
```

Verifies Dioxus WASM + Axum BFF compiles to `debian:bookworm-slim` image (HEALTHCHECK `/api/health` on `3000`/`3001`/`8080`).

### 6.2 K8s dry-run (non-mutating)

```bash
# Local render (no server needed — proves kustomize correctness)
kubectl kustomize infrastructure/kubernetes/overlays/prod > /tmp/kustomize-prod.yaml  # 362 lines OK 2026-08-30
# Validate against OpenAPI (requires colima running)
kubectl apply -k infrastructure/kubernetes/overlays/prod --dry-run=server --validate=true
# Or ignore API when colima down (still validates structure)
kubectl apply -k infrastructure/kubernetes/overlays/prod --dry-run=client
# (Phase infra done: ExternalName host.docker.internal → postgres-external:5432 etc.; hostAliases 192.168.5.1 kept for rollback)
```

Current 2026-08-30: `colima is not running` → `--dry-run=server` fails `dial tcp 127.0.0.1:62825: connection refused` (expected). `kustomize build` succeeds; socat bridges still running (`4750`/`4749`/`4748`/`4747` via `com.epsx.pay-port-bridge` LaunchAgent + `com.epsx.port-bridge` `8080/4810/9180`).

### 6.3 Live host checks (non-mutating)

```bash
ps aux | grep socat          # expect 4× socat for 4747-4752 + pay bridge
ps aux | grep cloudflared    # tunnel running
# When colima up:
kubectl get pods -n epsx-prod
kubectl logs -n epsx-prod deployment/epsx-backend
curl -sf http://localhost:30000/api/health   # frontend NodePort
curl -sf http://localhost:30001/api/health   # admin
curl -sf http://localhost:30080/api/health   # backend
curl -sf http://localhost:30085/api/health   # pay-bff (via 4752 bridge)
# Bare-metal PG/Redis/MinIO via host.docker.internal (192.168.5.1 alias)
psql "postgresql://$DB_USER:$DB_PASSWORD@host.docker.internal:5432/epsx_prod" -c "select 1"
redis-cli -h host.docker.internal -a epsx ping
curl -sf http://host.docker.internal:9100/minio/health/live
```

### 6.4 Cloudflare preview (isolated, teardown — optional)

Requires `wrangler login` (never commit token; use `CLOUDFLARE_API_TOKEN` env).

```bash
# Pages preview (isolated project, never prod domain)
bunx wrangler pages deploy dist --project-name epsx-preview --branch sim-$SHA
# Worker dry-run
bunx wrangler deploy --dry-run --config infrastructure/cloudflare/wrangler.preview.toml
# Remote dev only if testing Hyperdrive (warn: prod DB)
bunx wrangler dev --remote --config infrastructure/cloudflare/wrangler.preview.toml
# Verify
# - _headers (100 rules, 2,000 char/header) + _redirects (2100) limits
# - Turnstile widget renders
# - R2 binding (create isolated bucket epsx-preview-assets, delete after)

# Teardown (always)
bunx wrangler pages deployment delete --project-name epsx-preview --branch sim-$SHA
bunx wrangler r2 bucket delete epsx-preview-assets  # if created
# Never touch api/pay/minio ingress 30080/30085/9100 or Tunnel 6bee9b58-... host rules
```

### 6.5 Audits & Lighthouse

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask audit no-node --strict
cargo test --workspace --locked            # optional
cargo xtask e2e doctor --group 0           # env check
bunx lighthouse https://epsx-preview-xxxx.pages.dev --output json  # or compare_mobile_desktop on preview
```

---

## 7. Rollback to K8s

### 7.1 Kustomization

```bash
# Full stack
kubectl apply -k infrastructure/kubernetes/overlays/prod
kubectl rollout restart deployment -n epsx-prod -l app
kubectl rollout status -n epsx-prod deployment/epsx-backend
# Single service
kubectl apply -k infrastructure/kubernetes/base/backend
kubectl scale deployment/epsx-backend --replicas=1 -n epsx-prod
```

`base/kustomization.yaml` is limited to core prod (`backend`/`frontend`/`admin` + `postgres-external` etc.); `overlays/prod/kustomization.yaml` pins images `:prod` + `patches/{replicas,resources,services-nodeport}.yaml` + `pay-*.yaml`. Rollback = `git checkout main -- infrastructure/kubernetes/overlays/prod` then `kubectl apply -k`.

### 7.2 Tunnel

Restore `infrastructure/cloudflare/cloudflared-config.prod.yml`:

```yaml
tunnel: 6bee9b58-eede-4b4c-815c-94c0ee38fe58
ingress:
  - hostname: api.epsx.io
    service: http://localhost:30080
  - hostname: pay.epsx.io
    service: http://localhost:4752   # → 30085 → epsx-pay-bff:3002
  - hostname: minio.epsx.io
    service: http://localhost:9100
  - service: http_status:404
```

Then `cloudflared tunnel ingress validate` and `launchctl load ~/Library/LaunchAgents/com.epsx.port-bridge.plist` + `com.epsx.pay-port-bridge.plist`.

### 7.3 Secrets & DB

Secrets remain in `.env.prod` via `infrastructure/kubernetes/scripts/create-secrets.sh prod` → `epsx-backend-keys` (RSA `RSA_PRIVATE_KEY`/`RSA_PUBLIC_KEY`/`RSA_KEY_ID` mounted, never regenerated on restart).
DB: never `DROP`; use `ALTER TABLE ADD/RENAME` + `IF EXISTS/IF NOT EXISTS` per `AGENTS.md`; Diesel migrations run as `initContainer` `./migrate up`.

### 7.4 R2 dual-write rollback

If R2 cutover done, keep MinIO running (dual-write period). Rollback = DNS + Tunnel revert to MinIO `:9100` + delete R2 bucket bindings.

---

## 8. Assumptions

- Secrets stay in `.env.prod` → `create-secrets.sh prod`; Cloudflare token never committed.
- Free limits reset 00:00 UTC daily; breach = hard fail `exceededCpu`/`Error 1027`, no overage billing on Free — **Paid required before sustained breach**.
- `services/indexer` (`alloy` WS) + cron >5 stay off Free (Paid 250 crons) or on K8s/Containers.
- Colima profile `epsx` (`192.168.5.1` hostAliases → `host.docker.internal`) may drift; `postgres-external:5432` is the forward-fix, hostAliases kept for one release.
- `render.yaml` (starter Singapore `healthCheckPath: /health`, `RUST_ENV=production`) and `.vercelignore` are legacy drift; deletion is post-Stage-A doc approval (not in this PR).
- Staging/prod domains (`epsx.io`, `admin.epsx.io`, `api.epsx.io`, `pay.epsx.io`, `minio.epsx.io`) untouched until explicit `deploy to prod`.

---

## 9. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| SSR p99 CPU >10 ms on Free (workerd cold start + `dioxus_ssr` 7-15 ms) | `exceededCpu` 503 | Stage B with K8s fallback; Page cache + `Cache API`; graduate to Paid (30 M incl) |
| Diesel native TCP in Workers | Build fail | Hyperdrive `nodejs_compat` + `postgres` driver pool≤5; keep monolith on K8s for Stage 1-2 |
| `alloy` WS / long jobs | Not schedulable on Free | Containers (Paid) or K8s `Deployment` with Workflows orchestration |
| Free daily quota burst (100k) | 1027 throttling at 00:00 UTC | Monitor `workers observability` logs; set **Paid** alert at 80k/day; `rate limiting` rules |
| R2 SI Sippy/Super Slurper dual-write consistency | Stale assets | Verify `ETag` + `rclone check`; keep `MINIO_PUBLIC_URL` dual-publish |
| Bundle size (WASM 3 MB Free / 10 MB Paid) | Upload fail | `wasm-opt` + `lto/strip/codegen-units=1`; static assets offloaded to R2/Pages |
| Secrets drift (RSA re-gen) | Session expiry | Persistent `epsx-backend-keys` secret mounted; never auto-generate |

---

## 10. Immediate Next Steps (simulation PR)

1. **Create isolated `wrangler` preview scaffolding** (not deployed):
   - `infrastructure/cloudflare/wrangler.preview.toml` — `name=epsx-preview`, `compatibility_date=2024-09-23`, `nodejs_compat`, `[[r2_buckets]] binding=ASSETS`, `[[kv_namespaces]] binding=SESSION`, `observability` logs, never `route` to `epsx.io`.
   - `infrastructure/cloudflare/pages/_headers` (≤100 rules) + `_redirects` (≤2100) stubs.
2. **Run simulation locally** (see §6; colima up required for `--dry-run=server` + curl NodePorts; `kustomize build` already green).
3. **Delete `render.yaml` + `.vercelignore` Vercel drift** after doc review (PR separate, document here).
4. **Update `docs/01_deploy-guide.md`** to add Cloudflare option alongside Colima K8s, keep K8s as rollback path.
5. **When ready:** `wrangler pages deploy dist --project-name epsx-preview --branch sim-$SHA` → `*.pages.dev` → Lighthouse → teardown.

---

## 11. Appendix: Key File References

- `Cargo.toml` — resolver 2, 37 members (`shared/rust/*` ×15, `services/*` ×8, `apps/{frontend,admin,pay,backend,analytics,preview}` + `xtask`)
- `apps/frontend|admin|pay/src/main.rs` — Axum routers (`bff-frontend :3000`, `bff-admin :3001`, `bff-pay :3002`, `epsx :8080`)
- `shared/rust/{bff,browser-runtime,dioxus_ui,templates,renderer}` — SSR + design system
- `apps/frontend|admin|pay/Dockerfile` — `rust:slim-bookworm` → `debian:bookworm-slim`, `wasm-bindgen 0.2.123`, `cargo xtask browser-runtime build + cargo build --release --bin bff-*`, `HEALTHCHECK /api/health`
- `services/indexer/Cargo.toml` — `alloy` WS
- `infrastructure/kubernetes/{base/{frontend,backend,admin}/deployment.yaml,base/kustomization.yaml,overlays/prod/{kustomization.yaml,patches/*}}` — NodePorts `30000/30001/30080/30084/30085`, `hostAliases 192.168.5.1→host.docker.internal`, `postgres-external:5432`
- `infrastructure/cloudflare/cloudflared-config.prod.yml` — Tunnel `6bee9b58-...`
- `infrastructure/scripts/com.epsx.port-bridge.plist` + `com.epsx.pay-port-bridge.plist` — socat `8080/4810/9180` + `4747-4752`
- `render.yaml` / `.vercelignore` — legacy; to be deleted
- `docs/01_deploy-guide.md` (266 lines) — still docker-compose `args NEXT_PUBLIC_*`
- `xtask/src/main.rs` — `browser-runtime build`, `build --profile`, `audit no-node`, `assets verify`

## 12. References (verified 2026-08-30)

- https://developers.cloudflare.com/workers/platform/pricing/
- https://developers.cloudflare.com/workers/platform/limits/
- https://developers.cloudflare.com/r2/pricing/
- https://developers.cloudflare.com/pages/platform/limits/
- https://developers.cloudflare.com/workers/languages/rust/
- https://developers.cloudflare.com/kv/platform/pricing/ + /limits
- https://developers.cloudflare.com/d1/platform/pricing/ + /limits
- https://developers.cloudflare.com/queues/platform/pricing/ + /limits
- https://developers.cloudflare.com/durable-objects/platform/pricing/
- https://developers.cloudflare.com/containers/platform/pricing/
- https://developers.cloudflare.com/turnstile/ , /hyperdrive/configuration/ , /workers/static-assets/
- https://github.com/cloudflare/workers-rs , /workers-rs/tree/main/worker-build (wasm-bindgen, wasm-opt)
- https://github.com/vercel-labs/agent-browser (verify preview)
