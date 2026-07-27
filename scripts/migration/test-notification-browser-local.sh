#!/bin/sh
set -eu

# Guarded browser smoke proof for the Rust/Dioxus frontend BFF. This starts
# only a local process with an unreachable upstream URL; it performs no
# database, provider, staging, or production access.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
port=$((3200 + ($$ % 100)))
server_pid=""
server_log=""
browser_session="epsx-notification-browser-$$"

die() {
  echo "notification-browser-local: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --allow-local)
      allow_local=true
      shift
      ;;
    --port)
      [ "$#" -ge 2 ] || die "--port requires a local port"
      port=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command starts only a local frontend process"
case "$port" in
  *[!0-9]*|'') die "port must be numeric" ;;
esac
[ "$port" -ge 1024 ] && [ "$port" -le 65535 ] || die "port must be between 1024 and 65535"

command -v cargo >/dev/null 2>&1 || die "cargo is required"
command -v curl >/dev/null 2>&1 || die "curl is required"
command -v agent-browser >/dev/null 2>&1 || die "agent-browser is required"
command -v lsof >/dev/null 2>&1 || die "lsof is required for scoped local cleanup"

if lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null | grep -q .; then
  die "local port $port is already in use; choose another --port"
fi

server_log=$(mktemp "${TMPDIR:-/tmp}/epsx-notification-browser.XXXXXX")
cleanup() {
  agent-browser --session "$browser_session" close >/dev/null 2>&1 || true
  if [ -n "$server_pid" ] && kill -0 "$server_pid" 2>/dev/null; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  for listener_pid in $(lsof -nP -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null || true); do
    kill "$listener_pid" 2>/dev/null || true
  done
  rm -f -- "$server_log"
}
trap cleanup EXIT HUP INT TERM

EPSX_ENV=development \
API_URL=http://127.0.0.1:9 \
OIDC_ISSUER=http://127.0.0.1:9 \
PORT="$port" \
HOST=127.0.0.1 \
  cargo run -p epsx-frontend --bin bff-frontend --locked >"$server_log" 2>&1 &
server_pid=$!

ready=false
attempt=1
while [ "$attempt" -le 80 ]; do
  if health_code=$(curl -sS -o /dev/null -w '%{http_code}' "http://127.0.0.1:$port/api/health" 2>/dev/null) \
    && [ "$health_code" = 200 ]; then
    ready=true
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    die "frontend BFF exited before /api/health became ready"
  fi
  sleep 0.25
  attempt=$((attempt + 1))
done
[ "$ready" = true ] || die "frontend BFF did not become healthy"

agent-browser --session "$browser_session" --allowed-domains localhost \
  open "http://localhost:$port/" >/dev/null
agent-browser --session "$browser_session" --allowed-domains localhost \
  wait --load networkidle >/dev/null
home_title=$(agent-browser --session "$browser_session" get title)
[ "$home_title" = "Home — EPSX" ] || die "unexpected home title: $home_title"
home_snapshot=$(agent-browser --session "$browser_session" snapshot -i)
printf '%s' "$home_snapshot" | grep -q 'heading "Explore Market Analytics With Verified Data"' \
  || die "home accessibility heading is missing"

agent-browser --session "$browser_session" --allowed-domains localhost \
  open "http://localhost:$port/notifications" >/dev/null
agent-browser --session "$browser_session" --allowed-domains localhost \
  wait --load networkidle >/dev/null
notifications_url=$(agent-browser --session "$browser_session" get url)
case "$notifications_url" in
  *'/auth?return_url=%2Fnotifications'*) ;;
  *) die "signed-out notifications did not redirect to authentication: $notifications_url" ;;
esac

agent-browser --session "$browser_session" --allowed-domains localhost \
  open "http://localhost:$port/account" >/dev/null
agent-browser --session "$browser_session" --allowed-domains localhost \
  wait --load networkidle >/dev/null
account_text=$(agent-browser --session "$browser_session" get text body)
printf '%s' "$account_text" | grep -q 'Notification Preferences' \
  || die "account notification preferences section is missing"
printf '%s' "$account_text" | grep -q 'Sign in to view notification preferences' \
  || die "signed-out account state is not truthful"

agent-browser --session "$browser_session" --allowed-domains localhost \
  open "http://localhost:$port/" >/dev/null
agent-browser --session "$browser_session" --allowed-domains localhost \
  wait --load networkidle >/dev/null
home_runtime=$(agent-browser --session "$browser_session" eval \
  'JSON.stringify({targets:document.querySelectorAll("[data-notification-target], [data-notification-badge], [data-state=unavailable]").length, eventSource:[...document.scripts].some(s => (s.src||s.textContent).includes("EventSource"))})')
printf '%s' "$home_runtime" | grep -q 'targets.*0' \
  || die "signed-out home mounted a notification target: $home_runtime"
printf '%s' "$home_runtime" | grep -q 'eventSource.*false' \
  || die "signed-out home mounted an EventSource runtime: $home_runtime"

echo "notification-browser-local: PASS — Rust/Dioxus home rendered accessibly, signed-out notifications redirected, account preferences stayed truthful, and no signed-out notification runtime mounted"
echo "notification-browser-local: LIMIT — local browser/BFF smoke only; no authenticated browser, provider, staging, or production evidence"
