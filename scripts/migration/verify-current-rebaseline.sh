#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
CONTRACT="$ROOT/docs/migration/contracts/current-rebaseline.json"

command -v bun >/dev/null 2>&1 || { echo "current-rebaseline: bun is required" >&2; exit 1; }
command -v git >/dev/null 2>&1 || { echo "current-rebaseline: git is required" >&2; exit 1; }

bun -e '
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
const [root, contractPath] = process.argv.slice(1);
const fail = (message) => { console.error(`current-rebaseline: ERROR: ${message}`); process.exit(1); };
const git = (...args) => {
  const result = Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const readJson = (path) => { try { return JSON.parse(readFileSync(resolve(root, path), "utf8")); } catch (error) { fail(`invalid or missing JSON ${path}: ${error.message}`); } };
let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); } catch (error) { fail(`invalid contract: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.artifact !== "current-migration-rebaseline") fail("unexpected contract identity");
if (contract.productionReady !== false) fail("productionReady must remain false");
if (git("rev-parse", "development") !== contract.source.commit) fail("development source commit drifted");
if (git("rev-parse", contract.targetBase.commit + "^{commit}") !== contract.targetBase.commit) fail("target base commit is missing");
const targetBase = contract.targetBase.commit;
const targetAncestry = Bun.spawnSync(["git", "-C", root, "merge-base", "--is-ancestor", targetBase, "HEAD"], { stdout: "pipe", stderr: "pipe" });
if (targetAncestry.exitCode !== 0) fail(`current HEAD is not based on target commit ${targetBase}`);
const routes = readJson(contract.routeInventory.contract);
if (routes.sourceRef !== "development") fail("route inventory sourceRef must be development");
for (const [name, count, rootKey] of [["frontend", contract.routeInventory.frontendCount, "frontendSourceRoot"], ["admin", contract.routeInventory.adminCount, "adminSourceRoot"]]) {
  const app = routes.applications?.[name];
  if (!app || app.expectedCount !== count || app.sourceRoot !== contract.routeInventory[rootKey]) fail(`${name} route inventory drifted`);
  if (app.routes.length !== count) fail(`${name} route count is not ${count}`);
}
const api = readJson(contract.activeContracts.api);
if (api.baseline?.sourceRef !== "development" || api.baseline?.sourceCommit !== contract.source.commit) fail("API contract is not rebased");
const permission = readJson(contract.activeContracts.permission);
if (permission.sourceBaseline?.ref !== "development" || permission.sourceBaseline?.commit !== contract.source.commit) fail("permission contract is not rebased");
const migration = readJson(contract.activeContracts.migrationSafety);
if (migration.baseline?.sourceRef !== "development" || migration.baseline?.sourceCommit !== contract.source.commit) fail("migration safety contract is not rebased");
const serviceAuth = readJson(contract.activeContracts.serviceAuthorization);
const routesByService = (serviceAuth.services ?? []).flatMap((service) => service.routes ?? []);
if (routesByService.some((route) => route.classification === "unknown" || route.caseProfile === "unknown" || route.identitySource === "undecided-fail-closed")) fail("service authorization still contains unknown or undecided routes");
if (routesByService.length !== 166) fail(`service authorization route count drifted: ${routesByService.length}`);
const frontendMain = readFileSync(resolve(root, "apps/frontend/src/main.rs"), "utf8");
const frontendApi = readFileSync(resolve(root, "apps/frontend/src/api.rs"), "utf8");
for (const forbidden of ["api_subscription_plans", "api_subscription_subscribe", "api_subscription_create_plan", "api_wallet_connect", "sub_1", "pub async fn save_page("]) {
  if (frontendMain.includes(forbidden) || frontendApi.includes(forbidden)) fail(`frontend still contains removed producer: ${forbidden}`);
}
console.log(`current-rebaseline: development@${contract.source.commit}`);
console.log(`current-rebaseline: target-base@${contract.targetBase.commit}`);
console.log(`current-rebaseline: frontend ${contract.routeInventory.frontendCount}/${contract.routeInventory.frontendCount}, admin ${contract.routeInventory.adminCount}/${contract.routeInventory.adminCount}`);
console.log(`current-rebaseline: service authorization ${routesByService.length} routes, unknown=0`);
console.log("current-rebaseline: PASS — comparison/readiness guard only; production readiness is not claimed");
' -- "$ROOT" "$CONTRACT"
