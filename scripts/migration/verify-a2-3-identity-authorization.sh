#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a2-3-identity-authorization.json"
mode=""

die() {
  echo "identity-authorization: ERROR: $*" >&2
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

for name in DATABASE_URL IDENTITY_DATABASE_URL REDIS_URL REDIS_CLUSTER_URL JWKS_URL AUTH_JWKS_URL AUTH_BASE_URL BACKEND_URL NEXT_PUBLIC_BACKEND_URL; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts databases, Redis, JWKS, or services"
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
import { isAbsolute, relative, resolve } from "node:path";

const [rootInput, contractInput] = process.argv.slice(1);
const root = realpathSync(rootInput);
const fail = (message) => {
  console.error(`identity-authorization: ERROR: ${message}`);
  process.exit(1);
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" },
  });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const parse = (path, label) => {
  try { return JSON.parse(readFileSync(path, "utf8")); }
  catch (error) { fail(`invalid ${label} JSON: ${error.message}`); }
};
const parseText = (content, label) => {
  try { return JSON.parse(content); }
  catch (error) { fail(`invalid ${label} JSON: ${error.message}`); }
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const pieces = value.split("/");
  if (pieces.some((piece) => !piece || piece === "." || piece === "..")) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const candidate = resolve(root, value);
  const rel = relative(root, candidate);
  if (rel.startsWith("..") || isAbsolute(rel)) fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  return value;
};
const strings = (value, label) => {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || !item)) fail(`${label} must be an array of non-empty strings`);
  return value;
};
const anchored = (content, anchors, label) => {
  for (const [index, anchor] of strings(anchors, `${label}.anchors`).entries()) {
    if (anchor.length < 4) fail(`${label}.anchors[${index}] is too short`);
    if (!content.includes(anchor)) fail(`missing ${label} anchor: ${anchor}`);
  }
};

const contract = parse(contractInput, "identity authorization contract");
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-3-identity-authorization") fail("unexpected schemaVersion or artifact");
if (contract.contractId !== "A2.3h-identity-direct-service-authorization-audit") fail("unexpected contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || Object.values(contract.safety).some((value) => value !== false)) fail("all safety execution flags must remain false");

const source = contract.source;
if (!source || source.ref !== "origin/development" || source.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") fail("invalid pinned source ref/commit");
if (git("rev-parse", `${source.ref}^{commit}`) !== source.commit) fail("stale source ref/commit");
if (typeof source.interpretation !== "string" || !source.interpretation.includes("not automatically a safe production target")) fail("source interpretation must remain fail-closed");
if (!Array.isArray(source.evidence) || source.evidence.length !== 7) fail("exactly seven pinned source evidence records are required");

const evidenceIds = new Set();
for (const item of source.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  const file = safeRelative(item.file, item.id);
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid source blob`);
  if (git("rev-parse", `${source.commit}:${file}`) !== item.blob) fail(`${item.id}: stale source blob for ${file}`);
  anchored(git("show", `${source.commit}:${file}`), item.anchors, `source ${item.id}`);
}

const target = contract.targetSnapshot;
if (!target || target.ref !== "migration/dioxus-microservices" || target.commit !== "0cdd7ba1967d52e299000b7290873cd4d19dfd09") fail("invalid pinned target ref/commit");
if (git("rev-parse", `${target.commit}^{commit}`) !== target.commit) fail("missing pinned target commit");
if (typeof target.interpretation !== "string" || !target.interpretation.includes("do not describe current runtime status")) fail("target interpretation must remain historical and immutable");
if (!Array.isArray(target.evidence) || target.evidence.length !== 6) fail("exactly six pinned target evidence records are required");
for (const item of target.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  const file = safeRelative(item.file, item.id);
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid target blob`);
  if (git("rev-parse", `${target.commit}:${file}`) !== item.blob) fail(`${item.id}: stale target blob for ${file}`);
  anchored(git("show", `${target.commit}:${file}`), item.anchors, `target ${item.id}`);
  if (typeof item.finding !== "string" || !item.finding) fail(`${item.id}: finding is required`);
}

const serviceContract = parseText(git("show", `${target.commit}:docs/migration/contracts/service-authorization.json`), "target snapshot service authorization contract");
const identityService = serviceContract.services?.find((service) => service.name === "identity");
if (!identityService || !Array.isArray(identityService.routes) || identityService.routes.length !== 11) fail("A2 service contract must expose exactly eleven identity routes");
if (!Array.isArray(contract.routes) || contract.routes.length !== 11) fail("identity audit must contain exactly eleven routes");

const expectedPolicies = {
  "identity.get.health": ["GET", "/health", "partial", "none-public", null],
  "identity.post.auth-challenge": ["POST", "/api/v1/identity/auth/challenge", "blocked", "none-public-bootstrap", null],
  "identity.post.auth-siwe": ["POST", "/api/v1/identity/auth/siwe", "blocked", "one server-approved BFF audience selected by a proven flow", null],
  "identity.post.auth-refresh": ["POST", "/api/v1/identity/auth/refresh", "blocked", "server-bound original BFF audience; never caller-selected and never both", null],
  "identity.get.auth-me": ["GET", "/api/v1/identity/auth/me", "blocked", "epsx-frontend or epsx-admin, exactly one", null],
  "identity.post.auth-demo": ["POST", "/api/v1/identity/auth/demo", "blocked", "none-unreachable", null],
  "identity.get.users": ["GET", "/api/v1/identity/users", "blocked", "epsx-admin only", "admin:users:read"],
  "identity.post.users": ["POST", "/api/v1/identity/users", "blocked", "epsx-admin only", "admin:users:create"],
  "identity.get.user": ["GET", "/api/v1/identity/users/{id}", "blocked", "epsx-admin only", "admin:users:read"],
  "identity.put.user": ["PUT", "/api/v1/identity/users/{id}", "blocked", "epsx-admin only", "admin:users:update"],
  "identity.delete.user": ["DELETE", "/api/v1/identity/users/{id}", "blocked", "epsx-admin only", "admin:users:delete"],
};
const baselineById = new Map(identityService.routes.map((route) => [route.id, route]));
const seenRoutes = new Set();
const statuses = { aligned: 0, partial: 0, blocked: 0 };
const referencedBlockers = new Set();
for (const route of contract.routes) {
  if (!route || typeof route.id !== "string" || seenRoutes.has(route.id) || !Object.hasOwn(expectedPolicies, route.id)) fail(`invalid or duplicate route: ${route?.id}`);
  seenRoutes.add(route.id);
  const [method, path, status, audience, permission] = expectedPolicies[route.id];
  if (route.method !== method || route.path !== path || route.status !== status || route.audience !== audience || route.requiredPermission !== permission) fail(`${route.id}: route policy drifted`);
  const baseline = baselineById.get(route.id);
  if (!baseline || baseline.method !== method || baseline.path !== path || baseline.requiredPermission !== permission) fail(`${route.id}: route differs from A2 service authorization contract`);
  if (!Object.hasOwn(statuses, route.status)) fail(`${route.id}: invalid status`);
  statuses[route.status] += 1;
  if (typeof route.intendedBoundary !== "string" || !route.intendedBoundary || typeof route.observed !== "string" || !route.observed) fail(`${route.id}: boundary and observation are required`);
  if (route.ownership !== null && (typeof route.ownership !== "string" || !route.ownership)) fail(`${route.id}: ownership is invalid`);
  const ids = strings(route.blockerIds, `${route.id}.blockerIds`);
  if (ids.length === 0 || new Set(ids).size !== ids.length) fail(`${route.id}: unique blocker references are required`);
  for (const id of ids) referencedBlockers.add(id);
}
if (seenRoutes.size !== Object.keys(expectedPolicies).length || [...baselineById.keys()].some((id) => !seenRoutes.has(id))) fail("identity route set drifted");
if (statuses.aligned !== 0 || statuses.partial !== 1 || statuses.blocked !== 10) fail("route status count must remain conservative at 0 aligned, 1 partial, and 10 blocked");
if (JSON.stringify(statuses) !== JSON.stringify(contract.statusCounts)) fail("statusCounts disagrees with routes");

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 20) fail("exactly twenty STOP blockers are required");
const blockerIds = new Set();
for (const [index, blocker] of contract.blockers.entries()) {
  const expectedId = `B${String(index + 1).padStart(2, "0")}`;
  if (!blocker || blocker.id !== expectedId || blockerIds.has(blocker.id)) fail(`invalid or out-of-order blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: STOP blocker state changed without readiness proof`);
  for (const field of ["category", "summary", "resolution"]) if (typeof blocker[field] !== "string" || !blocker[field]) fail(`${blocker.id}: ${field} is required`);
}
for (const id of referencedBlockers) if (!blockerIds.has(id)) fail(`route references unknown blocker ${id}`);
for (const id of blockerIds) if (!referencedBlockers.has(id)) fail(`unreferenced blocker ${id}`);

