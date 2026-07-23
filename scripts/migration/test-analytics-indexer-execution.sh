#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-analytics-indexer-execution.sh"
contract="$repo_root/docs/migration/contracts/analytics-indexer-execution.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-analytics-indexer-execution.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "14 source pins, 40 target anchors, A2.4/A2.5/A2.6/A2.7/A2.8/A2.9/A2.10 boundary contracts, 4 separate domains, 16 surfaces, and 24 stop blockers" "$temp_dir/integrity.out"
grep -q "no database, Redis, chain, network, live market-data, deployment" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "analytics-indexer-execution self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "24 stop blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
EPSX_A2_4_EVIDENCE_ROOT="$temp_dir" EPSX_A2_4_STATIC_ONLY=1 \
EPSX_A2_5_EVIDENCE_ROOT="$temp_dir" EPSX_A2_5_STATIC_ONLY=1 \
EPSX_A2_6_EVIDENCE_ROOT="$temp_dir" EPSX_A2_6_STATIC_ONLY=1 \
EPSX_A2_7_EVIDENCE_ROOT="$temp_dir" EPSX_A2_7_STATIC_ONLY=1 \
EPSX_A2_8_EVIDENCE_ROOT="$temp_dir" EPSX_A2_8_STATIC_ONLY=1 \
EPSX_A2_9_EVIDENCE_ROOT="$temp_dir" EPSX_A2_9_STATIC_ONLY=1 \
EPSX_A2_10_EVIDENCE_ROOT="$temp_dir" EPSX_A2_10_STATIC_ONLY=1 \
  "$verify" --mode report >"$temp_dir/override-proof.json"
cmp "$temp_dir/report-one.json" "$temp_dir/override-proof.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
const expected = ["marketAnalytics", "eventAnalytics", "indexer", "identityRankingOffset"];
if (report.readinessExit !== 3 || report.productionReady !== false || report.blockers.length !== 24 || report.targetEvidence !== 40 || report.refreshedBoundaryEvidence !== 7 || report.surfaceContracts.length !== 16) process.exit(1);
if (JSON.stringify(report.composedBoundaryEvidence) !== JSON.stringify(["A2.4", "A2.5", "A2.6", "A2.7", "A2.8", "A2.9", "A2.10"])) process.exit(1);
if (expected.some((domain) => !report.domains[domain] || report.domains[domain].status !== "blocked")) process.exit(1);
' "$temp_dir/report-one.json"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/missing-anchor.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
contract.source.evidence[0].anchor = "tampered missing source anchor";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/missing-anchor.json" >"$temp_dir/missing-anchor.out" 2>&1
anchor_status=$?
set -e
if [ "$anchor_status" -ne 1 ]; then
  cat "$temp_dir/missing-anchor.out" >&2
  echo "analytics-indexer-execution self-test: expected missing-anchor exit 1, got $anchor_status" >&2
  exit 1
fi
grep -q "missing source anchor" "$temp_dir/missing-anchor.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/schema-boundary-anchor.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
contract.targetEvidence.find((item) => item.id === "tgt-indexer-schema-boundary").anchor = "\"productionReady\": false";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/schema-boundary-anchor.json" >"$temp_dir/schema-boundary-anchor.out" 2>&1
schema_boundary_status=$?
set -e
if [ "$schema_boundary_status" -ne 1 ]; then
  cat "$temp_dir/schema-boundary-anchor.out" >&2
  echo "analytics-indexer-execution self-test: expected schema-boundary-anchor exit 1, got $schema_boundary_status" >&2
  exit 1
fi
grep -q "refreshed schema/fake-sync evidence drifted" "$temp_dir/schema-boundary-anchor.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/stale-source.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
contract.source.commit = "0000000000000000000000000000000000000000";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-source.json" >"$temp_dir/stale-source.out" 2>&1
stale_status=$?
set -e
if [ "$stale_status" -ne 1 ]; then
  cat "$temp_dir/stale-source.out" >&2
  echo "analytics-indexer-execution self-test: expected stale-source exit 1, got $stale_status" >&2
  exit 1
