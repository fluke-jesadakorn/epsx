#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-analytics-indexer-execution.sh"
contract="$repo_root/docs/migration/contracts/analytics-indexer-execution.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-analytics-indexer-execution.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "14 source pins, 36 target anchors, 4 separate domains, 16 surfaces, and 24 stop blockers" "$temp_dir/integrity.out"
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
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
const expected = ["marketAnalytics", "eventAnalytics", "indexer", "identityRankingOffset"];
if (report.readinessExit !== 3 || report.productionReady !== false || report.blockers.length !== 24 || report.targetEvidence !== 36 || report.surfaceContracts.length !== 16) process.exit(1);
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
BSC_RPC_URL=https://example.invalid "$verify" --mode integrity >"$temp_dir/live-env.out" 2>&1
live_status=$?
set -e
if [ "$live_status" -ne 1 ]; then
  cat "$temp_dir/live-env.out" >&2
  echo "analytics-indexer-execution self-test: expected live-env exit 1, got $live_status" >&2
  exit 1
fi
grep -q "never contacts databases, Redis, chains, or live market-data providers" "$temp_dir/live-env.out"

echo "analytics-indexer-execution self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, anchor/stale/path/domain/prod/live tamper=1)"
