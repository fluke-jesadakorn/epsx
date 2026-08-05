#!/bin/sh
set -eu

# Authenticated browser proof for the Rust/Dioxus notification surfaces. The
# caller supplies a short-lived local access token; this script never prints,
# persists, or mutates the token and performs read-only browser assertions.

allow_local=false
browser_session="epsx-notification-browser-auth-$$"
base_url=${NOTIFICATION_BROWSER_BASE_URL:-http://localhost:3000}
access_token=${NOTIFICATION_BROWSER_AUTH_TOKEN:-}

die() {
  echo "notification-browser-authenticated-local: ERROR: $*" >&2
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
account_url=$(agent-browser --session "$browser_session" get url)
case "$account_url" in
  "$base_url/account"|"$base_url/account"\?*) ;;
  *) die "authenticated account redirected unexpectedly: $account_url" ;;
esac
account_text=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" get text body)
printf '%s' "$account_text" | grep -q 'Notification preferences loaded' || die "authenticated account did not load notification preferences"
printf '%s' "$account_text" | grep -q 'Save preferences' || die "native notification preferences form is missing"
printf '%s' "$account_text" | grep -q 'Browser notifications' || die "browser notification control is missing"
printf '%s' "$account_text" | grep -q 'Browser push is unavailable until the notification service is configured.' || die "local push capability did not fail closed"
push_control=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var root=document.querySelector("[data-epsx-notification-push=\"true\"]"); var enable=root&&root.querySelector("[data-push-action=\"enable\"]"); return root&&enable ? root.getAttribute("data-push-state")+"|"+enable.disabled+"|"+enable.hidden : "missing"})()')
push_control=${push_control#\"}
push_control=${push_control%\"}
[ "$push_control" = 'unavailable|true|false' ] || die "local push control was not disabled after capability check"
if printf '%s' "$account_text" | grep -q 'Notification preferences are unavailable'; then
  die "authenticated account reported notification preferences unavailable"
fi

agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" open "$base_url/notifications" >/dev/null
agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" wait --load networkidle >/dev/null
notifications_url=$(agent-browser --session "$browser_session" get url)
[ "$notifications_url" = "$base_url/notifications" ] || die "authenticated notifications redirected unexpectedly: $notifications_url"
notifications_text=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" get text body)
printf '%s' "$notifications_text" | grep -q 'Notifications' || die "authenticated notifications heading is missing"
if printf '%s' "$notifications_text" | grep -q 'Sign in to view notifications'; then
  die "authenticated notifications rendered signed-out state"
fi
printf '%s' "$notifications_text" | grep -q 'Live notification updates' || die "authenticated notifications live status is missing"
live_runtime=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var root=document.querySelector("[data-notifications-live-status=\"true\"]"); var runtime=document.querySelector("script[data-epsx-notification-realtime-runtime]"); return root&&runtime ? root.getAttribute("data-notifications-live-state")||"unset" : "missing"})()')
live_runtime=${live_runtime#\"}
live_runtime=${live_runtime%\"}
case "$live_runtime" in
  connected|connecting|reconnecting|paused|unavailable|unset) ;;
  *) die "authenticated notifications live controller entered an unknown state: $live_runtime" ;;
esac

echo "notification-browser-authenticated-local: PASS — authenticated Dioxus account preferences loaded with native controls and notifications remained owner-authenticated"
echo "notification-browser-authenticated-local: LIMIT — short-lived local bearer and browser only; no provider, staging, multi-instance, or production evidence"
