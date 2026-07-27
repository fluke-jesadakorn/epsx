#!/bin/sh
set -eu

# Local-only Web Push transport proof. The Rust test uses the checked-in
# standards-compliant test VAPID/subscription vectors and a loopback HTTP
# provider, so it exercises encryption, VAPID headers, bounded payloads, and
# deterministic provider IDs without contacting an external push service.

allow_local=false

die() {
  echo "notification-push-provider-local: ERROR: $*" >&2
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

[ "$allow_local" = true ] || die "pass --allow-local; this command uses a loopback provider only"
command -v cargo >/dev/null 2>&1 || die "cargo is required"

cargo test -p epsx-notification --bin notification --locked \
  tests::push_provider_acceptance_encrypts_payload_and_uses_stable_message_id \
  -- --exact --nocapture

echo "notification-push-provider-local: PASS — encrypted Web Push payload, VAPID authorization, loopback acceptance, and stable provider ID were verified"
echo "notification-push-provider-local: LIMIT — loopback provider only; no browser permission, external provider, staging, or production delivery evidence"
