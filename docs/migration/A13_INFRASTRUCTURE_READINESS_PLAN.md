# A13.0 non-production infrastructure readiness gate

Status: **artifact integrity can pass; deployment readiness is STOP**. This audit renders checked-in manifests locally. It never contacts a cluster, reads Kubernetes secrets, applies resources, changes Cloudflare/DNS, or authorizes a deployment.

The machine contract is [`contracts/infrastructure-readiness.json`](contracts/infrastructure-readiness.json). A13 remains dependent on completion of the P0 ledger plus the P1 domain selected for a future canary. Current evidence is A0 passed; A1, A2, A4, and A5 partial; and A3 and A6 blocked. The aggregate release dependency is therefore incomplete.

## Current rendered topology

The production overlay renders 15 resources: one namespace, seven Services, and seven Deployments. The observed public payment path is:

```text
pay.epsx.io
  -> Cloudflare localhost:4752
  -> socat 4752 -> NodePort 30085
  -> epsx-pay-bff:3002

internal pay service (not public)
  -> ClusterIP `epsx-pay-svc:8103`
```

| Deployment | Rendered image | Replica | Readiness | Important gap |
|---|---|---:|---|---|
| `epsx-admin` | `epsx-admin-frontend:prod` | 1 | HTTP `/api/health` | mutable tag, no digest, shallow |
| `epsx-analytics` | `epsx-analytics:wave12` | 1 | HTTP `/health` | mutable tag; identity dependencies not gated |
| `epsx-backend` | `epsx-backend:prod` (init and app) | 1 | HTTP `/health` | mutable tag; dependency depth unproven |
| `epsx-frontend` | `epsx-frontend:prod` | 1 | HTTP `/api/health` | mutable tag, no digest, shallow |
| `epsx-identity` | `epsx-identity:dev` | 1 | TCP `50051` | SSE `50052` is not checked; deployed stub is not candidate HTTP identity |
| `epsx-pay-bff` | `epsx-pay-bff:wave49` | 1 | HTTP `/api/health` | mutable tag; authenticated live ingress remains unproven |
| `epsx-pay-svc` | `epsx-pay-svc:wave49` | 1 | HTTP `/health` | raw NodePort, literal DB credentials, zero escrow, no webhook env |

All eight image occurrences use `IfNotPresent`; none uses an immutable digest. The six declared prod image-transform keys now exactly match the base image names and apply four visible replacements, while identity has no prod transform and remains the single `:dev` occurrence. This closes only the static key-resolution defect; mutable tags and image provenance remain blockers.

## Exposure, state, and dependency findings

- Prod NodePorts are `30000` frontend, `30001` admin, `30080` backend, `30081` analytics, `30084` pay service, and `30085` pay BFF. Identity remains ClusterIP on `50051/50052`.
- Prod and staging now declare distinct pay NodePort pairs (`30084/30085` vs `30082/30083`); the checked-in bridge maps prod through `4751/4752` and staging through `4747/4748`. Runtime cluster allocation and bridge reachability remain unproven.
- The prod Cloudflare artifact now maps pay through the BFF bridge (`4752 -> 30085`); it still contains no checked-in `epsx.io`, `admin.epsx.io`, or analytics ingress mapping. Other Cloudflare files describe different local topologies, so one reviewed ingress authority is not established.
- Backend/frontend/admin use rendered Secret references. Pay instead embeds a PostgreSQL URL containing credentials, renders `ESCROW_CONTRACT=0`, has no Secret reference, and has no webhook configuration. Rendered Secret resources are absent; existence and key compatibility are not proven by this artifact audit.
- The production base does not deploy candidate gateway, HTTP identity, wallet, subscription, content, notification, event-tracking analytics, or indexer services. The rendered analytics and identity deployments are different implementations documented in the production plan.
- Each production Deployment has liveness and readiness, but the probes remain shallow process endpoints or one TCP port and none has a startup probe. A dev/staging-only notification manifest has a bounded startup probe, but it is intentionally absent from the production render. The production manifests do not yet gate database, Redis, chain RPC, identity, webhook, or downstream readiness.
- Every Deployment is one replica. No checked-in strategy, disruption budget, topology spread, shadow route, traffic split, SLO abort threshold, immutable previous revision, or rehearsed rollback artifact closes the release loop.

## Required execution order

1. Preserve A0’s passing evidence; complete A1, A2, A4, and A5; unblock A3 and A6; then link the reviewed P0 ledger. Select only a P1 vertical slice whose contract, data, authorization, interaction, and recovery gates pass.
2. Make image transforms match base names and resolve each init/app image to an approved registry digest. Reject `:dev`, mutable-only tags, and unreviewed local-name resolution.
3. Make the raw pay service ClusterIP/internal-only and route `pay.epsx.io` through the authenticated BFF. Allocate environment-unique NodePorts only where host exposure is unavoidable.
4. Establish one checked-in ingress authority covering frontend, admin, API, analytics, and pay; lock every Cloudflare → host listener → bridge → NodePort → Service mapping.
5. Remove sensitive literals. Reference reviewed Secrets for pay DB, escrow, receiver/token, chain RPC, webhook identity/key ID, and related configuration; fail closed on missing/zero values.
6. Add the selected candidate services only after their P0/P1 gates pass. Preserve the monolith fallback and keep direct services inaccessible from public ingress.
7. Split liveness from readiness. Readiness must prove the dependencies required to safely accept that route class; add startup behavior for migrations, key loading, and slow dependencies.
8. Define shadow comparison, canary cohort/traffic weights, SLO and data-reconciliation abort thresholds, ownership, observation windows, and an immutable rollback revision.
9. Rehearse rollback in a non-production environment. Only reviewed evidence may change blocker states; integrity passing never grants production approval.

## Gate usage

```sh
./scripts/migration/verify-infrastructure-readiness.sh --mode integrity
./scripts/migration/verify-infrastructure-readiness.sh --mode readiness  # expected exit 3
./scripts/migration/verify-infrastructure-readiness.sh --mode report
./scripts/migration/test-infrastructure-readiness.sh
```

The verifier prefers local `kubectl kustomize`; if unavailable it uses `kustomize build`, writing only to a temporary directory. It fails clearly if neither renderer exists. It validates pinned repository anchors, path safety, rendered resource/image/port/probe/secret inventories, ingress/bridge mappings, blocker references, deterministic output, and tamper rejection. It contains no live-cluster command.
