#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a2-3-identity-fail-closed-runtime.json"
runtime_root="$repo_root"
mode=""

die() {
  echo "identity-fail-closed: ERROR: $*" >&2
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
    --runtime-root)
      [ "$#" -ge 2 ] || die "--runtime-root requires a local directory"
      runtime_root=$2
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
case "$runtime_root" in
  http://*|https://*) die "runtime root must be a local directory" ;;
esac
[ -f "$contract" ] || die "missing contract: $contract"
[ -d "$runtime_root" ] || die "missing runtime root: $runtime_root"
command -v bun >/dev/null 2>&1 || die "bun is required"

for name in DATABASE_URL IDENTITY_DATABASE_URL REDIS_URL REDIS_CLUSTER_URL JWKS_URL AUTH_JWKS_URL OIDC_JWKS_URL OIDC_ISSUER AUTH_BASE_URL BACKEND_URL NEXT_PUBLIC_BACKEND_URL; do
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
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";

const [rootInput, runtimeRootInput, contractInput] = process.argv.slice(1);
const root = realpathSync(rootInput);
const runtimeRoot = realpathSync(runtimeRootInput);
const fail = (message) => {
  console.error(`identity-fail-closed: ERROR: ${message}`);
  process.exit(1);
};
const parse = (path, label) => {
  try { return JSON.parse(readFileSync(path, "utf8")); }
  catch (error) { fail(`invalid ${label} JSON: ${error.message}`); }
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) fail(`unsafe path for ${label}`);
  const pieces = value.split("/");
  if (pieces.some((piece) => !piece || piece === "." || piece === "..")) fail(`unsafe path for ${label}`);
  const candidate = resolve(root, value);
  const rel = relative(root, candidate);
  if (rel.startsWith("..") || isAbsolute(rel)) fail(`unsafe path for ${label}`);
  return candidate;
};
const currentContent = (value, label) => {
  const candidate = safeRelative(value, label);
  if (!existsSync(candidate)) fail(`missing evidence file for ${label}`);
  const canonical = realpathSync(candidate);
  const rel = relative(root, canonical);
  if (rel.startsWith("..") || isAbsolute(rel)) fail(`evidence escapes repository for ${label}`);
  return readFileSync(canonical, "utf8");
};
const hash = (content) => new Bun.CryptoHasher("sha256").update(content).digest("hex");
const strings = (value, label) => {
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || !item)) fail(`${label} must contain non-empty strings`);
  return value;
};
const anchored = (content, anchors, label) => {
  for (const anchor of strings(anchors, `${label}.anchors`)) {
    if (anchor.length < 4 || !content.includes(anchor)) fail(`missing ${label} anchor: ${anchor}`);
  }
};
const normalizeRust = (content) => content.replace(/\s+/g, " ").trim();

