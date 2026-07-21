#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-a3-8-identity-lifecycle-schema.sh"
contract="$repo_root/docs/migration/contracts/a3-8-identity-lifecycle-schema.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a3-8-identity-schema.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "6,417-byte additive migration pinned; 4 tables, 6 indexes, client/lineage/u64 constraints" "$temp_dir/integrity.out"
grep -q "routes remain disabled; no runner, catalog adoption, backfill, upgrade, concurrency, JWKS, runtime, or database proof ran" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
[ "$readiness_status" -eq 3 ] || {
  cat "$temp_dir/readiness.out" >&2
  echo "a3-8 identity lifecycle schema self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
}
grep -q "ten residual A3.8 blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.productionReady !== false || report.readinessExit !== 3) process.exit(1);
if (report.source.evidenceItems !== 9 || report.source.schemaHistoryPaths !== 22) process.exit(1);
if (report.migrationRoot.pinnedBytes !== 6417 || report.migrationRoot.statements !== 10) process.exit(1);
if (report.migrationRoot.guardedTables !== 4 || report.migrationRoot.guardedIndexes !== 6 || report.migrationRoot.runner !== null) process.exit(1);
if (report.lifecycle.routesEnabled !== false || report.lifecycle.runtimeImplemented !== false || report.lifecycle.concurrencyProven !== false) process.exit(1);
if (report.lifecycle.crossClientReplayProven !== false || report.lifecycle.exactGenerationIncrementProven !== false) process.exit(1);
if (report.lifecycle.revokeVsRotateProven !== false) process.exit(1);
if (report.lifecycle.catalogCompatibilityProven !== false || report.lifecycle.digestAlgorithmClaimed !== false) process.exit(1);
if (report.lifecycle.challengeStorage !== "32-byte-digests-client-bound-single-consume-shape") process.exit(1);
if (report.blockers.length !== 10) process.exit(1);
' "$temp_dir/report-one.json"

