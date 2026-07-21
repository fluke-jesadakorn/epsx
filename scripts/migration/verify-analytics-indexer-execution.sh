#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/analytics-indexer-execution.json"
mode=""

die() {
  echo "analytics-indexer-execution: ERROR: $*" >&2
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

for name in DATABASE_URL ANALYTICS_DATABASE_URL INDEXER_DATABASE_URL REDIS_URL REDIS_CLUSTER_URL RPC_URL CHAIN_RPC_URL BSC_RPC_URL BSC_MAINNET_RPC_URL BSC_TESTNET_RPC_URL ETH_RPC_URL WEB3_PROVIDER_URL TRADINGVIEW_URL TRADINGVIEW_BASE_URL TRADINGVIEW_WEBSOCKET_URL MARKET_DATA_URL MARKET_DATA_API_KEY LIVE_DATA_URL; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts databases, Redis, chains, or live market-data providers"
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

for name in LIVE_DATA USE_LIVE_DATA ANALYTICS_LIVE_DATA INDEXER_SYNC_ON_START SYNC_ON_START; do
  eval "value=\${$name-}"
  normalized=$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')
  case "$normalized" in
    1|true|yes|on|live|enabled) die "$name enables a live-data or sync path" ;;
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
  console.error(`analytics-indexer-execution: ERROR: ${message}`);
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

if (contract.schemaVersion !== 1 || contract.contractId !== "A12.0-analytics-indexer-execution") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("safety flags must remain false");

const expectedDomains = {
  marketAnalytics: "apps/analytics",
  eventAnalytics: "services/analytics",
  indexer: "services/indexer",
  identityRankingOffset: "shared/rust/epsx-identity-service"
};
if (!contract.domains || JSON.stringify(Object.keys(contract.domains).sort()) !== JSON.stringify(Object.keys(expectedDomains).sort())) fail("the four domain boundaries must remain explicit and separate");
for (const [domain, owner] of Object.entries(expectedDomains)) {
  const item = contract.domains[domain];
  if (!item || item.owner !== owner || item.status !== "blocked" || typeof item.dataClass !== "string" || !item.dataClass || typeof item.authority !== "string" || !item.authority) fail(`${domain}: owner/authority boundary drifted`);
}
if (new Set(Object.values(contract.domains).map((item) => item.owner)).size !== 4) fail("domain owners must not be conflated");

const source = contract.source;
if (!source || source.ref !== "origin/development" || source.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") fail("invalid pinned source ref/commit");
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

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length !== 36) fail("exactly thirty-six target evidence records are required");
const targetDomainCounts = Object.fromEntries(Object.keys(expectedDomains).map((domain) => [domain, 0]));
for (const item of contract.targetEvidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  if (!Object.hasOwn(expectedDomains, item.domain)) fail(`${item.id}: unknown target domain ${item.domain}`);
  targetDomainCounts[item.domain] += 1;
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  const candidate = resolve(root, item.file);
  let actual;
  try { actual = realpathSync(candidate); }
  catch { fail(`missing target evidence file ${item.file}`); }
  if (actual !== root && !actual.startsWith(`${root}${sep}`)) fail(`unsafe evidence path for ${item.id}: ${JSON.stringify(item.file)}`);
  anchored(readFileSync(actual, "utf8"), item, "target");
}
for (const [domain, count] of Object.entries(targetDomainCounts)) if (count < 3) fail(`${domain}: insufficient target evidence (${count})`);

