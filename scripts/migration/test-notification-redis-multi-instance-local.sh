#!/bin/sh
set -eu

# Local-only Redis fan-out proof. The ignored Rust test starts two independent
# pubsub listeners, verifies one wake-up reaches both, and verifies the local
# PostgreSQL-backed wake-up path remains bounded when a Redis connection fails.

allow_local=false

die() {
  echo "notification-redis-multi-instance-local: ERROR: $*" >&2
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

[ "$allow_local" = true ] || die "pass --allow-local; this command uses only local Redis"
command -v redis-cli >/dev/null 2>&1 || die "redis-cli is required"
command -v cargo >/dev/null 2>&1 || die "cargo is required"

redis_url=${NOTIFICATION_RUNTIME_REDIS_URL:-redis://:epsx@127.0.0.1:6379}
redis_host=127.0.0.1
redis_port=6379
if [ "${redis_url#redis://}" != "$redis_url" ]; then
  redis_authority=${redis_url#redis://}
  redis_authority=${redis_authority#*@}
  redis_host=${redis_authority%%:*}
  redis_port=${redis_authority##*:}
fi
redis-cli -h "$redis_host" -p "$redis_port" -a epsx ping 2>/dev/null | grep -qx PONG \
  || die "local Redis did not respond to PING"

NOTIFICATION_RUNTIME_REDIS_URL="$redis_url" \
  cargo test -p epsx-notification --bin notification --locked \
    tests::redis_multi_instance_fanout_and_loss_fallback_are_bounded \
    -- --ignored --exact --nocapture

command -v redis-server >/dev/null 2>&1 || die "redis-server is required for the restart-recovery phase"
cargo test -p epsx-notification --bin notification --locked \
  tests::redis_broker_restart_recovers_multi_instance_listeners \
  -- --ignored --exact --nocapture

echo "notification-redis-multi-instance-local: PASS — two Redis listeners received fan-out, local replay survived a bounded failure, and both listeners recovered after an ephemeral broker restart"
echo "notification-redis-multi-instance-local: LIMIT — disposable local Redis/Rust listeners only; no browser receipt, external provider, staging, or production evidence"
