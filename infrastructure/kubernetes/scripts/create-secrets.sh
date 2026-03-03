#!/bin/bash
# create-secrets.sh <env>
# Creates all K8s secrets for a given environment by reading from .env file.
# Idempotent: uses --dry-run=client -o yaml | kubectl apply -f -
#
# Usage:
#   ./create-secrets.sh dev
#   ./create-secrets.sh staging
#   ./create-secrets.sh prod
#
# Expects env file at: /Users/fluke/epsx-runner/envs/.env.<env>
# Requires: kubectl with KUBECONFIG pointing to k3s cluster

set -euo pipefail

TARGET_ENV="${1:-}"
if [[ -z "$TARGET_ENV" ]]; then
  echo "Usage: $0 <env>  (dev|staging|prod)" >&2
  exit 1
fi

ENV_FILE="/Users/fluke/epsx-runner/envs/.env.${TARGET_ENV}"
if [[ ! -f "$ENV_FILE" ]]; then
  echo "Error: env file not found: $ENV_FILE" >&2
  exit 1
fi

# Load env vars (ENV in file may differ from TARGET_ENV; preserve TARGET_ENV)
set -a
# shellcheck source=/dev/null
source "$ENV_FILE"
set +a

NAMESPACE="epsx-${TARGET_ENV}"

# Helper: create/update a secret idempotently
apply_secret() {
  kubectl create secret generic "$@" \
    --dry-run=client -o yaml | kubectl apply -f -
}

echo "Creating secrets in namespace: $NAMESPACE"

# ── epsx-postgres ─────────────────────────────────────────────────────────────
apply_secret epsx-postgres \
  -n "$NAMESPACE" \
  --from-literal=POSTGRES_USER="${DB_USER}" \
  --from-literal=POSTGRES_PASSWORD="${DB_PASSWORD}" \
  --from-literal=POSTGRES_DB="${DB_NAME}"

# ── epsx-redis ────────────────────────────────────────────────────────────────
apply_secret epsx-redis \
  -n "$NAMESPACE" \
  --from-literal=REDIS_PASSWORD="${REDIS_PASSWORD}"

# ── epsx-minio ────────────────────────────────────────────────────────────────
apply_secret epsx-minio \
  -n "$NAMESPACE" \
  --from-literal=MINIO_ROOT_USER="${MINIO_ROOT_USER}" \
  --from-literal=MINIO_ROOT_PASSWORD="${MINIO_ROOT_PASSWORD}" \
  --from-literal=MINIO_PUBLIC_URL="${MINIO_PUBLIC_URL}"

# ── epsx-backend ──────────────────────────────────────────────────────────────
# Construct K8s-native connection strings (epsx-postgres, epsx-redis, epsx-minio)
DB_SUFFIX="${DB_NAME#epsx_}"

apply_secret epsx-backend \
  -n "$NAMESPACE" \
  --from-literal=ENV="${ENV}" \
  --from-literal=NODE_ENV="${NODE_ENV:-production}" \
  --from-literal=RUST_ENV="${RUST_ENV:-production}" \
  --from-literal=RUST_LOG="${RUST_LOG:-info}" \
  --from-literal=LOG_LEVEL="${LOG_LEVEL:-info}" \
  --from-literal=BACKEND_URL="${BACKEND_URL}" \
  --from-literal=FRONTEND_URL="${FRONTEND_URL}" \
  --from-literal=ADMIN_FRONTEND_URL="${ADMIN_FRONTEND_URL}" \
  --from-literal=DATABASE_MAX_CONNECTIONS="${DATABASE_MAX_CONNECTIONS:-10}" \
  --from-literal=DATABASE_MIN_CONNECTIONS="${DATABASE_MIN_CONNECTIONS:-2}" \
  --from-literal=DATABASE_ACQUIRE_TIMEOUT="${DATABASE_ACQUIRE_TIMEOUT:-30}" \
  --from-literal=DATABASE_IDLE_TIMEOUT="${DATABASE_IDLE_TIMEOUT:-600}" \
  --from-literal=WEB3_APP_SECRET="${WEB3_APP_SECRET}" \
  --from-literal=WALLET_SIGNATURE_SECRET="${WALLET_SIGNATURE_SECRET}" \
  --from-literal=WEB3_SESSION_SECRET="${WEB3_SESSION_SECRET}" \
  --from-literal=WEB3_SESSION_DURATION_HOURS="${WEB3_SESSION_DURATION_HOURS:-24}" \
  --from-literal=WEB3_SIGNATURE_TIMEOUT_MINUTES="${WEB3_SIGNATURE_TIMEOUT_MINUTES:-5}" \
  --from-literal=JWT_SECRET="${JWT_SECRET}" \
  --from-literal=BLOCKCHAIN_NETWORK="${BLOCKCHAIN_NETWORK}" \
  --from-literal=NEXT_PUBLIC_BLOCKCHAIN_NETWORK="${BLOCKCHAIN_NETWORK}" \
  --from-literal=COMPANY_WALLET_MAINNET="${COMPANY_WALLET_MAINNET}" \
  --from-literal=COMPANY_WALLET_TESTNET="${COMPANY_WALLET_TESTNET:-}" \
  --from-literal=BSC_MAINNET_RPC_URL="${BSC_MAINNET_RPC_URL}" \
  --from-literal=BSC_TESTNET_RPC_URL="${BSC_TESTNET_RPC_URL:-}" \
  --from-literal=BSC_RPC_URL="${BSC_MAINNET_RPC_URL}" \
  --from-literal=PAYMENT_RECEIVER_ADDRESS="${COMPANY_WALLET_MAINNET}" \
  --from-literal=BLOCKCHAIN_CONFIRMATION_TIMEOUT_SECONDS="${BLOCKCHAIN_CONFIRMATION_TIMEOUT_SECONDS:-60}" \
  --from-literal=BSC_REQUIRED_CONFIRMATIONS="${BSC_REQUIRED_CONFIRMATIONS:-3}" \
  --from-literal=BSC_TESTNET_REQUIRED_CONFIRMATIONS="${BSC_TESTNET_REQUIRED_CONFIRMATIONS:-1}" \
  --from-literal=MAX_PAYMENT_AGE_MINUTES="${MAX_PAYMENT_AGE_MINUTES:-60}" \
  --from-literal=TURNSTILE_SECRET_KEY="${TURNSTILE_SECRET_KEY:-}" \
  --from-literal=PAYMENTS_DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@epsx-postgres:5432/epsx_payments_${DB_SUFFIX}?sslmode=disable" \
  --from-literal=ANALYTICS_DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@epsx-postgres:5432/epsx_analytics_${DB_SUFFIX}?sslmode=disable" \
  --from-literal=NOTIFICATIONS_DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@epsx-postgres:5432/epsx_notifications_${DB_SUFFIX}?sslmode=disable"

