#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/content-lifecycle.json"
mode=""

die() {
  echo "content-lifecycle: ERROR: $*" >&2
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

for name in DATABASE_URL CONTENT_DATABASE_URL ANALYTICS_DATABASE_URL SUBSCRIPTION_DATABASE_URL REDIS_URL S3_ENDPOINT AWS_ENDPOINT_URL RPC_URL CHAIN_RPC_URL; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts databases, Redis, object storage, or a chain"
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
  console.error(`content-lifecycle: ERROR: ${message}`);
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
const exactIds = (items, expected, label) => {
  if (!Array.isArray(items) || items.length !== expected.length) fail(`${label} inventory must contain exactly ${expected.length} records`);
  const ids = items.map((item) => item?.id);
  if (new Set(ids).size !== ids.length || expected.some((id) => !ids.includes(id)) || ids.some((id) => !expected.includes(id))) {
    fail(`${label} inventory drifted`);
  }
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A10.0-content-lifecycle") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.lifecycleParity !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("safety flags must remain false");

const source = contract.source;
if (!source || source.ref !== "origin/development" || !/^[0-9a-f]{40}$/.test(source.commit)) fail("invalid pinned source ref/commit");
const resolvedRef = git("rev-parse", `${source.ref}^{commit}`);
if (resolvedRef !== source.commit) fail(`stale source ref/commit: ${source.ref}=${resolvedRef}, contract=${source.commit}`);
if (!Array.isArray(source.evidence) || source.evidence.length !== 14) fail("exactly fourteen pinned source evidence records are required");

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

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length !== 32) fail("exactly thirty-two target evidence records are required");
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

const boundary = contract.currentBoundary;
if (!boundary || boundary.authorization !== "partial" || boundary.editorRoutes !== "fail-closed-404" || boundary.lifecycleParityClaim !== false || boundary.productionClaim !== false) {
  fail("A2.3b boundary facts drifted or overclaim readiness");
}
if (!Array.isArray(boundary.evidenceIds) || boundary.evidenceIds.length < 2) fail("A2.3b boundary evidence is incomplete");
for (const id of boundary.evidenceIds) if (!evidenceIds.has(id)) fail(`currentBoundary: unknown evidence id ${id}`);

const expectedBatches = [
  "public-page-render",
  "public-content-discovery",
  "public-news-media",
  "public-plans",
  "public-rankings",
  "authenticated-portfolio-watchlist",
  "admin-content-lifecycle",
  "editor-sessions"
];
exactIds(contract.routeBatches, expectedBatches, "route batch");
for (const batch of contract.routeBatches) {
  if (batch.status !== "blocked" || typeof batch.ownerKey !== "string" || !batch.ownerKey) fail(`${batch.id}: route batch must remain blocked with an owner key`);
  if (!Array.isArray(batch.routes) || batch.routes.length === 0 || batch.routes.some((route) => typeof route !== "string" || !route.includes("/api/v1/"))) fail(`${batch.id}: route inventory is invalid`);
  if (typeof batch.contract !== "string" || !batch.contract) fail(`${batch.id}: route contract is required`);
  if (!Array.isArray(batch.blockerIds) || batch.blockerIds.length === 0) fail(`${batch.id}: blocker references are required`);
}

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 20) fail("exactly 20 stop blockers are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^B[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop blocker state changed without readiness proof`);
  if (typeof blocker.category !== "string" || !blocker.category || typeof blocker.summary !== "string" || !blocker.summary || typeof blocker.resolution !== "string" || !blocker.resolution) fail(`${blocker.id}: category/summary/resolution required`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
}
for (let n = 1; n <= 20; n += 1) {
  const id = `B${String(n).padStart(2, "0")}`;
  if (!blockerIds.has(id)) fail(`missing stop blocker ${id}`);
}
for (const batch of contract.routeBatches) for (const id of batch.blockerIds) if (!blockerIds.has(id)) fail(`${batch.id}: unknown blocker ${id}`);

const expectedRequirements = [
  "published-only-read",
  "locale-slug-not-found-cache",
  "page-crud-validation",
  "theme-crud-validation",
  "block-crud-validation",
  "editor-identity-session-ownership",
  "publish-workflow",
  "media-references",
  "file-watcher-sync-trust",
  "migration-backfill-reconciliation",
  "backend-owned-entitlement-ranking",
  "bff-wire-contract",
  "public-cache-security-headers",
  "audit-outbox-idempotency",
  "ui-state-machine",
  "rollback"
];
exactIds(contract.lifecycleRequirements, expectedRequirements, "lifecycle requirement");
for (const requirement of contract.lifecycleRequirements) {
  if (requirement.status !== "blocked") fail(`${requirement.id}: lifecycle requirement must remain blocked`);
  if (!Array.isArray(requirement.blockerIds) || requirement.blockerIds.length === 0) fail(`${requirement.id}: blocker references are required`);
  for (const id of requirement.blockerIds) if (!blockerIds.has(id)) fail(`${requirement.id}: unknown blocker ${id}`);
  if (!Array.isArray(requirement.acceptance) || requirement.acceptance.length < 2 || requirement.acceptance.some((item) => typeof item !== "string" || !item)) fail(`${requirement.id}: at least two acceptance checks are required`);
}

for (const section of ["backendOwnershipRules", "wireRules", "auditRules", "rollbackRules"]) {
  const rules = contract[section];
  if (!Array.isArray(rules) || rules.length < 3) fail(`${section} must contain at least three rules`);
  const ids = new Set();
  for (const rule of rules) {
    if (!rule || typeof rule.id !== "string" || ids.has(rule.id) || rule.status !== "required-unproven" || typeof rule.rule !== "string" || !rule.rule) fail(`${section}: invalid rule ${rule?.id}`);
    ids.add(rule.id);
  }
}

if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 12) fail("required execution order must contain exactly twelve phases");
for (let index = 0; index < contract.requiredExecutionOrder.length; index += 1) {
  const phase = contract.requiredExecutionOrder[index];
  const expectedStep = index + 1;
  if (!phase || phase.step !== expectedStep || typeof phase.phase !== "string" || !phase.phase || typeof phase.exitEvidence !== "string" || !phase.exitEvidence) fail(`invalid execution phase ${expectedStep}`);
  if (!Array.isArray(phase.dependsOn) || phase.dependsOn.some((dependency) => !Number.isInteger(dependency) || dependency < 1 || dependency >= expectedStep)) fail(`${phase.phase}: dependencies must reference earlier phases`);
}

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  targetEvidence: contract.targetEvidence.length,
  currentBoundary: {
    authorization: boundary.authorization,
    editorRoutes: boundary.editorRoutes,
    lifecycleParityClaim: false
  },
  routeBatches: contract.routeBatches.map((item) => ({ id: item.id, status: item.status, routes: item.routes.length })),
  lifecycleRequirements: contract.lifecycleRequirements.map((item) => ({ id: item.id, status: item.status })),
  rules: {
    backendOwnership: contract.backendOwnershipRules.length,
    wire: contract.wireRules.length,
    audit: contract.auditRules.length,
    rollback: contract.rollbackRules.length
  },
  executionPhases: contract.requiredExecutionOrder.map((item) => item.phase),
  blockers: contract.blockers.map((item) => ({ id: item.id, category: item.category, status: item.status })),
  productionReady: false,
  lifecycleParity: false,
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
  echo "content-lifecycle: PASS — pinned evidence and contract integrity verified (20 stop blockers, 8 route batches)"
  echo "content-lifecycle: LIMIT — A2.3b authorization is partial, editor routes remain fail-closed 404, and no lifecycle parity or production readiness was proven"
  exit 0
fi

echo "content-lifecycle: STOP — 20 stop blockers remain across 8 route batches; readiness is intentionally reserved as exit 3" >&2
echo "content-lifecycle: LIMIT — integrity may pass while content lifecycle parity remains unproven" >&2
exit 3
