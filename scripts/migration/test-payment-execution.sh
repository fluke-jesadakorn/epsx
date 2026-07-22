#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-payment-execution.sh"
contract="$repo_root/docs/migration/contracts/payment-execution.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-payment-execution.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "17 stop blockers" "$temp_dir/integrity.out"
grep -q "no database, chain, deployment" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "payment-execution self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "17 stop blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e 'const report = JSON.parse(await Bun.file(process.argv[1]).text()); if (report.readinessExit !== 3 || report.productionReady !== false || report.blockers.length !== 17 || report.targetEvidence !== 48) process.exit(1); if (report.schemaBoundaryEvidence.join(",") !== "tgt-pay-schema-boundary,tgt-subscription-schema-boundary") process.exit(1); if (report.authorityCrosswalk.decision !== "unresolved-do-not-cut-over-or-dual-write" || report.authorityCrosswalk.productionWriteAuthority !== null || report.authorityCrosswalk.systems.length !== 4) process.exit(1); if (report.authorityCrosswalk.databaseNames.join(",") !== "epsx_payment,epsx_pay,epsx_pay_dev,epsx_payments_dev,epsx_payments_staging|epsx_payments_prod,epsx_subscription") process.exit(1); if (report.remainingRuntimeDdl.total !== 3 || report.remainingRuntimeDdl.payAfter !== 0 || report.remainingRuntimeDdl.findings.join(",") !== "finding.002,finding.003,finding.004") process.exit(1);' "$temp_dir/report-one.json"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/missing-anchor.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.source.evidence[0].anchor = "tampered missing source anchor";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/missing-anchor.json" >"$temp_dir/missing-anchor.out" 2>&1
anchor_status=$?
set -e
if [ "$anchor_status" -ne 1 ]; then
  cat "$temp_dir/missing-anchor.out" >&2
  echo "payment-execution self-test: expected missing-anchor exit 1, got $anchor_status" >&2
  exit 1
