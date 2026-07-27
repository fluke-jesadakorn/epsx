#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-operations.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

backfill="docs/migration/fixtures/notification-backfill.jsonl"
legacy_backfill="docs/migration/fixtures/notification-backfill-legacy.jsonl"
source_fixture="docs/migration/fixtures/notification-reconcile-source.jsonl"
target_fixture="docs/migration/fixtures/notification-reconcile-target.jsonl"

cargo xtask notification-backfill --dry-run --input "$backfill" >"$temp_dir/backfill.out"
grep -q "records=3 eligible=3 invalid=0 duplicate_source_events=0" "$temp_dir/backfill.out"
grep -q "writes=0 network=0 database=0 checkpoint=start" "$temp_dir/backfill.out"

cargo xtask notification-backfill --dry-run --input "$backfill" --after notif-event-001 >"$temp_dir/checkpoint.out"
grep -q "records=3 eligible=2 invalid=0 duplicate_source_events=0" "$temp_dir/checkpoint.out"
grep -q "checkpoint=notif-event-001" "$temp_dir/checkpoint.out"

cargo xtask notification-backfill --dry-run --legacy --input "$legacy_backfill" >"$temp_dir/legacy-backfill.out"
grep -q "format=legacy records=3 eligible=3 invalid=0 duplicate_source_events=0 legacy_records=3" "$temp_dir/legacy-backfill.out"
grep -q "legacy_fields_preserved=27" "$temp_dir/legacy-backfill.out"
grep -q '"pending":1' "$temp_dir/legacy-backfill.out"
grep -q '"sent":1' "$temp_dir/legacy-backfill.out"
grep -q '"failed":1' "$temp_dir/legacy-backfill.out"
grep -q "writes=0 network=0 database=0 checkpoint=start" "$temp_dir/legacy-backfill.out"

sed 's/"topic_name":null/"topic_name":"unsupported-topic"/' "$legacy_backfill" >"$temp_dir/legacy-topic.jsonl"
set +e
cargo xtask notification-backfill --dry-run --legacy --input "$temp_dir/legacy-topic.jsonl" >"$temp_dir/legacy-topic.out" 2>&1
legacy_topic_status=$?
set -e
[ "$legacy_topic_status" -ne 0 ] || {
  cat "$temp_dir/legacy-topic.out" >&2
  echo "notification-operations self-test: expected topic-only legacy mapping to fail" >&2
  exit 1
}
grep -q "invalid=3" "$temp_dir/legacy-topic.out"

cargo xtask notification-reconcile --dry-run --source "$source_fixture" --target "$target_fixture" >"$temp_dir/reconcile.out"
bun -e '
const output = await Bun.file(process.argv[1]).text();
const json = JSON.parse(output.slice(0, output.indexOf("\nnotification-reconcile:")));
if (json.source_records !== 3 || json.target_records !== 3 || json.invalid_source_records !== 0 || json.invalid_target_records !== 0 || json.duplicate_source_events !== 0 || json.duplicate_target_events !== 0 || json.missing_target_events !== 0 || json.orphan_target_events !== 0 || json.target_sent_without_provider_id !== 0 || json.template_identity_drift !== 0 || json.preference_identity_drift !== 0 || json.provider_identity_drift !== 0 || !json.wallet_checksum_match || !json.source_target_event_set_match || !json.status_distribution_match || !json.broadcast_count_match || json.source_wallet_checksum !== "c07f8760e2c1053389d95533d6a4bfc660825ce99e6244cf397574acd735603e") process.exit(1);
' "$temp_dir/reconcile.out"
grep -q "writes=0 network=0 database=0" "$temp_dir/reconcile.out"

cp "$target_fixture" "$temp_dir/drift.jsonl"
sed -i.bak 's/provider-message-002/provider-message-drifted/' "$temp_dir/drift.jsonl"
set +e
cargo xtask notification-reconcile --dry-run --source "$source_fixture" --target "$temp_dir/drift.jsonl" >"$temp_dir/drift.out" 2>&1
drift_status=$?
set -e
[ "$drift_status" -ne 0 ] || {
  cat "$temp_dir/drift.out" >&2
  echo "notification-operations self-test: expected provider drift to fail" >&2
  exit 1
}
grep -q "provider_identity_drift" "$temp_dir/drift.out"

set +e
cargo xtask notification-backfill --dry-run --input "$backfill" --after missing-checkpoint >"$temp_dir/missing-checkpoint.out" 2>&1
checkpoint_status=$?
set -e
[ "$checkpoint_status" -ne 0 ] || {
  cat "$temp_dir/missing-checkpoint.out" >&2
  echo "notification-operations self-test: expected missing checkpoint to fail" >&2
  exit 1
}
grep -q "checkpoint was not found" "$temp_dir/missing-checkpoint.out"

echo "notification-operations self-test: PASS (backfill=3, legacy-map=3, checkpoint=2, reconcile=matched, drift-rejection=1)"
