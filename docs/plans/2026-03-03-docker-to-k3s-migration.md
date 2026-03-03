# Docker Compose Prod → k3s Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate all prod services from Docker Compose to the local k3d cluster, switch Cloudflare Tunnel routing to k3s, and remove the Docker Compose prod stack.

**Architecture:** The k3d cluster `epsx-local` is already running with `epsx-prod` namespace. All k8s manifests are already written in `infrastructure/kubernetes/`. Migration runs in 5 phases: push images → deploy k3s → migrate data → cut over cloudflared → decommission Docker.

**Tech Stack:** k3d v5.8.3, kubectl v1.34, kustomize, helm, Cloudflare Tunnel, PostgreSQL 17, Redis 7, MinIO

**Known values:**
- Mac Mini LAN IP: `192.168.0.33`
- Tunnel ID: `6bee9b58-eede-4b4c-815c-94c0ee38fe58`
- k3d cluster: `k3d-epsx-local-server-0`
- Env file: `/Users/fluke/epsx-runner/envs/.env.prod`

---

## Phase 1 — Images & Registry

### Task 1: Push prod images to GitLab registry

**Files:** none — pure Docker operations

**Step 1: Log in to the GitLab container registry**

```bash
docker login registry.jesadakorn.com
# Enter your GitLab username and a Personal Access Token (read_registry + write_registry scopes)
# Or your GitLab password directly
```

Expected: `Login Succeeded`

**Step 2: Tag and push backend**

```bash
docker tag epsx-backend:prod registry.jesadakorn.com/jesadakorn/epsx/backend:prod-latest
docker push registry.jesadakorn.com/jesadakorn/epsx/backend:prod-latest
```

Expected: push completes, shows digest

**Step 3: Tag and push frontend**

```bash
docker tag epsx-frontend:prod registry.jesadakorn.com/jesadakorn/epsx/frontend:prod-latest
docker push registry.jesadakorn.com/jesadakorn/epsx/frontend:prod-latest
```

**Step 4: Tag and push admin**

```bash
docker tag epsx-admin-frontend:prod registry.jesadakorn.com/jesadakorn/epsx/admin:prod-latest
docker push registry.jesadakorn.com/jesadakorn/epsx/admin:prod-latest
```

**Step 5: Verify all 3 tags exist in registry**

```bash
curl -sk -u "<user>:<token>" \
  "https://registry.jesadakorn.com/v2/jesadakorn/epsx/backend/tags/list"
# Expected: {"name":"...","tags":["prod-latest",...]}
```

---

### Task 2: Add imagePullSecrets to k8s deployments

**Files:**
- Modify: `infrastructure/kubernetes/base/backend/deployment.yaml`
- Modify: `infrastructure/kubernetes/base/frontend/deployment.yaml`
- Modify: `infrastructure/kubernetes/base/admin/deployment.yaml`
- Modify: `infrastructure/kubernetes/scripts/create-secrets.sh`

**Step 1: Add imagePullSecrets to backend deployment**

In `infrastructure/kubernetes/base/backend/deployment.yaml`, add after `spec:` inside `template.spec` (same level as `initContainers`):

```yaml
      imagePullSecrets:
        - name: epsx-registry
      initContainers:
```

**Step 2: Add imagePullSecrets to frontend deployment**

In `infrastructure/kubernetes/base/frontend/deployment.yaml`, add after `template.spec:` (same level as `containers:`):

```yaml
      imagePullSecrets:
        - name: epsx-registry
      containers:
```

**Step 3: Add imagePullSecrets to admin deployment**

Same change in `infrastructure/kubernetes/base/admin/deployment.yaml`.

**Step 4: Add registry secret creation to create-secrets.sh**

In `infrastructure/kubernetes/scripts/create-secrets.sh`, after the `epsx-cloudflared` block at the bottom, add:

```bash
# ── epsx-registry (image pull secret for all envs) ────────────────────────────
REGISTRY_CONFIG="${HOME}/.docker/config.json"
if [[ -f "$REGISTRY_CONFIG" ]]; then
  kubectl create secret generic epsx-registry \
    -n "$NAMESPACE" \
    --from-file=.dockerconfigjson="$REGISTRY_CONFIG" \
    --type=kubernetes.io/dockerconfigjson \
    --dry-run=client -o yaml | kubectl apply -f -
else
  echo "Warning: ~/.docker/config.json not found — epsx-registry secret skipped" >&2
fi
```

