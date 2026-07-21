#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(git -C "$script_dir" rev-parse --show-toplevel)"
repo_root="$(cd "$repo_root" && pwd -P)"
classifier="$script_dir/a3-2-history-classification.sh"
contract="$repo_root/docs/migration/contracts/a3-2-history-classification.json"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-2-self-test.XXXXXX")"
test_root="$(cd "$test_root" && pwd -P)"
trap 'rm -rf -- "$test_root"' EXIT

pass_count=0
pass() {
  pass_count=$((pass_count + 1))
  printf 'ok %d - %s\n' "$pass_count" "$1"
}

expect_status() {
  local expected="$1"
  local actual
  shift
  set +e
  "$@"
  actual="$?"
  set -e
  if [ "$actual" -ne "$expected" ]; then
    echo "expected status $expected, observed $actual: $*" >&2
    exit 1
  fi
}

rebuild_manifest() {
  local fixture="$1"
  local digest_lines="$fixture/.digests.jsonl"
  local domain suffix artifact digest
  : > "$digest_lines"
  for domain in core analytics notifications payments; do
    for suffix in discovery inspection; do
      artifact="$domain.$suffix.json"
      digest="$(shasum -a 256 "$fixture/$artifact" | awk '{print $1}')"
      jq -cn --arg file "$artifact" --arg sha256 "$digest" '{file:$file,sha256:$sha256}' >> "$digest_lines"
    done
  done
  jq -s 'sort_by(.file)' "$digest_lines" > "$fixture/.digests.json"
  jq -S -n --slurpfile artifacts "$fixture/.digests.json" '{
    schemaVersion:1,
    package:"A3.1a",
    purpose:"read-only-database-preflight",
    productionReady:false,
    status:"inspection-captured-operator-classification-required",
    artifacts:$artifacts[0],
    stopReasons:[]
  }' > "$fixture/manifest.json"
  rm -f "$digest_lines" "$fixture/.digests.json"
}

write_domain() {
  local fixture="$1"
  local domain="$2"
  local evidence_class="$3"
  local migration_schema="public"

  jq -S -n --arg domain "$domain" --arg migrationSchema "$migration_schema" '{
    identity:{
      database:("fixture_" + $domain),
      user:"read_only_fixture",
      serverAddress:"127.0.0.1",
      serverPort:5432,
      serverVersion:"fixture",
      searchPath:"public",
      transactionReadOnly:"on"
    },
    migrationTables:[{schema:$migrationSchema,name:"__diesel_schema_migrations"}]
  }' > "$fixture/$domain.discovery.json"

  jq -S \
    --arg domain "$domain" \
    --arg evidenceClass "$evidence_class" \
    --arg migrationSchema "$migration_schema" '
    . as $contract |
    first($contract.classes[] | select(.domain == $domain and .evidenceClass == $evidenceClass)) as $rule |
    $contract.schemaProfiles[$rule.profile] as $profile |
    (
      $contract.baseMigrationVersions +
      $rule.requiredVersions +
      (if $rule.followupHistory == "some"
       then [first($contract.knownMigrationVersions[$rule.domain][] | select(. as $v | $contract.baseMigrationVersions | index($v) == null))]
       else [] end)
      | unique | sort
    ) as $versions |
    ($profile.requiredRelations + [$migrationSchema + ".__diesel_schema_migrations"] | unique | sort) as $relations |
    {
      migrationTable:{schema:$migrationSchema,name:"__diesel_schema_migrations"},
      migrationHistory:[$versions[] | {version:.,run_on:"2026-01-01 00:00:00"}],
      relations:[$relations[] | split(".") | {
        schema_name:.[0],relation_name:.[1],relation_kind:"r",estimated_rows:0
      }],
      columns:(
        [$profile.requiredColumns[] | split(".") | {
          schema_name:.[0],relation_name:.[1],ordinal:1,column_name:.[2],
          data_type:"text",not_null:false,default_expression:null
        }] +
        [
          {schema_name:$migrationSchema,relation_name:"__diesel_schema_migrations",ordinal:1,column_name:"version",data_type:"character varying",not_null:true,default_expression:null},
          {schema_name:$migrationSchema,relation_name:"__diesel_schema_migrations",ordinal:2,column_name:"run_on",data_type:"timestamp without time zone",not_null:true,default_expression:"CURRENT_TIMESTAMP"}
        ]
      ),
      constraints:[],indexes:[],triggers:[],functions:[],partitions:[],views:[]
    }
  ' "$contract" > "$fixture/$domain.inspection.json"
}

