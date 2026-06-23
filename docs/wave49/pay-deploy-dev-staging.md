# wave49 — pay.epsx.io dev + staging deploy (slice 6)

**Status:** DEPLOYED + SMOKE-PASS on both dev (`epsx-dev` ns) and staging (`epsx-staging` ns). Not deployed to prod (out of scope).

**Commit:** `4f6bfcd9` (1 fix + 1 doc).

## Cluster

- colima profile `epsx` — aarch64 / 6 CPU / 12 GiB RAM / 30 GiB disk
- k3s v1.35.0+k3s1 (k3s-included) on macOS Virtualization.Framework
- 1 control-plane node `colima-epsx` (Ready)
- 2 namespaces: `epsx-dev`, `epsx-staging`
- Local Postgres 18 (Homebrew) listening on `*:5432` (was `localhost` only)
- Local PG `epsx` role + `epsx_pay` DB created (was missing on this host)

## Images

| Image | Tag | Pulled from |
|-------|-----|-------------|
| `epsx-pay-svc`  | `wave49-dev`, `wave49-staging` | `services/pay/Dockerfile` (local build) |
| `epsx-pay-bff`  | `wave49-dev`, `wave49-staging` | `apps/pay/Dockerfile` (local build) |

Both 38 MB / 166 MB compressed. Multi-stage `rust:slim-bookworm` → `debian:bookworm-slim`. `imagePullPolicy: IfNotPresent`.

## K8s resources applied (per namespace)

| Resource | NodePort (dev / staging) | ClusterIP port | Container port |
|----------|------|------|------|
| `Service/epsx-pay-svc` | 30106 / 30082 | 8103 | 8103 |
| `Service/epsx-pay-bff` | 30107 / 30083 | 3002 | 3002 |
| `Deployment/epsx-pay-svc` | n/a | n/a | 1 replica, 64Mi req / 256Mi lim |
| `Deployment/epsx-pay-bff` | n/a | n/a | 1 replica, 64Mi req / 256Mi lim |

Both deployments have `/health` + `/api/health` probes. Both 1/1 Ready after the env= fix.

## Smoke test results (all from `localhost` on the host, hitting the NodePorts)

```
DEV:
  curl http://localhost:30106/health                         → HTTP 200
  curl http://localhost:30106/api/v1/pay/intents             → HTTP 200, 2 intents
  curl http://localhost:30107/api/health                     → "ok" HTTP 200
  POST http://localhost:30106/api/v1/pay/intents (valid addr) → 200 + pay_url
  POST http://localhost:30107/api/v1/pay/links               → 200 + slug
  GET  http://localhost:30107/r/epsx-7c35e044b9ae           → 303 → /pay?intent=...

STAGING:
  curl http://localhost:30082/health                         → HTTP 200
  curl http://localhost:30082/api/v1/pay/intents             → HTTP 200, 2 intents
  curl http://localhost:30083/api/health                     → "ok" HTTP 200
  POST http://localhost:30082/api/v1/pay/intents (valid addr) → 200 + pay_url
  POST http://localhost:30083/api/v1/pay/links               → 200 + slug
  GET  http://localhost:30083/r/epsx-82d25f1a4db1           → 303 → /pay?intent=...
```

## Fix shipped (1 commit)

**`4f6bfcd9` — `fix(pay-svc): wire clap --database-url to DATABASE_URL env var`**

Root cause: `services/pay/src/main.rs` had `#[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_pay")]` for `database_url` — no `env = "DATABASE_URL"` attribute. The K8s manifest set `DATABASE_URL` via env, but clap's default won at parse time. Result: pay-svc panicked on boot with `Failed to connect to database: PoolTimedOut` because `localhost:5432` is not reachable from inside a pod.

Also added `env = "CHAIN_ID"` + `env = "ESCROW_CONTRACT"` to the other two args for consistency with the kustomize env vars (defaults still match: 56 + "0").

