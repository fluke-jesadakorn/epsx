#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: a3-2-history-classification.sh --input-dir CANONICAL_ABSOLUTE_DIRECTORY [--output CANONICAL_ABSOLUTE_NEW_FILE]

Consumes only the nine JSON files produced by a successful A3.1a read-only
database preflight. The command performs no database, network, migration,
repair, DDL, Docker, Kubernetes, or deployment operation.

Exit status: 0 = all four domains classified, 2 = evidence STOP, 64 = unsafe invocation.
Even exit 0 emits productionReady:false and does not authorize a migration.
EOF
}

fail_usage() {
  local message="$1"
  echo "a3-2-history-classification: ERROR: $message" >&2
  usage >&2
  exit 64
}

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"
repo_root="$(git -C "$script_dir" rev-parse --show-toplevel)"
repo_root="$(cd "$repo_root" && pwd -P)"
contract="$repo_root/docs/migration/contracts/a3-2-history-classification.json"
source_contract="$repo_root/docs/migration/contracts/a3-1-history-preflight.json"
input_dir=""
output_file=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --help)
      usage
      exit 0
      ;;
    --input-dir)
      [ "$#" -ge 2 ] || fail_usage "--input-dir requires a value"
      input_dir="$2"
      shift 2
      ;;
    --output)
      [ "$#" -ge 2 ] || fail_usage "--output requires a value"
      output_file="$2"
      shift 2
      ;;
    *)
      fail_usage "unknown option: $1"
      ;;
  esac
done

command -v jq >/dev/null 2>&1 || fail_usage "jq is required"
command -v shasum >/dev/null 2>&1 || fail_usage "shasum is required"
[ -n "$input_dir" ] || fail_usage "--input-dir is required"