**Step 5: Commit**

```bash
cd /Users/fluke/Desktop/Work/epsx
git add infrastructure/kubernetes/base/backend/deployment.yaml \
        infrastructure/kubernetes/base/frontend/deployment.yaml \
        infrastructure/kubernetes/base/admin/deployment.yaml \
        infrastructure/kubernetes/scripts/create-secrets.sh
git commit -m "feat(k8s): add imagePullSecrets to app deployments"
```

---

## Phase 2 — k3s Secrets & Deploy

### Task 3: Fill cloudflared configmap placeholders

**Files:**
- Modify: `infrastructure/kubernetes/overlays/prod/cloudflared/configmap.yaml`

**Step 1: Replace TUNNEL_ID placeholder**

In `infrastructure/kubernetes/overlays/prod/cloudflared/configmap.yaml`, replace every occurrence of `<TUNNEL_ID>` with `6bee9b58-eede-4b4c-815c-94c0ee38fe58`.

**Step 2: Replace MAC_MINI_LAN_IP placeholder**

Replace every occurrence of `<MAC_MINI_LAN_IP>` with `192.168.0.33`.

**Step 3: Verify the file has no remaining placeholders**

```bash
grep -n "<TUNNEL_ID>\|<MAC_MINI_LAN_IP>" \
  infrastructure/kubernetes/overlays/prod/cloudflared/configmap.yaml
# Expected: no output
```

**Step 4: Commit**

```bash
git add infrastructure/kubernetes/overlays/prod/cloudflared/configmap.yaml
git commit -m "feat(k8s): set cloudflared tunnel ID and LAN IP for prod"
```

---

### Task 4: Create all k8s secrets

**Files:** none — kubectl operations

**Step 1: Run create-secrets.sh for prod**

```bash
cd /Users/fluke/Desktop/Work/epsx/infrastructure/kubernetes/scripts
./create-secrets.sh prod
```

Expected output ends with:
```
Done. Secrets created in epsx-prod:
  epsx-postgres
  epsx-redis
  epsx-minio
  epsx-backend
  epsx-frontend
  epsx-admin
  epsx-cloudflared
  epsx-registry
```

**Step 2: Verify all secrets exist**

```bash
kubectl get secrets -n epsx-prod
```

Expected: 8 secrets listed, all with TYPE `Opaque` or `kubernetes.io/dockerconfigjson`.

---

### Task 5: Deploy the prod overlay to k3s

**Files:** none — kubectl operations

**Step 1: Apply the kustomize overlay**

```bash
kubectl apply -k /Users/fluke/Desktop/Work/epsx/infrastructure/kubernetes/overlays/prod
```

Expected: output shows `created` or `configured` for namespace, PVCs, configmaps, services, deployments, statefulsets.

**Step 2: Wait for stateful services (postgres, redis, minio)**

```bash
kubectl wait --for=condition=ready pod/epsx-postgres-0 -n epsx-prod --timeout=120s
kubectl wait --for=condition=ready pod/epsx-redis-0    -n epsx-prod --timeout=60s
kubectl wait --for=condition=ready pod/epsx-minio-0    -n epsx-prod --timeout=60s
```

Expected: `pod/epsx-postgres-0 condition met` etc.

**Step 3: Check overall pod status**

```bash
kubectl get pods -n epsx-prod
```

Expected: postgres-0, redis-0, minio-0 are `Running 1/1`. App pods (backend, frontend, admin) may be `Running` or still in `Init` (migrate job). cloudflared pods should be `Running`.

**Note:** App pods will be healthy once data is migrated in Task 6-8. They may crash-loop briefly on empty postgres — that is expected and will resolve after restore.

---

## Phase 3 — Data Migration

### Task 6: Migrate PostgreSQL

**Files:** none — shell operations

**Step 1: Dump all databases from Docker**

```bash
mkdir -p /tmp/k8s-backup-prod
docker exec epsx-prod-postgres \
  pg_dumpall -U epsx_user \
  > /tmp/k8s-backup-prod/postgres.sql
echo "Dump size: $(du -sh /tmp/k8s-backup-prod/postgres.sql | cut -f1)"
```

