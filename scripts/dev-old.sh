#!/usr/bin/env bash
# dev-old.sh — run the OLD Next.js apps (apps-old/frontend + apps-old/admin-frontend)
# locally for the wave-21 pixel-recheck visual diff.
#
# Why: the new Dioxus apps (apps/frontend + apps/admin) replace these
# Next.js apps in production. For the pixel-recheck tracks, we need a
# side-by-side diff: OLD on one URL, NEW on another. This script
# brings up the OLD side.
#
# Stack requirements (already up from wave 20):
#   - Postgres 5433 (epsx_user / dev_only) — `psql -h 127.0.0.1 -p 5433 -U epsx_user`
#   - Redis 6380 (requirepass dev_only) — `redis-cli -a dev_only -p 6380 PING`
#   - MinIO 9100 (root/rootrootroot) — http://localhost:9100
#   - K8s dev cluster (port-forwarded on :18080 → svc/epsx-backend :8080)
#
# This script:
#   1. Reuses the 5433/6380/9100 services from wave 20.
#   2. Writes a minimal `.env.development` in the repo root with the
#      dev DB URLs + port-forwarded backend URL.
#   3. Runs `pnpm install` in each OLD app (idempotent).
#   4. Installs the @tailwindcss/typography stub (committed at
#      scripts/old-stubs/) — the OLD code references this Tailwind
#      v4 plugin in styles/index.css but doesn't declare it in
#      package.json. The stub is a no-op so the dev server boots;
#      `prose` utility classes won't apply typography styles in this
#      mode (see stub's docstring for the trade-off rationale).
#   5. Starts each app via `next dev --webpack` on ports that don't
#      collide (suggest 5000 + 5001) and the new Dioxus dev loop
#      (4000 + 4001, see scripts/wave21-dev-loop.sh).
#
# Why `--webpack` and not `--turbo` (the default in Next.js 16):
#   - The OLD apps' tsconfig.json has `"@/shared/*": ["../../shared/*"]`,
#     a path alias that pulls files from outside the OLD app dir.
#   - With Turbopack (the Next 16 default), the @/shared/* alias
#     resolves to files OUTSIDE the project root, and the Edge
#     Middleware runtime can't follow bare-module imports from
#     those files. First symptom: `Can't resolve 'zod'` from
#     shared/env/schema.ts (zod lives at the OLD app's
#     node_modules, not at the workspace root).
#   - With `--webpack`, the OLD apps' next.config.ts takes effect
#     (it has `config.resolve.modules = [appNodeModules, ...]`)
#     and bare-module imports from shared/* resolve correctly.
#   - The downside is slower HMR vs Turbopack, but for a side-by-
#     side pixel diff that's fine — we're not editing the OLD code.
#
# Usage:
#   ./scripts/dev-old.sh up         # install + run both apps
#   ./scripts/dev-old.sh up frontend # only apps-old/frontend (port 5000)
#   ./scripts/dev-old.sh up admin    # only apps-old/admin-frontend (port 5001)
#   ./scripts/dev-old.sh down        # stop both
#   ./scripts/dev-old.sh status      # show what's running
#   ./scripts/dev-old.sh help        # this message
#
# Auth: by default the OLD apps go through the same SIWE gate as
# production. The new Dioxus dev loop has the auth bypass; the OLD
# side does NOT. To log in to the OLD side, use the same dev wallet
# flow you would in dev (or copy an existing valid `epsx_token`
# cookie from the dev K8s session).

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
OLD_FRONTEND_DIR=$REPO_ROOT/apps-old/frontend
OLD_ADMIN_DIR=$REPO_ROOT/apps-old/admin-frontend
STUB_SRC=$REPO_ROOT/scripts/old-stubs/@tailwindcss/typography
REACT_MARKDOWN_STUB_SRC=$REPO_ROOT/scripts/old-stubs/react-markdown
REMARK_GFM_STUB_SRC=$REPO_ROOT/scripts/old-stubs/remark-gfm

# Default ports — chosen to avoid the K8s cluster (30000/30001/30080),
# the prod bridges (4700/4701/9180), the new Dioxus dev loop
# (4000/4001), and the local DB/Redis/MinIO (5432/5433/6379/6380/9100).
OLD_FRONTEND_PORT=${OLD_FRONTEND_PORT:-5000}
OLD_ADMIN_PORT=${OLD_ADMIN_PORT:-5001}

