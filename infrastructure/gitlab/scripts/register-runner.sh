#!/usr/bin/env bash
# Register GitLab Runner with the self-hosted GitLab instance
# Usage: ./register-runner.sh <runner-token>
#
# Prerequisites:
#   1. GitLab is running and healthy: curl -sf https://gitlab.epsx.io/-/health
#   2. Create a runner token: GitLab Admin > CI/CD > Runners > New instance runner
#   3. Create host dirs: sudo mkdir -p /srv/epsx/{ci,envs}
#   4. Copy env files to /Users/fluke/epsx-runner/envs/.env.{dev,staging,prod}
#   5. Run this script with the token

set -euo pipefail

TOKEN="${1:?Usage: $0 <runner-token>}"
GITLAB_URL="${GITLAB_URL:-https://gitlab.epsx.io}"
RUNNER_NAME="${RUNNER_NAME:-epsx-mac-mini}"

echo "Registering runner '${RUNNER_NAME}' with ${GITLAB_URL}..."

docker exec -it epsx-gitlab-runner gitlab-runner register \
  --non-interactive \
  --url "${GITLAB_URL}" \
  --token "${TOKEN}" \
  --executor "docker" \
  --docker-image "docker:27" \
  --docker-volumes "/var/run/docker.sock:/var/run/docker.sock" \
  --docker-volumes "/Users/fluke/epsx-runner/builds:/Users/fluke/epsx-runner/builds" \
  --docker-volumes "/Users/fluke/epsx-runner/envs:/Users/fluke/epsx-runner/envs:ro" \
  --docker-network-mode "epsx_gitlab_network" \
  --docker-memory "4g" \
  --docker-cpus "4" \
  --docker-pull-policy "if-not-present" \
  --description "${RUNNER_NAME}" \
  --tag-list "docker,arm64,epsx" \
  --run-untagged="true" \
  --builds-dir "/Users/fluke/epsx-runner/builds"

# Set concurrency to 3 jobs
docker exec epsx-gitlab-runner sed -i 's/^concurrent = .*/concurrent = 3/' /etc/gitlab-runner/config.toml

echo ""
echo "Runner registered. Restarting runner to apply concurrency..."
docker restart epsx-gitlab-runner

echo ""
echo "Done. Verify: docker exec epsx-gitlab-runner gitlab-runner list"
echo ""
echo "IMPORTANT: Ensure env files exist at:"
echo "  /Users/fluke/epsx-runner/envs/.env.dev"
echo "  /Users/fluke/epsx-runner/envs/.env.staging"
echo "  /Users/fluke/epsx-runner/envs/.env.prod"
