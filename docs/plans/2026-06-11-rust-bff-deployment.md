# 2026-06-11 — Rust BFF Deployment Plan

> **Supersedes** `2026-04-16-vercel-hybrid-deployment.md`. That plan was written for
> the pre-migration Next.js apps. After Wave 1–4 the user-facing apps
> (`apps/frontend`, `apps/admin`) are Rust binaries (`bff-frontend`, `bff-admin`)
> — Dioxus 0.7 fullstack SSR + Axum. Vercel is no longer a valid target.

## Target topology

| Component     | Runtime         | Deployment target                | Public entrypoint     | Notes |
| ------------- | --------------- | -------------------------------- | --------------------- | ----- |
| `bff-frontend`| Rust/Axum/Dioxus| Self-hosted on k3s (Docker image) | `https://epsx.io`     | SSR for the user-facing site + JSON `/api/*` proxy to the gateway. |
| `bff-admin`   | Rust/Axum/Dioxus| Self-hosted on k3s (Docker image) | `https://admin.epsx.io` | SSR for the admin dashboard + JSON `/api/*` proxy. |
| `apps/backend`| Rust/Axum (existing) | Local k3s (unchanged)        | `https://api.epsx.io` | System of record — **stays where it is**. |
| `apps/contracts` | Foundry      | Local + on-chain                 | n/a                   | Out of scope. |

Both BFFs are stateless. They proxy every authenticated request to the gateway
via `epsx_client::ServiceClient`, then SSR the response with Dioxus.

## Repo changes in this wave

| Path | What | Why |
| --- | --- | --- |
| `apps/frontend/Dockerfile` | NEW — multi-stage `rust:slim-bookworm` → `debian:bookworm-slim` | Build the BFF binary in CI, run it with a non-root user, ship the `public/` static dir. |
| `apps/admin/Dockerfile`    | NEW — same shape as frontend | Same. |
| `apps/frontend/.dockerignore` | NEW | Exclude `target/`, `.git/`, `apps-old/`, etc. so the build context is small. |
| `apps/admin/.dockerignore`    | NEW | Same. |
| `infrastructure/kubernetes/base/frontend/deployment.yaml` | REWRITE | Old manifest pointed at the dead `frontend:prod-latest` Next.js image on port 3000. New manifest points at `bff-frontend:prod-latest` on port 3000, with the correct env vars and probe path. |
| `infrastructure/kubernetes/base/frontend/service.yaml`    | KEEP   | Port 3000 → port 3000 still correct for `bff-frontend`. |
| `infrastructure/kubernetes/base/admin/deployment.yaml`    | REWRITE | Same — `bff-admin` on port 3001. |
| `infrastructure/kubernetes/base/admin/service.yaml`       | KEEP   | Port 3001 → port 3001. |
| `infrastructure/kubernetes/overlays/prod/{frontend,admin}/kustomization.yaml` | UPDATE | New image tag + env source (Secret). |
| `infrastructure/cloudflare/cloudflared-config.prod.yml`   | UPDATE | Frontend tunnel points at the new `epsx-frontend` Service on port 3000; admin tunnel points at `epsx-admin` on port 3001. |
| `infrastructure/env-manifest.sh`                          | UPDATE | Add `REQUIRED_FRONTEND_BFF` and `REQUIRED_ADMIN_BFF` lists. |
| `docs/plans/2026-04-16-vercel-hybrid-deployment.md`       | DEPRECATE | Mark with a header pointing at this plan. |
| `docs/runbooks/2026-06-11-rust-bff-cutover.md`           | NEW | Step-by-step runbook the operator (you) follows. |

## BFF runtime contract

Both BFFs read the same env-var shape:

| Variable     | Required | Default                    | Notes |
| ------------ | -------- | -------------------------- | ----- |
| `API_URL`    | yes (prod) | `http://localhost:8080` (frontend) / `http://localhost:18081` (admin) | Where the BFF proxies `/api/*` requests. In prod this is `https://api.epsx.io` for both (the gateway ingress). |
| `PORT`       | no       | `3000` (frontend) / `3001` (admin) | Listen port. |
| `HOST`       | no       | `0.0.0.0`                  | Listen address. |
| `EPSX_ENABLE_DEMO_LOGIN` | no (frontend only) | unset | Set to `1` in non-prod to allow the demo login endpoint. |
| `RUST_LOG`   | no       | `info`                     | Standard `tracing` filter. |

The `apps/backend` already requires `BACKEND_URL`, `FRONTEND_URL`,
`ADMIN_FRONTEND_URL`. After the cutover:

- `FRONTEND_URL=https://epsx.io`
- `ADMIN_FRONTEND_URL=https://admin.epsx.io`

These are read by the gateway's CORS allowlist (it does literal `==`
comparisons on the Origin header). **Preview-deploy CORS is a known
follow-up** — the allowlist needs a predicate or wildcard for
`*.vercel.app` and similar. Not blocking production.

## Container shape

### `apps/frontend/Dockerfile` (and `apps/admin/Dockerfile`)

Pattern (mirrors `apps/backend/Dockerfile`):

1. `builder` stage: `rust:slim-bookworm`. Mount cargo cache. Stub `src/main.rs`
   so dependency resolution is cached, then copy real source and rebuild
   only the changed binary. Build the right `--bin`:
   - frontend: `--bin bff-frontend`
   - admin:    `--bin bff-admin`