LOG_DIR=${LOG_DIR:-/tmp/epsx-dev-old-logs}
PID_DIR=${PID_DIR:-/tmp/epsx-dev-old-pids}
mkdir -p "$LOG_DIR" "$PID_DIR"

# Color helpers.
log()  { printf '\033[1;35m[dev-old]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[dev-old]\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[1;31m[dev-old]\033[0m %s\n' "$*" >&2; }

is_port_listening() {
  local port=$1
  lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
}

wait_for_port() {
  local port=$1
  local label=$2
  local i
  for i in $(seq 1 100); do
    if is_port_listening "$port"; then
      log "$label listening on :$port"
      return 0
    fi
    sleep 0.3
  done
  err "$label did not bind to :$port within 30s; check $LOG_DIR/${label}.log"
  return 1
}

# ──────────────────────────────────────────────────────────────────────
# Pre-flight: verify wave-20 infra is up
# ──────────────────────────────────────────────────────────────────────
preflight() {
  log "preflight: verifying wave-20 infra..."

  # Postgres 5433
  if ! PGPASSWORD=dev_only psql -h 127.0.0.1 -p 5433 -U epsx_user -d epsx_dev -c "SELECT 1" >/dev/null 2>&1; then
    err "Postgres at 127.0.0.1:5433 is not reachable (epsx_user/dev_only)."
    err "Re-run the wave-20 recovery (see deliverable-wave20-dev-postgres-recovery.md)."
    return 1
  fi
  log "Postgres 5433 OK"

  # Redis 6380
  if ! redis-cli -a dev_only -p 6380 PING 2>/dev/null | grep -q PONG; then
    err "Redis at 127.0.0.1:6380 is not reachable (requirepass=dev_only)."
    err "Re-run the wave-20 recovery."
    return 1
  fi
  log "Redis 6380 OK"

  # MinIO 9100
  if ! curl -fsS -m 3 http://127.0.0.1:9100/minio/health/live >/dev/null 2>&1; then
    err "MinIO at 127.0.0.1:9100 is not reachable."
    return 1
  fi
  log "MinIO 9100 OK"

  # K8s dev backend port-forward (we don't own the port-forward, just warn)
  if ! is_port_listening 18080; then
    warn "K8s dev backend port-forward is NOT active on :18080."
    warn "Apps that call the backend will fail. Run:"
    warn "  KUBECONFIG=/tmp/k3s-default-clean.yaml kubectl port-forward -n epsx-dev svc/epsx-backend 18080:8080 &"
    warn "(Or set OLD_API_URL to wherever your backend is reachable.)"
  else
    log "K8s dev backend port-forward 18080 OK"
  fi
}

# ──────────────────────────────────────────────────────────────────────
# .env.development writer
# ──────────────────────────────────────────────────────────────────────
write_env() {
  local env_file=$REPO_ROOT/.env.development
  if [ -f "$env_file" ] && [ "${EPSX_FORCE_ENV_WRITE:-0}" != "1" ]; then
    log ".env.development already exists at $env_file — leaving it alone (set EPSX_FORCE_ENV_WRITE=1 to overwrite)"
    return 0
  fi
  log "writing $env_file (pointing at wave-20 dev infra)"
  cat > "$env_file" <<EOF
# Wave-21 dev-old.sh — auto-generated. Points the OLD Next.js apps
# at the wave-20 dev infra (5433/6380/9100) and the dev K8s backend
# (port-forwarded on :18080).
# Delete this file when you're done and the OLD apps will fall back
# to their prod-shaped defaults.

# Backend the OLD apps call
NEXT_PUBLIC_BACKEND_URL=http://localhost:18080
BACKEND_URL=http://localhost:18080
NEXT_PUBLIC_API_URL=http://localhost:18080

# Database (Diesel-friendly URL)
DATABASE_URL=postgres://epsx_user:dev_only@127.0.0.1:5433/epsx_dev
ANALYTICS_DATABASE_URL=postgres://epsx_user:dev_only@127.0.0.1:5433/epsx_analytics_dev
PAYMENTS_DATABASE_URL=postgres://epsx_user:dev_only@127.0.0.1:5433/epsx_payments_dev
NOTIFICATIONS_DATABASE_URL=postgres://epsx_user:dev_only@127.0.0.1:5433/epsx_notifications_dev

# Redis
REDIS_URL=redis://:dev_only@127.0.0.1:6380

# MinIO (default local creds; see launchctl com.epsx.minio)
MINIO_ENDPOINT=http://127.0.0.1:9100
MINIO_PUBLIC_URL=http://127.0.0.1:9100
MINIO_ACCESS_KEY=root
MINIO_SECRET_KEY=rootrootroot
NEXT_PUBLIC_CDN_URL=http://127.0.0.1:9100/public

# App identity (dev defaults — DO NOT use in prod)
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-frontend
NEXT_PUBLIC_APP_URL=http://localhost:${OLD_FRONTEND_PORT}
NEXT_PUBLIC_ADMIN_URL=http://localhost:${OLD_ADMIN_PORT}
NEXT_PUBLIC_DEPLOYMENT_ENV=development
EOF
  log "wrote $env_file"
}