make_fixture() {
  local fixture="$1"
  local core_class="$2"
  local analytics_class="$3"
  local notifications_class="$4"
  local payments_class="$5"
  mkdir "$fixture"
  write_domain "$fixture" core "$core_class"
  write_domain "$fixture" analytics "$analytics_class"
  write_domain "$fixture" notifications "$notifications_class"
  write_domain "$fixture" payments "$payments_class"
  rebuild_manifest "$fixture"
}

class_for_domain() {
  case "$1" in
    core) printf '%s' v5 ;;
    analytics) printf '%s' public-v2 ;;
    notifications) printf '%s' baseline-v2-table-present ;;
    payments) printf '%s' v3 ;;
    *) return 1 ;;
  esac
}

make_fixture "$test_root/base" v5 public-v2 baseline-v2-table-present v3

mkdir "$test_root/fake-bin"
for forbidden_command in psql curl wget docker kubectl; do
  printf '#!/usr/bin/env bash\ntouch %q\nexit 97\n' "$test_root/FORBIDDEN_COMMAND_CALLED" > "$test_root/fake-bin/$forbidden_command"
  chmod 0755 "$test_root/fake-bin/$forbidden_command"
done

PATH="$test_root/fake-bin:$PATH" "$classifier" --input-dir "$test_root/base" > "$test_root/base-report-1.json" 2> "$test_root/base-stderr-1"
PATH="$test_root/fake-bin:$PATH" "$classifier" --input-dir "$test_root/base" > "$test_root/base-report-2.json" 2> "$test_root/base-stderr-2"
cmp "$test_root/base-report-1.json" "$test_root/base-report-2.json"
[ ! -e "$test_root/FORBIDDEN_COMMAND_CALLED" ]
jq -e '
  .status == "classified-offline-evidence-only" and
  .productionReady == false and
  (.classifications | length) == 4 and
  .stopReasons == [] and
  all(.classifications[]; (.evidenceSha256 | test("^[0-9a-f]{64}$")))
' "$test_root/base-report-1.json" >/dev/null
! grep -Eq 'fixture_|127\.0\.0\.1|read_only_fixture|relation|column|run_on' "$test_root/base-report-1.json"
pass "deterministic redacted report classifies four domains without database, network, container, or cluster commands"

while IFS=$'\t' read -r domain evidence_class; do
  core_class="$(class_for_domain core)"
  analytics_class="$(class_for_domain analytics)"
  notifications_class="$(class_for_domain notifications)"
  payments_class="$(class_for_domain payments)"
  case "$domain" in
    core) core_class="$evidence_class" ;;
    analytics) analytics_class="$evidence_class" ;;
    notifications) notifications_class="$evidence_class" ;;
    payments) payments_class="$evidence_class" ;;
  esac
  safe_class="$(printf '%s' "$evidence_class" | tr -c 'A-Za-z0-9_' '_')"
  fixture="$test_root/class-$domain-$safe_class"
  make_fixture "$fixture" "$core_class" "$analytics_class" "$notifications_class" "$payments_class"
  "$classifier" --input-dir "$fixture" > "$fixture.report.json" 2> "$fixture.stderr"
  jq -e --arg domain "$domain" --arg evidenceClass "$evidence_class" '
    .productionReady == false and
    any(.classifications[]; .domain == $domain and .evidenceClass == $evidenceClass)
  ' "$fixture.report.json" >/dev/null
done < <(jq -r '.classes[] | [.domain,.evidenceClass] | @tsv' "$contract")
pass "all 13 and only the A3.1 recognized evidence classes have synthetic positive fixtures"

cp -R "$test_root/base" "$test_root/tampered"
printf '\n' >> "$test_root/tampered/core.inspection.json"
expect_status 2 "$classifier" --input-dir "$test_root/tampered" > "$test_root/tampered-report.json" 2> "$test_root/tampered-stderr"
jq -e '.productionReady == false and any(.stopReasons[]; .code == "artifact_digest_mismatch")' "$test_root/tampered-report.json" >/dev/null
pass "artifact bytes changed after manifest capture STOP on SHA-256 mismatch"

