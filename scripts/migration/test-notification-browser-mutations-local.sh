#!/bin/sh
set -eu

# Authenticated, disposable browser mutation proof for the Rust/Dioxus
# notification surfaces. The admin form enqueues one canonical-wallet in-app
# notification; the owner form saves a known-IANA timezone, marks that row
# read, acknowledges it, and removes it again. This is intentionally guarded
# to loopback origins and never accepts a provider or production URL.

allow_local=false
owner_token=${NOTIFICATION_BROWSER_MUTATION_OWNER_TOKEN:-}
admin_token=${NOTIFICATION_BROWSER_MUTATION_ADMIN_TOKEN:-}
frontend_url=${NOTIFICATION_BROWSER_MUTATION_FRONTEND_URL:-http://localhost:3000}
admin_url=${NOTIFICATION_BROWSER_MUTATION_ADMIN_URL:-http://localhost:3001}
owner_session="epsx-notification-owner-mutation-$$"
admin_session="epsx-notification-admin-mutation-$$"
marker="Dioxus mutation audit $(date +%s)-$$"
marker_json=$(printf '%s' "$marker" | jq -R .)

die() {
  echo "notification-browser-mutations-local: ERROR: $*" >&2
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

[ "$allow_local" = true ] || die "pass --allow-local; this audit is restricted to local browser origins"
[ -n "$owner_token" ] || die "NOTIFICATION_BROWSER_MUTATION_OWNER_TOKEN is required"
[ -n "$admin_token" ] || die "NOTIFICATION_BROWSER_MUTATION_ADMIN_TOKEN is required"

for url in "$frontend_url" "$admin_url"; do
  case "$url" in
    http://localhost:*|http://127.0.0.1:*) ;;
    *) die "browser URLs must be explicit local HTTP origins" ;;
  esac
done

command -v agent-browser >/dev/null 2>&1 || die "agent-browser is required"
command -v curl >/dev/null 2>&1 || die "curl is required"
command -v jq >/dev/null 2>&1 || die "jq is required"

cleanup() {
  agent-browser --session "$owner_session" close >/dev/null 2>&1 || true
  agent-browser --session "$admin_session" close >/dev/null 2>&1 || true
}
trap cleanup EXIT HUP INT TERM

frontend_domain=localhost
admin_domain=localhost
case "$frontend_url" in http://127.0.0.1:*) frontend_domain=127.0.0.1 ;; esac
case "$admin_url" in http://127.0.0.1:*) admin_domain=127.0.0.1 ;; esac

frontend_health=$(curl -sS -o /dev/null -w '%{http_code}' "$frontend_url/api/health")
[ "$frontend_health" = 200 ] || die "frontend BFF health returned $frontend_health"
admin_health=$(curl -sS -o /dev/null -w '%{http_code}' "$admin_url/api/health")
[ "$admin_health" = 200 ] || die "admin BFF health returned $admin_health"

admin_headers=$(printf '{"Authorization":"Bearer %s"}' "$admin_token")
agent-browser --session "$admin_session" --headers "$admin_headers" \
  --allowed-domains "$admin_domain" open "$admin_url/admin/notifications/create" >/dev/null
agent-browser --session "$admin_session" --allowed-domains "$admin_domain" wait --load networkidle >/dev/null
admin_text=$(agent-browser --session "$admin_session" --allowed-domains "$admin_domain" get text body)
printf '%s' "$admin_text" | grep -q 'Send an in-app notification' \
  || die "authenticated admin compose form did not render"

agent-browser --session "$admin_session" --allowed-domains "$admin_domain" \
  find label 'Recipient wallet' fill 0x1111111111111111111111111111111111111111 >/dev/null
agent-browser --session "$admin_session" --allowed-domains "$admin_domain" \
  find label 'Title' fill "$marker" >/dev/null
agent-browser --session "$admin_session" --allowed-domains "$admin_domain" \
  find label 'Message' fill "$marker" >/dev/null
agent-browser --session "$admin_session" --allowed-domains "$admin_domain" \
  find role button click --name 'Queue notification' >/dev/null
agent-browser --session "$admin_session" --allowed-domains "$admin_domain" wait --load networkidle >/dev/null
admin_url_after=$(agent-browser --session "$admin_session" get url)
case "$admin_url_after" in
  "$admin_url"/notifications/manage\?send=accepted*) ;;
  *) die "admin enqueue did not return the paired accepted redirect: $admin_url_after" ;;
esac
admin_text=$(agent-browser --session "$admin_session" --allowed-domains "$admin_domain" get text body)
printf '%s' "$admin_text" | grep -q 'Notification queued' \
  || die "admin enqueue did not render queued feedback"