const expectedSurfaces = [
  "market-public-rankings", "market-auth-rankings", "market-filters", "market-ui",
  "event-track", "event-reads", "event-revenue", "event-observability",
  "indexer-status", "indexer-block-transaction", "indexer-transfers", "indexer-sync",
  "identity-ranking-offset-query", "identity-ranking-offset-stream", "identity-ranking-offset-emit",
  "cross-domain-cutover"
];
if (!Array.isArray(contract.surfaceContracts) || contract.surfaceContracts.length !== expectedSurfaces.length) fail("exactly sixteen surface contracts are required");
const surfaceIds = new Set();
for (const surface of contract.surfaceContracts) {
  if (!surface || !expectedSurfaces.includes(surface.id) || surfaceIds.has(surface.id)) fail(`invalid or duplicate surface contract: ${surface?.id}`);
  surfaceIds.add(surface.id);
  if (!Object.hasOwn(expectedDomains, surface.domain)) fail(`${surface.id}: unknown domain ${surface.domain}`);
  if (surface.status !== "blocked" || typeof surface.sourceContract !== "string" || !surface.sourceContract || typeof surface.targetObserved !== "string" || !surface.targetObserved) fail(`${surface.id}: blocked source/target contract is required`);
  if (!Array.isArray(surface.blockerIds) || surface.blockerIds.length === 0) fail(`${surface.id}: blocker references are required`);
}
if (expectedSurfaces.some((id) => !surfaceIds.has(id))) fail("surface inventory drifted");

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 24) fail("exactly 24 stop blockers are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^B[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop blocker state changed without readiness proof`);
  if (typeof blocker.category !== "string" || !blocker.category || typeof blocker.summary !== "string" || !blocker.summary || typeof blocker.resolution !== "string" || !blocker.resolution) fail(`${blocker.id}: category/summary/resolution required`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
}
for (const surface of contract.surfaceContracts) for (const id of surface.blockerIds) if (!blockerIds.has(id)) fail(`${surface.id}: unknown blocker ${id}`);

const ruleSections = {
  ownershipRules: 7,
  freshnessRules: 6,
  backfillReconciliationRules: 7,
  observabilityRules: 6,
  cutoverRules: 5
};
for (const [section, expectedCount] of Object.entries(ruleSections)) {
  const rules = contract[section];
  if (!Array.isArray(rules) || rules.length !== expectedCount) fail(`${section} must contain exactly ${expectedCount} rules`);
  const ids = new Set();
  for (const rule of rules) {
    if (!rule || typeof rule.id !== "string" || ids.has(rule.id) || rule.status !== "required-unproven" || typeof rule.rule !== "string" || !rule.rule) fail(`${section}: invalid rule ${rule?.id}`);
    ids.add(rule.id);
  }
}

if (!Array.isArray(contract.nonProductionSurfaces) || contract.nonProductionSurfaces.length !== 12) fail("exactly twelve non-production surfaces are required");
for (const surface of contract.nonProductionSurfaces) {
  if (!surface || typeof surface.id !== "string" || typeof surface.reason !== "string" || !surface.reason || !Array.isArray(surface.evidenceIds) || surface.evidenceIds.length === 0) fail(`invalid non-production surface ${surface?.id}`);
  for (const id of surface.evidenceIds) if (!evidenceIds.has(id)) fail(`${surface.id}: unknown evidence id ${id}`);
}
if (!Array.isArray(contract.durableStateDependencies) || contract.durableStateDependencies.length !== 10) fail("durable state dependencies drifted");
if (!Array.isArray(contract.deploymentDependencies) || contract.deploymentDependencies.length !== 8) fail("deployment dependencies drifted");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 12) fail("required execution order drifted");

const surfaceDomainCounts = Object.fromEntries(Object.keys(expectedDomains).map((domain) => [domain, 0]));
for (const surface of contract.surfaceContracts) surfaceDomainCounts[surface.domain] += 1;
const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  domains: Object.fromEntries(Object.entries(contract.domains).map(([id, item]) => [id, { owner: item.owner, status: item.status, targetEvidence: targetDomainCounts[id], surfaces: surfaceDomainCounts[id] }])),
  targetEvidence: contract.targetEvidence.length,
  surfaceContracts: contract.surfaceContracts.map((item) => ({ id: item.id, domain: item.domain, status: item.status })),
  rules: Object.fromEntries(Object.keys(ruleSections).map((section) => [section, contract[section].length])),
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
  printf '%s\n' "$summary"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "analytics-indexer-execution: PASS — 14 source pins, 36 target anchors, 4 separate domains, 16 surfaces, and 24 stop blockers verified"
  echo "analytics-indexer-execution: LIMIT — no database, Redis, chain, network, live market-data, deployment, or production readiness was proven"
  exit 0
fi

echo "analytics-indexer-execution: STOP — 24 stop blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "analytics-indexer-execution: LIMIT — integrity may pass while all four execution domains remain non-production" >&2
exit 3
