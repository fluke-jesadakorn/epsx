#!/usr/bin/env bash
# Full reset: stop + remove container + remove image + rebuild + up.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CONTAINER="epsx-analytics-dev"
IMAGE="epsx-analytics:dev"

echo "==> stop + remove $CONTAINER"
docker rm -f "$CONTAINER" 2>/dev/null || true

echo "==> remove image $IMAGE"
docker rmi -f "$IMAGE" 2>/dev/null || true

echo "==> rebuild + up"
bash scripts/dev-up.sh