cp -R "$test_root/base" "$test_root/unknown-history"
jq -S '.migrationHistory += [{version:"99999999999999",run_on:"2026-01-02 00:00:00"}]' \
  "$test_root/unknown-history/core.inspection.json" > "$test_root/unknown-history/.next"
mv "$test_root/unknown-history/.next" "$test_root/unknown-history/core.inspection.json"
rebuild_manifest "$test_root/unknown-history"
expect_status 2 "$classifier" --input-dir "$test_root/unknown-history" > "$test_root/unknown-history-report.json" 2> "$test_root/unknown-history-stderr"
jq -e 'any(.stopReasons[]; .domain == "core" and .code == "unknown_migration_history")' "$test_root/unknown-history-report.json" >/dev/null
pass "unknown Diesel version STOPs after a valid manifest recapture"

cp -R "$test_root/base" "$test_root/unknown-relation"
jq -S '.relations += [{schema_name:"public",relation_name:"untracked_operator_table",relation_kind:"r",estimated_rows:1}]' \
  "$test_root/unknown-relation/payments.inspection.json" > "$test_root/unknown-relation/.next"
mv "$test_root/unknown-relation/.next" "$test_root/unknown-relation/payments.inspection.json"
rebuild_manifest "$test_root/unknown-relation"
expect_status 2 "$classifier" --input-dir "$test_root/unknown-relation" > "$test_root/unknown-relation-report.json" 2> "$test_root/unknown-relation-stderr"
jq -e 'any(.stopReasons[]; .domain == "payments" and .code == "unknown_relation")' "$test_root/unknown-relation-report.json" >/dev/null
pass "untracked application relation STOPs instead of being folded into a known class"

cp -R "$test_root/base" "$test_root/incomplete"
jq -S 'del(.relations[1])' "$test_root/incomplete/notifications.inspection.json" > "$test_root/incomplete/.next"
mv "$test_root/incomplete/.next" "$test_root/incomplete/notifications.inspection.json"
rebuild_manifest "$test_root/incomplete"
expect_status 2 "$classifier" --input-dir "$test_root/incomplete" > "$test_root/incomplete-report.json" 2> "$test_root/incomplete-stderr"
jq -e 'any(.stopReasons[]; .domain == "notifications" and .code == "unrecognized_or_incomplete_evidence")' "$test_root/incomplete-report.json" >/dev/null
pass "missing required schema landmark STOPs as incomplete evidence"

cp -R "$test_root/base" "$test_root/hybrid"
jq -S '.relations += [{schema_name:"infra_logs",relation_name:"api_key_usage_logs",relation_kind:"r",estimated_rows:0}]' \
  "$test_root/hybrid/analytics.inspection.json" > "$test_root/hybrid/.next"
mv "$test_root/hybrid/.next" "$test_root/hybrid/analytics.inspection.json"
rebuild_manifest "$test_root/hybrid"
expect_status 2 "$classifier" --input-dir "$test_root/hybrid" > "$test_root/hybrid-report.json" 2> "$test_root/hybrid-stderr"
jq -e 'any(.stopReasons[]; .domain == "analytics" and .code == "unrecognized_or_incomplete_evidence")' "$test_root/hybrid-report.json" >/dev/null
pass "public plus infra_logs hybrid analytics evidence STOPs"

cp -R "$test_root/base" "$test_root/credential"
jq -S '.identity.password = "forbidden-fixture-value"' "$test_root/credential/core.discovery.json" > "$test_root/credential/.next"
mv "$test_root/credential/.next" "$test_root/credential/core.discovery.json"
rebuild_manifest "$test_root/credential"
expect_status 2 "$classifier" --input-dir "$test_root/credential" > "$test_root/credential-report.json" 2> "$test_root/credential-stderr"
jq -e 'any(.stopReasons[]; .code == "credential_material_detected")' "$test_root/credential-report.json" >/dev/null
! grep -q 'forbidden-fixture-value' "$test_root/credential-report.json"
pass "credential-like evidence STOPs and is absent from the redacted report"

