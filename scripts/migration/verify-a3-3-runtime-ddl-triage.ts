#!/usr/bin/env bun

import {
  existsSync,
  lstatSync,
  readFileSync,
  realpathSync,
  statSync,
} from "node:fs";
import { dirname, isAbsolute, resolve } from "node:path";

type JsonObject = Record<string, unknown>;

const scriptPath = realpathSync(import.meta.path);
const repoRootResult = Bun.spawnSync(
  ["git", "-C", dirname(scriptPath), "rev-parse", "--show-toplevel"],
  { stdout: "pipe", stderr: "pipe" },
);
if (repoRootResult.exitCode !== 0) {
  console.error("a3-3-runtime-ddl-triage: ERROR: repository root is unavailable");
  process.exit(1);
}
const repoRoot = repoRootResult.stdout.toString().trim();

const args = process.argv.slice(2);
let contractPath = resolve(
  repoRoot,
  "docs/migration/contracts/a3-3-runtime-ddl-triage.json",
);
let upstreamPath = resolve(
  repoRoot,
  "docs/migration/contracts/migration-safety.json",
);
let jsonOutput = false;
let readiness = false;

const fail = (message: string): never => {
  console.error(`a3-3-runtime-ddl-triage: ERROR: ${message}`);
  process.exit(1);
};

for (let index = 0; index < args.length; index += 1) {
  const argument = args[index];
  if (argument === "--contract" || argument === "--upstream") {
    const value = args[index + 1];
    if (!value) fail(`${argument} requires a path`);
    const path = isAbsolute(value) ? value : resolve(repoRoot, value);
    if (argument === "--contract") contractPath = path;
    else upstreamPath = path;
    index += 1;
  } else if (argument === "--json") {
    jsonOutput = true;
  } else if (argument === "--readiness") {
    readiness = true;
  } else {
    fail(`unsupported argument: ${argument}`);
  }
}

const readRegularFile = (label: string, path: string): string => {
  if (!existsSync(path)) fail(`${label} does not exist: ${path}`);
  if (lstatSync(path).isSymbolicLink()) fail(`${label} must not be a symbolic link`);
  if (!statSync(path).isFile()) fail(`${label} must be a regular file`);
  return readFileSync(path, "utf8");
};

const parseJson = (label: string, raw: string): JsonObject => {
  try {
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      fail(`${label} must contain a JSON object`);
    }
    return parsed as JsonObject;
  } catch (error) {
    fail(`${label} is invalid JSON: ${(error as Error).message}`);
  }
};

const sha256 = (value: string): string => {
  const hasher = new Bun.CryptoHasher("sha256");
  hasher.update(value);
  return hasher.digest("hex");
};

