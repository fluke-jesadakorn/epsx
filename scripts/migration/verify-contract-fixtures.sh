#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
CONTRACT="$REPO_ROOT/docs/migration/contracts/api-contracts.json"

command -v bun >/dev/null 2>&1 || {
  echo "contract-fixtures: ERROR: bun is required" >&2
  exit 1
}

bun -e '
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const [root, contractPath] = process.argv.slice(1);
const fail = (message) => {
  console.error(`contract-fixtures: ERROR: ${message}`);
  process.exit(1);
};

let fixture;
try {
  fixture = JSON.parse(readFileSync(contractPath, "utf8"));
} catch (error) {
  fail(`invalid JSON: ${error.message}`);
}

if (fixture.schemaVersion !== 1) fail("schemaVersion must be 1");
if (fixture.purpose !== "fixture-integrity-only") fail("purpose must be fixture-integrity-only");
if (!fixture.baseline || fixture.baseline.sourceRef !== "origin/development") fail("origin/development baseline is required");
if (!Array.isArray(fixture.contracts) || fixture.contracts.length === 0) fail("contracts must be a non-empty array");
if (!Array.isArray(fixture.mandatoryIds) || !Array.isArray(fixture.knownP0Ids)) fail("mandatoryIds and knownP0Ids are required");

const allowedPriority = new Set(["P0", "P1", "P2"]);
const allowedStatus = new Set(["blocked", "partial", "aligned"]);
const allowedDependency = /^A(?:[0-9]|1[0-3])$/;
const ids = new Set();
let evidenceCount = 0;

for (const contract of fixture.contracts) {
  if (!contract || typeof contract !== "object") fail("each contract must be an object");
  if (typeof contract.id !== "string" || !/^[a-z][a-z0-9.-]+$/.test(contract.id)) fail(`invalid id: ${contract.id}`);
  if (ids.has(contract.id)) fail(`duplicate id: ${contract.id}`);
  ids.add(contract.id);
  if (typeof contract.surface !== "string" || !contract.surface) fail(`${contract.id}: surface is required`);
  if (!allowedPriority.has(contract.priority)) fail(`${contract.id}: invalid priority ${contract.priority}`);
  if (!Array.isArray(contract.dependencies) || contract.dependencies.some((item) => typeof item !== "string" || !allowedDependency.test(item))) fail(`${contract.id}: dependencies must be A0..A13 package IDs`);
  if (!contract.required || !Array.isArray(contract.required.methods) || contract.required.methods.length === 0) fail(`${contract.id}: required.methods is missing`);
  if (typeof contract.required.endpoint !== "string" || !contract.required.endpoint) fail(`${contract.id}: required.endpoint is missing`);
  if (!Array.isArray(contract.required.successStatuses) || contract.required.successStatuses.some((item) => !Number.isInteger(item))) fail(`${contract.id}: required.successStatuses must be integers`);
  if (!Array.isArray(contract.required.invariants) || contract.required.invariants.length === 0 || contract.required.invariants.some((item) => typeof item !== "string" || !item)) fail(`${contract.id}: required.invariants is missing`);
  if (!contract.observed || !allowedStatus.has(contract.observed.status) || typeof contract.observed.summary !== "string" || !contract.observed.summary) fail(`${contract.id}: observed status/summary is invalid`);
  if (!Array.isArray(contract.evidence) || contract.evidence.length === 0) fail(`${contract.id}: evidence is required`);

  for (const evidence of contract.evidence) {
    if (!evidence || typeof evidence.file !== "string" || !evidence.file || evidence.file.startsWith("/") || evidence.file.split("/").includes("..")) fail(`${contract.id}: evidence file must be repository-relative`);
    if (typeof evidence.anchor !== "string" || !evidence.anchor) fail(`${contract.id}: evidence anchor is required`);
    const path = resolve(root, evidence.file);
    let content;
    try {
      content = readFileSync(path, "utf8");
    } catch {
      fail(`${contract.id}: evidence file does not exist: ${evidence.file}`);
    }
    if (!content.includes(evidence.anchor)) fail(`${contract.id}: missing anchor in ${evidence.file}: ${JSON.stringify(evidence.anchor)}`);
    evidenceCount += 1;
  }
}

for (const id of fixture.mandatoryIds) {
  if (!ids.has(id)) fail(`mandatory contract is missing: ${id}`);
}
if (fixture.mandatoryIds.length !== ids.size || new Set(fixture.mandatoryIds).size !== fixture.mandatoryIds.length) fail("mandatoryIds must list every contract exactly once");

for (const id of fixture.knownP0Ids) {
  const contract = fixture.contracts.find((item) => item.id === id);
  if (!contract) fail(`known P0 contract is missing: ${id}`);
  if (contract.priority !== "P0") fail(`${id}: known P0 contract must have priority P0`);
  if (contract.observed.status !== "blocked") fail(`${id}: known P0 gap must remain blocked until executable runtime proof exists`);
}

console.log(`contract-fixtures: contracts ${ids.size}/${fixture.mandatoryIds.length}`);
console.log(`contract-fixtures: evidence anchors ${evidenceCount}/${evidenceCount}`);
console.log(`contract-fixtures: known P0 blockers ${fixture.knownP0Ids.length}/${fixture.knownP0Ids.length} labeled blocked`);
console.log("contract-fixtures: OK — fixture integrity only; this is NOT a production-readiness pass");
' -- "$REPO_ROOT" "$CONTRACT"