mutate_contract() {
  mutation=$1
  output=$2
  A3_CONTRACT_IN="$contract" A3_CONTRACT_OUT="$output" A3_MUTATION="$mutation" bun -e '
const contract = await Bun.file(process.env.A3_CONTRACT_IN).json();
const replaceAnchor = (expected, replacement) => {
  const index = contract.schemaDesign.requiredAnchors.indexOf(expected);
  if (index < 0) process.exit(3);
  contract.schemaDesign.requiredAnchors[index] = replacement;
};
switch (process.env.A3_MUTATION) {
  case "ready": contract.productionReady = true; break;
  case "hash": contract.migrationRoot.orderedMigrations[0].sha256 = "0".repeat(64); break;
  case "anchor": contract.schemaDesign.requiredAnchors[0] = "missing identity table anchor"; break;
  case "client": contract.schemaDesign.requiredAnchors[8] = "missing challenge client constraint"; break;
  case "cross_client": contract.lifecycleDesign.crossClientReplay = "Cross-client replay is authorized and proven."; break;
  case "chain": contract.schemaDesign.requiredAnchors[11] = "chain_id has no upper bound"; break;
  case "family": contract.schemaDesign.requiredAnchors[34] = "missing refresh family binding"; break;
  case "lineage": contract.schemaDesign.requiredAnchors[39] = "missing composite parent lineage"; break;
  case "self_parent": contract.schemaDesign.requiredAnchors[43] = "self-parent is authorized"; break;
  case "successor_unique": replaceAnchor("CONSTRAINT identity_refresh_sessions_parent_unique UNIQUE (parent_session_id)", "single successor uniqueness removed"); break;
  case "one_root": replaceAnchor("CREATE UNIQUE INDEX IF NOT EXISTS identity_refresh_sessions_one_root_per_family_idx", "one root uniqueness removed"); break;
  case "revoke_rotate": contract.lifecycleDesign.refreshRevocationRace = "Revoke-versus-rotate concurrency is fully proven by this static schema."; break;
  case "history": contract.sourceEvidence.schemaHistoryPaths.pop(); break;
  case "routes": contract.scope.routesEnabled = true; break;
  case "authority": contract.authority.failClosedRuntime.sha256 = "0".repeat(64); break;
  case "blocker": contract.blockers[0].status = "aligned"; break;
  case "catalog": contract.nonClaims = contract.nonClaims.filter((item) => !item.startsWith("IF NOT EXISTS provides name idempotence only:")); break;
  case "algorithm": contract.lifecycleDesign.refreshStorage = "The schema proves SHA-256 is the correct and fully verified digest implementation."; break;
  default: process.exit(2);
}
await Bun.write(process.env.A3_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
}

expect_integrity_failure() {
  fixture=$1
  expected=$2
  label=$3
  set +e
  "$verify" --mode integrity --contract "$fixture" >"$temp_dir/$label.out" 2>&1
  fixture_status=$?
  set -e
  [ "$fixture_status" -eq 1 ] || {
    cat "$temp_dir/$label.out" >&2
    echo "a3-8 identity lifecycle schema self-test: expected $label tamper exit 1" >&2
    exit 1
  }
  grep -Eq "$expected" "$temp_dir/$label.out"
}

mutate_contract ready "$temp_dir/ready.json"
expect_integrity_failure "$temp_dir/ready.json" "readiness sentinel changed" ready

mutate_contract hash "$temp_dir/hash.json"
expect_integrity_failure "$temp_dir/hash.json" "ordered migration pin drifted|identity lifecycle migration bytes changed" hash

mutate_contract anchor "$temp_dir/anchor.json"
expect_integrity_failure "$temp_dir/anchor.json" "required SQL anchors drifted|missing required anchor" anchor

mutate_contract client "$temp_dir/client.json"
expect_integrity_failure "$temp_dir/client.json" "required SQL anchors drifted|SIWE challenge client binding is missing" client

mutate_contract cross_client "$temp_dir/cross-client.json"
expect_integrity_failure "$temp_dir/cross-client.json" "cross-client replay requirement drifted|lifecycle design is not substantive" cross-client

mutate_contract chain "$temp_dir/chain.json"
expect_integrity_failure "$temp_dir/chain.json" "required SQL anchors drifted|exact u64 upper bound" chain

mutate_contract family "$temp_dir/family.json"
expect_integrity_failure "$temp_dir/family.json" "required SQL anchors drifted|family ownership binding" family

mutate_contract lineage "$temp_dir/lineage.json"
expect_integrity_failure "$temp_dir/lineage.json" "required SQL anchors drifted|parent lineage binding" lineage

mutate_contract self_parent "$temp_dir/self-parent.json"
expect_integrity_failure "$temp_dir/self-parent.json" "required SQL anchors drifted|self-parent denial" self-parent

mutate_contract successor_unique "$temp_dir/successor-unique.json"
expect_integrity_failure "$temp_dir/successor-unique.json" "required SQL anchors drifted|parent_unique|successor" successor-unique

mutate_contract one_root "$temp_dir/one-root.json"
expect_integrity_failure "$temp_dir/one-root.json" "required SQL anchors drifted|at most one root" one-root

mutate_contract revoke_rotate "$temp_dir/revoke-rotate.json"
expect_integrity_failure "$temp_dir/revoke-rotate.json" "refreshRevocationRace lifecycle design is not substantive|revoke-vs-rotate requirement drifted" revoke-rotate

mutate_contract history "$temp_dir/history.json"
expect_integrity_failure "$temp_dir/history.json" "schema-history inventory boundary drifted|schema-history inventory drifted" history

mutate_contract routes "$temp_dir/routes.json"
expect_integrity_failure "$temp_dir/routes.json" "scope drifted" routes

mutate_contract authority "$temp_dir/authority.json"
expect_integrity_failure "$temp_dir/authority.json" "failClosedRuntime authority drifted|authority bytes changed" authority

mutate_contract blocker "$temp_dir/blocker.json"
expect_integrity_failure "$temp_dir/blocker.json" "B01: residual blocker drifted" blocker

mutate_contract catalog "$temp_dir/catalog.json"
expect_integrity_failure "$temp_dir/catalog.json" "seven substantive non-claims are required|name-idempotence/catalog non-claim is missing" catalog

mutate_contract algorithm "$temp_dir/algorithm.json"
expect_integrity_failure "$temp_dir/algorithm.json" "refreshStorage lifecycle design is not substantive|digest-algorithm non-claim drifted" algorithm

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_env_status=$?
IDENTITY_DATABASE_URL=postgres://example.invalid/epsx "$verify" --mode integrity >"$temp_dir/database-env.out" 2>&1
database_env_status=$?
REDIS_URL=redis://example.invalid "$verify" --mode integrity >"$temp_dir/redis-env.out" 2>&1
redis_env_status=$?
set -e
[ "$production_env_status" -eq 1 ] || {
  cat "$temp_dir/production-env.out" >&2
  echo "a3-8 identity lifecycle schema self-test: expected production-env exit 1" >&2
  exit 1
}
[ "$database_env_status" -eq 1 ] || {
  cat "$temp_dir/database-env.out" >&2
  echo "a3-8 identity lifecycle schema self-test: expected database-env exit 1" >&2
  exit 1
}
[ "$redis_env_status" -eq 1 ] || {
  cat "$temp_dir/redis-env.out" >&2
  echo "a3-8 identity lifecycle schema self-test: expected Redis-env exit 1" >&2
  exit 1
}
grep -q "production-looking environment" "$temp_dir/production-env.out"
grep -q "never contacts a database or Redis" "$temp_dir/database-env.out"
grep -q "never contacts a database or Redis" "$temp_dir/redis-env.out"

echo "a3-8 identity lifecycle schema self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, readiness/hash/anchor/client/cross-client/chain/family/lineage/self-parent/successor-unique/one-root/revoke-rotate/history/routes/authority/blocker/catalog/algorithm/prod/db/redis tamper=1)"
