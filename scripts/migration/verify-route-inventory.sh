#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
CONTRACT="$REPO_ROOT/docs/migration/contracts/routes.json"
SOURCE_REF="origin/development"

command -v bun >/dev/null 2>&1 || { echo "route-inventory: ERROR: bun is required" >&2; exit 1; }
git -C "$REPO_ROOT" cat-file -e "$SOURCE_REF^{commit}" 2>/dev/null || {
  echo "route-inventory: ERROR: cannot resolve $SOURCE_REF to a commit" >&2
  exit 1
}

SOURCE_COMMIT="$(git -C "$REPO_ROOT" rev-parse --verify "$SOURCE_REF^{commit}")"

bun -e '
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const [root, contractPath, sourceRef, sourceCommit] = process.argv.slice(1);
const fail = (message) => { console.error(`route-inventory: ERROR: ${message}`); process.exit(1); };
let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid JSON: ${error.message}`); }

if (contract.schemaVersion !== 1) fail("schemaVersion must be 1");
if (contract.sourceRef !== sourceRef) fail(`sourceRef must be ${sourceRef}`);
if (!contract.applications || typeof contract.applications !== "object") fail("applications object is required");

const routeFromPage = (sourceRoot, file) => {
  const prefix = `${sourceRoot}/`;
  if (!file.startsWith(prefix) || !/\/page\.(?:js|jsx|ts|tsx)$/.test(file)) return null;
  const relative = file.slice(prefix.length).replace(/\/page\.(?:js|jsx|ts|tsx)$/, "");
  const segments = relative === "page.tsx" ? [] : relative.split("/")
    .filter((segment) => !(segment.startsWith("(") && segment.endsWith(")")))
    .filter((segment) => !segment.startsWith("@"));
  const urlSegments = segments.map((segment) => {
    let match = segment.match(/^\[([^.[\]]+)\]$/);
    if (match) return `:${match[1]}`;
    match = segment.match(/^\[\.\.\.([^\]]+)\]$/);
    if (match) return `*${match[1]}`;
    match = segment.match(/^\[\[\.\.\.([^\]]+)\]\]$/);
    if (match) return `*${match[1]}?`;
    return segment;
  });
  return urlSegments.length ? `/${urlSegments.join("/")}` : "/";
};

const allTargets = [];
let sourceTotal = 0;
for (const [appName, app] of Object.entries(contract.applications)) {
  if (!app || typeof app !== "object") fail(`${appName}: app contract must be an object`);
  if (typeof app.sourceRoot !== "string" || typeof app.dispatcher !== "string") fail(`${appName}: sourceRoot and dispatcher are required`);
  if (!Number.isInteger(app.expectedCount) || !Array.isArray(app.routes) || !Array.isArray(app.targetOnly)) fail(`${appName}: expectedCount, routes, and targetOnly are required`);
  if (app.routes.length !== app.expectedCount) fail(`${appName}: contract has ${app.routes.length} routes; expected ${app.expectedCount}`);

  const paths = new Set();
  const sourcePages = new Set();
  for (const route of app.routes) {
    if (!route || typeof route.path !== "string" || !route.path.startsWith("/")) fail(`${appName}: invalid route path`);
    if (paths.has(route.path)) fail(`${appName}: duplicate route path ${route.path}`);
    paths.add(route.path);
    if (typeof route.sourcePage !== "string" || sourcePages.has(route.sourcePage)) fail(`${appName}: missing or duplicate sourcePage for ${route.path}`);
    sourcePages.add(route.sourcePage);
    allTargets.push({ appName, path: route.path, target: route.target });
  }
  for (const extra of app.targetOnly) {
    if (!extra || typeof extra.path !== "string" || !extra.path.startsWith("/")) fail(`${appName}: invalid target-only path`);
    if (paths.has(extra.path)) fail(`${appName}: target-only path overlaps source route ${extra.path}`);
    paths.add(extra.path);
    allTargets.push({ appName, path: extra.path, target: extra.target });
  }

  const tree = spawnSync("git", ["-C", root, "ls-tree", "-r", "--name-only", sourceCommit, "--", app.sourceRoot], { encoding: "utf8" });
  if (tree.status !== 0) fail(`${appName}: git ls-tree failed: ${tree.stderr.trim()}`);
  const derived = tree.stdout.split(/\r?\n/).filter(Boolean)
    .filter((file) => /\/page\.(?:js|jsx|ts|tsx)$/.test(file))
    .map((file) => ({ path: routeFromPage(app.sourceRoot, file), sourcePage: file }));
  if (derived.some((item) => item.path === null)) fail(`${appName}: failed to derive a source route`);
  if (derived.length !== app.expectedCount) fail(`${appName}: ${sourceRef} has ${derived.length} page routes; expected ${app.expectedCount}`);
  const expectedPairs = new Set(app.routes.map((route) => `${route.path}\u0000${route.sourcePage}`));
  const derivedPairs = new Set(derived.map((route) => `${route.path}\u0000${route.sourcePage}`));
  const missing = [...derivedPairs].filter((pair) => !expectedPairs.has(pair));
  const stale = [...expectedPairs].filter((pair) => !derivedPairs.has(pair));
  if (missing.length || stale.length) fail(`${appName}: source inventory mismatch (missing=${missing.length}, stale=${stale.length})`);
  sourceTotal += derived.length;
  console.log(`route-inventory: ${appName} source pages ${derived.length}/${app.expectedCount}`);
}

if (sourceTotal !== 55) fail(`source route total is ${sourceTotal}; expected 55`);
if (!Array.isArray(contract.intentionalAdminRedirects) || contract.intentionalAdminRedirects.length !== 2) fail("intentionalAdminRedirects must contain exactly 2 entries");
const adminRoutes = contract.applications.admin.routes;
for (const redirect of contract.intentionalAdminRedirects) {
  const route = adminRoutes.find((candidate) => candidate.path === redirect.path);
  if (!route || route.target.kind !== "redirect" || route.target.redirectTo !== redirect.to || route.target.handler !== redirect.handler) {
    fail(`intentional admin redirect contract mismatch for ${redirect.path}`);
  }
}

const contentCache = new Map();
const verifyBlock = (appName, routePath, block) => {
  if (!block || typeof block.file !== "string" || !Array.isArray(block.evidence) || block.evidence.length === 0) fail(`${appName} ${routePath}: invalid target evidence block`);
  if (block.file.startsWith("/") || block.file.split("/").includes("..")) fail(`${appName} ${routePath}: target file must be repository-relative`);
  const absolute = resolve(root, block.file);
  let content = contentCache.get(absolute);
  if (content === undefined) {
    try { content = readFileSync(absolute, "utf8"); }
    catch { fail(`${appName} ${routePath}: target file does not exist: ${block.file}`); }
    contentCache.set(absolute, content);
  }
  for (const needle of block.evidence) if (typeof needle !== "string" || !content.includes(needle)) fail(`${appName} ${routePath}: missing evidence in ${block.file}: ${JSON.stringify(needle)}`);
};

for (const { appName, path, target } of allTargets) {
  if (!target || typeof target.handler !== "string" || !["static", "dynamic", "redirect", "fallback"].includes(target.kind)) fail(`${appName} ${path}: invalid target mapping`);
  if (target.kind === "dynamic" && !path.includes(":")) fail(`${appName} ${path}: dynamic mapping needs a dynamic template`);
  if (target.kind === "redirect" && (typeof target.redirectTo !== "string" || !target.redirectTo.startsWith("/"))) fail(`${appName} ${path}: redirectTo is required`);
  verifyBlock(appName, path, target);
  for (const extra of target.additionalEvidence ?? []) verifyBlock(appName, path, { file: extra.file, evidence: extra.contains });
}

console.log(`route-inventory: source ref ${sourceRef} -> ${sourceCommit}`);
console.log(`route-inventory: target evidence ${allTargets.length}/${allTargets.length}`);
console.log("route-inventory: intentional admin redirects 2/2");
console.log("route-inventory: OK");
' -- "$REPO_ROOT" "$CONTRACT" "$SOURCE_REF" "$SOURCE_COMMIT"
