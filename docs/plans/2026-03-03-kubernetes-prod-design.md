# Kubernetes Production Design — EPSX

**Date:** 2026-03-03
**Status:** Implemented

## Context

EPSX migrates from single-node Docker Compose (Mac Mini + Cloudflare Tunnel) to a multi-node k3s cluster with pull-based GitOps via GitLab Agent.

## Architecture

- **K8s distro**: k3s (arm64 + amd64 multi-arch)
- **Manifests**: Kustomize base + overlays (`infrastructure/kubernetes/`)
- **Deployment**: GitLab Agent for Kubernetes (pull-based GitOps)
- **Secrets**: GitLab CI masked variables → `kubectl create secret` (push-based, secrets only)
- **Ingress**: Cloudflare Tunnel (cloudflared Deployment, 2 replicas for HA)
- **Storage**: StatefulSets + PVCs on k3s `local-path` provisioner (Mac Mini)

## Cluster Topology

```
Mac Mini (arm64, 48GB RAM, 16 CPU)   Cloud VPS (amd64)
  Control Plane + Worker               Worker Node
  ─────────────────────────            ──────────────────
  PostgreSQL StatefulSet               Frontend (x2)
  Redis StatefulSet                    Admin (x2)
  MinIO StatefulSet                    Backend (x2)
  Cloudflared Deployment (x2)
```

Node labels:
- Mac Mini: `node-role=stateful`, `kubernetes.io/arch=arm64`
- VPS: `node-role=stateless`, `kubernetes.io/arch=amd64`

Stateful workloads use `nodeAffinity` to pin to Mac Mini (fast local storage).
Stateless workloads spread across both nodes via default scheduler.

## Directory Structure

```
infrastructure/kubernetes/
├── base/
│   ├── kustomization.yaml
│   ├── namespace.yaml
│   ├── backend/
│   │   ├── deployment.yaml      # init container runs ./migrate
│   │   └── service.yaml
│   ├── frontend/
│   │   ├── deployment.yaml
│   │   └── service.yaml
│   ├── admin/
│   │   ├── deployment.yaml
│   │   └── service.yaml
│   ├── postgres/
│   │   ├── statefulset.yaml     # nodeAffinity: stateful
│   │   ├── service.yaml
│   │   ├── configmap-init.yaml
│   │   └── configmap-postgres.yaml
│   ├── redis/
│   │   ├── statefulset.yaml     # nodeAffinity: stateful
│   │   ├── service.yaml
│   │   └── configmap.yaml
│   └── minio/
│       ├── statefulset.yaml     # nodeAffinity: stateful
│       └── service.yaml
└── overlays/
    ├── prod/
    │   ├── kustomization.yaml   # image tags updated by CI
    │   ├── pvcs.yaml            # 200Gi postgres, 100Gi minio, 10Gi redis
    │   ├── cloudflared/
    │   │   ├── deployment.yaml  # 2 replicas
    │   │   ├── service.yaml
    │   │   └── configmap.yaml
    │   └── patches/
    │       ├── replicas.yaml    # prod replica counts (2x stateless)
    │       ├── resources.yaml   # prod resource limits
    │       ├── postgres-conf.yaml
    │       └── redis-conf.yaml
    ├── staging/
    └── dev/

.gitlab/agents/epsx-prod/config.yaml   # GitLab Agent GitOps config
```

## Service Specs

| Service | Kind | Replicas | Mem Limit | CPU Limit | Node Affinity |
|---------|------|----------|-----------|-----------|---------------|
| backend | Deployment | 2 | 2Gi | 4 | any |
| frontend | Deployment | 2 | 1Gi | 2 | any |
| admin | Deployment | 2 | 1Gi | 2 | any |
| postgres | StatefulSet | 1 | 6Gi | 4 | stateful (Mac Mini) |
| redis | StatefulSet | 1 | 1Gi | 2 | stateful (Mac Mini) |
| minio | StatefulSet | 1 | 1Gi | 2 | stateful (Mac Mini) |
| cloudflared | Deployment | 2 | 256Mi | 1 | any |

## Key Design Decisions

### Database Migrations
Backend Deployment uses an init container (`migrate`) that runs `./migrate` before the main container starts, ensuring DB schema is up-to-date on every deploy.

### Multi-arch Images
CI build jobs use `docker buildx --platform linux/arm64,linux/amd64` so images run on both Mac Mini (arm64) and VPS (amd64) nodes.

### Cloudflared Ingress
cloudflared routes Cloudflare Tunnel traffic to K8s Services:
- `epsx.io` → `epsx-frontend:3000`
- `admin.epsx.io` → `epsx-admin:3000`
- `api.epsx.io` → `epsx-backend:8080`
- `cdn.epsx.io` / `minio.epsx.io` → `epsx-minio:9000`
- `storage.epsx.io` → `epsx-minio:9001`

### Secrets Management
All secrets stored in GitLab CI masked/protected variables. `deploy:k8s:secrets` CI job creates K8s secrets from those variables. Secret values never committed to Git.

### GitOps Flow
1. CI builds multi-arch images, pushes to `registry.jesadakorn.com`
2. `deploy:k8s:tag` job updates image tag in `overlays/prod/kustomization.yaml` and commits
3. GitLab Agent detects change, pulls updated manifests, applies to cluster
4. k8s rolling update: drains old pods, starts new ones with init-container migrations

## Cluster Setup

### Phase 1: Install k3s

```bash
# Mac Mini (control plane + worker)
curl -sfL https://get.k3s.io | sh -

# Cloud VPS (worker node)
curl -sfL https://get.k3s.io | K3S_URL=https://<mac-mini-ip>:6443 K3S_TOKEN=<token> sh -

# Label nodes
kubectl label node <mac-mini> node-role=stateful kubernetes.io/arch=arm64
kubectl label node <vps> node-role=stateless kubernetes.io/arch=amd64
```

### Phase 2: Install GitLab Agent

```bash
helm repo add gitlab https://charts.gitlab.io
helm install gitlab-agent gitlab/gitlab-agent \
  --namespace gitlab-agent --create-namespace \
  --set config.token=<agent-token> \
  --set config.kasAddress=wss://kas.jesadakorn.com
```

### Phase 3: Initial Deployment

```bash
# Apply secrets
./infrastructure/kubernetes/scripts/create-secrets.sh prod

# Tag trigger (or run deploy:k8s:tag CI job)
cd infrastructure/kubernetes/overlays/prod
kustomize edit set image \
  registry.jesadakorn.com/jesadakorn/epsx/backend:prod-<sha> \
  registry.jesadakorn.com/jesadakorn/epsx/frontend:prod-<sha> \
  registry.jesadakorn.com/jesadakorn/epsx/admin:prod-<sha>
git commit -am "deploy: prod <sha>"
git push
```

## Verification

```bash
# Cluster health
kubectl get nodes

# Pod health
kubectl get pods -n epsx-prod

# Migration logs
kubectl logs -n epsx-prod <backend-pod> -c migrate

# Service health
curl https://api.epsx.io/health
curl -o /dev/null -w "%{http_code}" https://epsx.io        # 200
curl -o /dev/null -w "%{http_code}" https://admin.epsx.io  # 307

# GitOps working
kubectl rollout status deployment/epsx-backend -n epsx-prod
```