2. `runtime` stage: `debian:bookworm-slim`. `ca-certificates` + `curl` (for
   the HEALTHCHECK). Non-root `nonroot` user (uid 1001). The static `public/`
   directory is `COPY`'d next to the binary — `tower_http::services::ServeDir`
   reads it from CWD, so we run with `WORKDIR /app`.

Frontend binary: `target/release/bff-frontend`, port 3000, probes
`/api/health`.

Admin binary: `target/release/bff-admin`, port 3001, probes
`/api/health`.

Both have `HEALTHCHECK` inline so `docker inspect` and the k8s
`livenessProbe` agree.

## k3s manifests (new shape)

`infrastructure/kubernetes/base/frontend/deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: epsx-frontend
spec:
  replicas: 2
  selector:
    matchLabels: { app: epsx-frontend }
  template:
    metadata:
      labels: { app: epsx-frontend }
    spec:
      containers:
        - name: epsx-frontend
          image: registry.jesadakorn.com/jesadakorn/epsx/bff-frontend:prod-latest
          imagePullPolicy: IfNotPresent
          envFrom:
            - secretRef: { name: epsx-frontend-bff }
          env:
            - { name: PORT,  value: "3000" }
            - { name: HOST,  value: "0.0.0.0" }
            - { name: API_URL, value: "https://api.epsx.io" }
            - { name: RUST_LOG, value: "info" }
          ports:
            - { containerPort: 3000 }
          livenessProbe:
            httpGet: { path: /api/health, port: 3000 }
            initialDelaySeconds: 20
            periodSeconds: 15
            timeoutSeconds: 5
            failureThreshold: 3
          readinessProbe:
            httpGet: { path: /api/health, port: 3000 }
            initialDelaySeconds: 10
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
          volumeMounts:
            - { name: tmp, mountPath: /tmp }
          resources:
            limits:   { memory: 1Gi, cpu: "1" }
            requests: { memory: 256Mi, cpu: "0.25" }
      volumes:
        - name: tmp
          emptyDir: { sizeLimit: 100Mi }
```

`infrastructure/kubernetes/base/admin/deployment.yaml` mirrors it with
`bff-admin:prod-latest`, port 3001, the `epsx-admin-bff` secret, and a
slightly larger request (admin renders wider pages).

The base `service.yaml` for both is unchanged — ClusterIP, targetPort =
containerPort.

## Cloudflare Tunnel

`infrastructure/cloudflare/cloudflared-config.prod.yml` already has the
shape. The cutover:

- Existing `epsx-frontend` tunnel entry that points at
  `http://epsx-frontend:3000` — keep, it just becomes correct once the
  new Deployment is rolled.
- Existing `epsx-admin` tunnel entry — same.
- The tunnels run **inside the cluster** (cloudflared DaemonSet already
  exists in the namespace), so they don't care what container the Service
  points to as long as the Service name + port are stable. The Service
  names in the new manifests match (`epsx-frontend` / `epsx-admin`), so
  no tunnel change is required for the production hostname routing.
- DNS records on Cloudflare (`epsx.io` CNAME → the tunnel) do not need
  to move.

## CORS for preview deploys (deferred)

Not in scope this wave. The Rust gateway's CORS allowlist uses
explicit origin comparison, so previews like
`epsx-frontend-git-wave5-rust-bff-deploy-fluke-jesadakorn.vercel.app`
will be rejected by the browser. Production custom domains work
unchanged. We'll address preview CORS when we re-introduce preview
deploys (likely on a different host than Vercel).

## Build / verify sequence (per-track, for the team plan)

Each track runs `cargo check --workspace` plus its own crate's
`cargo test --lib` to satisfy the per-track verify cap. The integration
gate (last step) does:

1. `docker build -f apps/frontend/Dockerfile -t bff-frontend:local .`
2. `docker build -f apps/admin/Dockerfile -t bff-admin:local .`
3. `colima start --cpu 4 --memory 6 --disk 30` (or the local cluster equivalent)
4. `docker save` each image → `ctr -n k8s.io images import` (k3s) or
   `kind load docker-image` (kind)
5. `kubectl apply -k infrastructure/kubernetes/overlays/prod`
6. Wait for `kubectl rollout status` to settle.
7. `kubectl port-forward svc/epsx-frontend 3000:3000` and `curl
   http://localhost:3000/api/health` (expect `ok`). Same for admin.
8. From a browser pointed at `http://localhost:3000/`, load the home
   page — Dioxus SSR should render a real HTML doc with the design
   system.

If the integration gate passes, the same manifests apply to the
production k3s cluster via `kubectl apply -k
infrastructure/kubernetes/overlays/prod` with the right image tag and
secret.

## Open questions for the operator (you)

1. **Image registry path** — used `registry.jesadakorn.com/jesadakorn/epsx/bff-frontend`
   to match the existing `frontend` and `admin` images. Confirm the
   path/credentials are right for the new images.
2. **Pull credentials** — does the cluster already have an `imagePullSecret`
   for that registry, or does the namespace need one created?
3. **Secret per BFF** — `epsx-frontend-bff` and `epsx-admin-bff` are
   referenced as `envFrom.secretRef`. Are these names right (the current
   manifests use `epsx-frontend` / `epsx-admin` for the old env)?
4. **CORS for the admin** — admin calls the gateway with credentials;
   confirm `admin.epsx.io` is in the gateway's allowlist.

## Out of scope (deferred)

- Preview-deploy CORS
- Replacing `apps/backend` Dockerfile (already fine, no changes)
- Dropping `apps-old/`
- Cloudflare Tunnel DaemonSet changes (the service names are stable)