const stable = (value: unknown): string => {
  if (Array.isArray(value)) return `[${value.map(stable).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.entries(value as JsonObject)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, item]) => `${JSON.stringify(key)}:${stable(item)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
};

const exact = (label: string, expected: unknown, actual: unknown): void => {
  if (stable(expected) !== stable(actual)) {
    fail(`${label} differs from the checksum-pinned scanner evidence`);
  }
};

const git = (...gitArgs: string[]): string => {
  const result = Bun.spawnSync(["git", "-C", repoRoot, ...gitArgs], {
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    fail(`git ${gitArgs.join(" ")} failed: ${result.stderr.toString().trim()}`);
  }
  return result.stdout.toString();
};

const rawContract = readRegularFile("triage contract", contractPath);
const rawUpstream = readRegularFile("migration-safety contract", upstreamPath);
if (/postgres(?:ql)?:\/\/[^\s"/]+:[^\s"@]+@/i.test(rawContract + rawUpstream)) {
  fail("contracts must not contain database credentials");
}
const contract = parseJson("triage contract", rawContract) as any;
const upstream = parseJson("migration-safety contract", rawUpstream) as any;

if (contract.schemaVersion !== 1) fail("schemaVersion must be 1");
if (contract.purpose !== "offline-static-runtime-ddl-triage-only") {
  fail("purpose must be offline-static-runtime-ddl-triage-only");
}
if (contract.productionReady !== false) fail("productionReady must remain false");
if (contract.upstream?.path !== "docs/migration/contracts/migration-safety.json") {
  fail("the canonical migration-safety contract path must remain pinned");
}
if (!/^[a-f0-9]{64}$/.test(contract.upstream?.sha256 ?? "")) {
  fail("upstream.sha256 must be a lowercase SHA-256 digest");
}
if (sha256(rawUpstream) !== contract.upstream.sha256) {
  fail("migration-safety contract checksum changed; refresh triage explicitly");
}

if (upstream.schemaVersion !== 1 || upstream.purpose !== "fixture-and-static-integrity-only") {
  fail("unexpected migration-safety contract schema/purpose");
}
exact(
  "runtime Rust DDL baseline",
  {
    trackedFiles: upstream.staticScan?.runtimeRustDdl?.trackedFiles,
    findings: upstream.staticScan?.runtimeRustDdl?.findings,
    sha256: upstream.staticScan?.runtimeRustDdl?.sha256,
    reviewedExceptions: upstream.staticScan?.reviewedExceptions?.length,
    actionable:
      upstream.staticScan?.runtimeRustDdl?.findings -
      upstream.staticScan?.reviewedExceptions?.length,
  },
  contract.upstream.runtimeRustDdl,
);

if (
  contract.statusPolicy?.actionableStatus !== "blocked" ||
  contract.statusPolicy?.readinessResult !== "STOP"
) {
  fail("actionable findings must remain blocked and readiness must remain STOP");
}
for (const key of ["exceptionMeaning", "actionableMeaning"]) {
  if (typeof contract.statusPolicy?.[key] !== "string" || contract.statusPolicy[key].length < 30) {
    fail(`statusPolicy.${key} must explain the triage status`);
  }
}

const riskIds = ["runtime.service-schema-ddl", "runtime.missing-service-migrations"];
const riskRequirements = riskIds.map((id) => {
  const risk = upstream.risks?.find((candidate: any) => candidate.id === id);
  if (!risk || risk.observed?.status !== "blocked") {
    fail(`${id} must exist and remain blocked upstream`);
  }
  return {
    id,
    dataSafetyInvariant: risk.dataSafetyInvariant,
    remediationPackage: risk.requiredRemediation?.package,
    remediationMode: risk.requiredRemediation?.mode,
    summary: risk.requiredRemediation?.summary,
    requiredProof: risk.requiredRemediation?.requiredProof,
  };
});
exact("risk requirements", riskRequirements, contract.riskRequirements);

const normalizeLine = (line: string): string => line.trim().replace(/\s+/g, " ");
const stripComments = (content: string, marker: string): string[] => {
  let inBlock = false;
  return content.split(/\r?\n/).map((line) => {
    let result = "";
    for (let index = 0; index < line.length; index += 1) {
      if (inBlock) {
        if (line.slice(index, index + 2) === "*/") {
          inBlock = false;
          index += 1;
        }
        continue;
      }
      if (line.slice(index, index + 2) === "/*") {
        inBlock = true;
        index += 1;
        continue;
      }
      if (line.slice(index, index + marker.length) === marker) break;
      result += line[index];
    }
    return result;
  });
};

const runtimeDdlPattern =
  /\b(?:CREATE|ALTER|DROP|TRUNCATE)\s+(?:OR\s+REPLACE\s+)?(?:TABLE|SCHEMA|INDEX|TYPE|VIEW|MATERIALIZED\s+VIEW|DATABASE)\b/i;
const trackedRustFiles = git("ls-files", "-z")
  .split("\0")
  .filter(Boolean)
  .filter((file) => file.endsWith(".rs"))
  .sort();
const scannerFindings: Array<{
  file: string;
  line: number;
  kind: string;
  text: string;
}> = [];
for (const file of trackedRustFiles) {
  const path = resolve(repoRoot, file);
  if (!existsSync(path) || !statSync(path).isFile()) fail(`tracked Rust file is missing: ${file}`);
  stripComments(readFileSync(path, "utf8"), "//").forEach((line, index) => {
    const match = line.match(runtimeDdlPattern);
    if (match) {
      scannerFindings.push({
        file,
        line: index + 1,
        kind: normalizeLine(match[0]).toUpperCase(),
        text: normalizeLine(line),
      });
    }
  });
}
scannerFindings.sort(
  (left, right) =>
    left.file.localeCompare(right.file) ||
    left.line - right.line ||
    left.text.localeCompare(right.text),
);
const scannerDigest = sha256(
  scannerFindings
    .map((item) => `${item.file}:${item.line}:${item.kind}:${item.text}`)
    .join("\n"),
);
exact(
  "live scanner baseline",
  upstream.staticScan.runtimeRustDdl,
  {
    trackedFiles: trackedRustFiles.length,
    findings: scannerFindings.length,
    sha256: scannerDigest,
  },
);

const exceptionByFinding = new Map<string, string>();
const exceptionIds = new Set<string>();
for (const exception of upstream.staticScan.reviewedExceptions) {
  if (exceptionIds.has(exception.id)) fail(`duplicate upstream exception: ${exception.id}`);
  exceptionIds.add(exception.id);
  const matches = scannerFindings.filter(
    (finding) =>
      finding.file === exception.file && finding.text.includes(exception.anchor),
  );
  if (matches.length !== 1) {
    fail(`${exception.id} must match exactly one scanner finding, observed ${matches.length}`);
  }
  const finding = matches[0];
  const key = `${finding.file}:${finding.line}:${finding.text}`;
  if (exceptionByFinding.has(key)) fail(`multiple exceptions match ${finding.file}:${finding.line}`);
  exceptionByFinding.set(key, exception.id);
}

const classify = (finding: (typeof scannerFindings)[number], index: number) => {
  const key = `${finding.file}:${finding.line}:${finding.text}`;
  const exceptionId = exceptionByFinding.get(key) ?? null;
  let service: string;
  let bootTimeRisk: string;
  if (exceptionId) {
    service = finding.file.includes("web3_security_tests")
      ? "backend-security-test"
      : "backend-smoke-test";
    bootTimeRisk = "reviewed-exception-not-runtime-ddl";
  } else if (finding.file === "apps/backend/src/bin/blockchain_monitor.rs") {
    service = "backend-blockchain-monitor";
    bootTimeRisk = "lexical-match-not-schema-ddl";
  } else if (finding.file === "apps/backend/src/main.rs") {
    service = "backend-main";
    bootTimeRisk = "lexical-match-not-schema-ddl";
  } else if (finding.file === "apps/backend/src/bin/migrate.rs") {
    service = "backend-migrate";
    bootTimeRisk = "runtime-database-bootstrap";
  } else {
    const match = finding.file.match(/^services\/([^/]+)\/src\//);
    if (!match) fail(`actionable finding lacks an evidence-backed service group: ${finding.file}`);
    service = match[1];
    bootTimeRisk = "service-startup-schema-mutation";
  }
  return {
    id: `finding.${String(index + 1).padStart(3, "0")}`,
    file: finding.file,
    line: finding.line,
    ddlKind: finding.kind,
    classification: exceptionId ? "reviewed-exception" : "actionable",
    reviewedExceptionId: exceptionId,
    service,
    bootTimeRisk,
    status: "blocked",
  };
};
const classified = scannerFindings.map(classify);

if (!Array.isArray(contract.findings) || contract.findings.length !== 35) {
  fail("the contract must enumerate exactly 35 scanner findings");
}
exact("enumerated findings", classified, contract.findings);
if (new Set(classified.map((item) => item.id)).size !== classified.length) {
  fail("finding IDs must be unique");
}
if (classified.some((item) => item.status !== "blocked")) {
  fail("every enumerated finding must remain blocked in this integrity package");
}
const actionable = classified.filter((item) => item.classification === "actionable");
const reviewedExceptions = classified.filter(
  (item) => item.classification === "reviewed-exception",
);
if (actionable.length !== 29 || reviewedExceptions.length !== 6) {
  fail(`expected 29 actionable and 6 exceptions, observed ${actionable.length}/${reviewedExceptions.length}`);
}
if (actionable.some((item) => item.reviewedExceptionId !== null)) {
  fail("an actionable finding cannot carry a reviewed exception ID");
}
exact(
  "reviewed exception IDs",
  [...exceptionIds].sort(),
  reviewedExceptions.map((item) => item.reviewedExceptionId).sort(),
);

const group = (key: "classification" | "ddlKind" | "bootTimeRisk" | "service" | "file") =>
  Object.fromEntries(
    [...new Set(classified.map((item) => item[key]))]
      .sort()
      .map((value) => [value, classified.filter((item) => item[key] === value).length]),
  );
const groups = {
  classification: group("classification"),
  ddlKind: group("ddlKind"),
  bootTimeRisk: group("bootTimeRisk"),
  service: group("service"),
  file: group("file"),
};
exact("grouped triage counts", groups, contract.expectedGroups);

const report = {
  schemaVersion: 1,
  purpose: contract.purpose,
  upstreamSha256: contract.upstream.sha256,
  scanner: {
    trackedRustFiles: trackedRustFiles.length,
    findings: classified.length,
    reviewedExceptions: reviewedExceptions.length,
    actionable: actionable.length,
    sha256: scannerDigest,
  },
  groups,
  riskIds,
  actionableStatus: "blocked",
  readiness: "STOP",
  productionReady: false,
};

if (jsonOutput) {
  console.log(JSON.stringify(report, null, 2));
} else {
  console.log(
    `a3-3-runtime-ddl-triage: Rust ${trackedRustFiles.length} files, ${classified.length} findings, ${reviewedExceptions.length} exact reviewed exceptions, ${actionable.length} actionable`,
  );
  console.log(
    `a3-3-runtime-ddl-triage: ${Object.keys(groups.service).length} service groups, ${Object.keys(groups.file).length} file groups, ${Object.keys(groups.ddlKind).length} DDL kinds, ${Object.keys(groups.bootTimeRisk).length} boot-time risk groups`,
  );
  console.log(
    "a3-3-runtime-ddl-triage: OK — deterministic offline/static integrity only; all 29 actionable findings remain blocked",
  );
}

if (readiness) {
  console.error(
    "a3-3-runtime-ddl-triage: STOP — productionReady=false; runtime DDL remediation and executable database proof are absent",
  );
  process.exit(2);
}
