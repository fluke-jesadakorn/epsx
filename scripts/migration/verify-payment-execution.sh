#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/payment-execution.json"
mode=""

die() {
  echo "payment-execution: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      [ "$#" -ge 2 ] || die "--mode requires integrity, readiness, or report"
      mode=$2
      shift 2
      ;;
    --contract)
      [ "$#" -ge 2 ] || die "--contract requires a local JSON file"
      contract=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

case "$mode" in
  integrity|readiness|report) ;;
  *) die "--mode must be integrity, readiness, or report" ;;
esac

case "$contract" in
  http://*|https://*) die "contract must be a local file" ;;
esac
[ -f "$contract" ] || die "missing contract: $contract"
command -v bun >/dev/null 2>&1 || die "bun is required"
command -v git >/dev/null 2>&1 || die "git is required"

for name in DATABASE_URL PAY_DATABASE_URL PAYMENTS_DATABASE_URL SUBSCRIPTION_DATABASE_URL REDIS_URL RPC_URL CHAIN_RPC_URL; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts databases, Redis, or a chain"
done

for name in EPSX_ENV APP_ENV ENVIRONMENT NODE_ENV RUST_ENV DEPLOY_ENV; do
  eval "value=\${$name-}"
  normalized=$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')
  case "$normalized" in
    prod|production|prod-*|production-*|*-prod|*-production)
      die "$name identifies a production-looking environment"
      ;;
  esac
done

export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

