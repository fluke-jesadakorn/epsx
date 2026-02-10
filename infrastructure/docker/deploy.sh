#!/bin/bash
# ============================================================================
# EPSX Deploy Script — Build Local, Deploy Remote
# ============================================================================
# Usage:
#   ./deploy.sh prod        # Build & deploy production
#   ./deploy.sh dev         # Build & deploy development
#   ./deploy.sh prod build  # Build only (no transfer)
#   ./deploy.sh prod push   # Transfer & deploy only (no build)
#
# Prerequisites:
#   - Docker Desktop running on local Mac
#   - SSH access to server (configure SERVER_HOST below)
# ============================================================================

set -euo pipefail

# --- Configuration ---
SERVER_HOST="${DEPLOY_SERVER:-100.109.131.15}"
SERVER_USER="${DEPLOY_USER:-$(whoami)}"
SERVER_DIR="${DEPLOY_DIR:-~/epsx}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# --- Architecture ---
# Default: arm64 for Mac Mini server. Override: DOCKER_PLATFORM=linux/amd64
PLATFORM="${DOCKER_PLATFORM:-linux/arm64}"

# --- Parse Args ---
ENVIRONMENT="${1:-}"
ACTION="${2:-all}"  # all, build, push

if [[ -z "$ENVIRONMENT" ]] || [[ ! "$ENVIRONMENT" =~ ^(prod|dev)$ ]]; then
  echo "❌ Usage: $0 <prod|dev> [build|push|all]"
  exit 1
fi

echo "============================================"
echo "  EPSX Deploy — ${ENVIRONMENT^^}"
echo "  Server: ${SERVER_USER}@${SERVER_HOST}"
echo "  Platform: ${PLATFORM}"
echo "============================================"

# --- Load env for build args ---
ENV_FILE="${PROJECT_ROOT}/infrastructure/docker/.env.${ENVIRONMENT}.template"
if [[ -f "${PROJECT_ROOT}/infrastructure/docker/.env.${ENVIRONMENT}" ]]; then
  ENV_FILE="${PROJECT_ROOT}/infrastructure/docker/.env.${ENVIRONMENT}"
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "❌ Env file not found: $ENV_FILE"
  echo "   Copy .env.${ENVIRONMENT}.template → .env.${ENVIRONMENT} and fill in values"
  exit 1
fi

# Source env vars
set -a
source "$ENV_FILE"
set +a

TARBALL="deploy-${ENVIRONMENT}.tar.gz"

# ============================================================================
# BUILD
# ============================================================================
if [[ "$ACTION" == "all" ]] || [[ "$ACTION" == "build" ]]; then
  echo ""
  echo "🔨 Building images for ${PLATFORM}..."
  cd "$PROJECT_ROOT"

  export DOCKER_DEFAULT_PLATFORM="${PLATFORM}"

  # Frontend
  echo "  → Building frontend..."
  docker build \
    --platform "${PLATFORM}" \
    --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID="${WALLETCONNECT_PROJECT_ID}" \
    --build-arg NEXT_PUBLIC_APP_URL="${FRONTEND_URL}" \
    --build-arg NEXT_PUBLIC_BACKEND_URL="${BACKEND_URL}" \
    --build-arg NEXT_PUBLIC_ADMIN_URL="${ADMIN_FRONTEND_URL}" \
    --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID="${OAUTH_CLIENT_ID:-epsx-frontend}" \
    --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK="${BLOCKCHAIN_NETWORK}" \
    --build-arg NEXT_PUBLIC_CHAIN_ID="${CHAIN_ID}" \
    --build-arg NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS="${COMPANY_WALLET_MAINNET}" \
    --build-arg NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS="${COMPANY_WALLET_TESTNET}" \
    -f apps/frontend/Dockerfile -t "epsx-frontend:${ENVIRONMENT}" .

  # Admin Frontend
  echo "  → Building admin-frontend..."
  docker build \
    --platform "${PLATFORM}" \
    --build-arg NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID="${WALLETCONNECT_PROJECT_ID}" \
    --build-arg NEXT_PUBLIC_APP_URL="${ADMIN_FRONTEND_URL}" \
    --build-arg NEXT_PUBLIC_BACKEND_URL="${BACKEND_URL}" \
    --build-arg NEXT_PUBLIC_ADMIN_URL="${ADMIN_FRONTEND_URL}" \
    --build-arg NEXT_PUBLIC_OAUTH_CLIENT_ID="${OAUTH_CLIENT_ID:-epsx-admin}" \
    --build-arg NEXT_PUBLIC_BLOCKCHAIN_NETWORK="${BLOCKCHAIN_NETWORK}" \
    --build-arg NEXT_PUBLIC_CHAIN_ID="${CHAIN_ID}" \
    --build-arg NEXT_PUBLIC_PAYMENT_MAINNET_ADDRESS="${COMPANY_WALLET_MAINNET}" \
    --build-arg NEXT_PUBLIC_PAYMENT_TESTNET_ADDRESS="${COMPANY_WALLET_TESTNET}" \
    -f apps/admin-frontend/Dockerfile -t "epsx-admin-frontend:${ENVIRONMENT}" .

  # Backend
  echo "  → Building backend..."
  docker build \
    --platform "${PLATFORM}" \
    -f apps/backend/Dockerfile -t "epsx-backend:${ENVIRONMENT}" .

  # Save images
  echo "  → Saving images to ${TARBALL}..."
  docker save \
    "epsx-frontend:${ENVIRONMENT}" \
    "epsx-admin-frontend:${ENVIRONMENT}" \
    "epsx-backend:${ENVIRONMENT}" \
    | gzip > "${TARBALL}"

  SIZE=$(du -sh "${TARBALL}" | cut -f1)
  echo "  ✅ Built: ${TARBALL} (${SIZE})"
