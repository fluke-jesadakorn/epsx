#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
CONTRACT="$REPO_ROOT/docs/migration/contracts/migration-safety.json"

command -v bun >/dev/null 2>&1 || {
  echo "migration-safety: ERROR: bun is required" >&2
  exit 1
}

bun -e '
import { existsSync, readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";

const [root, contractPath] = process.argv.slice(1);
const fail = (message) => {
  console.error(`migration-safety: ERROR: ${message}`);
  process.exit(1);
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed: ${result.stderr.toString().trim()}`);
  return result.stdout.toString();
};
const hash = (value) => {
  const hasher = new Bun.CryptoHasher("sha256");
  hasher.update(value);
  return hasher.digest("hex");
};
const validRelativePath = (value) =>
  typeof value === "string" && value.length > 0 && !value.startsWith("/") && !value.split("/").includes("..");
const normalizeLine = (line) => line.trim().replace(/\s+/g, " ");

let fixture;
try {
  fixture = JSON.parse(readFileSync(contractPath, "utf8"));
} catch (error) {
  fail(`invalid JSON: ${error.message}`);
}

if (fixture.schemaVersion !== 1) fail("schemaVersion must be 1");
if (fixture.purpose !== "fixture-and-static-integrity-only") fail("purpose must be fixture-and-static-integrity-only");
if (fixture.baseline?.sourceRef !== "origin/development") fail("origin/development baseline is required");
if (git("rev-parse", fixture.baseline.sourceRef).trim() !== fixture.baseline.sourceCommit) fail("sourceCommit does not match origin/development");
if (!Array.isArray(fixture.migrationRoots) || fixture.migrationRoots.length === 0) fail("migrationRoots must be non-empty");
if (!Array.isArray(fixture.mandatoryRootIds)) fail("mandatoryRootIds is required");
if (!Array.isArray(fixture.risks) || fixture.risks.length === 0) fail("risks must be non-empty");
if (!Array.isArray(fixture.mandatoryRiskIds) || !Array.isArray(fixture.knownBlockedIds)) fail("mandatoryRiskIds and knownBlockedIds are required");
if (!Array.isArray(fixture.allowedRiskClasses) || fixture.allowedRiskClasses.length === 0) fail("allowedRiskClasses is required");

const rawFixture = readFileSync(contractPath, "utf8");
if (/postgres(?:ql)?:\/\/[^\s"/]+:[^\s"@]+@/i.test(rawFixture)) fail("fixture must not contain database credentials");

const allowedStatus = new Set(["blocked", "partial", "aligned"]);
const allowedSeverity = new Set(["critical", "high", "medium", "low"]);
const allowedRootKind = new Set(["active-diesel", "tracked-archive", "manual-runner"]);
const allowedRiskClass = new Set(fixture.allowedRiskClasses);
const allowedDependency = /^A(?:[0-9]|1[0-3])$/;
const idPattern = /^[a-z][a-z0-9.-]+$/;
let evidenceCount = 0;

const validateEvidence = (owner, evidence) => {
  if (!Array.isArray(evidence) || evidence.length === 0) fail(`${owner}: evidence is required`);
  for (const item of evidence) {
    if (!item || !["worktree", fixture.baseline.sourceRef].includes(item.ref)) fail(`${owner}: invalid evidence ref`);
    if (!validRelativePath(item.file)) fail(`${owner}: evidence file must be repository-relative`);
    if (typeof item.anchor !== "string" || item.anchor.length === 0) fail(`${owner}: evidence anchor is required`);
    let content;
    if (item.ref === "worktree") {
      const path = resolve(root, item.file);
      if (!existsSync(path) || !statSync(path).isFile()) fail(`${owner}: evidence file does not exist: ${item.file}`);
      content = readFileSync(path, "utf8");
    } else {
      content = git("show", `${item.ref}:${item.file}`);
    }
    if (!content.includes(item.anchor)) fail(`${owner}: missing anchor in ${item.ref}:${item.file}: ${JSON.stringify(item.anchor)}`);
    evidenceCount += 1;
  }
};

const rootIds = new Set();
for (const item of fixture.migrationRoots) {
  if (!item || typeof item !== "object" || !idPattern.test(item.id ?? "")) fail(`invalid migration root id: ${item?.id}`);
  if (rootIds.has(item.id)) fail(`duplicate migration root id: ${item.id}`);
  rootIds.add(item.id);
  if (!validRelativePath(item.path)) fail(`${item.id}: root path must be repository-relative`);
  const path = resolve(root, item.path);
  if (!existsSync(path) || !statSync(path).isDirectory()) fail(`${item.id}: root path does not exist: ${item.path}`);
  if (!allowedRootKind.has(item.kind)) fail(`${item.id}: invalid root kind ${item.kind}`);
  if (typeof item.database !== "string" || !item.database) fail(`${item.id}: database is required`);
  if (!item.observed || !allowedStatus.has(item.observed.status) || typeof item.observed.summary !== "string" || !item.observed.summary) fail(`${item.id}: observed status/summary is invalid`);
  if (item.runner !== null) {
    if (!validRelativePath(item.runner) || !existsSync(resolve(root, item.runner))) fail(`${item.id}: runner does not exist: ${item.runner}`);
  }
  validateEvidence(item.id, item.evidence);
}
if (new Set(fixture.mandatoryRootIds).size !== fixture.mandatoryRootIds.length) fail("mandatoryRootIds contains duplicates");
if (fixture.mandatoryRootIds.length !== rootIds.size) fail("mandatoryRootIds must list every migration root exactly once");
for (const id of fixture.mandatoryRootIds) if (!rootIds.has(id)) fail(`mandatory migration root is missing: ${id}`);

const riskIds = new Set();
for (const risk of fixture.risks) {
  if (!risk || typeof risk !== "object" || !idPattern.test(risk.id ?? "")) fail(`invalid risk id: ${risk?.id}`);
  if (riskIds.has(risk.id)) fail(`duplicate risk id: ${risk.id}`);
  riskIds.add(risk.id);
  if (!allowedRiskClass.has(risk.riskClass)) fail(`${risk.id}: invalid riskClass ${risk.riskClass}`);
  if (!allowedSeverity.has(risk.severity)) fail(`${risk.id}: invalid severity ${risk.severity}`);
  if (typeof risk.database !== "string" || !risk.database) fail(`${risk.id}: database is required`);
  if (typeof risk.dataSafetyInvariant !== "string" || !risk.dataSafetyInvariant) fail(`${risk.id}: dataSafetyInvariant is required`);
  if (!Array.isArray(risk.dependencies) || risk.dependencies.length === 0 || risk.dependencies.some((value) => typeof value !== "string" || !allowedDependency.test(value))) fail(`${risk.id}: dependencies must be A0..A13 package IDs`);
  if (!risk.requiredRemediation || !/^A3(?:\.[0-9]+)?$/.test(risk.requiredRemediation.package ?? "")) fail(`${risk.id}: A3 remediation package is required`);
  if (!new Set(["additive-remediation", "backfill", "reconcile", "forward-fix"]).has(risk.requiredRemediation.mode)) fail(`${risk.id}: invalid remediation mode`);
  if (typeof risk.requiredRemediation.summary !== "string" || !risk.requiredRemediation.summary) fail(`${risk.id}: remediation summary is required`);
  if (!Array.isArray(risk.requiredRemediation.requiredProof) || risk.requiredRemediation.requiredProof.length === 0 || risk.requiredRemediation.requiredProof.some((value) => typeof value !== "string" || !value)) fail(`${risk.id}: requiredProof is required`);
  if (!risk.observed || !allowedStatus.has(risk.observed.status) || typeof risk.observed.summary !== "string" || !risk.observed.summary) fail(`${risk.id}: observed status/summary is invalid`);
  if (risk.observed.status === "aligned" && !Array.isArray(risk.executableProof)) fail(`${risk.id}: aligned status requires executableProof`);
  validateEvidence(risk.id, risk.evidence);
}

if (new Set(fixture.mandatoryRiskIds).size !== fixture.mandatoryRiskIds.length) fail("mandatoryRiskIds contains duplicates");
if (fixture.mandatoryRiskIds.length !== riskIds.size) fail("mandatoryRiskIds must list every risk exactly once");
for (const id of fixture.mandatoryRiskIds) if (!riskIds.has(id)) fail(`mandatory risk is missing: ${id}`);
for (const id of fixture.knownBlockedIds) {
  const risk = fixture.risks.find((item) => item.id === id);
  if (!risk) fail(`known blocker is missing: ${id}`);
  if (risk.observed.status !== "blocked") fail(`${id}: known destructive/data-loss/drift gap must remain blocked until executable upgrade proof exists`);
}

const tracked = git("ls-files", "-z").split("\0").filter(Boolean).sort();
const migrationSqlFiles = tracked.filter((file) => file.endsWith(".sql") && /(^|\/)migrations?\//.test(file));
const rustFiles = tracked.filter((file) => file.endsWith(".rs"));

const stripComments = (content, marker) => {
  let inBlock = false;
  return content.split(/\r?\n/).map((line) => {
    let result = "";
    for (let index = 0; index < line.length; index += 1) {
      if (inBlock) {
        if (line.slice(index, index + 2) === "*/") { inBlock = false; index += 1; }
        continue;
      }
      if (line.slice(index, index + 2) === "/*") { inBlock = true; index += 1; continue; }
      if (line.slice(index, index + marker.length) === marker) break;
      result += line[index];
    }
    return result;
  });
};

const destructivePattern = /\b(?:DROP\s+(?:TABLE|SCHEMA|COLUMN|CONSTRAINT|INDEX|TYPE|VIEW|MATERIALIZED\s+VIEW|FUNCTION|TRIGGER|DATABASE)|TRUNCATE(?:\s+TABLE)?|DELETE\s+FROM|CASCADE)\b/i;
const runtimeDdlPattern = /\b(?:CREATE|ALTER|DROP|TRUNCATE)\s+(?:OR\s+REPLACE\s+)?(?:TABLE|SCHEMA|INDEX|TYPE|VIEW|MATERIALIZED\s+VIEW|DATABASE)\b/i;
const scanFiles = (files, marker, pattern) => {
  const findings = [];
  for (const file of files) {
    const lines = stripComments(readFileSync(resolve(root, file), "utf8"), marker);
    lines.forEach((line, index) => {
      const match = line.match(pattern);
      if (match) findings.push({ file, line: index + 1, kind: normalizeLine(match[0]).toUpperCase(), text: normalizeLine(line) });
    });
  }
  return findings.sort((left, right) => left.file.localeCompare(right.file) || left.line - right.line || left.text.localeCompare(right.text));
};

const sqlFindings = scanFiles(migrationSqlFiles, "--", destructivePattern);
const rustFindings = scanFiles(rustFiles, "//", runtimeDdlPattern);
const digestFindings = (findings) => hash(findings.map((item) => `${item.file}:${item.line}:${item.kind}:${item.text}`).join("\n"));

const exceptions = fixture.staticScan?.reviewedExceptions;
if (!Array.isArray(exceptions)) fail("staticScan.reviewedExceptions is required");
const exceptionIds = new Set();
const exceptedRuntime = new Set();
for (const exception of exceptions) {
  if (!idPattern.test(exception.id ?? "") || exceptionIds.has(exception.id)) fail(`invalid/duplicate exception id: ${exception.id}`);
  exceptionIds.add(exception.id);
  if (exception.scope !== "runtimeRustDdl") fail(`${exception.id}: unsupported exception scope`);
  if (!validRelativePath(exception.file) || typeof exception.anchor !== "string" || !exception.anchor || typeof exception.reason !== "string" || exception.reason.length < 20) fail(`${exception.id}: exact file, anchor, and substantive reason are required`);
  const matches = rustFindings.filter((finding) => finding.file === exception.file && finding.text.includes(exception.anchor));
  if (matches.length !== 1) fail(`${exception.id}: reviewed exception must match exactly one runtime finding, found ${matches.length}`);
  exceptedRuntime.add(`${matches[0].file}:${matches[0].line}:${matches[0].text}`);
}

const historyLines = git("diff", "--name-status", fixture.baseline.sourceRef, "--", "apps/backend/migrations", "apps/backend/diesel.toml", "apps/backend/diesel_analytics.toml", "apps/backend/diesel_notifications.toml", "apps/backend/diesel_payments.toml")
  .split(/\r?\n/).filter(Boolean).sort();

const expectedSql = fixture.staticScan?.migrationSql;
const expectedRust = fixture.staticScan?.runtimeRustDdl;
const expectedDiff = fixture.staticScan?.sourceDiff;
if (!expectedSql || !expectedRust || !expectedDiff) fail("staticScan baselines are required");
const actualSql = { trackedFiles: migrationSqlFiles.length, findings: sqlFindings.length, sha256: digestFindings(sqlFindings) };
const actualRust = { trackedFiles: rustFiles.length, findings: rustFindings.length, sha256: digestFindings(rustFindings) };
const actualDiff = { changedPaths: historyLines.length, sha256: hash(historyLines.join("\n")) };
const compareBaseline = (label, expected, actual) => {
  for (const key of Object.keys(actual)) {
    if (expected[key] !== actual[key]) fail(`${label} ${key} changed: expected ${expected[key]}, observed ${actual[key]}; inventory the exact finding or add an exact reviewed exception with reason`);
  }
};
compareBaseline("migrationSql", expectedSql, actualSql);
compareBaseline("runtimeRustDdl", expectedRust, actualRust);
compareBaseline("sourceDiff", expectedDiff, actualDiff);

const actionableRuntime = rustFindings.filter((finding) => !exceptedRuntime.has(`${finding.file}:${finding.line}:${finding.text}`));
const statusCounts = Object.fromEntries(["blocked", "partial", "aligned"].map((status) => [status, fixture.risks.filter((risk) => risk.observed.status === status).length]));
console.log(`migration-safety: roots ${rootIds.size}/${rootIds.size}`);
console.log(`migration-safety: risks ${riskIds.size}/${fixture.mandatoryRiskIds.length} (blocked=${statusCounts.blocked}, partial=${statusCounts.partial}, aligned=${statusCounts.aligned})`);
console.log(`migration-safety: evidence anchors ${evidenceCount}/${evidenceCount}`);
console.log(`migration-safety: migration SQL ${migrationSqlFiles.length} files, ${sqlFindings.length} destructive findings (digest locked)`);
console.log(`migration-safety: runtime Rust ${rustFiles.length} files, ${rustFindings.length} DDL findings, ${exceptions.length} exact reviewed exceptions, ${actionableRuntime.length} actionable (digest locked)`);
console.log(`migration-safety: source comparison ${historyLines.length} changed migration/config paths (digest locked)`);
console.log("migration-safety: OK — fixture/static integrity only; this is NOT a database upgrade test or production-readiness pass");
' -- "$REPO_ROOT" "$CONTRACT"
