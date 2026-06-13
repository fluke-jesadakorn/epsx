#!/usr/bin/env bash
# Tear down the wave-12 dev env container. Keep the image.
set -euo pipefail

CONTAINER="epsx-analytics-dev"
echo "==> stop + remove $CONTAINER"
docker rm -f "$CONTAINER" 2>/dev/null || echo "    (was not running)"

cat <<EOF

✔ container removed; image epsx-analytics:dev still on disk
  scripts/dev-up.sh     # bring it back
  scripts/dev-reset.sh  # full reset (rmi + rebuild)
EOF