Diff: 11 insertions, 3 deletions, 1 file.

## What still needs operator action

1. **Cloudflare DNS** — manual step, not done here:
   - `pay.epsx.io`        CNAME → `<tunnel>.cfargotunnel.com` (prod tunnel)
   - `staging-pay.epsx.io` CNAME → same tunnel (path-routed)
   - `dev-pay.epsx.io`     CNAME → same tunnel (path-routed)
   The tunnel configs in `infrastructure/cloudflare/cloudflared-config.{prod,dev}.yml` already
   have `pay.epsx.io` entries pointing at the local port-bridge `localhost:4747` →
   `socat` → NodePort 30082 (prod). Staging entry needs the same treatment.

2. **Cloudflared `pay.epsx.io` route** — same configs as above. Currently the
   prod tunnel has `pay.epsx.io` → `http://localhost:4747`. The dev tunnel has
   `dev-pay.epsx.io` → `http://localhost:4747`. Staging is missing the entry.

3. **Port-bridge plist** — `infrastructure/scripts/com.epsx.pay-port-bridge.plist`
   is in the repo but not loaded into launchctl. `launchctl load` is the operator
   step. Once loaded, ports 4747 + 4748 bridge to NodePort 30082 + 30083.

4. **Prod DB rename** — `ALTER DATABASE epsx_payments RENAME TO epsx_pay;` is
   a manual ops step on Neon. The pay-svc DATABASE_URL points to `epsx_pay`
   (per the wave49(slice-1) rename plan in the kustomize env). Until the rename
   is run, prod pay-svc will fail to connect.

5. **Isolation between dev and staging** — both namespaces currently point to
   the same `epsx_pay` database on the local PG. The list endpoint returns
   the same 2 intents for both. If true isolation is needed, change the
   `DATABASE_URL` env in the dev overlay to `...epsx_pay_dev?sslmode=disable`
   (the dev DB already exists; the staging DB would need to be created).

## Files applied (dev + staging)

- `/tmp/dev-pay.yaml` — 5 docs (Namespace/epsx-dev + 2 Services + 2 Deployments)
- `/tmp/staging-pay.yaml` — 5 docs (Namespace/epsx-staging + 2 Services + 2 Deployments)

Extracted from `kubectl kustomize infrastructure/kubernetes/overlays/{dev,staging}`
using a Python filter that keeps only Namespace + Service/Deployment where the
name matches `epsx-pay-*`. The other 6 services in each overlay
(`epsx-backend`, `epsx-frontend`, `epsx-admin`, `epsx-analytics`, `epsx-identity`,
`epsx-admin-frontend`) are intentionally not applied here — they would need
their own pre-built images.

## Verification commands (operator rerun)

```bash
# Pods
kubectl get pods -n epsx-dev -l app=epsx-pay-svc
kubectl get pods -n epsx-staging -l app=epsx-pay-svc

# Health (host)
curl -s http://localhost:30106/health
curl -s http://localhost:30082/health

# BFF (host)
curl -s http://localhost:30107/api/health
curl -s http://localhost:30083/api/health

# End-to-end intent flow
INTENT=$(curl -s -X POST http://localhost:30106/api/v1/pay/intents \
  -H 'Content-Type: application/json' \
  -d '{"payer":"0x1234567890123456789012345678901234567890","payee":"0x0987654321098765432109876543210987654321","amount":"100","token":"USDC"}' \
  | python3 -c 'import json,sys;print(json.load(sys.stdin)["intent"]["id"])')
SLUG=$(curl -s -X POST http://localhost:30107/api/v1/pay/links \
  -H 'Content-Type: application/json' \
  -d "{\"intent_id\":\"$INTENT\"}" \
  | python3 -c 'import json,sys;print(json.load(sys.stdin)["link"]["slug"])')
curl -sv http://localhost:30107/r/$SLUG 2>&1 | grep -i location
```
