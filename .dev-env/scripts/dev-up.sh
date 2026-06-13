#!/usr/bin/env bash
# Bring up the wave-12 dev env: build image + run isolated container.
# No K8s, no Cloudflare, no prod touch. Idempotent.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

IMAGE="epsx-analytics:dev"
CONTAINER="epsx-analytics-dev"
HOST_PORT="${HOST_PORT:-18080}"
CONTAINER_PORT=8080

echo "==> 1/4  build image $IMAGE from apps/analytics/Dockerfile"
docker build -f apps/analytics/Dockerfile -t "$IMAGE" .

echo "==> 2/4  stop + remove any prior $CONTAINER (ignore if absent)"
docker rm -f "$CONTAINER" >/dev/null 2>&1 || true

echo "==> 3/4  run $CONTAINER (host port $HOST_PORT -> container $CONTAINER_PORT)"
docker run -d \
  --name "$CONTAINER" \
  -p "${HOST_PORT}:${CONTAINER_PORT}" \
  --restart unless-stopped \
  "$IMAGE" >/dev/null

echo "==> 4/4  wait for /health to return 200 (max 30s)"
ok=0
for i in {1..30}; do
  if curl -fsS "http://localhost:${HOST_PORT}/health" >/dev/null 2>&1; then
    ok=1
    echo "    healthy after ${i}s"
    break
  fi
  sleep 1
done
if [[ $ok -ne 1 ]]; then
  echo "    FAILED to come up in 30s; recent logs:"
  docker logs --tail 50 "$CONTAINER"
  exit 1
fi

cat <<EOF

✔ wave-12 dev env up
  container:   $CONTAINER
  image:       $IMAGE
  bind:        http://localhost:${HOST_PORT}
  health:      curl -fsS http://localhost:${HOST_PORT}/health
  5 routes:    /api/analytics/{rankings,filters,countries,available-countries,sectors}

Next:
  scripts/smoke-container.sh    # hit all 5 routes
  scripts/dev-down.sh           # stop + remove
EOF
