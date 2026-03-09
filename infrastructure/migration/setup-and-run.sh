#!/usr/bin/env bash
# =============================================================================
# EPSX New Machine Setup — Install Docker + Run All Services
# =============================================================================
# Sets up dependencies, Docker networks, and starts all environments.
#
# Services started:
#   Production: backend, frontend, admin, redis, minio, cloudflared
#   Dev:        postgres, redis, traefik, minio
#   GitLab:     gitlab-ce, runner, traefik
#
# PostgreSQL runs bare metal (brew services start postgresql@14)
# Backend connects to PG via host.docker.internal:5432
#
# Usage: ./setup-and-run.sh
# =============================================================================
set -euo pipefail

REPO_DIR="${HOME}/Desktop/Work/epsx"
INFRA_DIR="${REPO_DIR}/infrastructure/docker"
GITLAB_DIR="${HOME}/gitlab-central"
GITLAB_ENV="${HOME}/epsx-runner/envs/.env.gitlab"

# Colors
R='\033[0;31m' G='\033[0;32m' Y='\033[1;33m' B='\033[0;34m' C='\033[0;36m' NC='\033[0m'
info()  { echo -e "${B}[INFO]${NC}  $*"; }
ok()    { echo -e "${G}[OK]${NC}    $*"; }
warn()  { echo -e "${Y}[WARN]${NC}  $*"; }
fail()  { echo -e "${R}[FAIL]${NC}  $*"; exit 1; }
step()  { echo -e "\n${C}━━━ $* ━━━${NC}"; }

# ─── 1. Docker Desktop ──────────────────────────────────────────────────
step "1. Docker Desktop"
if command -v docker &>/dev/null && docker info &>/dev/null; then
  ok "Docker running: $(docker --version)"
else
  fail "Docker Desktop not installed or not running.
  Download: https://docs.docker.com/desktop/install/mac-install/
  Install, start Docker Desktop, then re-run this script."
fi

# ─── 2. Dependencies ────────────────────────────────────────────────────
step "2. Dependencies"

# Bun
if command -v bun &>/dev/null; then
  ok "bun $(bun --version)"
else
  info "Installing Bun..."
  curl -fsSL https://bun.sh/install | bash
  export PATH="${HOME}/.bun/bin:$PATH"
  ok "bun installed"
fi

# Homebrew packages
for pkg in sops cloudflared age; do
  if brew list "$pkg" &>/dev/null 2>&1; then
    ok "$pkg installed"
  else
    info "Installing $pkg..."
    brew install "$pkg"
    ok "$pkg installed"
  fi
done

# Rust
if command -v rustc &>/dev/null; then
  ok "rust $(rustc --version | awk '{print $2}')"
else
  info "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "${HOME}/.cargo/env"
  ok "rust installed"
fi

# ─── 3. Bare metal PostgreSQL ───────────────────────────────────────────
step "3. Bare metal PostgreSQL"
if pg_isready -q 2>/dev/null; then
  ok "PostgreSQL running on port 5432"
  # Show databases
  psql postgres -t -c "SELECT datname FROM pg_database WHERE datname LIKE 'epsx%' ORDER BY datname;" 2>/dev/null \
    | sed 's/^ */  /'

  # Ensure epsx_user role exists with CREATEDB
  psql postgres -v ON_ERROR_STOP=1 <<-'EOSQL' 2>/dev/null
    DO $$
    BEGIN
      IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'epsx_user') THEN
        CREATE ROLE epsx_user LOGIN CREATEDB PASSWORD 'password';
        RAISE NOTICE 'Created role epsx_user';
      ELSE
        ALTER ROLE epsx_user CREATEDB;
      END IF;
    END $$;
EOSQL
  ok "epsx_user role ready (LOGIN + CREATEDB)"
else
  warn "PostgreSQL not running — start with: brew services start postgresql@17"
fi

# ─── 4. Bare metal Redis ────────────────────────────────────────────────
step "4. Bare metal Redis"
if redis-cli ping &>/dev/null 2>&1; then
  ok "Redis running — DBSIZE: $(redis-cli DBSIZE 2>/dev/null | grep -oE '[0-9]+')"
else
  warn "Redis not running — start with: brew services start redis"
fi

# ─── 5. SOPS decryption ─────────────────────────────────────────────────
step "5. SOPS decryption check"
cd "$REPO_DIR"
if [ -f "${HOME}/.config/sops/age/keys.txt" ]; then
  if sops -d --input-type dotenv --output-type dotenv "${INFRA_DIR}/.env.prod.enc" > /dev/null 2>&1; then
    ok "SOPS decryption works"
  else
    fail "SOPS decryption failed — check ~/.config/sops/age/keys.txt"
  fi
else
  fail "SOPS age key not found at ~/.config/sops/age/keys.txt"
fi

# Decrypt prod env
sops -d --input-type dotenv --output-type dotenv "${INFRA_DIR}/.env.prod.enc" > /tmp/.env.prod
ok "Decrypted .env.prod"

# ─── 6. Docker networks ─────────────────────────────────────────────────
step "6. Docker networks"
for net in epsx_prod_network epsx_dev_network epsx_staging_network epsx_gitlab_network epsx_review_network; do
  if docker network inspect "$net" &>/dev/null; then
    ok "$net exists"
  else
    docker network create "$net"
    ok "$net created"
  fi
done

# ─── 7. Production services ─────────────────────────────────────────────
step "7. Production services (backend, frontend, admin, redis, minio, cloudflared)"
cd "$INFRA_DIR"

# Pre-flight: verify cloudflared bind-mount paths
[ -f "${REPO_DIR}/infrastructure/cloudflare/cloudflared-config.prod.yml" ] \
  || fail "Cloudflared config not found: infrastructure/cloudflare/cloudflared-config.prod.yml"
