#!/bin/bash
# EPSX Docker Cleanup Script
# Prunes unused resources and keeps only the latest tags for development/staging/production images.

set -e

echo "--------------------------------------------------------"
echo "  Docker Cleanup Started — $(date)"
echo "--------------------------------------------------------"

# 1. Prune dangling images (untagged)
echo "[1/5] Pruning dangling images..."
docker image prune -f

# 2. Prune unused volumes
# WARNING: This removes all volumes not used by at least one container.
echo "[2/5] Pruning unused volumes..."
docker volume prune -f

# 3. Prune builder cache
echo "[3/5] Pruning builder cache (older than 24h)..."
docker builder prune --filter "until=24h" -f

# 4. Remove untagged images (named <none>)
echo "[4/5] Removing untagged images..."
DANGLING_IMAGES=$(docker images -f "dangling=true" -q)
if [ -n "$DANGLING_IMAGES" ]; then
    docker rmi $DANGLING_IMAGES 2>/dev/null || true
fi

# 5. Selective image cleanup for EPSX apps
# We keep 'latest' tags and the most recent 3 versioned tags per environment.
echo "[5/5] Performing selective image retention..."

# Components to check
COMPONENTS=("frontend" "admin" "backend")
# Environments to check
ENVS=("dev" "staging" "prod")

for comp in "${COMPONENTS[@]}"; do
    for env in "${ENVS[@]}"; do
        echo "--> Processing $comp (env: $env)"
        
        # Get all tags for this component and environment, excluding 'latest'
        # Format: Repo:Tag|CreatedAt
        # Sort by CreatedAt descending
        IMAGE_LIST=$(docker images --format "{{.Repository}}:{{.Tag}}|{{.CreatedAt}}" | \
            grep "/$comp" | \
            grep ":$env-" | \
            grep -v ":$env-latest" | \
            sort -t '|' -k 2 -r | \
            cut -d '|' -f 1)
        
        if [ -z "$IMAGE_LIST" ]; then
            echo "    No versioned images found for $comp:$env"
            continue
        fi

        COUNT=0
        while read -r img; do
            if [ -n "$img" ]; then
                COUNT=$((COUNT + 1))
                if [ $COUNT -gt 3 ]; then
                    echo "    [DELETE] $img ($COUNT/keep 3)"
                    docker rmi "$img" 2>/dev/null || true
                else
                    echo "    [KEEP]   $img ($COUNT/keep 3)"
                fi
            fi
        done <<< "$IMAGE_LIST"
    done
done

echo "--------------------------------------------------------"
echo "  Docker Cleanup Finished — $(date)"
echo "--------------------------------------------------------"
docker system df
