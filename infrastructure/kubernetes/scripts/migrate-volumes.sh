#!/bin/bash
# migrate-volumes.sh <env>
# Migrates data from Docker Compose volumes to k3s PVCs.
# Run WHILE Docker is still up (for pg_dump), then again after k3s pods are ready.
#
# Steps:
#   1. Dump data from running Docker containers
#   2. Transfer dumps into Multipass VM
#   3. Restore into k3s pods (run after kubectl apply -k and pods are ready)
#
# Usage:
#   ./migrate-volumes.sh dump <env>     # step 1+2: dump from Docker + transfer
#   ./migrate-volumes.sh restore <env>  # step 3: restore into k3s pods

set -euo pipefail

CMD="${1:-}"
ENV="${2:-}"

if [[ -z "$CMD" || -z "$ENV" ]]; then
  echo "Usage: $0 <dump|restore> <env>  (dev|staging|prod)" >&2
  exit 1
fi

ENV_FILE="/Users/fluke/epsx-runner/envs/.env.${ENV}"
set -a; source "$ENV_FILE"; set +a

NAMESPACE="epsx-${ENV}"
BACKUP_DIR="/tmp/k8s-backup-${ENV}"
VM_BACKUP_DIR="/tmp/k8s-backup-${ENV}"
CONTAINER_PREFIX="epsx-${ENV}"

case "$CMD" in
  dump)
    echo "=== Dumping data from Docker (env: $ENV) ==="
    mkdir -p "$BACKUP_DIR"

    # PostgreSQL — live dump from running container
    echo "Dumping PostgreSQL..."
    docker exec "${CONTAINER_PREFIX}-postgres" \
      pg_dumpall -U "$DB_USER" \
      > "${BACKUP_DIR}/postgres.sql"
    echo "  Postgres dump: ${BACKUP_DIR}/postgres.sql ($(du -sh "${BACKUP_DIR}/postgres.sql" | cut -f1))"

    # Redis — trigger BGSAVE then copy RDB
    echo "Dumping Redis..."
    docker exec "${CONTAINER_PREFIX}-redis" \
      redis-cli -a "$REDIS_PASSWORD" BGSAVE
    sleep 3
    docker cp "${CONTAINER_PREFIX}-redis:/data/dump.rdb" "${BACKUP_DIR}/redis.rdb"
    echo "  Redis dump: ${BACKUP_DIR}/redis.rdb ($(du -sh "${BACKUP_DIR}/redis.rdb" | cut -f1))"

    # MinIO — tar archive
    echo "Dumping MinIO..."
    docker run --rm \
      -v "epsx_${ENV}_minio_data:/src" \
      -v "${BACKUP_DIR}:/backup" \
      alpine tar -czf /backup/minio.tar.gz -C /src .
    echo "  MinIO dump: ${BACKUP_DIR}/minio.tar.gz ($(du -sh "${BACKUP_DIR}/minio.tar.gz" | cut -f1))"

    # Transfer to Multipass VM
    echo "Transferring to k3s-node VM..."
    multipass exec k3s-node -- mkdir -p "$VM_BACKUP_DIR"
    multipass transfer "${BACKUP_DIR}/postgres.sql" "k3s-node:${VM_BACKUP_DIR}/postgres.sql"
    multipass transfer "${BACKUP_DIR}/redis.rdb"    "k3s-node:${VM_BACKUP_DIR}/redis.rdb"
    multipass transfer "${BACKUP_DIR}/minio.tar.gz" "k3s-node:${VM_BACKUP_DIR}/minio.tar.gz"

    echo "=== Dump complete. Files in VM at: ${VM_BACKUP_DIR} ==="
    ;;

  restore)
    echo "=== Restoring data into k3s (env: $ENV, namespace: $NAMESPACE) ==="

    # PostgreSQL
    echo "Restoring PostgreSQL..."
    kubectl wait --for=condition=ready pod/epsx-postgres-0 \
      -n "$NAMESPACE" --timeout=120s
    kubectl cp "${VM_BACKUP_DIR}/postgres.sql" \
      "${NAMESPACE}/epsx-postgres-0:/tmp/restore.sql"
    kubectl exec -n "$NAMESPACE" epsx-postgres-0 -- \
      psql -U "$DB_USER" -f /tmp/restore.sql
    kubectl exec -n "$NAMESPACE" epsx-postgres-0 -- \
      rm /tmp/restore.sql
    echo "  PostgreSQL restored."

    # Redis — copy RDB into PVC path, then scale up
    echo "Restoring Redis..."
    kubectl scale statefulset epsx-redis -n "$NAMESPACE" --replicas=0
    kubectl wait --for=delete pod/epsx-redis-0 -n "$NAMESPACE" --timeout=60s 2>/dev/null || true

    # Find PVC path inside VM
    REDIS_PVC=$(kubectl get pvc epsx-redis-data -n "$NAMESPACE" \
      -o jsonpath='{.spec.volumeName}')
    REDIS_PVC_PATH=$(multipass exec k3s-node -- \
      sudo find /var/lib/rancher/k3s/storage -maxdepth 1 -name "*${REDIS_PVC}*" 2>/dev/null | head -1)

    if [[ -z "$REDIS_PVC_PATH" ]]; then
      echo "  Warning: Could not find Redis PVC path. Skipping Redis restore." >&2
    else
      multipass exec k3s-node -- \
        sudo cp "${VM_BACKUP_DIR}/redis.rdb" "${REDIS_PVC_PATH}/dump.rdb"
      echo "  Redis RDB copied to PVC."
    fi

    kubectl scale statefulset epsx-redis -n "$NAMESPACE" --replicas=1
    kubectl wait --for=condition=ready pod/epsx-redis-0 -n "$NAMESPACE" --timeout=60s
    echo "  Redis restored."

    # MinIO
    echo "Restoring MinIO..."
    kubectl wait --for=condition=ready pod/epsx-minio-0 \
      -n "$NAMESPACE" --timeout=60s
    kubectl cp "${VM_BACKUP_DIR}/minio.tar.gz" \
      "${NAMESPACE}/epsx-minio-0:/tmp/minio.tar.gz"
    kubectl exec -n "$NAMESPACE" epsx-minio-0 -- \
      tar -xzf /tmp/minio.tar.gz -C /data
    kubectl exec -n "$NAMESPACE" epsx-minio-0 -- \
      rm /tmp/minio.tar.gz
    echo "  MinIO restored."

    echo "=== Restore complete ==="
    ;;

  *)
    echo "Unknown command: $CMD. Use 'dump' or 'restore'." >&2
    exit 1
    ;;
esac
