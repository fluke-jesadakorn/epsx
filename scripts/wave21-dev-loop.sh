#!/usr/bin/env bash
# wave21-dev-loop.sh — start the new Dioxus BFFs (frontend + admin) locally
# against the dev K8s cluster's backend.
#
# This is the dev loop for the pixel-recheck tracks:
#   - NO K8s manifests touched
#   - NO `:dev` images rebuilt
#   - NO port collisions with the prod K8s NodePorts
#   - Reuses the dev K8s backend via `kubectl port-forward` (read-only
#     connection to the cluster, no cluster state change)
#
# Ports:
#   18080 — local → K8s svc/epsx-backend :8080 (the BFFs' API_URL)
#   4000  — bff-frontend (user-facing pages, dev loop)
#   4001  — bff-admin (admin pages + admin API, dev loop)
#
# These ports are deliberately:
#   - Not 3000/3001 (which the K8s pods use inside the cluster)
#   - Not 30080/30101/30102/... (the K8s NodePorts on the host)
#   - Not 4700/4701/9180 (the prod Cloudflare-tunnel bridges)
#   - Not 5432/5433/6379/6380/9100 (the local DB/Redis/MinIO ports)
# So nothing should collide.
#
# Usage:
#   # default: builds, then runs with the dev auth bypass ON
#   ./scripts/wave21-dev-loop.sh up
#
#   # without auth bypass (default = ON; use this to see the SIWE gate):
#   ./scripts/wave21-dev-loop.sh up --no-bypass
#
#   # only the frontend BFF (port 4000)
#   ./scripts/wave21-dev-loop.sh frontend
#
#   # only the admin BFF (port 4001)
#   ./scripts/wave21-dev-loop.sh admin
#
#   # port-forward only (no BFFs) — useful if you just want a shell
#   # into the cluster's backend
#   ./scripts/wave21-dev-loop.sh port-forward
#
#   # tear down
#   ./scripts/wave21-dev-loop.sh down
#
#   # status
#   ./scripts/wave21-dev-loop.sh status

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
WORKTREE_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

# Configurable
KUBECONFIG_PATH=${KUBECONFIG:-/tmp/k3s-default-clean.yaml}
K8S_NAMESPACE=${K8S_NAMESPACE:-epsx-dev}
LOCAL_API_PORT=18080
FRONTEND_PORT=4000
ADMIN_PORT=4001
FRONTEND_BIN=bff-frontend
ADMIN_BIN=bff-admin
LOG_DIR=${LOG_DIR:-/tmp/epsx-wave21-dev-loop-logs}
PID_DIR=${PID_DIR:-/tmp/epsx-wave21-dev-loop-pids}

mkdir -p "$LOG_DIR" "$PID_DIR"

log()  { printf '\033[1;34m[wave21]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[wave21]\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[1;31m[wave21]\033[0m %s\n' "$*" >&2; }

is_port_listening() {
  local port=$1
  lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
}

wait_for_port() {
  local port=$1
  local label=$2
  local i
  for i in $(seq 1 50); do
    if is_port_listening "$port"; then
      log "$label listening on :$port"
      return 0
    fi
    sleep 0.2
  done
  err "$label did not bind to :$port within 10s; check $LOG_DIR/${label}.log"
  return 1
}

start_port_forward() {
  log "Starting kubectl port-forward svc/epsx-backend :$LOCAL_API_PORT → :8080 (cluster)..."
  if is_port_listening "$LOCAL_API_PORT"; then
    warn "port $LOCAL_API_PORT already in use; assuming port-forward is already running"
    return 0
  fi
  KUBECONFIG="$KUBECONFIG_PATH" kubectl port-forward \
    -n "$K8S_NAMESPACE" svc/epsx-backend "$LOCAL_API_PORT":8080 \
    > "$LOG_DIR/port-forward.log" 2>&1 &
  echo $! > "$PID_DIR/port-forward.pid"
  wait_for_port "$LOCAL_API_PORT" "port-forward"
}

