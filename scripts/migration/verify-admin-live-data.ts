import { existsSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";

type Json = Record<string, any>;

const fail = (message: string): never => {
  console.error(`admin-live-data: ERROR: ${message}`);
  process.exit(1);
};

const args = process.argv.slice(2);
let mode = "";
let fixtureInput = "docs/migration/contracts/admin-live-data.json";
let rootInput = process.cwd();
for (let index = 0; index < args.length; index += 1) {
  const arg = args[index];
  const value = args[index + 1];
  if (arg === "--mode" && value) {
    mode = value;
    index += 1;
  } else if ((arg === "--fixture" || arg === "--contract") && value) {
    fixtureInput = value;
    index += 1;
  } else if (arg === "--root" && value) {
    rootInput = value;
    index += 1;
  } else {
    fail(`unsupported or incomplete argument: ${arg}`);
  }
}
if (!new Set(["integrity", "readiness", "emit"]).has(mode)) {
  fail("--mode must be integrity, readiness, or emit");
}

const root = realpathSync(rootInput);
const fixturePath = isAbsolute(fixtureInput) ? fixtureInput : resolve(root, fixtureInput);
if (!existsSync(fixturePath)) fail(`missing fixture: ${fixturePath}`);

const parse = (path: string, label: string): Json => {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`cannot parse ${label}: ${String(error)}`);
  }
};
const contract = parse(fixturePath, "admin live-data fixture");

const git = (...gitArgs: string[]): string => {
  const result = Bun.spawnSync(["git", ...gitArgs], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" },
  });
  if (result.exitCode !== 0) {
    fail(`git ${gitArgs.join(" ")} failed: ${result.stderr.toString().trim()}`);
  }
  return result.stdout.toString().trim();
};

const safeRelative = (value: unknown, label: string): string => {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.includes("\0") ||
    value.includes("\\") ||
    isAbsolute(value)
  ) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const pieces = value.split("/");
  if (pieces.some((piece) => piece === "" || piece === "." || piece === "..")) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const candidate = resolve(root, value);
  const rel = relative(root, candidate);
  if (rel.startsWith("..") || isAbsolute(rel)) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  return value;
};

const currentFile = (value: unknown, label: string): string => {
  const rel = safeRelative(value, label);
  const absolute = resolve(root, rel);
  if (!existsSync(absolute)) fail(`missing target evidence file for ${label}: ${rel}`);
  const canonical = realpathSync(absolute);
  const relCanonical = relative(root, canonical);
  if (relCanonical.startsWith("..") || isAbsolute(relCanonical)) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  return readFileSync(canonical, "utf8");
};

const strings = (value: unknown, label: string): string[] => {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string" || entry.length === 0)) {
    fail(`${label} must be an array of non-empty strings`);
  }
  return value;
};

if (contract.schemaVersion !== 1 || contract.artifact !== "a8-admin-live-data") {
  fail("unexpected schemaVersion or artifact");
}
if (contract.contractId !== "A8.0-admin-live-data-and-mutation-readiness") {
  fail("unexpected contractId");
}
if (contract.source?.ref !== "origin/development") fail("source ref must be origin/development");
if (contract.source?.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") {
  fail("source commit must be the accepted full development baseline SHA");
}
const resolvedSource = git("rev-parse", contract.source.ref);
if (resolvedSource !== contract.source.commit) {
  fail(`stale source ref/commit: ${contract.source.ref} resolves to ${resolvedSource}, fixture pins ${contract.source.commit}`);
}
git("cat-file", "-e", `${contract.source.commit}^{commit}`);

const inventoryRel = safeRelative(contract.routeInventory, "routeInventory");
const inventory = parse(resolve(root, inventoryRel), "route inventory");
const adminInventory = inventory.applications?.admin;
const expected = adminInventory?.routes;
if (!Array.isArray(expected) || adminInventory.expectedCount !== 27) {
  fail("route inventory admin application must contain the checked 27-source-route contract");
}
if (contract.expectedRouteCount !== 27 || !Array.isArray(contract.routes) || contract.routes.length !== 27) {
  fail("admin live-data fixture must contain exactly 27 source routes");
}

const expectedDependencyKeys = ["A1", "A10", "A11", "A12", "A2", "A3", "A4", "A5", "A6", "A9"];
const dependencyKeys = Object.keys(contract.dependencyDefinitions ?? {}).sort();
if (JSON.stringify(dependencyKeys) !== JSON.stringify(expectedDependencyKeys)) {
  fail("dependencyDefinitions must be exactly A1-A6 and A9-A12");
}
const stateValues = strings(contract.stateValues, "stateValues");
if (JSON.stringify([...stateValues].sort()) !== JSON.stringify(["missing", "not-applicable", "present"])) {
  fail("stateValues must be present, missing, and not-applicable");
}

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length < 4) {
  fail("at least four cross-cutting target evidence records are required");
}
for (const [index, evidence] of contract.targetEvidence.entries()) {
  const content = currentFile(evidence?.file, `targetEvidence[${index}].file`);
  for (const [anchorIndex, anchor] of strings(evidence?.anchors, `targetEvidence[${index}].anchors`).entries()) {
    if (!content.includes(anchor)) {
      fail(`missing target anchor targetEvidence[${index}].anchors[${anchorIndex}]: ${anchor}`);
    }
  }
  if (typeof evidence?.finding !== "string" || evidence.finding.length === 0) {
    fail(`targetEvidence[${index}].finding is required`);
  }
}

