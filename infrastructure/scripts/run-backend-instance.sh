#!/bin/bash
set -euo pipefail

ENV_NAME="${1:-}"
if [[ -z "$ENV_NAME" ]]; then
  echo "Usage: $0 <dev|staging|prod>" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKEND_BIN="$REPO_ROOT/apps/backend/target/release/epsx"
ENV_DIR="$REPO_ROOT/.secret/backend"
ENV_FILE="$ENV_DIR/$ENV_NAME.env"

case "$ENV_NAME" in
  dev)
    DEFAULT_ENV="development"
    DEFAULT_NODE_ENV="development"
    DEFAULT_RUST_ENV="development"
    DEFAULT_PORT="18080"
    DEFAULT_BACKEND_URL="https://dev-api.epsx.io"
    DEFAULT_FRONTEND_URL="https://dev.epsx.io"
    DEFAULT_ADMIN_URL="https://dev-admin.epsx.io"
    DEFAULT_NETWORK="testnet"
    ;;
  staging)
    DEFAULT_ENV="staging"
    DEFAULT_NODE_ENV="production"
    DEFAULT_RUST_ENV="staging"
    DEFAULT_PORT="28080"
    DEFAULT_BACKEND_URL="https://staging-api.epsx.io"
    DEFAULT_FRONTEND_URL="https://staging.epsx.io"
    DEFAULT_ADMIN_URL="https://staging-admin.epsx.io"
    DEFAULT_NETWORK="testnet"
    ;;
  prod)
    DEFAULT_ENV="production"
    DEFAULT_NODE_ENV="production"
    DEFAULT_RUST_ENV="production"
    DEFAULT_PORT="38080"
    DEFAULT_BACKEND_URL="https://api.epsx.io"
    DEFAULT_FRONTEND_URL="https://epsx.io"
    DEFAULT_ADMIN_URL="https://admin.epsx.io"
    DEFAULT_NETWORK="mainnet"
    ;;
  *)
    echo "Unsupported environment: $ENV_NAME" >&2
    exit 1
    ;;
esac

if [[ ! -x "$BACKEND_BIN" ]]; then
  echo "Backend binary not found: $BACKEND_BIN" >&2
  echo "Run ./infrastructure/scripts/build-backend-local.sh first." >&2
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing environment file: $ENV_FILE" >&2
  echo "Copy one of the templates from infrastructure/local-backend/ into .secret/backend/." >&2
  exit 1
fi

set -a
source "$ENV_FILE"
set +a

: "${ENV:=$DEFAULT_ENV}"
: "${NODE_ENV:=$DEFAULT_NODE_ENV}"
: "${RUST_ENV:=$DEFAULT_RUST_ENV}"
: "${HOST:=127.0.0.1}"
: "${PORT:=$DEFAULT_PORT}"
: "${BACKEND_URL:=$DEFAULT_BACKEND_URL}"
: "${FRONTEND_URL:=$DEFAULT_FRONTEND_URL}"
: "${ADMIN_FRONTEND_URL:=$DEFAULT_ADMIN_URL}"

: "${NEXT_PUBLIC_BLOCKCHAIN_NETWORK:=${BLOCKCHAIN_NETWORK:-$DEFAULT_NETWORK}}"
: "${BLOCKCHAIN_NETWORK:=$NEXT_PUBLIC_BLOCKCHAIN_NETWORK}"

if [[ -z "${DATABASE_URL:-}" ]] && [[ -n "${DB_USER:-}" ]] && [[ -n "${DB_PASSWORD:-}" ]] && [[ -n "${DB_NAME:-}" ]]; then
  DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST:-127.0.0.1}:${DB_PORT:-5432}/${DB_NAME}"
  export DATABASE_URL
fi

if [[ -z "${REDIS_URL:-}" ]] && [[ -n "${REDIS_PASSWORD:-}" ]]; then
  REDIS_URL="redis://:${REDIS_PASSWORD}@${REDIS_HOST:-127.0.0.1}:${REDIS_PORT:-6379}/${REDIS_DB:-0}"
  export REDIS_URL
fi

required_vars=(
  DATABASE_URL
  BACKEND_URL
  FRONTEND_URL
  ADMIN_FRONTEND_URL
  JWT_SECRET
  NEXT_PUBLIC_BLOCKCHAIN_NETWORK
)

for var in "${required_vars[@]}"; do
  if [[ -z "${!var:-}" ]]; then
    echo "Missing required environment variable: $var" >&2
    exit 1
  fi
done

echo "Starting EPSX backend [$ENV_NAME]"
echo "  bind: ${HOST}:${PORT}"
echo "  public: ${BACKEND_URL}"
echo "  frontend: ${FRONTEND_URL}"
echo "  admin: ${ADMIN_FRONTEND_URL}"

exec "$BACKEND_BIN"