fi
grep -q "invalid pinned source ref/commit\|stale source ref/commit" "$temp_dir/stale-source.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
contract.targetEvidence[0].file = "../outside";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
if [ "$traversal_status" -ne 1 ]; then
  cat "$temp_dir/traversal.out" >&2
  echo "analytics-indexer-execution self-test: expected traversal exit 1, got $traversal_status" >&2
  exit 1
fi
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/conflated-domain.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
contract.domains.eventAnalytics.owner = contract.domains.marketAnalytics.owner;
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/conflated-domain.json" >"$temp_dir/conflated-domain.out" 2>&1
conflated_status=$?
set -e
if [ "$conflated_status" -ne 1 ]; then
  cat "$temp_dir/conflated-domain.out" >&2
  echo "analytics-indexer-execution self-test: expected conflated-domain exit 1, got $conflated_status" >&2
  exit 1
fi
grep -q "owner/authority boundary drifted\|domain owners must not be conflated" "$temp_dir/conflated-domain.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/blocker-inventory.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
contract.blockers[0].id = "B99";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/blocker-inventory.json" >"$temp_dir/blocker-inventory.out" 2>&1
blocker_inventory_status=$?
set -e
if [ "$blocker_inventory_status" -ne 1 ]; then
  cat "$temp_dir/blocker-inventory.out" >&2
  echo "analytics-indexer-execution self-test: expected blocker-inventory exit 1, got $blocker_inventory_status" >&2
  exit 1