if (!Array.isArray(contract.redirects) || contract.redirects.length !== 2) {
  fail("exactly two intentional target redirects are required");
}
const inventoryRedirects = expected
  .filter((route: Json) => route.target?.kind === "redirect")
  .map((route: Json) => `${route.path}\0${route.target.redirectTo}`)
  .sort();
const contractRedirects = contract.redirects
  .map((redirect: Json, index: number) => {
    if (redirect.status !== "partial" || typeof redirect.blocker !== "string" || redirect.blocker.length === 0) {
      fail(`redirects[${index}] must remain partial with a blocker until redirect behavior is proven`);
    }
    if (typeof redirect.transport !== "string" || typeof redirect.sourceSemantics !== "string") {
      fail(`redirects[${index}] must record transport and source semantics`);
    }
    return `${redirect.path}\0${redirect.target}`;
  })
  .sort();
if (JSON.stringify(inventoryRedirects) !== JSON.stringify(contractRedirects)) {
  fail("redirect set must equal the two redirect-classified routes in routes.json");
}

if (!Array.isArray(contract.batches) || contract.batches.length !== 7) {
  fail("exactly seven executable batches are required");
}
const batchIds = new Set<string>();
const batchMembership = new Map<string, string>();
for (const [index, batch] of contract.batches.entries()) {
  if (typeof batch.id !== "string" || !/^B[1-7]-[a-z-]+$/.test(batch.id) || batchIds.has(batch.id)) {
    fail(`invalid or duplicate batch id at batches[${index}]`);
  }
  batchIds.add(batch.id);
  const members = strings(batch.routes, `batches[${index}].routes`);
  if (members.length < 3 || members.length > 4) fail(`${batch.id} must contain three or four routes`);
  for (const path of members) {
    if (batchMembership.has(path)) fail(`route ${path} appears in more than one batch`);
    batchMembership.set(path, batch.id);
  }
}

if (!Array.isArray(contract.stopBlockers) || contract.stopBlockers.length !== 20) {
  fail("exactly 20 cross-cutting STOP blockers are required");
}
strings(contract.stopBlockers, "stopBlockers");

const expectedByPath = new Map(expected.map((route: Json) => [route.path, route]));
const seen = new Set<string>();
const statuses = { aligned: 0, partial: 0, blocked: 0 };
const validStatuses = new Set(["aligned", "partial", "blocked"]);
const stateKeys = ["loading", "empty", "error", "retry", "forbidden", "conflict"];

