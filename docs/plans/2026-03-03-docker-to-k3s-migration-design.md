# Docker Compose Prod → k3s Migration Design

**Date:** 2026-03-03
**Status:** Approved

## Context

Production currently runs as 7 Docker Compose containers on Mac Mini (arm64, 48GB RAM) with Cloudflare Tunnel for external access. A k3d (k3s-in-Docker) cluster `epsx-local` is already running with the `epsx-prod` namespace created. All k8s manifests are complete under `infrastructure/kubernetes/`.

Goal: migrate prod from Docker Compose to k3s, switch Cloudflare Tunnel routing to k3s services, and remove the Docker Compose prod stack.

## What Stays on Docker

- `epsx-gitlab` + `epsx-gitlab-runner` (gitlab-central compose)
- `epsx-traefik` (review apps)
- Dev and staging environments

The k3s cloudflared routes these through the Mac Mini LAN IP (`192.168.0.33`).

## Migration Phases

### Phase 1 — Images & Registry
- Tag local prod images (`epsx-frontend:prod` etc.) as `registry.jesadakorn.com/jesadakorn/epsx/*:prod-latest`
- Push to GitLab registry
- Create k8s `imagePullSecret` (`epsx-registry`) in `epsx-prod` namespace
- Add `imagePullSecrets` to the 3 app deployments (backend, frontend, admin)

### Phase 2 — k3s Secrets & Deploy
- Run `infrastructure/kubernetes/scripts/create-secrets.sh prod`
- Fill real values in cloudflared configmap: `<TUNNEL_ID>` → `6bee9b58-eede-4b4c-815c-94c0ee38fe58`, `<MAC_MINI_LAN_IP>` → `192.168.0.33`
- `kubectl apply -k infrastructure/kubernetes/overlays/prod`
- Wait for postgres, redis, minio pods Running (empty volumes)

### Phase 3 — Data Migration
- PostgreSQL: `pg_dumpall` from Docker → `kubectl cp` + `psql` restore in k3s pod
- Redis: `BGSAVE` → `docker cp` RDB → scale k3s redis to 0 → copy RDB to PVC → scale to 1
- MinIO: `docker run --rm -v epsx_prod_minio_data:/src alpine tar` → `kubectl cp` + extract

### Phase 4 — Atomic Cloudflared Cutover
- `docker stop epsx-prod-cloudflared` (old routing dies)
- k3s cloudflared (deployed in Phase 2) takes over immediately
- Verify `epsx.io`, `admin.epsx.io`, `api.epsx.io` respond

### Phase 5 — Decommission
- `docker compose --env-file .env.prod -f docker-compose.prod.yml down` (removes containers, keeps volumes)
- Delete `infrastructure/docker/cloudflared-config.prod.yml` (routing now in k8s configmap)

## Key Files Changed

| File | Change |
|------|--------|
| `base/backend/deployment.yaml` | Add `imagePullSecrets: [{name: epsx-registry}]` |
| `base/frontend/deployment.yaml` | Add `imagePullSecrets` |
| `base/admin/deployment.yaml` | Add `imagePullSecrets` |
| `overlays/prod/cloudflared/configmap.yaml` | Replace `<TUNNEL_ID>` and `<MAC_MINI_LAN_IP>` |
| `infrastructure/docker/cloudflared-config.prod.yml` | Delete |

## Cloudflared Routing (post-migration)

The k3s cloudflared handles all environments from one pod:
- **prod**: k8s service FQDNs (`epsx-frontend.epsx-prod.svc.cluster.local`)
- **dev/staging/review**: Docker via `192.168.0.33` host ports
- **gitlab/registry**: Docker via `192.168.0.33:8929` / `:5050`

## Verification Checklist

1. `kubectl get pods -n epsx-prod` — all pods Running
2. `curl https://api.epsx.io/health` — backend responds
3. `curl -o /dev/null -w "%{http_code}" https://epsx.io` — 200
4. `curl -o /dev/null -w "%{http_code}" https://admin.epsx.io` — 307 (auth redirect = OK)
5. `curl https://gitlab.jesadakorn.com/-/health` — GitLab still reachable (Docker)
6. `docker ps | grep epsx-prod` — no prod containers running