const contract = parse(contractInput, "identity fail-closed contract");
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-3-identity-fail-closed-runtime") fail("unexpected schemaVersion or artifact");
if (contract.contractId !== "A2.3i-identity-direct-service-fail-closed-runtime") fail("unexpected contractId");
if (contract.purpose !== "hermetic-fail-closed-runtime-evidence") fail("unexpected purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || Object.keys(contract.safety).length !== 9 || Object.values(contract.safety).some((value) => value !== false)) fail("all nine execution safety flags must remain false");

const authority = contract.authority;
if (!authority || authority.contract !== "docs/migration/contracts/a2-3-identity-authorization.json" || authority.contractId !== "A2.3h-identity-direct-service-authorization-audit") fail("invalid A2.3h authority");
if (!/^[0-9a-f]{64}$/.test(authority.sha256)) fail("invalid authority digest");
if (typeof authority.interpretation !== "string" || !authority.interpretation.includes("does not authorize identity lifecycle or persistence behavior")) fail("authority interpretation must remain fail-closed");
const authorityContent = currentContent(authority.contract, "A2.3h authority");
if (hash(authorityContent) !== authority.sha256) fail("A2.3h authority digest drifted");
const audit = JSON.parse(authorityContent);
if (audit.contractId !== authority.contractId || audit.productionReady !== false || !Array.isArray(audit.routes) || audit.routes.length !== 11) fail("A2.3h authority is not the expected STOP contract");

if (!Array.isArray(contract.runtimeEvidence) || contract.runtimeEvidence.length !== 3) fail("exactly three runtime evidence records are required");
const evidence = new Map();
for (const item of contract.runtimeEvidence) {
  if (!item || typeof item.file !== "string" || evidence.has(item.file)) fail(`invalid or duplicate runtime evidence: ${item?.file}`);
  const candidate = safeRelative(item.file, item.file);
  const runtimeCandidate = resolve(runtimeRoot, relative(root, candidate));
  const runtimeRelative = relative(runtimeRoot, runtimeCandidate);
  if (runtimeRelative.startsWith("..") || isAbsolute(runtimeRelative) || !existsSync(runtimeCandidate)) fail(`missing runtime evidence file for ${item.file}`);
  const runtimeCanonical = realpathSync(runtimeCandidate);
  const runtimeCanonicalRelative = relative(runtimeRoot, runtimeCanonical);
  if (runtimeCanonicalRelative.startsWith("..") || isAbsolute(runtimeCanonicalRelative)) fail(`runtime evidence escapes runtime root for ${item.file}`);
  const content = readFileSync(runtimeCanonical, "utf8");
  anchored(content, item.anchors, item.file);
  evidence.set(item.file, content);
}
for (const file of ["services/identity/src/lib.rs", "services/identity/src/main.rs", "services/identity/Cargo.toml"]) if (!evidence.has(file)) fail(`missing runtime evidence ${file}`);

const lib = evidence.get("services/identity/src/lib.rs");
const main = evidence.get("services/identity/src/main.rs");
const cargo = evidence.get("services/identity/Cargo.toml");

const topology = contract.sourceTopology;
if (!topology || !Array.isArray(topology.runtimePins) || topology.runtimePins.length !== 3) fail("exactly three runtime topology pins are required");
const expectedPins = new Map([
  ["services/identity/src/lib.rs", "#[cfg(test)]"],
  ["services/identity/src/main.rs", null],
  ["services/identity/Cargo.toml", null],
]);
const productionEvidence = new Map();
for (const pin of topology.runtimePins) {
  if (!pin || !expectedPins.has(pin.file) || productionEvidence.has(pin.file) || pin.endMarker !== expectedPins.get(pin.file) || !/^[0-9a-f]{64}$/.test(pin.sha256) || !Number.isInteger(pin.bytes) || pin.bytes <= 0) fail(`invalid runtime topology pin ${pin?.file}`);
  const content = evidence.get(pin.file);
  let productionContent = content;
  if (pin.endMarker !== null) {
    const marker = content.indexOf(pin.endMarker);
    if (marker < 0 || content.indexOf(pin.endMarker, marker + pin.endMarker.length) >= 0) fail(`${pin.file} production topology marker drifted`);
    productionContent = content.slice(0, marker);
  }
  if (Buffer.byteLength(productionContent, "utf8") !== pin.bytes) fail(`${pin.file} production topology byte length drifted`);
  if (hash(productionContent) !== pin.sha256) fail(`${pin.file} production topology digest drifted`);
  productionEvidence.set(pin.file, productionContent);
}
if (productionEvidence.size !== 3) fail("runtime topology pin set drifted");

const routerSurface = productionEvidence.get("services/identity/src/lib.rs");
const actualRouterPaths = [...routerSurface.matchAll(/\.route\s*\(\s*"([^"]+)"/g)].map((match) => match[1]);
const expectedRouterPaths = [
  "/health",
  "/api/v1/identity/auth/challenge",
  "/api/v1/identity/auth/siwe",
  "/api/v1/identity/auth/refresh",
  "/api/v1/identity/auth/me",
  "/api/v1/identity/auth/demo",
  "/api/v1/identity/users",
  "/api/v1/identity/users/{id}",
];
if (JSON.stringify(topology.routerPaths) !== JSON.stringify(expectedRouterPaths)) fail("contract router path inventory drifted");
if (JSON.stringify(actualRouterPaths) !== JSON.stringify(expectedRouterPaths)) fail("runtime router path inventory drifted");
const normalizedRouter = normalizeRust(routerSurface);
for (const fragment of strings(topology.routerFragments, "sourceTopology.routerFragments")) {
  if (!normalizedRouter.includes(fragment)) fail(`missing exact router fragment: ${fragment}`);
}
if (topology.routerFragments.length !== 10) fail("exactly ten router fragments are required");
for (const forbidden of [".merge(", ".nest(", ".nest_service(", ".route_service(", ".fallback_service(", ".without_v07_checks("]) {
  if (normalizedRouter.includes(forbidden)) fail(`unapproved router composition remains: ${forbidden}`);
}
if (topology.anonymousPolicy !== "(&Method::GET | &Method::HEAD, \"/health\") => AccessPolicy::Public") fail("contract anonymous policy inventory drifted");
if (!normalizedRouter.includes(topology.anonymousPolicy)) fail("runtime anonymous policy inventory drifted");
if ((normalizedRouter.match(/=> AccessPolicy::Public/g) ?? []).length !== 1) fail("runtime has an unapproved anonymous classifier policy");
for (const forbidden of ["CREATE TABLE", "EPSX_BOOTSTRAP_ADMIN", "EPSX_ENABLE_DEMO_LOGIN", "JwtService", "jwt_secret", "database_url", "redis_url", "sqlx::", "deadpool", "SiweVerifier::", "generate_tokens(", "verify_token("]) {
  if (main.includes(forbidden)) fail(`unsafe startup behavior remains: ${forbidden}`);
}
for (const forbidden of ["CREATE TABLE", "ALTER TABLE", "DROP TABLE", "sqlx::", "deadpool", "SiweVerifier::", "generate_tokens(", "verify_token("]) {
  if (lib.includes(forbidden)) fail(`unsafe identity library behavior remains: ${forbidden}`);
}
for (const forbidden of ["epsx-crypto", "sqlx.workspace", "redis.workspace", "deadpool-redis"]) {
  if (cargo.includes(forbidden)) fail(`disabled candidate dependency remains: ${forbidden}`);
}
for (const required of [
  ".route(\"/health\", get(health))",
  ".route(\"/api/v1/identity/auth/challenge\", post(not_found))",
  ".route(\"/api/v1/identity/auth/siwe\", post(not_found))",
  ".route(\"/api/v1/identity/auth/refresh\", post(not_found))",
  ".route(\"/api/v1/identity/auth/me\", get(not_found))",
  ".route(\"/api/v1/identity/auth/demo\", post(not_found))",
  "(&Method::GET, \"/api/v1/identity/users\")",
  "(&Method::POST, \"/api/v1/identity/users\")",
  "AccessPolicy::AdminPermission(USERS_READ_PERMISSION)",
  "AccessPolicy::AdminPermission(USERS_CREATE_PERMISSION)",
  "AccessPolicy::AdminPermission(USERS_UPDATE_PERMISSION)",
  "AccessPolicy::AdminPermission(USERS_DELETE_PERMISSION)",
]) if (!lib.includes(required)) fail(`missing strict runtime classifier anchor: ${required}`);

const expected = {
  "identity.get.health": [["GET", "HEAD"], "/health", "public-strip-authorization-and-spoofable-identity-headers", "aligned", "none-public", null],
  "identity.post.auth-challenge": [["POST"], "/api/v1/identity/auth/challenge", "404-before-auth-body-redis-or-handler", "blocked", "none-disabled", null],
  "identity.post.auth-siwe": [["POST"], "/api/v1/identity/auth/siwe", "404-before-auth-body-signature-redis-sql-or-handler", "blocked", "none-disabled", null],
  "identity.post.auth-refresh": [["POST"], "/api/v1/identity/auth/refresh", "404-before-auth-body-credential-store-or-handler", "blocked", "none-disabled", null],
  "identity.get.auth-me": [["GET"], "/api/v1/identity/auth/me", "canonical-access-token-frontend-or-admin-then-404-before-identity-store", "blocked", "epsx-frontend-or-epsx-admin-exactly-one", null],
  "identity.post.auth-demo": [["POST"], "/api/v1/identity/auth/demo", "404-before-auth-body-token-sql-or-handler", "blocked", "none-disabled", null],
  "identity.get.users": [["GET"], "/api/v1/identity/users", "admin-literal-permission-then-404-before-store", "blocked", "epsx-admin", "admin:users:read"],
  "identity.post.users": [["POST"], "/api/v1/identity/users", "admin-literal-permission-then-404-before-body-or-store", "blocked", "epsx-admin", "admin:users:create"],
  "identity.get.user": [["GET"], "/api/v1/identity/users/{id}", "admin-literal-permission-then-404-before-selector-or-store", "blocked", "epsx-admin", "admin:users:read"],
  "identity.put.user": [["PUT"], "/api/v1/identity/users/{id}", "admin-literal-permission-then-404-before-selector-body-or-store", "blocked", "epsx-admin", "admin:users:update"],
  "identity.delete.user": [["DELETE"], "/api/v1/identity/users/{id}", "admin-literal-permission-then-404-before-selector-or-store", "blocked", "epsx-admin", "admin:users:delete"],
};
if (!Array.isArray(contract.routes) || contract.routes.length !== 11) fail("exactly eleven route records are required");
const seen = new Set();
let functionalityAligned = 0;
let functionalityBlocked = 0;
const auditById = new Map(audit.routes.map((route) => [route.id, route]));
for (const route of contract.routes) {
  if (!route || seen.has(route.id) || !Object.hasOwn(expected, route.id)) fail(`invalid or duplicate route ${route?.id}`);
  seen.add(route.id);
  const [methods, path, boundary, functionality, audience, permission] = expected[route.id];
  if (JSON.stringify(route.methods) !== JSON.stringify(methods) || route.path !== path || route.boundary !== boundary || route.functionality !== functionality || route.audience !== audience || route.permission !== permission) fail(`${route.id}: runtime disposition drifted`);
  const audited = auditById.get(route.id);
  if (!audited || audited.path !== path || audited.requiredPermission !== permission || !methods.includes(audited.method)) fail(`${route.id}: differs from A2.3h route authority`);
  if (functionality === "aligned") functionalityAligned += 1;
  else functionalityBlocked += 1;
}
if (seen.size !== 11 || auditById.size !== 11) fail("route set differs from A2.3h authority");
if (contract.statusCounts?.boundaryAligned !== 11 || contract.statusCounts?.functionalityAligned !== functionalityAligned || contract.statusCounts?.functionalityBlocked !== functionalityBlocked || functionalityAligned !== 1 || functionalityBlocked !== 10) fail("status counts must remain 11 boundary aligned, 1 functionality aligned, 10 functionality blocked");

if (strings(contract.invariants, "invariants").length !== 10) fail("exactly ten runtime invariants are required");
if (strings(contract.removedUnsafeStartup, "removedUnsafeStartup").length !== 5) fail("exactly five removed unsafe startup behaviors are required");
if (strings(contract.residualStopBlockers, "residualStopBlockers").length !== 12) fail("exactly twelve residual STOP blockers are required");

process.stdout.write(JSON.stringify({
  schemaVersion: 1,
  contractId: contract.contractId,
  authority: { contractId: authority.contractId, sha256: authority.sha256 },
  runtimePins: topology.runtimePins.length,
  routerPaths: actualRouterPaths.length,
  routes: contract.routes.length,
  boundaryAligned: contract.statusCounts.boundaryAligned,
  functionalityAligned,
  functionalityBlocked,
  invariants: contract.invariants.length,
  removedUnsafeStartup: contract.removedUnsafeStartup.length,
  stopBlockers: contract.residualStopBlockers.length,
  productionReady: contract.productionReady,
  readinessExit: contract.readinessExit,
}));
' "$repo_root" "$runtime_root" "$contract")

case "$mode" in
  integrity)
    echo "identity fail-closed integrity: PASS (3 exact runtime pins and 8 router paths; 11 boundaries aligned; functionality 1 aligned, 10 blocked; 12 STOP blockers; no database, Redis, JWKS, service, migration, or deployment executed)"
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    echo "identity fail-closed readiness: STOP (10 functional routes blocked; 12 residual STOP blockers; productionReady=false)" >&2
    exit 3
    ;;
esac
