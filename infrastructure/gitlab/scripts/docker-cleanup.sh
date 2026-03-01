#!/usr/bin/env bash
# Docker cleanup for CI runner host (Mac Mini)
# Removes dangling images/containers from CI builds + review apps.
#
# Install crontab entries:
#   crontab -e
#   0 4 * * * /Users/fluke/Desktop/Work/epsx/infrastructure/gitlab/scripts/docker-cleanup.sh
#   0 4 * * 0 docker exec epsx-gitlab gitlab-ctl registry-garbage-collect >> /var/log/registry-gc.log 2>&1

set -euo pipefail

LOG="/var/log/docker-cleanup.log"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Starting Docker cleanup" >> "$LOG"

# Prune stopped containers, dangling images, and build cache older than 72h
docker system prune -f --filter "until=72h" >> "$LOG" 2>&1

# Remove review app images older than 7 days
docker images --filter "reference=registry.epsx.io/epsx/review:*" --format '{{.ID}} {{.CreatedAt}}' | \
  while read -r id created; do
    age=$(( ($(date +%s) - $(date -d "$created" +%s 2>/dev/null || date -j -f "%Y-%m-%d %H:%M:%S %z" "$created" +%s 2>/dev/null || echo 0)) / 86400 ))
    if [ "$age" -gt 7 ]; then
      docker rmi -f "$id" >> "$LOG" 2>&1 || true
    fi
  done

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Cleanup complete" >> "$LOG"
