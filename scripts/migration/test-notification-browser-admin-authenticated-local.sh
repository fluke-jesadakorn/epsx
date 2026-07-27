#!/bin/sh
set -eu

# Authenticated browser proof for the Rust/Dioxus admin notification inventory.
# The caller supplies a short-lived local admin bearer; this script never
# prints, persists, or mutates the token and performs a read-only page check.

allow_local=false
browser_session="epsx-notification-admin-browser-auth-$$"
base_url=${NOTIFICATION_ADMIN_BROWSER_BASE_URL:-http://localhost:3001}
access_token=${NOTIFICATION_ADMIN_BROWSER_AUTH_TOKEN:-}

die() {
  echo "notification-browser-admin-authenticated-local: ERROR: $*" >&2
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
[ -n "$access_token" ] || die "NOTIFICATION_ADMIN_BROWSER_AUTH_TOKEN is required"
case "$base_url" in
  http://localhost:*) allowed_domain=localhost ;;
  http://127.0.0.1:*) allowed_domain=127.0.0.1 ;;
  *) die "NOTIFICATION_ADMIN_BROWSER_BASE_URL must be an explicit local HTTP origin" ;;
esac

command -v agent-browser >/dev/null 2>&1 || die "agent-browser is required"
command -v curl >/dev/null 2>&1 || die "curl is required"

cleanup() {
  agent-browser --session "$browser_session" close >/dev/null 2>&1 || true
}
trap cleanup EXIT HUP INT TERM

headers=$(printf '{"Authorization":"Bearer %s"}' "$access_token")
health_code=$(curl -sS -o /dev/null -w '%{http_code}' "$base_url/api/health")
[ "$health_code" = 200 ] || die "admin BFF health returned $health_code"

agent-browser --session "$browser_session" --headers "$headers" --allowed-domains "$allowed_domain" open "$base_url/notifications/manage" >/dev/null
agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" wait --load networkidle >/dev/null
page_url=$(agent-browser --session "$browser_session" get url)
case "$page_url" in
  "$base_url/notifications/manage"|"$base_url/notifications/manage"\?*) ;;
  *) die "authenticated admin notifications redirected unexpectedly: $page_url" ;;
esac

page_text=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" get text body)
printf '%s' "$page_text" | grep -q 'Notifications' || die "admin notification heading is missing"
printf '%s' "$page_text" | grep -Eq 'Delivery inventory|No notifications found' || die "admin notification inventory state is missing"
inventory_state=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var node=document.querySelector("[data-admin-notifications-state]"); return node ? node.getAttribute("data-admin-notifications-state") : "missing"})()')
inventory_state=${inventory_state#\"}
inventory_state=${inventory_state%\"}
case "$inventory_state" in
  ready|empty) ;;
  *) die "admin notification inventory entered an untrusted state: $inventory_state" ;;
esac
if printf '%s' "$page_text" | grep -q 'Sign in required'; then
  die "authenticated admin notifications rendered signed-out state"
fi

agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" open "$base_url/admin/notifications/create" >/dev/null
agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" wait --load networkidle >/dev/null
compose_url=$(agent-browser --session "$browser_session" get url)
[ "$compose_url" = "$base_url/admin/notifications/create" ] || die "authenticated admin compose redirected unexpectedly: $compose_url"
compose_text=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" get text body)
printf '%s' "$compose_text" | grep -q 'Send an in-app notification' || die "bounded admin compose form is missing"
printf '%s' "$compose_text" | grep -q 'Recipient wallet' || die "canonical wallet recipient field is missing"
printf '%s' "$compose_text" | grep -q 'Queue notification' || die "bounded queue action is missing"
compose_fields=$(agent-browser --session "$browser_session" --allowed-domains "$allowed_domain" eval '(()=>{var f=document.querySelector("form[action=\"/notifications/create\"]"); if(!f)return "missing"; var names=Array.from(f.querySelectorAll("input,textarea,select")).map(x=>x.getAttribute("name")); var forbidden=["broadcast","plan_id","schedule","image_url","action_url","data"].some(x=>names.includes(x)); return [f.getAttribute("method"),names.join(","),forbidden].join("|")})()')
compose_fields=${compose_fields#\"}
compose_fields=${compose_fields%\"}
[ "$compose_fields" = 'post|recipient_wallet_address,title,message|false' ] || die "admin compose fields exceeded the bounded canonical-wallet contract: $compose_fields"

echo "notification-browser-admin-authenticated-local: PASS — authenticated Dioxus admin inventory and bounded canonical-wallet compose rendered through the permissioned BFF"
echo "notification-browser-admin-authenticated-local: LIMIT — short-lived local admin bearer and browser only; no provider, staging, multi-instance, or production evidence"