Expected: file created, size shown (typically 10–200MB depending on data volume).

**Step 2: Wait for k3s postgres to be ready (if not already)**

```bash
kubectl wait --for=condition=ready pod/epsx-postgres-0 -n epsx-prod --timeout=120s
```

**Step 3: Copy dump into the k3s pod**

```bash
kubectl cp /tmp/k8s-backup-prod/postgres.sql \
  epsx-prod/epsx-postgres-0:/tmp/restore.sql
```

**Step 4: Restore all databases**

```bash
kubectl exec -n epsx-prod epsx-postgres-0 -- \
  psql -U epsx_user -f /tmp/restore.sql
```

Expected: many `CREATE DATABASE`, `ALTER ROLE`, `CREATE TABLE` etc. lines. Ignore `role already exists` warnings — those are normal.

**Step 5: Clean up dump from pod**

```bash
kubectl exec -n epsx-prod epsx-postgres-0 -- rm /tmp/restore.sql
```

**Step 6: Verify databases exist**

```bash
kubectl exec -n epsx-prod epsx-postgres-0 -- \
  psql -U epsx_user -c "\l"
```

Expected: lists `epsx_prod`, `epsx_analytics_prod`, `epsx_notifications_prod`, `epsx_payments_prod`.

---

### Task 7: Migrate Redis

**Files:** none — shell operations

**Step 1: Trigger BGSAVE on Docker Redis**

```bash
REDIS_PASS=$(grep "^REDIS_PASSWORD=" /Users/fluke/epsx-runner/envs/.env.prod | cut -d= -f2)
docker exec epsx-prod-redis redis-cli -a "$REDIS_PASS" BGSAVE
sleep 3
```

**Step 2: Copy RDB from Docker**

```bash
docker cp epsx-prod-redis:/data/dump.rdb /tmp/k8s-backup-prod/redis.rdb
echo "Redis dump: $(du -sh /tmp/k8s-backup-prod/redis.rdb | cut -f1)"
```

**Step 3: Scale k3s Redis to 0 (so it releases the RDB file)**

```bash
kubectl scale statefulset epsx-redis -n epsx-prod --replicas=0
kubectl wait --for=delete pod/epsx-redis-0 -n epsx-prod --timeout=60s 2>/dev/null || true
```

**Step 4: Find the Redis PVC path inside the k3d container**

```bash
REDIS_PVC=$(kubectl get pvc epsx-redis-data -n epsx-prod \
  -o jsonpath='{.spec.volumeName}')
REDIS_PVC_PATH=$(docker exec k3d-epsx-local-server-0 \
  find /var/lib/rancher/k3s/storage -maxdepth 1 -name "*${REDIS_PVC}*" -type d 2>/dev/null | head -1)
echo "PVC path: $REDIS_PVC_PATH"
```

Expected: prints a path like `/var/lib/rancher/k3s/storage/pvc-<uuid>_epsx-prod_epsx-redis-data`

**Step 5: Copy RDB into the PVC**

```bash
docker cp /tmp/k8s-backup-prod/redis.rdb \
  "k3d-epsx-local-server-0:${REDIS_PVC_PATH}/dump.rdb"
```

**Step 6: Scale Redis back to 1**

```bash
kubectl scale statefulset epsx-redis -n epsx-prod --replicas=1
kubectl wait --for=condition=ready pod/epsx-redis-0 -n epsx-prod --timeout=60s
```

**Step 7: Verify Redis has data**

```bash
REDIS_PASS=$(grep "^REDIS_PASSWORD=" /Users/fluke/epsx-runner/envs/.env.prod | cut -d= -f2)
kubectl exec -n epsx-prod epsx-redis-0 -- \
  redis-cli -a "$REDIS_PASS" DBSIZE
```

Expected: non-zero number (e.g., `42`). If `0`, data did not restore — check PVC path was correct.

---

### Task 8: Migrate MinIO

**Files:** none — shell operations

**Step 1: Tar MinIO data from Docker volume**