# ──────────────────────────────────────────────────────────────────────
# pnpm install (idempotent — skips if node_modules already present)
# ──────────────────────────────────────────────────────────────────────
ensure_deps() {
  local dir=$1
  if [ -d "$dir/node_modules" ]; then
    log "deps already installed in $dir — skipping pnpm install"
  else
    log "installing deps in $dir (this may take a few minutes the first time)..."
    ( cd "$dir" && pnpm install --no-frozen-lockfile )
  fi
  # Always install the typography stub — it lives in node_modules and
  # pnpm could blow it away on a re-install. The stub is a small
  # no-op Tailwind plugin; see scripts/old-stubs/@tailwindcss/typography/.
  install_typography_stub "$dir"
  # Same story for react-markdown + remark-gfm — imported by
  # apps-old/frontend/components/news/news-detail.tsx but never
  # declared in the OLD app's package.json. The stubs render raw
  # markdown as <pre> so the news article body is at least
  # inspectable. See scripts/old-stubs/{react-markdown,remark-gfm}/.
  install_react_markdown_stub "$dir"
  install_remark_gfm_stub "$dir"
}

install_typography_stub() {
  local dir=$1
  local dest=$dir/node_modules/@tailwindcss/typography
  if [ ! -d "$STUB_SRC/src" ]; then
    warn "stub source $STUB_SRC not found; skipping (OLD pages may 500 on Tailwind compile)"
    return 0
  fi
  mkdir -p "$dest/src"
  cp -f "$STUB_SRC/package.json" "$dest/package.json"
  cp -f "$STUB_SRC/src/index.js" "$dest/src/index.js"
  log "typography stub installed at $dest (no-op plugin; see stub docstring)"
}

install_react_markdown_stub() {
  local dir=$1
  local dest=$dir/node_modules/react-markdown
  if [ ! -d "$REACT_MARKDOWN_STUB_SRC/src" ]; then
    warn "stub source $REACT_MARKDOWN_STUB_SRC not found; skipping (OLD /news/[slug] may 500)"
    return 0
  fi
  mkdir -p "$dest/src"
  cp -f "$REACT_MARKDOWN_STUB_SRC/package.json" "$dest/package.json"
  cp -f "$REACT_MARKDOWN_STUB_SRC/src/index.js" "$dest/src/index.js"
  log "react-markdown stub installed at $dest (renders <pre>; see stub docstring)"
}

install_remark_gfm_stub() {
  local dir=$1
  local dest=$dir/node_modules/remark-gfm
  if [ ! -d "$REMARK_GFM_STUB_SRC/src" ]; then
    warn "stub source $REMARK_GFM_STUB_SRC not found; skipping"
    return 0
  fi
  mkdir -p "$dest/src"
  cp -f "$REMARK_GFM_STUB_SRC/package.json" "$dest/package.json"
  cp -f "$REMARK_GFM_STUB_SRC/src/index.js" "$dest/src/index.js"
  log "remark-gfm stub installed at $dest (no-op passthrough; see stub docstring)"
}

# ──────────────────────────────────────────────────────────────────────
# Start apps
# ──────────────────────────────────────────────────────────────────────
start_frontend() {
  log "starting apps-old/frontend on :$OLD_FRONTEND_PORT (next dev --webpack)..."
  (
    cd "$OLD_FRONTEND_DIR"
    PORT="$OLD_FRONTEND_PORT" \
    ENV=development \
    NEXT_TELEMETRY_DISABLED=1 \
    ./node_modules/.bin/next dev -H 0.0.0.0 -p "$OLD_FRONTEND_PORT" --webpack \
      > "$LOG_DIR/old-frontend.log" 2>&1 &
    echo $! > "$PID_DIR/old-frontend.pid"
  )
  wait_for_port "$OLD_FRONTEND_PORT" "old-frontend"
}