fi

# ============================================================================
# PUSH & DEPLOY
# ============================================================================
if [[ "$ACTION" == "all" ]] || [[ "$ACTION" == "push" ]]; then
  echo ""
  echo "📦 Transferring to ${SERVER_USER}@${SERVER_HOST}..."

  # Ensure server directory exists
  ssh "${SERVER_USER}@${SERVER_HOST}" "mkdir -p ${SERVER_DIR}/scripts"

  # Transfer tarball
  scp "${TARBALL}" "${SERVER_USER}@${SERVER_HOST}:${SERVER_DIR}/"

  # Transfer compose & config files
  scp "${PROJECT_ROOT}/infrastructure/docker/docker-compose.${ENVIRONMENT}.yml" \
      "${SERVER_USER}@${SERVER_HOST}:${SERVER_DIR}/"

  scp "${PROJECT_ROOT}/infrastructure/docker/scripts/init-databases-${ENVIRONMENT}.sh" \
      "${SERVER_USER}@${SERVER_HOST}:${SERVER_DIR}/scripts/"

  # Transfer cloudflare config (prod only has separate config)
  if [[ "$ENVIRONMENT" == "prod" ]]; then
    scp "${PROJECT_ROOT}/infrastructure/cloudflare/cloudflared-config.prod.yml" \
        "${SERVER_USER}@${SERVER_HOST}:${SERVER_DIR}/"
  else
    scp "${PROJECT_ROOT}/infrastructure/cloudflare/cloudflared-config.yml" \
        "${SERVER_USER}@${SERVER_HOST}:${SERVER_DIR}/cloudflared-config.dev.yml"
  fi

  echo ""
  echo "🚀 Deploying on server..."
  ssh "${SERVER_USER}@${SERVER_HOST}" bash -s <<EOF
    cd ${SERVER_DIR}
    echo "  → Loading images..."
    gzip -d -c ${TARBALL} | docker load
    echo "  → Starting containers..."
    docker compose --env-file .env.${ENVIRONMENT} -f docker-compose.${ENVIRONMENT}.yml up -d
    echo "  → Container status:"
    docker compose --env-file .env.${ENVIRONMENT} -f docker-compose.${ENVIRONMENT}.yml ps
EOF

  echo ""
  echo "✅ Deployment complete!"
  echo ""
  echo "📋 Next steps:"
  echo "   1. Verify: ssh ${SERVER_USER}@${SERVER_HOST} 'docker ps'"
  if [[ "$ENVIRONMENT" == "prod" ]]; then
    echo "   2. Check: https://api.epsx.io/health"
    echo "   3. Visit: https://epsx.io"
  else
    echo "   2. Check: https://dev-api.epsx.io/health"
    echo "   3. Visit: https://dev.epsx.io"
  fi
  echo ""
  echo "🗃️  Run migrations (from local Mac):"
  echo "   ssh -L 5434:localhost:${DB_PORT:-5432} ${SERVER_USER}@${SERVER_HOST} -Nf"
  echo "   export DATABASE_URL=postgres://${DB_USER}:<password>@localhost:5434/${DB_NAME}"
  echo "   diesel migration run --config apps/backend/diesel.toml"
  echo "   diesel migration run --config apps/backend/diesel_analytics.toml"
  echo "   diesel migration run --config apps/backend/diesel_notifications.toml"
  echo "   diesel migration run --config apps/backend/diesel_payments.toml"
fi