```bash
docker run --rm \
  -v epsx_prod_minio_data:/src \
  -v /tmp/k8s-backup-prod:/backup \
  alpine tar -czf /backup/minio.tar.gz -C /src .
echo "MinIO dump: $(du -sh /tmp/k8s-backup-prod/minio.tar.gz | cut -f1)"
```

**Step 2: Wait for k3s MinIO to be ready**

```bash
kubectl wait --for=condition=ready pod/epsx-minio-0 -n epsx-prod --timeout=60s
```

**Step 3: Copy tar into MinIO pod**

```bash
kubectl cp /tmp/k8s-backup-prod/minio.tar.gz \
  epsx-prod/epsx-minio-0:/tmp/minio.tar.gz
```

**Step 4: Extract into /data**

```bash
kubectl exec -n epsx-prod epsx-minio-0 -- \
  tar -xzf /tmp/minio.tar.gz -C /data
kubectl exec -n epsx-prod epsx-minio-0 -- \
  rm /tmp/minio.tar.gz
```

**Step 5: Verify bucket list**

```bash
kubectl exec -n epsx-prod epsx-minio-0 -- \
  mc alias set local http://localhost:9000 \
    "$(kubectl get secret epsx-minio -n epsx-prod -o jsonpath='{.data.MINIO_ROOT_USER}' | base64 -d)" \
    "$(kubectl get secret epsx-minio -n epsx-prod -o jsonpath='{.data.MINIO_ROOT_PASSWORD}' | base64 -d)" \
  && mc ls local/
```

Expected: your existing buckets listed.

---

### Task 9: Verify app pods are healthy after data migration

**Files:** none

**Step 1: Check all pods**

```bash
kubectl get pods -n epsx-prod
```

Expected: all pods `Running` with `1/1` (or `2/2` if replicas=2). No `CrashLoopBackOff`.

If backend pods are still crashing, check logs:
```bash
kubectl logs -n epsx-prod -l app=epsx-backend --tail=50
```

**Step 2: Smoke-test backend health inside the cluster**

```bash
kubectl exec -n epsx-prod deploy/epsx-backend -- \
  curl -sf http://localhost:8080/health
```

Expected: `{"status":"ok"}` or similar JSON.

---

## Phase 4 — Cloudflared Cutover

### Task 10: Cut over Cloudflare Tunnel from Docker to k3s

**Files:** none — Docker/network operations

**IMPORTANT:** This causes ~5 seconds of downtime. Do this during low-traffic period.

**Step 1: Confirm k3s cloudflared pods are Running**

```bash
kubectl get pods -n epsx-prod -l app=epsx-cloudflared
```

Expected: 2 pods `Running 1/1`.

**Step 2: Stop Docker cloudflared**

```bash
docker stop epsx-prod-cloudflared
```