start_admin() {
  log "starting apps-old/admin-frontend on :$OLD_ADMIN_PORT (next dev --webpack)..."
  (
    cd "$OLD_ADMIN_DIR"
    PORT="$OLD_ADMIN_PORT" \
    ENV=development \
    NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin \
    NEXT_TELEMETRY_DISABLED=1 \
    ./node_modules/.bin/next dev -H 0.0.0.0 -p "$OLD_ADMIN_PORT" --webpack \
      > "$LOG_DIR/old-admin.log" 2>&1 &
    echo $! > "$PID_DIR/old-admin.pid"
  )
  wait_for_port "$OLD_ADMIN_PORT" "old-admin"
}

print_urls() {
  cat <<EOF

  ──────────────────────────────────────────────────────────
  EPSX WAVE 21 DEV-OLD (Next.js apps) READY
  ──────────────────────────────────────────────────────────
  OLD frontend (apps-old/frontend)       http://localhost:$OLD_FRONTEND_PORT
  OLD admin    (apps-old/admin-frontend) http://localhost:$OLD_ADMIN_PORT
  (NEW Dioxus frontend is on :4000, NEW Dioxus admin is on :4001)
  ──────────────────────────────────────────────────────────
  Logs:  $LOG_DIR/{old-frontend,old-admin}.log
  PIDs:  $PID_DIR/{old-frontend,old-admin}.pid
  Auth:  OLD apps use the dev SIWE flow (no bypass). To log in,
         use the dev wallet / paste a valid epsx_token cookie from
         a real dev session.
  Mode:  next dev --webpack (Turbopack off — see header comment
         for why; the OLD apps' tsconfig path-aliases for @/shared/*
         don't play with Turbopack's project-root sandbox).
  ──────────────────────────────────────────────────────────

EOF
}

# ──────────────────────────────────────────────────────────────────────
# Subcommand dispatch
# ──────────────────────────────────────────────────────────────────────
cmd_up() {
  local frontend=1 admin=1
  while [ $# -gt 0 ]; do
    case "$1" in
      frontend) admin=0; shift ;;
      admin)    frontend=0; shift ;;
      *) warn "unknown up flag: $1"; shift ;;
    esac
  done
  preflight
  write_env
  [ "$frontend" = 1 ] && ensure_deps "$OLD_FRONTEND_DIR"
  [ "$admin" = 1 ] && ensure_deps "$OLD_ADMIN_DIR"
  [ "$frontend" = 1 ] && start_frontend
  [ "$admin" = 1 ] && start_admin
  print_urls
}

cmd_down() {
  for pid_file in "$PID_DIR"/*.pid; do
    [ -f "$pid_file" ] || continue
    local pid; pid=$(cat "$pid_file")
    local name; name=$(basename "$pid_file" .pid)
    if kill -0 "$pid" 2>/dev/null; then
      log "stopping $name (pid $pid)"
      kill "$pid" 2>/dev/null || true
      sleep 0.3
      kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
    else
      log "$name (pid $pid) not running"
    fi
    rm -f "$pid_file"
  done
  log "all dev-old processes stopped"
}

cmd_status() {
  for pid_file in "$PID_DIR"/*.pid; do
    [ -f "$pid_file" ] || continue
    local pid; pid=$(cat "$pid_file")
    local name; name=$(basename "$pid_file" .pid)
    if kill -0 "$pid" 2>/dev/null; then
      log "$name (pid $pid) RUNNING"
    else
      log "$name (pid $pid) STOPPED (stale pid file)"
    fi
  done
  for port in "$OLD_FRONTEND_PORT" "$OLD_ADMIN_PORT"; do
    if is_port_listening "$port"; then
      log "port :$port LISTENING (old apps)"
    else
      log "port :$port free"
    fi
  done
}

usage() {
  sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
}

main() {
  local cmd=${1:-usage}
  shift || true
  case "$cmd" in
    up)      cmd_up "$@" ;;
    down)    cmd_down "$@" ;;
    status)  cmd_status "$@" ;;
    help|--help|-h) usage ;;
    *)       usage; exit 1 ;;
  esac
}

main "$@"
