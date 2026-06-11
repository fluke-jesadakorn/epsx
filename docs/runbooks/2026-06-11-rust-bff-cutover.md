# 2026-06-11 — Rust BFF Production Cutover Runbook

Step-by-step for an operator with cluster + registry access. Replaces the
"Vercel" cutover that was planned in `2026-04-16-vercel-hybrid-deployment.md`
— that plan is no longer applicable because the user-facing apps are
now Rust binaries, not Next.js.

Plan doc: [`../plans/2026-06-11-rust-bff-deployment.md`](../plans/2026-06-11-rust-bff-deployment.md)

## 0. Pre-flight (in this session, before you start)

These all happen in `/private/tmp/epsx-wave5/` (the worktree on branch
`wave5/rust-bff-deploy`).

- [x] `apps/frontend/Dockerfile` written
- [x] `apps/admin/Dockerfile` written
- [x] `.dockerignore` at repo root written
- [x] `infrastructure/kubernetes/base/frontend/deployment.yaml` rewritten
- [x] `infrastructure/kubernetes/base/admin/deployment.yaml` rewritten
- [x] `infrastructure/kubernetes/base/admin/service.yaml` port 3000 → 3001
- [x] `infrastructure/kubernetes/overlays/prod/kustomization.yaml` image overrides updated
- [x] `infrastructure/kubernetes/overlays/prod/patches/services-nodeport.yaml` admin port fixed
- [x] `infrastructure/kubernetes/overlays/staging/kustomization.yaml` updated
- [x] `infrastructure/kubernetes/overlays/staging/patches/services-nodeport.yaml` updated
- [x] `infrastructure/cloudflare/cloudflared-config.prod.yml` `epsx.io` and `admin.epsx.io` added
- [x] `infrastructure/env-manifest.sh` adds `REQUIRED_FRONTEND_BFF` / `REQUIRED_ADMIN_BFF`

These are the operator-only steps. None of them can be done from this
session.

## 1. Local verify (colima on the operator's machine)

```bash
# 1. Start colima with enough resources to actually build Rust.
colima start --cpu 4 --memory 6 --disk 30

# 2. Build the images.
cd /private/tmp/epsx-wave5
docker build -f apps/frontend/Dockerfile -t bff-frontend:local .
docker build -f apps/admin/Dockerfile    -t bff-admin:local .

# 3. Import into k3s (colima-epsx is the local k3s context).
kubectl --context colima-epsx create namespace epsx-local
docker save bff-frontend:local | sudo ctr -n k8s.io images import -
docker save bff-admin:local    | sudo ctr -n k8s.io images import -

# 4. Edit the kustomization image overrides temporarily to use the
#    local "bff-frontend:local" / "bff-admin:local" tags, or
#    `kubectl set image` after apply.
kubectl --context colima-epsx apply -k infrastructure/kubernetes/overlays/prod

# 5. Wait for rollout, then port-forward and curl.
kubectl --context colima-epsx -n epsx-prod rollout status deploy/epsx-frontend
kubectl --context colima-epsx -n epsx-prod rollout status deploy/epsx-admin
kubectl --context colima-epsx -n epsx-prod port-forward svc/epsx-frontend 3000:3000 &
curl -sf http://localhost:3000/api/health      # expect: ok
curl -sfI http://localhost:3000/ | head -5     # expect: 200 OK, Content-Type: text/html
```

If `curl http://localhost:3000/api/health` returns `ok` and the home page
renders a real HTML doc, the local verify is good.

## 2. Build prod images and push to registry

```bash
cd /private/tmp/epsx-wave5
# Use BuildKit to get the cache mounts.
DOCKER_BUILDKIT=1 docker build -f apps/frontend/Dockerfile \
  -t registry.jesadakorn.com/jesadakorn/epsx/bff-frontend:prod-$(date +%Y%m%d-%H%M%S) .
DOCKER_BUILDKIT=1 docker build -f apps/admin/Dockerfile \
  -t registry.jesadakorn.com/jesadakorn/epsx/bff-admin:prod-$(date +%Y%m%d-%H%M%S) .

docker push registry.jesadakorn.com/jesadakorn/epsx/bff-frontend:prod-YYYYMMDD-HHMMSS
docker push registry.jesadakorn.com/jesadakorn/epsx/bff-admin:prod-YYYYMMDD-HHMMSS

# Re-tag the new images as :prod-latest.
docker tag  <new-fe> registry.jesadakorn.com/jesadakorn/epsx/bff-frontend:prod-latest
docker tag  <new-ad> registry.jesadakorn.com/jesadakorn/epsx/bff-admin:prod-latest
docker push registry.jesadakorn.com/jesadakorn/epsx/bff-frontend:prod-latest
docker push registry.jesadakorn.com/jesadakorn/epsx/bff-admin:prod-latest
```

