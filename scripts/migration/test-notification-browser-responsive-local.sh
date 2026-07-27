#!/bin/sh
set -eu

# Authenticated responsive/accessibility smoke for the Rust/Dioxus notification
# pages. This is intentionally read-only and local-only; it does not exercise
# provider delivery or mutate notification state.
allow_local=false
browser_session="epsx-notification-browser-responsive-$$"
base_url=${NOTIFICATION_BROWSER_BASE_URL:-http://localhost:3000}
access_token=${NOTIFICATION_BROWSER_AUTH_TOKEN:-}

die() {
  echo "notification-browser-responsive-local: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --allow-local)
      allow_local=true
      shift
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command is restricted to a local browser origin"
[ -n "$access_token" ] || die "NOTIFICATION_BROWSER_AUTH_TOKEN is required"
case "$base_url" in
  http://localhost:*) allowed_domain=localhost ;;
  http://127.0.0.1:*) allowed_domain=127.0.0.1 ;;
  *) die "NOTIFICATION_BROWSER_BASE_URL must be an explicit local HTTP origin" ;;
esac

command -v agent-browser >/dev/null 2>&1 || die "agent-browser is required"
command -v curl >/dev/null 2>&1 || die "curl is required"

cleanup() {
  agent-browser --session "$browser_session" close >/dev/null 2>&1 || true
}
trap cleanup EXIT HUP INT TERM

headers=$(printf '{"Authorization":"Bearer %s"}' "$access_token")
health_code=$(curl -sS -o /dev/null -w '%{http_code}' "$base_url/api/health")
[ "$health_code" = 200 ] || die "frontend BFF health returned $health_code"

agent-browser --session "$browser_session" --headers "$headers" --allowed-domains "$allowed_domain" open "$base_url/account" >/dev/null
agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" wait --load networkidle >/dev/null
agent-browser --session "$browser_session" set viewport 390 844 >/dev/null
account_shape=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var root=document.querySelector("[data-epsx-notification-push=\"true\"]"); var enable=root&&root.querySelector("[data-push-action=\"enable\"]"); return [window.matchMedia("(max-width: 640px)").matches,document.documentElement.scrollWidth<=window.innerWidth,!!root,!!enable,!!(enable&&((enable.getAttribute("aria-label")||enable.textContent||"").trim()))].join("|")})()')
account_shape=${account_shape#\"}
account_shape=${account_shape%\"}
[ "$account_shape" = 'true|true|true|true|true' ] || die "mobile account layout/control accessibility contract failed: $account_shape"

agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" open "$base_url/notifications" >/dev/null
agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" wait --load networkidle >/dev/null
notifications_shape=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var list=document.querySelector("[data-notifications-window]"); var status=document.querySelector("[data-notifications-live-status=\"true\"]"); var first=list&&list.querySelector("h3.notification-title"); var empty=document.querySelector(".notifications-empty"); return [window.matchMedia("(max-width: 640px)").matches,document.documentElement.scrollWidth<=window.innerWidth,!!list,!!status,!!(first||empty)].join("|")})()')
notifications_shape=${notifications_shape#\"}
notifications_shape=${notifications_shape%\"}
[ "$notifications_shape" = 'true|true|true|true|true' ] || die "mobile notifications layout/status contract failed: $notifications_shape"

focus_result=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var link=document.querySelector("a[href=\"/notifications\"]"); if(!link)return "missing"; link.focus(); return document.activeElement===link ? "focused" : "not-focused"})()')
focus_result=${focus_result#\"}
focus_result=${focus_result%\"}
[ "$focus_result" = focused ] || die "notification navigation did not retain a native keyboard focus target: $focus_result"

echo "notification-browser-responsive-local: PASS — authenticated Dioxus account and notifications pages fit a 390px viewport with native focusable controls and live status"
echo "notification-browser-responsive-local: LIMIT — local responsive/accessibility browser smoke only; no mobile device matrix, provider, staging, or production evidence"