case "$input_dir" in
  /*) ;;
  *) fail_usage "--input-dir must be absolute" ;;
esac
[ -d "$input_dir" ] || fail_usage "--input-dir must be an existing directory"
[ ! -L "$input_dir" ] || fail_usage "--input-dir may not be a symlink"
canonical_input="$(cd "$input_dir" && pwd -P)"
[ "$input_dir" = "$canonical_input" ] || fail_usage "--input-dir must be canonical and traversal-free"
case "$canonical_input" in
  "$repo_root"|"$repo_root/"*) fail_usage "--input-dir must be outside the repository" ;;
esac

if [ -n "$output_file" ]; then
  case "$output_file" in
    /*) ;;
    *) fail_usage "--output must be absolute" ;;
  esac
  [ "$output_file" != "/" ] || fail_usage "--output may not be filesystem root"
  [ ! -e "$output_file" ] && [ ! -L "$output_file" ] || fail_usage "--output refuses an existing path"
  output_parent="$(dirname -- "$output_file")"
  [ -d "$output_parent" ] || fail_usage "--output parent must exist"
  [ ! -L "$output_parent" ] || fail_usage "--output parent may not be a symlink"
  canonical_output_parent="$(cd "$output_parent" && pwd -P)"
  canonical_output="$canonical_output_parent/$(basename -- "$output_file")"
  [ "$output_file" = "$canonical_output" ] || fail_usage "--output must be canonical and traversal-free"
  case "$canonical_output" in
    "$repo_root"|"$repo_root/"*) fail_usage "--output must be outside the repository" ;;
  esac
fi

jq -e '
  .schemaVersion == 1 and
  .package == "A3.2" and
  .purpose == "offline-history-classification-only" and
  .productionReady == false and
  (.domains == ["core", "analytics", "notifications", "payments"]) and
  (.classes | type == "array" and length == 13) and
  ([.classes[] | {domain,evidenceClass}] | length == (unique | length)) and
  all(.classes[]; .followupHistory == "none" or .followupHistory == "some" or .followupHistory == "any")
' "$contract" >/dev/null || fail_usage "classification contract is invalid"

expected_source_sha="$(jq -r '.sourceContract.sha256' "$contract")"
actual_source_sha="$(shasum -a 256 "$source_contract" | awk '{print $1}')"
[ "$actual_source_sha" = "$expected_source_sha" ] || fail_usage "A3.1 source contract checksum does not match A3.2"

work_root="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-2-classification.XXXXXX")"
trap 'rm -rf -- "$work_root"' EXIT

jq -S '[.recoveryMatrix[] | .domain as $domain | .acceptedEvidenceClasses[] | {domain:$domain,evidenceClass:.}] | sort_by(.domain,.evidenceClass)' \
  "$source_contract" > "$work_root/source-classes.json"
jq -S '[.classes[] | {domain,evidenceClass}] | sort_by(.domain,.evidenceClass)' \
  "$contract" > "$work_root/classifier-classes.json"
cmp -s "$work_root/source-classes.json" "$work_root/classifier-classes.json" || \
  fail_usage "classifier classes do not exactly match A3.1 recognized evidence classes"

jq -e --slurpfile c "$contract" '
  $c[0] as $contract |
  all(.classes[];
    . as $rule |
    ($contract.domains | index($rule.domain)) != null and
    ($contract.schemaProfiles[$rule.profile] | type) == "object" and
    all($rule.requiredVersions[]; . as $v | $contract.knownMigrationVersions[$rule.domain] | index($v) != null) and
    all($rule.forbiddenVersions[]; . as $v | $contract.knownMigrationVersions[$rule.domain] | index($v) != null)
  )
' "$contract" >/dev/null 2>&1 || fail_usage "classifier rules reference an invalid domain, profile, or version"

stop_lines="$work_root/stops.jsonl"
classification_lines="$work_root/classifications.jsonl"
: > "$stop_lines"
: > "$classification_lines"

add_stop() {
  local domain="$1"
  local code="$2"
  local detail="$3"
  if [ -n "$domain" ]; then
    jq -cn --arg domain "$domain" --arg code "$code" --arg detail "$detail" \
      '{domain:$domain,code:$code,detail:$detail}' >> "$stop_lines"
  else
    jq -cn --arg code "$code" --arg detail "$detail" \
      '{code:$code,detail:$detail}' >> "$stop_lines"
  fi
}

emit_report() {
  local status="$1"
  local manifest_sha_value="$2"
  jq -s 'sort_by(.domain // "", .code)' "$stop_lines" > "$work_root/stops.json"
  jq -s 'sort_by(.domain)' "$classification_lines" > "$work_root/classifications.json"
  jq -S -n \
    --arg status "$status" \
    --arg manifestSha256 "$manifest_sha_value" \
    --slurpfile classifications "$work_root/classifications.json" \
    --slurpfile stops "$work_root/stops.json" \
    '{
      schemaVersion:1,
      package:"A3.2",
      purpose:"offline-history-classification-only",
      productionReady:false,
      status:$status,
      input:{manifestSha256:(if $manifestSha256 == "" then null else $manifestSha256 end)},
      classifications:$classifications[0],
      stopReasons:$stops[0]
  }' > "$work_root/report.json"
  if [ -n "$output_file" ]; then
    if ! (
      set -o noclobber
      umask 077
      cat "$work_root/report.json" > "$output_file"
    ) 2>/dev/null; then
      fail_usage "--output became occupied; refusing to overwrite it"
    fi
  else
    cat "$work_root/report.json"
  fi
}

expected_entries="$work_root/expected-entries.txt"
{
  printf '%s\n' manifest.json
  jq -r '.domains[] as $d | .input.artifactSuffixes[] | $d + "." + .' "$contract"
} | LC_ALL=C sort > "$expected_entries"

actual_entries="$work_root/actual-entries.txt"
: > "$actual_entries"
while IFS= read -r entry; do
  [ ! -L "$entry" ] || fail_usage "input artifacts may not be symlinks"
  basename -- "$entry" >> "$actual_entries"
  [ -f "$entry" ] || add_stop "" "invalid_artifact_set" "every input entry must be a regular file"
done < <(find "$canonical_input" -mindepth 1 -maxdepth 1 -print)
LC_ALL=C sort -o "$actual_entries" "$actual_entries"
if ! cmp -s "$expected_entries" "$actual_entries"; then
  add_stop "" "invalid_artifact_set" "input directory must contain exactly manifest.json and the eight domain artifacts"
fi

manifest="$canonical_input/manifest.json"
manifest_sha=""
if [ -f "$manifest" ] && [ ! -L "$manifest" ]; then
  manifest_sha="$(shasum -a 256 "$manifest" | awk '{print $1}')"
else
  add_stop "" "manifest_missing" "manifest.json is missing or not a regular file"
fi

expected_artifacts="$work_root/expected-artifacts.json"
jq -S '[.domains[] as $d | .input.artifactSuffixes[] | $d + "." + .] | sort' "$contract" > "$expected_artifacts"

if [ -n "$manifest_sha" ]; then
  if ! jq -e --slurpfile expected "$expected_artifacts" '
    .schemaVersion == 1 and
    .package == "A3.1a" and
    .purpose == "read-only-database-preflight" and
    .productionReady == false and
    .status == "inspection-captured-operator-classification-required" and
    .stopReasons == [] and
    (.artifacts | type == "array" and length == 8) and
    all(.artifacts[];
      (keys | sort) == ["file", "sha256"] and
      (.file | type == "string") and
      (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
    ) and
    ([.artifacts[].file] | sort) == $expected[0] and
    ([.artifacts[].file] | length) == ([.artifacts[].file] | unique | length)
  ' "$manifest" >/dev/null 2>&1; then
    add_stop "" "manifest_invalid" "manifest metadata or exact artifact inventory is invalid"
  fi
fi

while IFS= read -r expected_name; do
  json_file="$canonical_input/$expected_name"
  [ -f "$json_file" ] || continue
  if ! jq -e . "$json_file" >/dev/null 2>&1; then
    add_stop "" "invalid_json" "one or more evidence files are not valid JSON"
    continue
  fi
  if jq -e '
    paths(scalars) as $p |
    ($p[-1] | tostring) as $key |
    getpath($p) as $value |
    select(
      ($key | test("^(password|passwd|secret|credential|database_?url|dsn|private_?key|access_?token|refresh_?token)$"; "i")) or
      (($value | type) == "string" and ($value | test("postgres(?:ql)?://|-----BEGIN [A-Z ]*PRIVATE KEY-----"; "i")))
    )
  ' "$json_file" >/dev/null 2>&1; then
    add_stop "" "credential_material_detected" "credential-like material is forbidden in preflight artifacts"
  fi
done < "$expected_entries"

if [ -n "$manifest_sha" ] && jq -e '.artifacts | type == "array"' "$manifest" >/dev/null 2>&1; then
  while IFS=$'\t' read -r artifact_name expected_sha; do
    case "$artifact_name" in
      core.discovery.json|core.inspection.json|analytics.discovery.json|analytics.inspection.json|notifications.discovery.json|notifications.inspection.json|payments.discovery.json|payments.inspection.json) ;;
      *) continue ;;
    esac
    artifact_path="$canonical_input/$artifact_name"
    if [ ! -f "$artifact_path" ] || [ -L "$artifact_path" ]; then
      add_stop "" "artifact_missing" "a manifest artifact is missing or unsafe"
      continue
    fi
    actual_sha="$(shasum -a 256 "$artifact_path" | awk '{print $1}')"
    if [ "$actual_sha" != "$expected_sha" ]; then
      add_stop "" "artifact_digest_mismatch" "an artifact SHA-256 does not match the manifest"
    fi
  done < <(jq -r '.artifacts[]? | [.file,.sha256] | @tsv' "$manifest")
fi

if [ -s "$stop_lines" ]; then
  emit_report "stop" "$manifest_sha"
  echo "a3-2-history-classification: STOP — artifact integrity failed" >&2
  exit 2
fi

for domain in core analytics notifications payments; do
  discovery="$canonical_input/$domain.discovery.json"
  inspection="$canonical_input/$domain.inspection.json"

  if ! jq -e '
    type == "object" and
    (.identity | type == "object") and
    .identity.transactionReadOnly == "on" and
    (.migrationTables | type == "array" and length == 1) and
    .migrationTables[0].name == "__diesel_schema_migrations" and
    (.migrationTables[0].schema | type == "string" and test("^[A-Za-z_][A-Za-z0-9_$]*$"))
  ' "$discovery" >/dev/null 2>&1; then
    add_stop "$domain" "invalid_discovery_fingerprint" "read-only identity or the single migration-table fingerprint is invalid"
    continue
  fi

  migration_schema="$(jq -r '.migrationTables[0].schema' "$discovery")"
  if ! jq -e --arg migrationSchema "$migration_schema" '
    type == "object" and
    .migrationTable == {schema:$migrationSchema,name:"__diesel_schema_migrations"} and
    (.migrationHistory | type == "array") and
    (.relations | type == "array") and
    (.columns | type == "array") and
    (.constraints | type == "array") and
    (.indexes | type == "array") and
    (.triggers | type == "array") and
    (.functions | type == "array") and
    (.partitions | type == "array") and
    (.views | type == "array") and
    all(.migrationHistory[]; (keys | sort) == ["run_on","version"] and (.version | type == "string" and test("^[0-9]+$")) and (.run_on | type == "string")) and
    all(.relations[];
      (.schema_name | type == "string" and test("^[A-Za-z_][A-Za-z0-9_$]*$")) and
      (.relation_name | type == "string" and test("^[A-Za-z_][A-Za-z0-9_$]*$")) and
      (.relation_kind | type == "string")
    ) and
    all(.columns[];
      (.schema_name | type == "string" and test("^[A-Za-z_][A-Za-z0-9_$]*$")) and
      (.relation_name | type == "string" and test("^[A-Za-z_][A-Za-z0-9_$]*$")) and
      (.column_name | type == "string" and test("^[A-Za-z_][A-Za-z0-9_$]*$"))
    ) and
    ([.relations[] | select(.relation_name == "__diesel_schema_migrations")] | length) == 1 and
    any(.relations[]; .schema_name == $migrationSchema and .relation_name == "__diesel_schema_migrations" and .relation_kind == "r")
  ' "$inspection" >/dev/null 2>&1; then
    add_stop "$domain" "invalid_inspection_fingerprint" "catalog evidence or migration-table identity is structurally incomplete"
    continue
  fi

  evidence="$work_root/$domain.evidence.json"
  jq -S '{
    versions:[.migrationHistory[].version],
    relations:([.relations[] | select(.relation_name != "__diesel_schema_migrations") | .schema_name + "." + .relation_name] | sort),
    columns:([.columns[] | select(.relation_name != "__diesel_schema_migrations") | .schema_name + "." + .relation_name + "." + .column_name] | sort)
  }' "$inspection" > "$evidence"

  if ! jq -e '.versions == (.versions | sort) and (.versions | length) == (.versions | unique | length)' "$evidence" >/dev/null; then
    add_stop "$domain" "noncanonical_migration_history" "migration versions must be ordered and unique"
    continue
  fi
  if ! jq -e '(.relations | length) == (.relations | unique | length) and (.columns | length) == (.columns | unique | length)' "$evidence" >/dev/null; then
    add_stop "$domain" "duplicate_schema_fingerprint" "relation and column fingerprints must be unique"
    continue
  fi
  if ! jq -e --slurpfile e "$evidence" '
    $e[0] as $evidence |
    all($evidence.columns[]; split(".") as $p | ($evidence.relations | index($p[0] + "." + $p[1])) != null)
  ' "$evidence" >/dev/null; then
    add_stop "$domain" "orphan_column_fingerprint" "every captured column must reference a captured relation"
    continue
  fi
  if ! jq -e --arg domain "$domain" --slurpfile e "$evidence" '
    $e[0] as $evidence |
    (.knownMigrationVersions[$domain]) as $known |
    all($evidence.versions[]; . as $v | $known | index($v) != null) and
    all(.baseMigrationVersions[]; . as $v | $evidence.versions | index($v) != null)
  ' "$contract" >/dev/null; then
    add_stop "$domain" "unknown_migration_history" "history contains an unknown version or omits a required base version"
    continue
  fi
  if ! jq -e --arg domain "$domain" --slurpfile e "$evidence" '
    $e[0] as $evidence |
    (.allowedRelations[$domain]) as $allowed |
    all($evidence.relations[]; . as $r | $allowed | index($r) != null)
  ' "$contract" >/dev/null; then
    add_stop "$domain" "unknown_relation" "schema contains a relation outside the domain allowlist"
    continue
  fi

  matches="$work_root/$domain.matches.json"
  jq -S --arg domain "$domain" --slurpfile e "$evidence" '
    def has_all($required; $actual): all($required[]; . as $x | $actual | index($x) != null);
    def has_none($forbidden; $actual): all($forbidden[]; . as $x | $actual | index($x) == null);
    . as $contract |
    $e[0] as $evidence |
    ($evidence.versions - $contract.baseMigrationVersions | length) as $followupCount |
    [
      $contract.classes[] |
      select(.domain == $domain) |
      . as $rule |
      $contract.schemaProfiles[$rule.profile] as $profile |
      select(
        ($rule.followupHistory == "any" or
         ($rule.followupHistory == "none" and $followupCount == 0) or
         ($rule.followupHistory == "some" and $followupCount > 0)) and
        has_all($rule.requiredVersions; $evidence.versions) and
        has_none($rule.forbiddenVersions; $evidence.versions) and
        has_all($profile.requiredRelations; $evidence.relations) and
        has_none($profile.forbiddenRelations; $evidence.relations) and
        has_all($profile.requiredColumns; $evidence.columns) and
        has_none($profile.forbiddenColumns; $evidence.columns)
      ) |
      .evidenceClass
    ] | sort
  ' "$contract" > "$matches"

  match_count="$(jq 'length' "$matches")"
  if [ "$match_count" -eq 0 ]; then
    add_stop "$domain" "unrecognized_or_incomplete_evidence" "evidence matches no recognized A3.1 recovery class"
    continue
  fi
  if [ "$match_count" -ne 1 ]; then
    add_stop "$domain" "hybrid_evidence" "evidence matches more than one recovery class"
    continue
  fi

  evidence_sha="$(shasum -a 256 "$evidence" | awk '{print $1}')"
  evidence_class="$(jq -r '.[0]' "$matches")"
  jq -cn --arg domain "$domain" --arg evidenceClass "$evidence_class" --arg evidenceSha256 "$evidence_sha" \
    '{domain:$domain,evidenceClass:$evidenceClass,evidenceSha256:$evidenceSha256}' >> "$classification_lines"
done

if [ -s "$stop_lines" ]; then
  : > "$classification_lines"
  emit_report "stop" "$manifest_sha"
  echo "a3-2-history-classification: STOP — evidence is unknown, hybrid, incomplete, or unsafe" >&2
  exit 2
fi

if [ "$(wc -l < "$classification_lines" | tr -d ' ')" -ne 4 ]; then
  add_stop "" "incomplete_domain_set" "all four domains must classify exactly once"
  : > "$classification_lines"
  emit_report "stop" "$manifest_sha"
  exit 2
fi

emit_report "classified-offline-evidence-only" "$manifest_sha"
echo "a3-2-history-classification: four domains classified; NOT production-ready" >&2