## 3. Create / update k8s Secrets

The Deployment `envFrom.secretRef` references `epsx-frontend-bff` and
`epsx-admin-bff`. Confirm these exist in `epsx-prod`:

```bash
kubectl -n epsx-prod get secret epsx-frontend-bff -o yaml
kubectl -n epsx-prod get secret epsx-admin-bff    -o yaml
```

If they don't exist, create them. The Rust BFFs need:

- `EPSX_ENABLE_DEMO_LOGIN` (frontend only) — `"0"` in prod.

That's it. The Dioxus side doesn't read any other env vars from the
secret; everything else (`API_URL`, `PORT`, `HOST`, `RUST_LOG`) is set
inline in the Deployment.

## 4. Apply to production

```bash
cd /private/tmp/epsx-wave5
kubectl -n epsx-prod apply -k infrastructure/kubernetes/overlays/prod
kubectl -n epsx-prod rollout status deploy/epsx-frontend
kubectl -n epsx-prod rollout status deploy/epsx-admin
```

## 5. Restart the cloudflared DaemonSet to pick up the new config

The new `cloudflared-config.prod.yml` has two new ingress entries
(`epsx.io` and `admin.epsx.io`). Apply the updated ConfigMap and roll
the DaemonSet:

```bash
kubectl -n epsx-prod create configmap cloudflared-config \
  --from-file=cloudflared-config.prod.yml=infrastructure/cloudflare/cloudflared-config.prod.yml \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl -n epsx-prod rollout restart daemonset cloudflared
kubectl -n epsx-prod rollout status    daemonset cloudflared
```

## 6. DNS

Add (or update) CNAMEs in Cloudflare:

- `epsx.io`       → `<tunnel-id>.cfargotunnel.com` (Proxied)
- `admin.epsx.io` → `<tunnel-id>.cfargotunnel.com` (Proxied)

The other two (`api.epsx.io`, `minio.epsx.io`) are already pointed at
the tunnel. No DNS change needed for them.

## 7. Smoke test

From a workstation outside the cluster:

```bash
# 7.1 Frontend health.
curl -sf https://epsx.io/api/health
# 7.2 Admin health.
curl -sf https://admin.epsx.io/api/health
# 7.3 Frontend HTML render.
curl -sfI https://epsx.io/ | head -5
# 7.4 Admin HTML render.
curl -sfI https://admin.epsx.io/ | head -5
# 7.5 BFF -> gateway path.
curl -sf https://epsx.io/api/v1/news | jq '.items | length'   # expect: a number, not an error
# 7.6 CORS pre-flight from a browser origin.
curl -sI -X OPTIONS https://api.epsx.io/api/v1/news \
  -H 'Origin: https://epsx.io' \
  -H 'Access-Control-Request-Method: GET' | head -10
#   expect: Access-Control-Allow-Origin: https://epsx.io
```

If any of these fail, the rollback is straightforward — the old
`frontend:prod-latest` image and `admin:prod-latest` image are still
in the registry (don't delete them yet); the old kustomize is in git
history at `migration/dioxus-microservices` HEAD~. `git revert` the
wave5 commit and `kubectl apply -k` the old overlay.

## 8. Post-cutover follow-ups (out of scope for this runbook)

- CORS allowlist needs a predicate for preview deploys (not blocking).
- Drop `apps-old/` after 1 week of stable production traffic.
- Resolve the 15 pre-existing dead-code warnings on the BFFs (admin
  BFF will start using `require_user` / `require_admin` once the
  admin auth handlers are ported to them).
- Re-introduce preview deploys (likely on Fly.io, not Vercel — Rust
  doesn't fit Vercel's runtime).