cp -R "$test_root/base" "$test_root/manifest-traversal"
jq -S '.artifacts[0].file = "../core.discovery.json"' "$test_root/manifest-traversal/manifest.json" > "$test_root/manifest-traversal/.next"
mv "$test_root/manifest-traversal/.next" "$test_root/manifest-traversal/manifest.json"
expect_status 2 "$classifier" --input-dir "$test_root/manifest-traversal" > "$test_root/manifest-traversal-report.json" 2> "$test_root/manifest-traversal-stderr"
jq -e 'any(.stopReasons[]; .code == "manifest_invalid")' "$test_root/manifest-traversal-report.json" >/dev/null
pass "manifest path traversal cannot alter the exact artifact inventory"

cp -R "$test_root/base" "$test_root/symlink"
cp "$test_root/symlink/core.discovery.json" "$test_root/symlink-target.json"
rm "$test_root/symlink/core.discovery.json"
ln -s "$test_root/symlink-target.json" "$test_root/symlink/core.discovery.json"
expect_status 64 "$classifier" --input-dir "$test_root/symlink" >/dev/null 2>&1
pass "artifact symlinks are refused as unsafe invocation"

traversal_input="$test_root/base/../base"
expect_status 64 "$classifier" --input-dir "$traversal_input" >/dev/null 2>&1
pass "noncanonical input traversal is refused before evidence access"

cp -R "$test_root/base" "$test_root/extra-file"
printf '{}\n' > "$test_root/extra-file/operator-note.json"
expect_status 2 "$classifier" --input-dir "$test_root/extra-file" > "$test_root/extra-file-report.json" 2> "$test_root/extra-file-stderr"
jq -e 'any(.stopReasons[]; .code == "invalid_artifact_set")' "$test_root/extra-file-report.json" >/dev/null
pass "extra artifacts STOP the exact domain and filename gate"

output_file="$test_root/classified-output.json"
"$classifier" --input-dir "$test_root/base" --output "$output_file" >/dev/null 2> "$test_root/output-stderr"
jq -e '.productionReady == false and .status == "classified-offline-evidence-only"' "$output_file" >/dev/null
expect_status 64 "$classifier" --input-dir "$test_root/base" --output "$output_file" >/dev/null 2>&1

real_shasum="$(command -v shasum)"
mkdir "$test_root/race-bin"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  'if [ -n "${A3_2_RACE_SIGNAL:-}" ] && [ ! -e "$A3_2_RACE_SIGNAL" ]; then' \
  '  : > "$A3_2_RACE_SIGNAL"' \
  '  while [ ! -e "$A3_2_RACE_RELEASE" ]; do sleep 0.01; done' \
  'fi' \
  "exec $(printf '%q' "$real_shasum") \"\$@\"" \
  > "$test_root/race-bin/shasum"
chmod 0755 "$test_root/race-bin/shasum"

race_output="$test_root/race-output.json"
race_signal="$test_root/race-signal"
race_release="$test_root/race-release"
set +e
PATH="$test_root/race-bin:$PATH" \
A3_2_RACE_SIGNAL="$race_signal" \
A3_2_RACE_RELEASE="$race_release" \
  "$classifier" --input-dir "$test_root/base" --output "$race_output" \
  > "$test_root/race-stdout" 2> "$test_root/race-stderr" &
race_pid="$!"
set -e
for _attempt in $(seq 1 200); do
  [ -e "$race_signal" ] && break
  sleep 0.01
done
if [ ! -e "$race_signal" ]; then
  kill "$race_pid" >/dev/null 2>&1 || true
  wait "$race_pid" >/dev/null 2>&1 || true
  echo "race test did not reach the post-output-check SHA gate" >&2
  exit 1
fi
printf '%s\n' 'concurrent-owner-marker' > "$race_output"
: > "$race_release"
set +e
wait "$race_pid"
race_status="$?"
set -e
[ "$race_status" -eq 64 ] || { echo "output race must exit 64, observed $race_status" >&2; exit 1; }
[ "$(cat "$race_output")" = "concurrent-owner-marker" ]
pass "explicit output is mode-private, non-production, atomic-exclusive, and never overwrites existing or raced files"

printf '1..%d\n' "$pass_count"