The k3s cloudflared immediately takes over (it was already connected to Cloudflare's network).

**Step 3: Verify prod URLs (give ~10s for DNS/routing to settle)**

```bash
sleep 10
curl -sf https://api.epsx.io/health
echo "Backend: $?"

curl -s -o /dev/null -w "%{http_code}" https://epsx.io
echo " <- epsx.io"

curl -s -o /dev/null -w "%{http_code}" https://admin.epsx.io
echo " <- admin.epsx.io"
```

Expected:
- Backend: `{"status":"ok"}` (exit 0)
- `epsx.io`: 200
- `admin.epsx.io`: 307 (auth redirect = OK)

**Step 4: Verify GitLab is still reachable (still on Docker)**

```bash
curl -s -o /dev/null -w "%{http_code}" https://gitlab.jesadakorn.com/-/health
echo " <- gitlab"
```

Expected: 200

If any check fails, restore Docker cloudflared immediately:
```bash
docker start epsx-prod-cloudflared
# Then diagnose before retrying
```

---

## Phase 5 — Decommission Docker Compose Prod

### Task 11: Remove Docker Compose prod stack and old cloudflared config

**Files:**
- Delete: `infrastructure/cloudflare/cloudflared-config.prod.yml`

**Step 1: Confirm everything works — wait 5 minutes of production traffic**

Monitor `kubectl logs -n epsx-prod -l app=epsx-backend --tail=20 -f` for a few minutes to confirm no errors.

**Step 2: Stop and remove prod Docker Compose stack**

```bash
cd /Users/fluke/Desktop/Work/epsx/infrastructure/docker
docker compose --env-file .env.prod -f docker-compose.prod.yml down
```

Expected: removes containers `epsx-prod-postgres`, `epsx-prod-redis`, `epsx-prod-minio`, `epsx-prod-frontend`, `epsx-prod-admin`, `epsx-prod-backend`, and the `epsx_prod_network`. Volumes are preserved.

**Step 3: Confirm no epsx-prod-* containers remain**

```bash
docker ps | grep epsx-prod
```

Expected: no output.

**Step 4: Delete old cloudflared config file (routing now lives in k8s)**

```bash
git rm /Users/fluke/Desktop/Work/epsx/infrastructure/cloudflare/cloudflared-config.prod.yml
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(infra): migrate prod from Docker Compose to k3s

- All prod services now running in k3d cluster (epsx-local)
- Cloudflare Tunnel routing moved to k8s cloudflared deployment
- Docker Compose prod stack decommissioned
- Old cloudflared-config.prod.yml removed (replaced by k8s configmap)"
```

---

## Phase 6 — Update CI/CD Pipeline

### Task 12: Update .gitlab-ci.yml to deploy via k3s instead of Docker Compose

**Files:**
- Modify: `.gitlab-ci.yml` — find the `deploy:prod` job

**Step 1: Find the deploy:prod job**

```bash
grep -n "deploy:prod\|docker compose.*prod\|epsx-backend:prod" .gitlab-ci.yml | head -20
```

**Step 2: Replace Docker Compose deploy with registry push + kubectl rollout**

In the `deploy:prod` job, replace the steps that:
- `docker tag ... epsx-backend:prod`
- `docker compose ... up -d --force-recreate`

With:
```yaml
    - docker tag $BACKEND_IMAGE registry.jesadakorn.com/jesadakorn/epsx/backend:prod-latest
    - docker tag $FRONTEND_IMAGE registry.jesadakorn.com/jesadakorn/epsx/frontend:prod-latest
    - docker tag $ADMIN_IMAGE registry.jesadakorn.com/jesadakorn/epsx/admin:prod-latest
    - docker push registry.jesadakorn.com/jesadakorn/epsx/backend:prod-latest
    - docker push registry.jesadakorn.com/jesadakorn/epsx/frontend:prod-latest
    - docker push registry.jesadakorn.com/jesadakorn/epsx/admin:prod-latest
    - kubectl rollout restart deployment/epsx-backend  -n epsx-prod
    - kubectl rollout restart deployment/epsx-frontend -n epsx-prod
    - kubectl rollout restart deployment/epsx-admin    -n epsx-prod
    - kubectl rollout status  deployment/epsx-backend  -n epsx-prod --timeout=120s
    - kubectl rollout status  deployment/epsx-frontend -n epsx-prod --timeout=120s
    - kubectl rollout status  deployment/epsx-admin    -n epsx-prod --timeout=120s
```

**Step 3: Verify the job also removes the verification steps that check Docker containers**

Remove or replace any steps like:
```bash
for c in epsx-prod-backend epsx-prod-frontend epsx-prod-admin; do
  docker inspect ...
```

Replace with:
```bash
kubectl get pods -n epsx-prod
```

**Step 4: Commit**

```bash
git add .gitlab-ci.yml
git commit -m "ci: update deploy:prod to push prod-latest to registry and rollout k3s"
```

---

## Verification Checklist (final)

```bash
# 1. All k3s pods running
kubectl get pods -n epsx-prod

# 2. Backend health
curl -sf https://api.epsx.io/health

# 3. Frontend accessible
curl -s -o /dev/null -w "%{http_code}" https://epsx.io

# 4. Admin accessible (307 = auth redirect = OK)
curl -s -o /dev/null -w "%{http_code}" https://admin.epsx.io

# 5. GitLab still up (Docker)
curl -s -o /dev/null -w "%{http_code}" https://gitlab.jesadakorn.com/-/health

# 6. No prod Docker containers
docker ps | grep epsx-prod   # expect: no output

# 7. Cloudflared running in k3s
kubectl get pods -n epsx-prod -l app=epsx-cloudflared
```