summary=$(bun -e '
import { readFileSync, realpathSync } from "node:fs";
import { isAbsolute, resolve, sep } from "node:path";

const [rootInput, contractPath] = process.argv.slice(1);
const root = realpathSync(rootInput);
const fail = (message) => {
  console.error(`payment-execution: ERROR: ${message}`);
  process.exit(1);
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", ...args], { cwd: root, stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const segments = value.split("/");
  if (segments.some((segment) => !segment || segment === "." || segment === "..")) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
};
const anchored = (content, item, scope) => {
  if (typeof item.anchor !== "string" || item.anchor.length < 4) fail(`${scope} ${item.id}: invalid anchor`);
  if (!content.includes(item.anchor)) fail(`missing ${scope} anchor ${item.id} in ${item.file}`);
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A6.0-payment-execution") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("safety flags must remain false");

const source = contract.source;
if (!source || source.ref !== "origin/development" || !/^[0-9a-f]{40}$/.test(source.commit)) fail("invalid pinned source ref/commit");
const resolvedRef = git("rev-parse", `${source.ref}^{commit}`);
if (resolvedRef !== source.commit) fail(`stale source ref/commit: ${source.ref}=${resolvedRef}, contract=${source.commit}`);
if (!Array.isArray(source.evidence) || source.evidence.length < 8) fail("at least eight pinned source evidence records are required");

const evidenceIds = new Set();
for (const item of source.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid source blob`);
  const actualBlob = git("rev-parse", `${source.commit}:${item.file}`);
  if (actualBlob !== item.blob) fail(`${item.id}: stale source blob for ${item.file}`);
  const content = git("show", `${source.commit}:${item.file}`);
  anchored(content, item, "source");
}

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length < 20) fail("at least twenty target evidence records are required");
for (const item of contract.targetEvidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  const candidate = resolve(root, item.file);
  let actual;
  try { actual = realpathSync(candidate); }
  catch { fail(`missing target evidence file ${item.file}`); }
  if (actual !== root && !actual.startsWith(`${root}${sep}`)) fail(`unsafe evidence path for ${item.id}: ${JSON.stringify(item.file)}`);
  anchored(readFileSync(actual, "utf8"), item, "target");
}

const expectedRoutes = [
  "payment-submit", "payment-status", "payment-validate", "payment-history", "plan-lifecycle",
  "admin-payment", "escrow-mutations", "on-chain-webhook", "subscription-lifecycle"
];
if (!Array.isArray(contract.routeContracts) || contract.routeContracts.length !== expectedRoutes.length) fail("nine route contracts are required");
const routeIds = new Set();
for (const route of contract.routeContracts) {
  if (!route || !expectedRoutes.includes(route.id) || routeIds.has(route.id)) fail(`invalid or duplicate route contract: ${route?.id}`);
  routeIds.add(route.id);
  if (route.status !== "blocked" || typeof route.ownerKey !== "string" || !route.ownerKey) fail(`${route.id}: route must remain blocked with an owner key`);
  if (!route.source || typeof route.source.method !== "string" || typeof route.source.path !== "string" || !route.source.path.startsWith("/")) fail(`${route.id}: invalid source method/path`);
  if (!Array.isArray(route.source.body) || !Array.isArray(route.source.successStatuses) || route.source.successStatuses.length === 0) fail(`${route.id}: body/status contract is required`);
  if (!Array.isArray(route.blockerIds) || route.blockerIds.length === 0) fail(`${route.id}: blocker references are required`);
}
if (expectedRoutes.some((id) => !routeIds.has(id))) fail("route contract inventory drifted");

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 17) fail("exactly 17 stop blockers are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^B[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop blocker state changed without readiness proof`);
  if (typeof blocker.summary !== "string" || typeof blocker.resolution !== "string" || !blocker.summary || !blocker.resolution) fail(`${blocker.id}: summary/resolution required`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
}
for (const route of contract.routeContracts) for (const id of route.blockerIds) if (!blockerIds.has(id)) fail(`${route.id}: unknown blocker ${id}`);

for (const section of ["ownershipRules", "idempotencyRules", "finalityRules"]) {
  const rules = contract[section];
  if (!Array.isArray(rules) || rules.length === 0) fail(`${section} must not be empty`);
  const ids = new Set();
  for (const rule of rules) {
    if (!rule || typeof rule.id !== "string" || ids.has(rule.id) || rule.status !== "required-unproven" || typeof rule.rule !== "string" || !rule.rule) fail(`${section}: invalid rule ${rule?.id}`);
    ids.add(rule.id);
  }
}
if (!contract.statusSemantics || contract.statusSemantics.acceptedForMonitoring !== 202 || contract.statusSemantics.foreignOrMissing !== 404 || contract.statusSemantics.conflictOrReplayMismatch !== 409) fail("status semantics drifted");
if (!Array.isArray(contract.nonProductionSurfaces) || contract.nonProductionSurfaces.length < 7) fail("non-production surface inventory is incomplete");
for (const surface of contract.nonProductionSurfaces) {
  if (!surface || typeof surface.id !== "string" || typeof surface.reason !== "string" || !surface.reason || !Array.isArray(surface.evidenceIds) || surface.evidenceIds.length === 0) fail(`invalid non-production surface ${surface?.id}`);
  for (const id of surface.evidenceIds) if (!evidenceIds.has(id)) fail(`${surface.id}: unknown evidence id ${id}`);
}
if (!Array.isArray(contract.durableStateDependencies) || contract.durableStateDependencies.length < 7) fail("durable state dependencies are incomplete");
if (!Array.isArray(contract.deploymentDependencies) || contract.deploymentDependencies.length < 6) fail("deployment dependencies are incomplete");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 9) fail("required execution order drifted");

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  targetEvidence: contract.targetEvidence.length,
  routeContracts: contract.routeContracts.map((item) => item.id),
  rules: {
    ownership: contract.ownershipRules.length,
    idempotency: contract.idempotencyRules.length,
    finality: contract.finalityRules.length
  },
  nonProductionSurfaces: contract.nonProductionSurfaces.map((item) => item.id),
  durableStateDependencies: contract.durableStateDependencies.length,
  deploymentDependencies: contract.deploymentDependencies.length,
  blockers: contract.blockers.map((item) => ({ id: item.id, category: item.category, status: item.status })),
  productionReady: false,
  readinessExit: 3
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s' "$summary"
  printf '\n'
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "payment-execution: PASS — pinned evidence and contract integrity verified (17 stop blockers)"
  echo "payment-execution: LIMIT — no database, chain, deployment, or production readiness was proven"
  exit 0
fi

echo "payment-execution: STOP — 17 stop blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "payment-execution: LIMIT — integrity may pass while payment execution remains non-production" >&2
exit 3