printf '%s' "$admin_text" | grep -q "$marker" \
  || die "admin inventory did not reload the queued marker"

owner_headers=$(printf '{"Authorization":"Bearer %s"}' "$owner_token")
agent-browser --session "$owner_session" --headers "$owner_headers" \
  --allowed-domains "$frontend_domain" open "$frontend_url/account" >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" wait --load networkidle >/dev/null
owner_text=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" get text body)
printf '%s' "$owner_text" | grep -q 'Notification preferences loaded' \
  || die "owner preferences did not load before mutation"
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  snapshot -i >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  find label Timezone fill Asia/Bangkok >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  find role button click --name 'Save preferences' >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  wait --load networkidle >/dev/null
owner_url_after=$(agent-browser --session "$owner_session" get url)
case "$owner_url_after" in
  "$frontend_url"/account\?preferences=saved*) ;;
  *) die "owner preference save did not return the paired saved redirect: $owner_url_after" ;;
esac
owner_text=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" get text body)
printf '%s' "$owner_text" | grep -q 'Preferences saved' \
  || die "owner preference save did not render saved feedback"
printf '%s' "$owner_text" | grep -q 'Timezone: Asia/Bangkok' \
  || die "owner preference save was not reloaded from the service"

agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  open "$frontend_url/notifications" >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  wait --load networkidle >/dev/null
marker_id=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  eval "(()=>{const button=[...document.querySelectorAll('[data-notification-id]')].find(node=>node.closest('li')?.innerText.includes($marker_json)); return button?.closest('li')?.getAttribute('data-notification-id')||''})()")
marker_id=${marker_id#\"}
marker_id=${marker_id%\"}
[ -n "$marker_id" ] || die "owner notification list did not contain the queued marker"
case "$marker_id" in
  0x[0-9a-fA-F]*) ;;
  *) die "owner notification identity was not canonical: $marker_id" ;;
esac

agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  eval "document.querySelector(\"[data-notification-mutation='read'][data-notification-id='$marker_id']\").click()" >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" wait --load networkidle >/dev/null
row_class=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  eval "document.querySelector(\"li[data-notification-id='$marker_id']\")?.className||\"missing\"")
row_class=${row_class#\"}
row_class=${row_class%\"}
printf '%s' "$row_class" | grep -q 'notification-row-read' \
  || die "owner mark-read mutation did not reload the row as read: $row_class"

agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  eval "document.querySelector(\"[data-notification-mutation='acknowledge'][data-notification-id='$marker_id']\").click()" >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" wait --load networkidle >/dev/null
mutation_state=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  eval 'document.querySelector("[data-notification-mutation-status=\"true\"]")?.getAttribute("data-notification-mutation-state")||"missing"')
mutation_state=${mutation_state#\"}
mutation_state=${mutation_state%\"}
case "$mutation_state" in
  ready|missing) ;;
  error) die "owner acknowledgement mutation reported an error state" ;;
  *) die "owner acknowledgement mutation returned an unexpected state: $mutation_state" ;;
esac
owner_text=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" get text body)
printf '%s' "$owner_text" | grep -q 'Changes are saved by the notification service' \
  || die "owner acknowledgement mutation did not reload the saved notification page"

agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  eval "document.querySelector(\"[data-notification-mutation='delete'][data-notification-id='$marker_id']\").click()" >/dev/null 2>&1 || true
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" dialog accept >/dev/null 2>&1 || true
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" wait --load networkidle >/dev/null
owner_text=$(agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" get text body)
if printf '%s' "$owner_text" | grep -q "$marker"; then
  die "owner delete mutation left the disposable marker visible"
fi

# Restore the local owner's original neutral timezone so this audit does not
# leave a preference mutation behind for a later local check.
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  open "$frontend_url/account" >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" wait --load networkidle >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" snapshot -i >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  find label Timezone fill UTC >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  find role button click --name 'Save preferences' >/dev/null
agent-browser --session "$owner_session" --allowed-domains "$frontend_domain" \
  wait --load networkidle >/dev/null
owner_url_after=$(agent-browser --session "$owner_session" get url)
case "$owner_url_after" in
  "$frontend_url"/account\?preferences=saved*) ;;
  *) die "owner preference restoration did not return the saved redirect: $owner_url_after" ;;
esac

echo "notification-browser-mutations-local: PASS — authenticated Dioxus admin enqueue, owner preference save/reload, mark-read, acknowledgement, and disposable delete all completed through same-origin Rust BFFs"
echo "notification-browser-mutations-local: LIMIT — loopback bearer and disposable local row only; no provider, staging, multi-instance, device-matrix, or production evidence"