[ -f "${HOME}/.cloudflared/6bee9b58-eede-4b4c-815c-94c0ee38fe58.json" ] \
  || fail "Tunnel credentials not found: ~/.cloudflared/6bee9b58-*.json"

docker compose --env-file /tmp/.env.prod -f docker-compose.prod.yml up -d --force-recreate
info "Waiting 30s for services to stabilize..."
sleep 30

for svc in epsx-prod-redis epsx-prod-backend epsx-prod-frontend epsx-prod-admin epsx-prod-minio epsx-prod-cloudflared; do
  STATUS=$(docker inspect -f '{{.State.Status}}' "$svc" 2>/dev/null || echo "missing")
  HEALTH=$(docker inspect -f '{{.State.Health.Status}}' "$svc" 2>/dev/null || echo "n/a")
  if [ "$STATUS" = "running" ]; then
    ok "$svc — $HEALTH"
  else
    warn "$svc — $STATUS"
  fi
done

# ─── 8. Dev services ────────────────────────────────────────────────────
step "8. Dev services (postgres, redis, traefik, minio)"
if [ -f "${INFRA_DIR}/.env.dev" ]; then
  docker compose --env-file "${INFRA_DIR}/.env.dev" -f "${INFRA_DIR}/docker-compose.dev.yml" up -d --force-recreate
  sleep 10
  for svc in epsx-dev-postgres epsx-dev-redis epsx-dev-traefik epsx-dev-minio; do
    STATUS=$(docker inspect -f '{{.State.Status}}' "$svc" 2>/dev/null || echo "missing")
    if [ "$STATUS" = "running" ]; then
      ok "$svc running"
    else
      warn "$svc: $STATUS"
    fi
  done
else
  warn ".env.dev not found — skipping dev services"
fi

# ─── 9. GitLab + Runner ─────────────────────────────────────────────────
step "9. GitLab + Runner + Traefik"
if [ -f "$GITLAB_ENV" ] && [ -f "${GITLAB_DIR}/docker-compose.yml" ]; then
  cd "$GITLAB_DIR"
  docker compose --env-file "$GITLAB_ENV" up -d --force-recreate
  info "GitLab takes 3-5 min for first boot..."

  TRIES=0
  until docker exec epsx-gitlab curl -sf http://localhost:80/-/health 2>/dev/null; do
    sleep 30
    TRIES=$((TRIES + 1))
    info "Waiting for GitLab... (${TRIES}/20)"
    [ "$TRIES" -ge 20 ] && { warn "GitLab slow to start — check manually"; break; }
  done

  for svc in epsx-gitlab epsx-gitlab-runner epsx-traefik; do
    STATUS=$(docker inspect -f '{{.State.Status}}' "$svc" 2>/dev/null || echo "missing")
    if [ "$STATUS" = "running" ]; then
      ok "$svc running"
    else
      warn "$svc: $STATUS"
    fi
  done
else
  warn "GitLab compose or env not found — skipping"
  [ ! -f "$GITLAB_ENV" ] && warn "  Missing: $GITLAB_ENV"
  [ ! -f "${GITLAB_DIR}/docker-compose.yml" ] && warn "  Missing: ${GITLAB_DIR}/docker-compose.yml"
fi

# ─── 10. Verify endpoints ───────────────────────────────────────────────
step "10. Endpoint verification"

# Local
HEALTH=$(curl -sf http://localhost:9180/health 2>/dev/null || echo "FAIL")
[ "$HEALTH" != "FAIL" ] && ok "backend local: $HEALTH" || warn "backend local: not responding"

FE=$(curl -so /dev/null -w "%{http_code}" http://localhost:4700 2>/dev/null || echo "000")
[ "$FE" = "200" ] && ok "frontend local: HTTP $FE" || warn "frontend local: HTTP $FE"

ADMIN=$(curl -so /dev/null -w "%{http_code}" http://localhost:4701 2>/dev/null || echo "000")
[ "$ADMIN" = "200" ] || [ "$ADMIN" = "307" ] && ok "admin local: HTTP $ADMIN" || warn "admin local: HTTP $ADMIN"

# Tunnel
echo ""
info "Tunnel endpoints:"
for url in "https://api.epsx.io/health" "https://epsx.io" "https://admin.epsx.io" "https://gitlab.jesadakorn.com/-/health"; do
  CODE=$(curl -so /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
  [ "$CODE" = "200" ] || [ "$CODE" = "307" ] && ok "$url → HTTP $CODE" || warn "$url → HTTP $CODE"
done

# ─── Summary ─────────────────────────────────────────────────────────────
step "All services started"
echo ""
echo "  Production:"
echo "    backend       → localhost:9180 → api.epsx.io"
echo "    frontend      → localhost:4700 → epsx.io"
echo "    admin         → localhost:4701 → admin.epsx.io"
echo "    redis         → Docker (epsx_prod_network)"
echo "    minio         → localhost:9100/9101 → cdn.epsx.io"
echo "    cloudflared   → tunnel 6bee9b58-..."
echo "    postgresql    → bare metal :5432"
echo ""
echo "  Dev:"
echo "    postgres      → localhost:5433"
echo "    redis         → localhost:6380"
echo "    traefik       → localhost:4800"
echo "    minio         → localhost:9200/9201"
echo ""
echo "  GitLab:"
echo "    gitlab        → localhost:8929 → gitlab.jesadakorn.com"
echo "    registry      → localhost:5050 → registry.jesadakorn.com"
echo "    runner        → Docker executor"
echo "    traefik       → localhost:8930 (review apps)"
echo ""

# Cleanup temp env
rm -f /tmp/.env.prod