start_frontend() {
  local api_url=${API_URL:-http://localhost:$LOCAL_API_PORT}
  local bypass=${EPSX_DEV_AUTH_BYPASS:-1}
  log "Starting $FRONTEND_BIN on :$FRONTEND_PORT (api=$api_url, bypass=$bypass)..."
  (
    cd "$WORKTREE_ROOT"
    EPSX_DEV_AUTH_BYPASS="$bypass" \
    PORT="$FRONTEND_PORT" \
    API_URL="$api_url" \
    cargo run -p epsx-frontend --bin "$FRONTEND_BIN" --release \
      > "$LOG_DIR/frontend.log" 2>&1 &
    echo $! > "$PID_DIR/frontend.pid"
  )
  wait_for_port "$FRONTEND_PORT" "$FRONTEND_BIN"
}

start_admin() {
  local api_url=${API_URL:-http://localhost:$LOCAL_API_PORT}
  local bypass=${EPSX_DEV_AUTH_BYPASS:-1}
  log "Starting $ADMIN_BIN on :$ADMIN_PORT (api=$api_url, bypass=$bypass)..."
  (
    cd "$WORKTREE_ROOT"
    EPSX_DEV_AUTH_BYPASS="$bypass" \
    PORT="$ADMIN_PORT" \
    API_URL="$api_url" \
    cargo run -p epsx-admin --bin "$ADMIN_BIN" --release \
      > "$LOG_DIR/admin.log" 2>&1 &
    echo $! > "$PID_DIR/admin.pid"
  )
  wait_for_port "$ADMIN_PORT" "$ADMIN_BIN"
}

print_urls() {
  cat <<EOF

  ──────────────────────────────────────────────────────────
  EPSX WAVE 21 DEV LOOP READY
  ──────────────────────────────────────────────────────────
  bff-frontend  (user pages)  http://localhost:$FRONTEND_PORT
  bff-admin     (admin pages) http://localhost:$ADMIN_PORT
  K8s backend   (port-fwd)    http://localhost:$LOCAL_API_PORT
  API_URL (BFF)               http://localhost:$LOCAL_API_PORT
  ──────────────────────────────────────────────────────────
  Logs:  $LOG_DIR/{frontend,admin,port-forward}.log
  PIDs:  $PID_DIR/{frontend,admin,port-forward}.pid
  Auth bypass: EPSX_DEV_AUTH_BYPASS=${EPSX_DEV_AUTH_BYPASS:-1}
               (unset to require SIWE login)
  ──────────────────────────────────────────────────────────

EOF
}

cmd_up() {
  local frontend=1 admin=1 pf=1
  while [ $# -gt 0 ]; do
    case "$1" in
      --no-bypass) export EPSX_DEV_AUTH_BYPASS=0; shift ;;
      --bypass)    export EPSX_DEV_AUTH_BYPASS=1; shift ;;
      --no-frontend) frontend=0; shift ;;
      --no-admin)    admin=0; shift ;;
      *) warn "unknown up flag: $1"; shift ;;
    esac
  done
  [ "$pf" = 1 ] && start_port_forward
  [ "$frontend" = 1 ] && start_frontend
  [ "$admin" = 1 ] && start_admin
  print_urls
}

cmd_frontend() { start_port_forward; start_frontend; print_urls; }
cmd_admin()    { start_port_forward; start_admin;    print_urls; }
cmd_port_forward() { start_port_forward; print_urls; }

cmd_down() {
  for pid_file in "$PID_DIR"/*.pid; do
    [ -f "$pid_file" ] || continue
    local pid; pid=$(cat "$pid_file")
    local name; name=$(basename "$pid_file" .pid)
    if kill -0 "$pid" 2>/dev/null; then
      log "stopping $name (pid $pid)"
      kill "$pid" 2>/dev/null || true
      # give it a moment, then SIGKILL if still alive
      sleep 0.3
      kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
    else
      log "$name (pid $pid) not running"
    fi
    rm -f "$pid_file"
  done
  log "all wave21 dev-loop processes stopped"
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
  for port in "$LOCAL_API_PORT" "$FRONTEND_PORT" "$ADMIN_PORT"; do
    if is_port_listening "$port"; then
      log "port :$port LISTENING"
    else
      log "port :$port free"
    fi
  done
}

usage() {
  sed -n '2,55p' "$0" | sed 's/^# \{0,1\}//'
}

main() {
  local cmd=${1:-usage}
  shift || true
  case "$cmd" in
    up)            cmd_up "$@" ;;
    frontend)      cmd_frontend "$@" ;;
    admin)         cmd_admin "$@" ;;
    port-forward)  cmd_port_forward "$@" ;;
    down)          cmd_down "$@" ;;
    status)        cmd_status "$@" ;;
    help|--help|-h) usage ;;
    *)             usage; exit 1 ;;
  esac
}

main "$@"