# ── epsx-frontend ─────────────────────────────────────────────────────────────
apply_secret epsx-frontend \
  -n "$NAMESPACE" \
  --from-literal=ENV="${ENV}" \
  --from-literal=NODE_ENV="${NODE_ENV:-production}" \
  --from-literal=BACKEND_URL="${BACKEND_URL}" \
  --from-literal=FRONTEND_URL="${FRONTEND_URL}" \
  --from-literal=ADMIN_FRONTEND_URL="${ADMIN_FRONTEND_URL}" \
  --from-literal=NEXT_PUBLIC_BACKEND_URL="${BACKEND_URL}" \
  --from-literal=NEXT_PUBLIC_APP_URL="${FRONTEND_URL}" \
  --from-literal=NEXT_PUBLIC_ADMIN_URL="${ADMIN_FRONTEND_URL}" \
  --from-literal=NEXT_PUBLIC_BLOCKCHAIN_NETWORK="${BLOCKCHAIN_NETWORK}" \
  --from-literal=NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID="${WALLETCONNECT_PROJECT_ID}" \
  --from-literal=NEXT_PUBLIC_CHAIN_ID="${CHAIN_ID}" \
  --from-literal=NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET="${NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET:-${PAYMENT_ESCROW_CONTRACT_MAINNET:-}}" \
  --from-literal=NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET="${NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET:-${COMPANY_WALLET_MAINNET}}" \
  --from-literal=NEXT_PUBLIC_TURNSTILE_SITE_KEY="${NEXT_PUBLIC_TURNSTILE_SITE_KEY:-}" \
  --from-literal=NEXT_PUBLIC_OAUTH_CLIENT_ID="${OAUTH_CLIENT_ID}" \
  --from-literal=NEXT_PUBLIC_CDN_URL="${MINIO_PUBLIC_URL}"

# ── epsx-admin ────────────────────────────────────────────────────────────────
apply_secret epsx-admin \
  -n "$NAMESPACE" \
  --from-literal=ENV="${ENV}" \
  --from-literal=NODE_ENV="${NODE_ENV:-production}" \
  --from-literal=BACKEND_URL="${BACKEND_URL}" \
  --from-literal=FRONTEND_URL="${FRONTEND_URL}" \
  --from-literal=ADMIN_FRONTEND_URL="${ADMIN_FRONTEND_URL}" \
  --from-literal=NEXT_PUBLIC_BACKEND_URL="${BACKEND_URL}" \
  --from-literal=NEXT_PUBLIC_APP_URL="${ADMIN_FRONTEND_URL}" \
  --from-literal=NEXT_PUBLIC_ADMIN_URL="${ADMIN_FRONTEND_URL}" \
  --from-literal=NEXT_PUBLIC_BLOCKCHAIN_NETWORK="${BLOCKCHAIN_NETWORK}" \
  --from-literal=NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID="${WALLETCONNECT_PROJECT_ID}" \
  --from-literal=NEXT_PUBLIC_CHAIN_ID="${CHAIN_ID}" \
  --from-literal=NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET="${NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET:-${PAYMENT_ESCROW_CONTRACT_MAINNET:-}}" \
  --from-literal=NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET="${NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET:-${COMPANY_WALLET_MAINNET}}" \
  --from-literal=NEXT_PUBLIC_TURNSTILE_SITE_KEY="${NEXT_PUBLIC_TURNSTILE_SITE_KEY:-}" \
  --from-literal=NEXT_PUBLIC_OAUTH_CLIENT_ID="epsx-admin" \
  --from-literal=NEXT_PUBLIC_CDN_URL="${MINIO_PUBLIC_URL}"

# ── epsx-cloudflared (prod only) ──────────────────────────────────────────────
if [[ "$TARGET_ENV" == "prod" ]]; then
  CREDS_FILE="${HOME}/.cloudflared/6bee9b58-eede-4b4c-815c-94c0ee38fe58.json"
  if [[ ! -f "$CREDS_FILE" ]]; then
    echo "Warning: cloudflared credentials not found at $CREDS_FILE" >&2
  else
    kubectl create secret generic epsx-cloudflared \
      -n "$NAMESPACE" \
      --from-literal=TUNNEL_TOKEN="${CLOUDFLARE_TUNNEL_TOKEN}" \
      --from-file=credentials.json="$CREDS_FILE" \
      --dry-run=client -o yaml | kubectl apply -f -
  fi
fi

echo "Done. Secrets created in $NAMESPACE:"
kubectl get secrets -n "$NAMESPACE" --no-headers | awk '{print "  " $1}'