if (!Array.isArray(contract.invariants) || contract.invariants.length !== 10) fail("exactly ten required invariants are required");
for (const [index, invariant] of contract.invariants.entries()) {
  const expectedId = `I${String(index + 1).padStart(2, "0")}`;
  if (!invariant || invariant.id !== expectedId || invariant.status !== "required-unproven" || typeof invariant.rule !== "string" || !invariant.rule) fail(`invalid invariant ${invariant?.id}`);
}
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 12) fail("exactly twelve ordered execution steps are required");
for (const [index, step] of contract.requiredExecutionOrder.entries()) {
  const prefix = `E${String(index + 1).padStart(2, "0")} `;
  if (typeof step !== "string" || !step.startsWith(prefix)) fail(`execution order step ${index + 1} drifted`);
}

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  targetSnapshot: { ref: target.ref, commit: target.commit, evidence: target.evidence.length },
  routeCount: contract.routes.length,
  statuses,
  routes: contract.routes.map((route) => ({ id: route.id, method: route.method, path: route.path, status: route.status, audience: route.audience, requiredPermission: route.requiredPermission, blockerCount: route.blockerIds.length })),
  invariantCount: contract.invariants.length,
  blockerCount: contract.blockers.length,
  executionSteps: contract.requiredExecutionOrder.length,
  productionReady: false,
  readinessExit: 3,
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s\n' "$summary"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "identity-authorization: PASS integrity (7 source pins; 6 immutable target pins; 11 routes: 0 aligned, 1 partial, 10 blocked; 10 invariants; 20 STOP blockers)"
  echo "identity-authorization: LIMIT — no database, Redis, JWKS, service, browser, migration, deployment, or production readiness was proven"
  exit 0
fi

echo "identity-authorization: STOP readiness (10 blocked routes, 1 partial route, 20 STOP blockers)" >&2
echo "identity-authorization: LIMIT — integrity success is an offline audit only and never production readiness" >&2
exit 3
