#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/subscription-execution.json"
mode=""

die() {
  echo "subscription-execution: ERROR: $*" >&2
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

for name in DATABASE_URL PAY_DATABASE_URL PAYMENTS_DATABASE_URL SUBSCRIPTION_DATABASE_URL IDENTITY_DATABASE_URL ANALYTICS_DATABASE_URL REDIS_URL RPC_URL CHAIN_RPC_URL; do
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
  console.error(`subscription-execution: ERROR: ${message}`);
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
const requiredRuleList = (contract, section, minimum) => {
  const rules = contract[section];
  if (!Array.isArray(rules) || rules.length < minimum) fail(`${section} is incomplete`);
  const ids = new Set();
  for (const rule of rules) {
    if (!rule || typeof rule.id !== "string" || ids.has(rule.id) || rule.status !== "required-unproven" || typeof rule.rule !== "string" || !rule.rule) {
      fail(`${section}: invalid rule ${rule?.id}`);
    }
    ids.add(rule.id);
  }
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A9.0-subscription-execution") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || contract.safety.writesDatabase !== false || contract.safety.contactsChain !== false || contract.safety.contactsNetwork !== false || contract.safety.deploys !== false || contract.safety.mutatesRuntime !== false) fail("safety flags must remain false");

const source = contract.source;
if (!source || source.ref !== "origin/development" || !/^[0-9a-f]{40}$/.test(source.commit)) fail("invalid pinned source ref/commit");
const resolvedRef = git("rev-parse", `${source.ref}^{commit}`);
if (resolvedRef !== source.commit) fail(`stale source ref/commit: ${source.ref}=${resolvedRef}, contract=${source.commit}`);
if (!Array.isArray(source.evidence) || source.evidence.length !== 18) fail("exactly eighteen pinned source evidence records are required");

const evidenceIds = new Set();
for (const item of source.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid source blob`);
  const actualBlob = git("rev-parse", `${source.commit}:${item.file}`);
  if (actualBlob !== item.blob) fail(`${item.id}: stale source blob for ${item.file}`);
  anchored(git("show", `${source.commit}:${item.file}`), item, "source");
}

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length !== 25) fail("exactly twenty-five target evidence records are required");
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
  "public-plan-read", "owner-current-access", "owner-create-activation", "owner-list",
  "owner-detail", "owner-cancel", "owner-switch-preview", "renewal-expiry",
  "verified-payment-activation", "admin-subscription-mutations", "admin-plan-mutations",
  "entitlement-ranking-projection"
];
if (!Array.isArray(contract.routeContracts) || contract.routeContracts.length !== expectedRoutes.length) fail("twelve route/lifecycle contracts are required");
const routeIds = new Set();
for (const route of contract.routeContracts) {
  if (!route || !expectedRoutes.includes(route.id) || routeIds.has(route.id)) fail(`invalid or duplicate route contract: ${route?.id}`);
  routeIds.add(route.id);
  if (route.status !== "blocked" || typeof route.ownerKey !== "string" || !route.ownerKey) fail(`${route.id}: route must remain blocked with an owner key`);
  if (!route.source || typeof route.source.method !== "string" || typeof route.source.path !== "string" || !route.source.path) fail(`${route.id}: invalid source method/path`);
  if (!Array.isArray(route.source.body) || !Array.isArray(route.source.successStatuses) || route.source.successStatuses.length === 0) fail(`${route.id}: body/status contract is required`);
  if (!Array.isArray(route.targetObserved) || route.targetObserved.length === 0) fail(`${route.id}: target observations are required`);
  if (!Array.isArray(route.blockerIds) || route.blockerIds.length === 0) fail(`${route.id}: blocker references are required`);
}
if (expectedRoutes.some((id) => !routeIds.has(id))) fail("route/lifecycle contract inventory drifted");

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 20) fail("exactly 20 stop blockers are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^B[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop blocker state changed without readiness proof`);
  if (typeof blocker.category !== "string" || typeof blocker.summary !== "string" || typeof blocker.resolution !== "string" || !blocker.category || !blocker.summary || !blocker.resolution) fail(`${blocker.id}: category/summary/resolution required`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
}
for (const route of contract.routeContracts) for (const id of route.blockerIds) if (!blockerIds.has(id)) fail(`${route.id}: unknown blocker ${id}`);

if (!contract.planAuthority || contract.planAuthority.status !== "required-unproven" || typeof contract.planAuthority.rule !== "string" || !Array.isArray(contract.planAuthority.currentDrift) || contract.planAuthority.currentDrift.length < 3) fail("planAuthority is incomplete");
requiredRuleList(contract, "ownershipRules", 5);
requiredRuleList(contract, "idempotencyRules", 5);
requiredRuleList(contract, "uiStateRules", 4);
requiredRuleList(contract, "schedulerOutboxRules", 3);

const lifecycle = contract.lifecycleStateMachine;
if (!lifecycle || lifecycle.status !== "required-unproven" || !Array.isArray(lifecycle.states) || lifecycle.states.length !== 7 || !lifecycle.states.includes("pending_payment") || !lifecycle.states.includes("active") || !lifecycle.states.includes("expired") || !Array.isArray(lifecycle.allowedTransitions) || lifecycle.allowedTransitions.length < 7 || !Array.isArray(lifecycle.forbiddenTransitions) || lifecycle.forbiddenTransitions.length < 4 || typeof lifecycle.autoRenewPolicy !== "string") fail("lifecycle state machine is incomplete");
if (!contract.activeSubscriptionPolicy || contract.activeSubscriptionPolicy.status !== "required-unproven" || !Array.isArray(contract.activeSubscriptionPolicy.databaseProofRequired) || contract.activeSubscriptionPolicy.databaseProofRequired.length < 5) fail("active subscription overlap policy is incomplete");
const semantics = contract.statusSemantics;
if (!semantics || semantics.createPending !== 202 || semantics.createAdmin !== 201 || semantics.readSuccess !== 200 || semantics.cancelSuccess !== 200 || semantics.foreignOrMissing !== 404 || semantics.validationFailure !== 400 || semantics.unauthenticated !== 401 || semantics.forbidden !== 403 || semantics.conflictOrReplayMismatch !== 409 || semantics.dependencyUnavailable !== 503 || typeof semantics.envelope !== "string") fail("status/envelope semantics drifted");
if (!Array.isArray(contract.durableStateDependencies) || contract.durableStateDependencies.length !== 10) fail("durable state dependencies are incomplete");
if (!contract.migrationAndReconciliation || contract.migrationAndReconciliation.status !== "required-unproven" || !Array.isArray(contract.migrationAndReconciliation.steps) || contract.migrationAndReconciliation.steps.length !== 8) fail("migration/reconciliation plan is incomplete");
if (!contract.rollback || contract.rollback.status !== "required-unproven" || !Array.isArray(contract.rollback.triggers) || contract.rollback.triggers.length < 5 || !Array.isArray(contract.rollback.actions) || contract.rollback.actions.length < 6) fail("rollback plan is incomplete");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 9) fail("required execution order drifted");
for (let index = 0; index < contract.requiredExecutionOrder.length; index++) {
  const item = contract.requiredExecutionOrder[index];
  if (!item || item.order !== index + 1 || typeof item.id !== "string" || !item.id || typeof item.acceptance !== "string" || !item.acceptance) fail(`invalid execution order item ${index + 1}`);
}

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  targetEvidence: contract.targetEvidence.length,
  routeContracts: contract.routeContracts.map((item) => item.id),
  rules: {
    ownership: contract.ownershipRules.length,
    idempotency: contract.idempotencyRules.length,
    uiStates: contract.uiStateRules.length,
    schedulerOutbox: contract.schedulerOutboxRules.length
  },
  lifecycleStates: contract.lifecycleStateMachine.states,
  durableStateDependencies: contract.durableStateDependencies.length,
  executionOrder: contract.requiredExecutionOrder.map((item) => item.id),
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
  echo "subscription-execution: PASS — pinned evidence and contract integrity verified (20 stop blockers)"
  echo "subscription-execution: LIMIT — no database, chain, deployment, migration, or production readiness was proven"
  exit 0
fi

echo "subscription-execution: STOP — 20 stop blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "subscription-execution: LIMIT — integrity may pass while subscription execution remains non-production" >&2
exit 3