for (const [routeIndex, route] of contract.routes.entries()) {
  const label = `routes[${routeIndex}]`;
  if (typeof route.path !== "string" || seen.has(route.path)) fail(`${label}.path is missing or duplicated`);
  seen.add(route.path);
  const baseline = expectedByPath.get(route.path);
  if (!baseline) fail(`${label}.path is not in the checked admin route inventory: ${route.path}`);
  if (batchMembership.get(route.path) !== route.batch) fail(`${label}.batch disagrees with batch membership`);
  if (!validStatuses.has(route.status)) fail(`${label}.status is invalid`);
  statuses[route.status as keyof typeof statuses] += 1;

  const dependencies = strings(route.dependencies, `${label}.dependencies`);
  if (new Set(dependencies).size !== dependencies.length || dependencies.some((item) => !dependencyKeys.includes(item))) {
    fail(`${label}.dependencies contains duplicates or an unsupported dependency`);
  }
  const blockers = strings(route.blockers, `${label}.blockers`);
  if (route.status === "aligned" && blockers.length !== 0) fail(`${label} aligned route cannot have blockers`);
  if (route.status !== "aligned" && blockers.length === 0) fail(`${label} non-aligned route must have a blocker`);

  if (route.source?.file !== baseline.sourcePage) fail(`${label}.source.file disagrees with routes.json`);
  if (route.target?.handler !== baseline.target?.handler) fail(`${label}.target.handler disagrees with routes.json`);
  const sourceFile = safeRelative(route.source.file, `${label}.source.file`);
  const sourceContent = git("show", `${contract.source.commit}:${sourceFile}`);
  for (const [anchorIndex, anchor] of strings(route.source.anchors, `${label}.source.anchors`).entries()) {
    if (!sourceContent.includes(anchor)) fail(`missing source anchor ${label}.source.anchors[${anchorIndex}]: ${anchor}`);
  }
  const targetContent = currentFile(route.target?.file, `${label}.target.file`);
  for (const [anchorIndex, anchor] of strings(route.target.anchors, `${label}.target.anchors`).entries()) {
    if (!targetContent.includes(anchor)) fail(`missing target anchor ${label}.target.anchors[${anchorIndex}]: ${anchor}`);
  }

  const paramNames = strings(route.params?.names, `${label}.params.names`);
  if (new Set(paramNames).size !== paramNames.length || typeof route.params?.current !== "string") {
    fail(`${label}.params is invalid`);
  }
  if (baseline.target?.kind === "dynamic" && paramNames.length === 0) {
    fail(`${label} dynamic route must inventory at least one parameter`);
  }
  strings(route.data?.reads, `${label}.data.reads`);
  strings(route.data?.endpoints, `${label}.data.endpoints`);
  strings(route.data?.fallbacks, `${label}.data.fallbacks`);
  if (typeof route.data?.current !== "string" || route.data.current.length === 0) fail(`${label}.data.current is required`);

  strings(route.permissions?.read, `${label}.permissions.read`);
  strings(route.permissions?.manage, `${label}.permissions.manage`);
  if (typeof route.permissions?.current !== "string" || route.permissions.current.length === 0) {
    fail(`${label}.permissions.current is required`);
  }
  if (!Array.isArray(route.mutations)) fail(`${label}.mutations must be an array`);
  for (const [mutationIndex, mutation] of route.mutations.entries()) {
    for (const key of ["name", "method", "endpoint", "current"]) {
      if (typeof mutation?.[key] !== "string" || mutation[key].length === 0) {
        fail(`${label}.mutations[${mutationIndex}].${key} is required`);
      }
    }
  }
  strings(route.contracts?.bodies, `${label}.contracts.bodies`);
  strings(route.contracts?.envelopes, `${label}.contracts.envelopes`);
  strings(route.contracts?.statuses, `${label}.contracts.statuses`);
  for (const key of stateKeys) {
    if (!stateValues.includes(route.states?.[key])) fail(`${label}.states.${key} is invalid`);
  }
  for (const key of ["keyboard", "responsive", "hydration"]) {
    if (typeof route.ux?.[key] !== "string" || route.ux[key].length === 0) fail(`${label}.ux.${key} is required`);
  }
}

const expectedPaths = [...expectedByPath.keys()].sort();
const actualPaths = [...seen].sort();
if (JSON.stringify(expectedPaths) !== JSON.stringify(actualPaths)) fail("27-route set differs from routes.json");
if (batchMembership.size !== 27 || [...batchMembership.keys()].some((path) => !seen.has(path))) {
  fail("batch membership must cover the exact 27-source-route set");
}
if (statuses.aligned !== 2 || statuses.partial !== 2 || statuses.blocked !== 23) {
  fail("baseline status count must remain conservative at 2 aligned, 2 partial, and 23 blocked until evidence is updated deliberately");
}

const nonAligned = statuses.partial + statuses.blocked;
const emitted = {
  artifact: contract.artifact,
  contractId: contract.contractId,
  baseline: { ref: contract.source.ref, commit: contract.source.commit },
  routeCount: contract.routes.length,
  redirectCount: contract.redirects.length,
  stopBlockerCount: contract.stopBlockers.length,
  statuses,
  productionReady: nonAligned === 0 && contract.stopBlockers.length === 0,
  readinessExit: nonAligned === 0 && contract.stopBlockers.length === 0 ? 0 : 3,
  batches: contract.batches.map((batch: Json) => ({ id: batch.id, routes: batch.routes })),
  redirects: contract.redirects.map((redirect: Json) => ({
    path: redirect.path,
    target: redirect.target,
    transport: redirect.transport,
    currentStatus: redirect.currentStatus,
    status: redirect.status,
  })),
  routes: contract.routes.map((route: Json) => ({
    path: route.path,
    batch: route.batch,
    status: route.status,
    dependencies: route.dependencies,
    blockerCount: route.blockers.length,
    mutationCount: route.mutations.length,
  })),
};

if (mode === "emit") {
  process.stdout.write(`${JSON.stringify(emitted, null, 2)}\n`);
} else if (mode === "integrity") {
  console.log(`admin-live-data: PASS integrity (27 source routes; 2 redirects; ${statuses.aligned} aligned, ${statuses.partial} partial, ${statuses.blocked} blocked; 20 STOP blockers; deterministic offline evidence only)`);
} else if (!emitted.productionReady) {
  console.error(`admin-live-data: STOP readiness (${nonAligned} non-aligned routes: ${statuses.partial} partial, ${statuses.blocked} blocked; ${contract.stopBlockers.length} cross-cutting blockers)`);
  process.exit(3);
} else {
  console.log("admin-live-data: PASS readiness (all 27 source routes aligned and cross-cutting blockers cleared)");
}