fi
grep -q "exact B01..B24 blocker inventory drifted" "$temp_dir/blocker-inventory.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-5-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B04");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-market-a2-5-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-5-link.json" >"$temp_dir/a2-5-link.out" 2>&1
a2_5_link_status=$?
set -e
if [ "$a2_5_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-5-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-5-link exit 1, got $a2_5_link_status" >&2
  exit 1
fi
grep -q "B04 must retain the canonical A2.5 provider-boundary evidence link" "$temp_dir/a2-5-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-6-drifted-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const evidence = contract.targetEvidence.find((item) => item.id === "tgt-market-a2-6-authority-failure-contract");
evidence.anchor = "\"productionReady\": false";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-6-drifted-link.json" >"$temp_dir/a2-6-drifted-link.out" 2>&1
a2_6_drifted_link_status=$?
set -e
if [ "$a2_6_drifted_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-6-drifted-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-6-drifted-link exit 1, got $a2_6_drifted_link_status" >&2
  exit 1
fi
grep -q "A2.6 authority-failure boundary evidence is missing or drifted" "$temp_dir/a2-6-drifted-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-6-missing-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B03");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-market-a2-6-authority-failure-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-6-missing-link.json" >"$temp_dir/a2-6-missing-link.out" 2>&1
a2_6_missing_link_status=$?
set -e
if [ "$a2_6_missing_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-6-missing-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-6-missing-link exit 1, got $a2_6_missing_link_status" >&2
  exit 1
fi
grep -q "B03 must retain A2.6 fail-closed evidence plus A2.10 unwired workload-auth" "$temp_dir/a2-6-missing-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-7-drifted-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const evidence = contract.targetEvidence.find((item) => item.id === "tgt-identity-a2-7-entitlement-snapshot-contract");
evidence.anchor = "\"productionReady\": false";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-7-drifted-link.json" >"$temp_dir/a2-7-drifted-link.out" 2>&1
a2_7_drifted_link_status=$?
set -e
if [ "$a2_7_drifted_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-7-drifted-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-7-drifted-link exit 1, got $a2_7_drifted_link_status" >&2
  exit 1
fi
grep -q "A2.7 entitlement-snapshot evidence is missing or drifted" "$temp_dir/a2-7-drifted-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-7-missing-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B07");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-identity-a2-7-entitlement-snapshot-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-7-missing-link.json" >"$temp_dir/a2-7-missing-link.out" 2>&1
a2_7_missing_link_status=$?
set -e
if [ "$a2_7_missing_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-7-missing-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-7-missing-link exit 1, got $a2_7_missing_link_status" >&2
  exit 1
fi
grep -q "B07 must retain A2.6/A2.7/A2.8/A2.9/A2.10 evidence" "$temp_dir/a2-7-missing-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-8-drifted-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const evidence = contract.targetEvidence.find((item) => item.id === "tgt-identity-a2-8-core-snapshot-adapter-contract");
evidence.anchor = "\"productionReady\": false";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-8-drifted-link.json" >"$temp_dir/a2-8-drifted-link.out" 2>&1
a2_8_drifted_link_status=$?
set -e
if [ "$a2_8_drifted_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-8-drifted-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-8-drifted-link exit 1, got $a2_8_drifted_link_status" >&2
  exit 1
fi
grep -q "A2.8 core-snapshot-adapter evidence is missing or drifted" "$temp_dir/a2-8-drifted-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-8-missing-b07-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B07");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-identity-a2-8-core-snapshot-adapter-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-8-missing-b07-link.json" >"$temp_dir/a2-8-missing-b07-link.out" 2>&1
a2_8_missing_b07_status=$?
set -e
if [ "$a2_8_missing_b07_status" -ne 1 ]; then
  cat "$temp_dir/a2-8-missing-b07-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-8-missing-b07-link exit 1, got $a2_8_missing_b07_status" >&2
  exit 1
fi
grep -q "B07 must retain A2.6/A2.7/A2.8/A2.9/A2.10 evidence" "$temp_dir/a2-8-missing-b07-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-8-missing-b21-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B21");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-identity-a2-8-core-snapshot-adapter-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-8-missing-b21-link.json" >"$temp_dir/a2-8-missing-b21-link.out" 2>&1
a2_8_missing_b21_status=$?
set -e
if [ "$a2_8_missing_b21_status" -ne 1 ]; then
  cat "$temp_dir/a2-8-missing-b21-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-8-missing-b21-link exit 1, got $a2_8_missing_b21_status" >&2
  exit 1
fi
grep -q "B21 must keep A2.7 fixtures and A2.8 static SQL distinct from reconciliation" "$temp_dir/a2-8-missing-b21-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-9-drifted-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const evidence = contract.targetEvidence.find((item) => item.id === "tgt-identity-a2-9-event-containment-contract");
evidence.anchor = "\"productionReady\": false";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-9-drifted-link.json" >"$temp_dir/a2-9-drifted-link.out" 2>&1
a2_9_drifted_link_status=$?
set -e
if [ "$a2_9_drifted_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-9-drifted-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-9-drifted-link exit 1, got $a2_9_drifted_link_status" >&2
  exit 1
fi
grep -q "A2.9 identity-event-containment evidence is missing or drifted" "$temp_dir/a2-9-drifted-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-9-missing-b08-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B08");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-identity-a2-9-event-containment-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-9-missing-b08-link.json" >"$temp_dir/a2-9-missing-b08-link.out" 2>&1
a2_9_missing_b08_status=$?
set -e
if [ "$a2_9_missing_b08_status" -ne 1 ]; then
  cat "$temp_dir/a2-9-missing-b08-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-9-missing-b08-link exit 1, got $a2_9_missing_b08_status" >&2
  exit 1
fi
grep -q "B08 must retain A2.9 containment" "$temp_dir/a2-9-missing-b08-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-10-drifted-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const evidence = contract.targetEvidence.find((item) => item.id === "tgt-identity-a2-10-authenticated-ranking-rpc-contract");
evidence.anchor = "\"productionReady\": false";
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-10-drifted-link.json" >"$temp_dir/a2-10-drifted-link.out" 2>&1
a2_10_drifted_link_status=$?
set -e
if [ "$a2_10_drifted_link_status" -ne 1 ]; then
  cat "$temp_dir/a2-10-drifted-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-10-drifted-link exit 1, got $a2_10_drifted_link_status" >&2
  exit 1
fi
grep -q "A2.10 authenticated-ranking-RPC evidence is missing or drifted" "$temp_dir/a2-10-drifted-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-10-missing-b03-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B03");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-identity-a2-10-authenticated-ranking-rpc-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-10-missing-b03-link.json" >"$temp_dir/a2-10-missing-b03-link.out" 2>&1
a2_10_missing_b03_status=$?
set -e
if [ "$a2_10_missing_b03_status" -ne 1 ]; then
  cat "$temp_dir/a2-10-missing-b03-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-10-missing-b03-link exit 1, got $a2_10_missing_b03_status" >&2
  exit 1
fi
grep -q "B03 must retain A2.6 fail-closed evidence plus A2.10 unwired workload-auth" "$temp_dir/a2-10-missing-b03-link.out"

A12_CONTRACT_IN="$contract" A12_CONTRACT_OUT="$temp_dir/a2-10-missing-b07-link.json" bun -e '
const contract = await Bun.file(process.env.A12_CONTRACT_IN).json();
const blocker = contract.blockers.find((item) => item.id === "B07");
blocker.evidenceIds = blocker.evidenceIds.filter((id) => id !== "tgt-identity-a2-10-authenticated-ranking-rpc-contract");
await Bun.write(process.env.A12_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/a2-10-missing-b07-link.json" >"$temp_dir/a2-10-missing-b07-link.out" 2>&1
a2_10_missing_b07_status=$?
set -e
if [ "$a2_10_missing_b07_status" -ne 1 ]; then
  cat "$temp_dir/a2-10-missing-b07-link.out" >&2
  echo "analytics-indexer-execution self-test: expected a2-10-missing-b07-link exit 1, got $a2_10_missing_b07_status" >&2
  exit 1
fi
grep -q "B07 must retain A2.6/A2.7/A2.8/A2.9/A2.10 evidence" "$temp_dir/a2-10-missing-b07-link.out"

set +e
EPSX_ENV=production "$verify" --mode integrity >"$temp_dir/production-env.out" 2>&1
production_status=$?
set -e
if [ "$production_status" -ne 1 ]; then
  cat "$temp_dir/production-env.out" >&2
  echo "analytics-indexer-execution self-test: expected production-env exit 1, got $production_status" >&2
  exit 1
fi
grep -q "production-looking environment" "$temp_dir/production-env.out"

set +e
ENV=production "$verify" --mode integrity >"$temp_dir/generic-production-env.out" 2>&1
generic_production_status=$?
set -e
if [ "$generic_production_status" -ne 1 ]; then
  cat "$temp_dir/generic-production-env.out" >&2
  echo "analytics-indexer-execution self-test: expected generic-production-env exit 1, got $generic_production_status" >&2
  exit 1
fi
grep -q "production-looking environment" "$temp_dir/generic-production-env.out"

set +e
TEST_DATABASE_URL=postgres://example.invalid/test "$verify" --mode integrity >"$temp_dir/test-database-env.out" 2>&1
test_database_status=$?
set -e
if [ "$test_database_status" -ne 1 ]; then
  cat "$temp_dir/test-database-env.out" >&2
  echo "analytics-indexer-execution self-test: expected test-database-env exit 1, got $test_database_status" >&2
  exit 1
fi
grep -q "never contacts databases" "$temp_dir/test-database-env.out"

set +e
BSC_RPC_URL=https://example.invalid "$verify" --mode integrity >"$temp_dir/live-env.out" 2>&1
live_status=$?
set -e
if [ "$live_status" -ne 1 ]; then
  cat "$temp_dir/live-env.out" >&2
  echo "analytics-indexer-execution self-test: expected live-env exit 1, got $live_status" >&2
  exit 1
fi
grep -q "never contacts databases, Redis, chains, or live market-data providers" "$temp_dir/live-env.out"

echo "analytics-indexer-execution self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, source/schema-anchor/A2.5/A2.6/A2.7/A2.8/A2.9/A2.10-link/stale/path/domain/blocker/prod/live tamper=1)"