fi
grep -q "missing source anchor" "$temp_dir/missing-anchor.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/wrong-source-anchor.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.source.evidence[0].anchor = "export interface PaymentConfirmRequest {";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/wrong-source-anchor.json" >"$temp_dir/wrong-source-anchor.out" 2>&1
wrong_source_anchor_status=$?
set -e
if [ "$wrong_source_anchor_status" -ne 1 ]; then
  cat "$temp_dir/wrong-source-anchor.out" >&2
  echo "payment-execution self-test: expected existing-but-wrong source anchor exit 1, got $wrong_source_anchor_status" >&2
  exit 1
fi
grep -q "source semantic evidence pins drifted" "$temp_dir/wrong-source-anchor.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/stale-source.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.source.commit = "0000000000000000000000000000000000000000";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-source.json" >"$temp_dir/stale-source.out" 2>&1
stale_status=$?
set -e
if [ "$stale_status" -ne 1 ]; then
  cat "$temp_dir/stale-source.out" >&2
  echo "payment-execution self-test: expected stale-source exit 1, got $stale_status" >&2
  exit 1
fi
grep -q "stale source ref/commit" "$temp_dir/stale-source.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.targetEvidence[0].file = "../outside";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
if [ "$traversal_status" -ne 1 ]; then
  cat "$temp_dir/traversal.out" >&2
  echo "payment-execution self-test: expected traversal exit 1, got $traversal_status" >&2
  exit 1
fi
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/wrong-target-anchor.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.targetEvidence.find((item) => item.id === "tgt-pay-schema-boundary").anchor = "let app = Router::new()";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/wrong-target-anchor.json" >"$temp_dir/wrong-target-anchor.out" 2>&1
wrong_target_anchor_status=$?
set -e
if [ "$wrong_target_anchor_status" -ne 1 ]; then
  cat "$temp_dir/wrong-target-anchor.out" >&2
  echo "payment-execution self-test: expected existing-but-wrong target anchor exit 1, got $wrong_target_anchor_status" >&2
  exit 1
fi
grep -q "target semantic evidence pin drifted" "$temp_dir/wrong-target-anchor.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/wrong-runtime-db-anchor.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.targetEvidence.find((item) => item.id === "tgt-db-epsx-payments-runtime-dev").anchor = "PAYMENTS_DATABASE_URL: postgresql://${DB_USER:-epsx_user}:${DB_PASSWORD:-password}@postgres:5432/epsx_pay_dev";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/wrong-runtime-db-anchor.json" >"$temp_dir/wrong-runtime-db-anchor.out" 2>&1
wrong_runtime_db_anchor_status=$?
set -e
if [ "$wrong_runtime_db_anchor_status" -ne 1 ]; then
  cat "$temp_dir/wrong-runtime-db-anchor.out" >&2
  echo "payment-execution self-test: expected existing-but-wrong runtime DB anchor exit 1, got $wrong_runtime_db_anchor_status" >&2
  exit 1
fi
grep -q "tgt-db-epsx-payments-runtime-dev: target semantic evidence pin drifted" "$temp_dir/wrong-runtime-db-anchor.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/stale-pay-ddl.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
const item = contract.targetEvidence.find((entry) => entry.id === "tgt-pay-schema-boundary");
item.id = "tgt-pay-runtime-ddl";
for (const blocker of contract.blockers) blocker.evidenceIds = blocker.evidenceIds.map((id) => id === "tgt-pay-schema-boundary" ? "tgt-pay-runtime-ddl" : id);
for (const surface of contract.nonProductionSurfaces) surface.evidenceIds = surface.evidenceIds.map((id) => id === "tgt-pay-schema-boundary" ? "tgt-pay-runtime-ddl" : id);
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-pay-ddl.json" >"$temp_dir/stale-pay-ddl.out" 2>&1
stale_pay_ddl_status=$?
set -e
if [ "$stale_pay_ddl_status" -ne 1 ]; then
  cat "$temp_dir/stale-pay-ddl.out" >&2
  echo "payment-execution self-test: expected stale-pay-DDL exit 1, got $stale_pay_ddl_status" >&2
  exit 1
fi
grep -q "stale runtime-DDL evidence returned" "$temp_dir/stale-pay-ddl.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/authority-selected.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.authorityCrosswalk.decision = "canonical-backend-is-authority";
contract.authorityCrosswalk.productionWriteAuthority = "canonical-backend";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/authority-selected.json" >"$temp_dir/authority-selected.out" 2>&1
authority_status=$?
set -e
if [ "$authority_status" -ne 1 ]; then
  cat "$temp_dir/authority-selected.out" >&2
  echo "payment-execution self-test: expected authority-selection exit 1, got $authority_status" >&2
  exit 1
fi
grep -q "payment authority crosswalk drifted" "$temp_dir/authority-selected.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/runtime-db-role.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.authorityCrosswalk.databaseNameCrosswalk.find((item) => item.name === "epsx_payments_dev").role = "canonical-backend-compose-migrator-payments-url-candidate";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/runtime-db-role.json" >"$temp_dir/runtime-db-role.out" 2>&1
runtime_db_role_status=$?
set -e
if [ "$runtime_db_role_status" -ne 1 ]; then
  cat "$temp_dir/runtime-db-role.out" >&2
  echo "payment-execution self-test: expected runtime-DB-role exit 1, got $runtime_db_role_status" >&2
  exit 1
fi
grep -q "payment authority crosswalk drifted" "$temp_dir/runtime-db-role.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/uniform-mutation-404.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.authorityCrosswalk.systems.find((item) => item.id === "current-pay-candidate").writeReachability = "all-financial-mutations-404";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/uniform-mutation-404.json" >"$temp_dir/uniform-mutation-404.out" 2>&1
uniform_mutation_status=$?
set -e
if [ "$uniform_mutation_status" -ne 1 ]; then
  cat "$temp_dir/uniform-mutation-404.out" >&2
  echo "payment-execution self-test: expected uniform-mutation-404 exit 1, got $uniform_mutation_status" >&2
  exit 1
fi
grep -q "payment authority crosswalk drifted" "$temp_dir/uniform-mutation-404.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/stale-b06.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.blockers.find((item) => item.id === "B06").summary = "Pay reads are owner-unscoped.";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-b06.json" >"$temp_dir/stale-b06.out" 2>&1
stale_b06_status=$?
set -e
if [ "$stale_b06_status" -ne 1 ]; then
  cat "$temp_dir/stale-b06.out" >&2
  echo "payment-execution self-test: expected stale-B06 exit 1, got $stale_b06_status" >&2
  exit 1
fi
grep -q "B06 corrected ownership blocker drifted" "$temp_dir/stale-b06.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/stale-subscription-observation.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.routeContracts.find((item) => item.id === "subscription-lifecycle").targetObserved = ["separate unauthenticated service CRUD"];
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-subscription-observation.json" >"$temp_dir/stale-subscription-observation.out" 2>&1
stale_subscription_observation_status=$?
set -e
if [ "$stale_subscription_observation_status" -ne 1 ]; then
  cat "$temp_dir/stale-subscription-observation.out" >&2
  echo "payment-execution self-test: expected stale-subscription-observation exit 1, got $stale_subscription_observation_status" >&2
  exit 1
fi
grep -q "subscription-lifecycle ownership observation drifted" "$temp_dir/stale-subscription-observation.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/stale-b12.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.blockers.find((item) => item.id === "B12").summary = "Subscription CRUD is unauthenticated.";
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-b12.json" >"$temp_dir/stale-b12.out" 2>&1
stale_b12_status=$?
set -e
if [ "$stale_b12_status" -ne 1 ]; then
  cat "$temp_dir/stale-b12.out" >&2
  echo "payment-execution self-test: expected stale-B12 exit 1, got $stale_b12_status" >&2
  exit 1
fi
grep -q "B12 corrected subscription blocker drifted" "$temp_dir/stale-b12.out"

PAYMENT_CONTRACT_IN="$contract" PAYMENT_CONTRACT_OUT="$temp_dir/stale-ddl-correction.json" bun -e '
const contract = await Bun.file(process.env.PAYMENT_CONTRACT_IN).json();
contract.remainingRuntimeDdlCorrection.totalFindings = 13;
await Bun.write(process.env.PAYMENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-ddl-correction.json" >"$temp_dir/stale-ddl-correction.out" 2>&1
stale_ddl_correction_status=$?
set -e
if [ "$stale_ddl_correction_status" -ne 1 ]; then
  cat "$temp_dir/stale-ddl-correction.out" >&2
  echo "payment-execution self-test: expected stale-DDL-correction exit 1, got $stale_ddl_correction_status" >&2
  exit 1
fi
grep -q "remaining runtime DDL correction drifted" "$temp_dir/stale-ddl-correction.out"

echo "payment-execution self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, source/target/runtime-DB/role/reachability/subscription/B06/B12/DDL tamper=1)"
