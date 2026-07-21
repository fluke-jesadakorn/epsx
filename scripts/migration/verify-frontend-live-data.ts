import { existsSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";

type Json = Record<string, any>;

const die = (message: string): never => {
  console.error(`frontend-live-data: ERROR: ${message}`);
  process.exit(1);
};

const args = process.argv.slice(2);
let mode = "";
let fixtureInput = "docs/migration/contracts/frontend-live-data.json";
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
    die(`unsupported or incomplete argument: ${arg}`);
  }
}
if (!new Set(["integrity", "readiness", "emit"]).has(mode)) {
  die("--mode must be integrity, readiness, or emit");
}

const root = realpathSync(rootInput);
const fixturePath = isAbsolute(fixtureInput) ? fixtureInput : resolve(root, fixtureInput);
if (!existsSync(fixturePath)) die(`missing fixture: ${fixturePath}`);

const parse = (path: string, label: string): Json => {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    die(`cannot parse ${label}: ${String(error)}`);
  }
};
const contract = parse(fixturePath, "frontend live-data fixture");

const git = (...gitArgs: string[]): string => {
  const result = Bun.spawnSync(["git", ...gitArgs], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" },
  });
  if (result.exitCode !== 0) {
    die(`git ${gitArgs.join(" ")} failed: ${result.stderr.toString().trim()}`);
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
    die(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const pieces = value.split("/");
  if (pieces.some((piece) => piece === "" || piece === "." || piece === "..")) {
    die(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const candidate = resolve(root, value);
  const rel = relative(root, candidate);
  if (rel.startsWith("..") || isAbsolute(rel)) {
    die(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  return value;
};

const currentFile = (value: unknown, label: string): string => {
  const rel = safeRelative(value, label);
  const absolute = resolve(root, rel);
  if (!existsSync(absolute)) die(`missing target evidence file for ${label}: ${rel}`);
  const canonical = realpathSync(absolute);
  const relCanonical = relative(root, canonical);
  if (relCanonical.startsWith("..") || isAbsolute(relCanonical)) {
    die(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  return readFileSync(canonical, "utf8");
};

const stringArray = (value: unknown, label: string): string[] => {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string" || entry.length === 0)) {
    die(`${label} must be an array of non-empty strings`);
  }
  return value;
};

if (contract.schemaVersion !== 1 || contract.artifact !== "a7-frontend-live-data") {
  die("unexpected schemaVersion or artifact");
}
if (contract.source?.ref !== "origin/development") die("source ref must be origin/development");
if (!/^[0-9a-f]{40}$/.test(contract.source?.commit ?? "")) die("source commit must be a full SHA-1");
const resolvedSource = git("rev-parse", contract.source.ref);
if (resolvedSource !== contract.source.commit) {
  die(`stale source ref/commit: ${contract.source.ref} resolves to ${resolvedSource}, fixture pins ${contract.source.commit}`);
}
git("cat-file", "-e", `${contract.source.commit}^{commit}`);

const inventoryRel = safeRelative(contract.routeInventory, "routeInventory");
const inventory = parse(resolve(root, inventoryRel), "route inventory");
const expected = inventory.applications?.frontend?.routes;
if (!Array.isArray(expected) || inventory.applications.frontend.expectedCount !== 28) {
  die("route inventory frontend application must contain the checked 28-route contract");
}
if (contract.expectedRouteCount !== 28 || !Array.isArray(contract.routes) || contract.routes.length !== 28) {
  die("frontend live-data fixture must contain exactly 28 routes");
}

const dependencyKeys = Object.keys(contract.dependencyDefinitions ?? {}).sort();
if (JSON.stringify(dependencyKeys) !== JSON.stringify(["A1", "A4", "A5", "A6"])) {
  die("dependencyDefinitions must be exactly A1, A4, A5, and A6");
}
const stateValues = stringArray(contract.stateValues, "stateValues");
if (JSON.stringify([...stateValues].sort()) !== JSON.stringify(["missing", "not-applicable", "present"])) {
  die("stateValues must be present, missing, and not-applicable");
}

if (!Array.isArray(contract.batches) || contract.batches.length !== 7) die("exactly seven executable batches are required");
const batchIds = new Set<string>();
const batchMembership = new Map<string, string>();
for (const [batchIndex, batch] of contract.batches.entries()) {
  if (typeof batch.id !== "string" || !/^B[1-7]-[a-z-]+$/.test(batch.id) || batchIds.has(batch.id)) {
    die(`invalid or duplicate batch id at batches[${batchIndex}]`);
  }
  batchIds.add(batch.id);
  const members = stringArray(batch.routes, `batches[${batchIndex}].routes`);
  if (members.length !== 4) die(`${batch.id} must contain exactly four routes`);
  for (const path of members) {
    if (batchMembership.has(path)) die(`route ${path} appears in more than one batch`);
    batchMembership.set(path, batch.id);
  }
}

const expectedByPath = new Map(expected.map((route: Json) => [route.path, route]));
const seen = new Set<string>();
const statuses = { aligned: 0, partial: 0, blocked: 0 };
const interactionKeys = ["forms", "pagination", "search", "wallet", "keyboard", "controls"];
const stateKeys = ["loading", "empty", "error", "retry"];
const hydrationNeeds = new Set(["none", "browser"]);
const hydrationStatuses = new Set(["not-applicable", "implemented", "partial", "missing"]);

for (const [routeIndex, route] of contract.routes.entries()) {
  const label = `routes[${routeIndex}]`;
  if (typeof route.path !== "string" || seen.has(route.path)) die(`${label}.path is missing or duplicated`);
  seen.add(route.path);
  const baseline = expectedByPath.get(route.path);
  if (!baseline) die(`${label}.path is not in the checked frontend route inventory: ${route.path}`);
  if (batchMembership.get(route.path) !== route.batch) die(`${label}.batch disagrees with batches membership`);
  if (!new Set(["aligned", "partial", "blocked"]).has(route.status)) die(`${label}.status is invalid`);
  statuses[route.status as keyof typeof statuses] += 1;

  const dependencies = stringArray(route.dependencies, `${label}.dependencies`);
  if (new Set(dependencies).size !== dependencies.length || dependencies.some((item) => !dependencyKeys.includes(item))) {
    die(`${label}.dependencies contains duplicates or an unsupported dependency`);
  }
  const blockers = stringArray(route.blockers, `${label}.blockers`);
  if (route.status === "aligned" && blockers.length !== 0) die(`${label} aligned route cannot have blockers`);
  if (route.status !== "aligned" && blockers.length === 0) die(`${label} non-aligned route must have a blocker`);

  if (route.source?.file !== baseline.sourcePage) die(`${label}.source.file disagrees with routes.json`);
  if (route.target?.handler !== baseline.target?.handler) die(`${label}.target.handler disagrees with routes.json`);
  const sourceFile = safeRelative(route.source.file, `${label}.source.file`);
  const sourceContent = git("show", `${contract.source.commit}:${sourceFile}`);
  for (const [anchorIndex, anchor] of stringArray(route.source.anchors, `${label}.source.anchors`).entries()) {
    if (!sourceContent.includes(anchor)) die(`missing source anchor ${label}.source.anchors[${anchorIndex}]: ${anchor}`);
  }
  const targetContent = currentFile(route.target?.file, `${label}.target.file`);
  for (const [anchorIndex, anchor] of stringArray(route.target.anchors, `${label}.target.anchors`).entries()) {
    if (!targetContent.includes(anchor)) die(`missing target anchor ${label}.target.anchors[${anchorIndex}]: ${anchor}`);
  }

  stringArray(route.payloads?.staticOrSample, `${label}.payloads.staticOrSample`);
  stringArray(route.payloads?.placeholderOrSkeleton, `${label}.payloads.placeholderOrSkeleton`);
  if (typeof route.loader?.kind !== "string" || route.loader.kind.length === 0) die(`${label}.loader.kind is required`);
  stringArray(route.loader.endpoints, `${label}.loader.endpoints`);
  if (!Array.isArray(route.loader.evidence)) die(`${label}.loader.evidence must be an array`);
  for (const [evidenceIndex, evidence] of route.loader.evidence.entries()) {
    const content = currentFile(evidence?.file, `${label}.loader.evidence[${evidenceIndex}].file`);
    if (typeof evidence?.anchor !== "string" || evidence.anchor.length === 0 || !content.includes(evidence.anchor)) {
      die(`missing target anchor ${label}.loader.evidence[${evidenceIndex}]: ${String(evidence?.anchor)}`);
    }
  }
  if (typeof route.authOwner?.auth !== "string" || typeof route.authOwner?.owner !== "string") {
    die(`${label}.authOwner must inventory auth and ownership`);
  }
  for (const key of interactionKeys) stringArray(route.interactions?.[key], `${label}.interactions.${key}`);
  for (const key of stateKeys) {
    if (!stateValues.includes(route.states?.[key])) die(`${label}.states.${key} is invalid`);
  }
  if (!hydrationNeeds.has(route.hydration?.need) || !hydrationStatuses.has(route.hydration?.status) || typeof route.hydration?.reason !== "string") {
    die(`${label}.hydration is invalid`);
  }
}

const expectedPaths = [...expectedByPath.keys()].sort();
const actualPaths = [...seen].sort();
if (JSON.stringify(expectedPaths) !== JSON.stringify(actualPaths)) die("28-route set differs from routes.json");
if (batchMembership.size !== 28 || [...batchMembership.keys()].some((path) => !seen.has(path))) {
  die("batch membership must cover the exact 28-route set");
}

const nonAligned = statuses.partial + statuses.blocked;
const emitted = {
  artifact: contract.artifact,
  baseline: { ref: contract.source.ref, commit: contract.source.commit },
  routeCount: contract.routes.length,
  statuses,
  productionReady: nonAligned === 0,
  readinessExit: nonAligned === 0 ? 0 : 3,
  batches: contract.batches.map((batch: Json) => ({ id: batch.id, routes: batch.routes })),
  routes: contract.routes.map((route: Json) => ({
    path: route.path,
    batch: route.batch,
    status: route.status,
    dependencies: route.dependencies,
    blockerCount: route.blockers.length,
  })),
};

if (mode === "emit") {
  process.stdout.write(`${JSON.stringify(emitted, null, 2)}\n`);
} else if (mode === "integrity") {
  console.log(`frontend-live-data: PASS integrity (28 routes; ${statuses.aligned} aligned, ${statuses.partial} partial, ${statuses.blocked} blocked; deterministic offline evidence only)`);
} else if (nonAligned > 0) {
  console.error(`frontend-live-data: STOP readiness (${nonAligned} non-aligned routes: ${statuses.partial} partial, ${statuses.blocked} blocked)`);
  process.exit(3);
} else {
  console.log("frontend-live-data: PASS readiness (all 28 routes aligned)");
}
