#!/usr/bin/env bun

import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  closeSync,
  existsSync,
  lstatSync,
  openSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, isAbsolute, relative, resolve, sep } from "node:path";

type RootContract = {
  id: string;
  path: string;
  canonicalBaseline: string;
  alternateBaselines: string[];
  recoveryClass: string;
};

type HashContract = { path: string; sha256: string };
type RelocationContract = { sourcePath: string; archivePath: string; sha256: string };

type Contract = {
  schemaVersion: number;
  package: string;
  purpose: string;
  productionReady: boolean;
  source: { ref: string; commit: string };
  roots: RootContract[];
  trustedSourceFiles: HashContract[];
  approvedRelocations: RelocationContract[];
  approvedWorktreeAdditions: HashContract[];
  stopPolicy: Record<string, boolean>;
  recoveryMatrix: unknown[];
};

type StopReason = {
  code: string;
  domain: string;
  path?: string;
  version?: string;
  detail: string;
};

type SqlFingerprint = {
  domain: string;
  path: string;
  migration: string;
  version: string;
  direction: "up" | "down";
  classification: "unchanged" | "modified" | "deleted" | "added" | "approved-added" | "relocated";
  sourceSha256: string | null;
  worktreeSha256: string | null;
  archivePath: string | null;
};

const usage = `Usage: a3-1-history-preflight.ts [options]

Options:
  --repo PATH       Repository to inspect (default: enclosing git repository)
  --contract PATH   Contract JSON (default: docs/migration/contracts/a3-1-history-preflight.json)
  --output PATH     Write JSON exclusively to an outside-repository path (default: stdout)
  --help            Show this help

Exit status: 0 = static history clear, 2 = stop reasons found, 64 = unsafe/invalid invocation.`;

const dieUsage = (message: string): never => {
  process.stderr.write(`a3-1-history-preflight: ERROR: ${message}\n${usage}\n`);
  process.exit(64);
};

const args = process.argv.slice(2);
let requestedRepo: string | null = null;
let requestedContract: string | null = null;
let outputPath = "-";
for (let index = 0; index < args.length; index += 1) {
  const arg = args[index];
  if (arg === "--help") {
    process.stdout.write(`${usage}\n`);
    process.exit(0);
  }
  if (!["--repo", "--contract", "--output"].includes(arg)) dieUsage(`unknown option: ${arg}`);
  const value = args[index + 1];
  if (!value || value.startsWith("--")) dieUsage(`${arg} requires a value`);
  if (arg === "--repo") requestedRepo = value;
  if (arg === "--contract") requestedContract = value;
  if (arg === "--output") outputPath = value;
  index += 1;
}

const run = (command: string, commandArgs: string[], cwd?: string, allowFailure = false): Buffer => {
  const result = spawnSync(command, commandArgs, {
    cwd,
    encoding: null,
    maxBuffer: 64 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) dieUsage(`${command} failed to start: ${result.error.message}`);
  if (!allowFailure && result.status !== 0) {
    const stderr = result.stderr?.toString("utf8").trim();
    dieUsage(`${command} ${commandArgs.join(" ")} failed${stderr ? `: ${stderr}` : ""}`);
  }
  return result.stdout ?? Buffer.alloc(0);
};

const initialPath = resolve(requestedRepo ?? dirname(new URL(import.meta.url).pathname));
const repoRoot = run("git", ["-C", initialPath, "rev-parse", "--show-toplevel"]).toString("utf8").trim();
if (!repoRoot || !statSync(repoRoot).isDirectory()) dieUsage("could not resolve repository root");
const realRepoRoot = realpathSync(repoRoot);
const contractPath = resolve(requestedContract ?? resolve(realRepoRoot, "docs/migration/contracts/a3-1-history-preflight.json"));

const safeRelativePath = (value: unknown): value is string =>
  typeof value === "string" &&
  value.length > 0 &&
  !isAbsolute(value) &&
  !value.includes("\0") &&
  !value.split(/[\\/]/).includes("..");

const sha256 = (data: Buffer | string): string => createHash("sha256").update(data).digest("hex");
const normalizedVersion = (migrationName: string): string => migrationName.split("_")[0].replaceAll("-", "");
const inside = (parent: string, child: string): boolean => {
  const path = relative(parent, child);
  return path === "" || (!path.startsWith(`..${sep}`) && path !== "..");
};

let contract: Contract;
try {
  contract = JSON.parse(readFileSync(contractPath, "utf8")) as Contract;
} catch (error) {
  dieUsage(`cannot read contract ${contractPath}: ${(error as Error).message}`);
}

if (contract.schemaVersion !== 1) dieUsage("contract schemaVersion must be 1");
if (contract.package !== "A3.1a" || contract.purpose !== "diesel-history-preflight-only") {
  dieUsage("contract package/purpose is not A3.1a history preflight");
}
if (contract.productionReady !== false) dieUsage("contract must explicitly set productionReady=false");
if (!contract.source?.ref || !/^[0-9a-f]{40}$/.test(contract.source?.commit ?? "")) {
  dieUsage("contract source ref and 40-character commit are required");
}
if (!Array.isArray(contract.roots) || contract.roots.length !== 4) dieUsage("contract must define exactly four roots");
if (!Array.isArray(contract.trustedSourceFiles) || contract.trustedSourceFiles.length === 0) {
  dieUsage("contract trustedSourceFiles must be non-empty");
}
if (!Array.isArray(contract.approvedRelocations) || !Array.isArray(contract.approvedWorktreeAdditions)) {
  dieUsage("contract relocation/addition allowlists are required");
}

const rootIds = new Set<string>();
const rootPaths = new Set<string>();
for (const root of contract.roots) {
  if (!/^[a-z][a-z0-9-]*$/.test(root.id ?? "") || rootIds.has(root.id)) dieUsage(`invalid/duplicate root id: ${root.id}`);
  if (!safeRelativePath(root.path) || rootPaths.has(root.path)) dieUsage(`invalid/duplicate root path: ${root.path}`);
  if (!root.canonicalBaseline || !Array.isArray(root.alternateBaselines)) dieUsage(`${root.id}: baseline contract is incomplete`);
  const absolute = resolve(realRepoRoot, root.path);
  if (!inside(realRepoRoot, absolute) || !existsSync(absolute) || !statSync(absolute).isDirectory()) {
    dieUsage(`${root.id}: migration root does not exist: ${root.path}`);
  }
  rootIds.add(root.id);
  rootPaths.add(root.path);
}

const validateHashEntry = (entry: HashContract, owner: string): void => {
  if (!safeRelativePath(entry?.path) || !/^[0-9a-f]{64}$/.test(entry?.sha256 ?? "")) {
    dieUsage(`${owner}: invalid path or SHA-256`);
  }
};
contract.trustedSourceFiles.forEach((entry) => validateHashEntry(entry, "trustedSourceFiles"));
contract.approvedWorktreeAdditions.forEach((entry) => validateHashEntry(entry, "approvedWorktreeAdditions"));
for (const entry of contract.approvedRelocations) {
  if (!safeRelativePath(entry?.sourcePath) || !safeRelativePath(entry?.archivePath) || !/^[0-9a-f]{64}$/.test(entry?.sha256 ?? "")) {
    dieUsage("approvedRelocations: invalid source/archive path or SHA-256");
  }
  if (contract.roots.some((root) => inside(resolve(realRepoRoot, root.path), resolve(realRepoRoot, entry.archivePath)))) {
    dieUsage(`approvedRelocations: archive remains inside an active root: ${entry.archivePath}`);
  }
}

const reasons: StopReason[] = [];
const addReason = (reason: StopReason): void => {
  reasons.push(reason);
};

const resolvedSourceRef = run("git", ["-C", realRepoRoot, "rev-parse", contract.source.ref]).toString("utf8").trim();
if (resolvedSourceRef !== contract.source.commit) {
  addReason({
    code: "source_ref_moved",
    domain: "all",
    detail: `${contract.source.ref} resolves to ${resolvedSourceRef}, expected pinned ${contract.source.commit}`,
  });
}

const sourceSql = new Map<string, Buffer>();
const worktreeSql = new Map<string, Buffer>();
const domainByPath = new Map<string, string>();
const currentMigrations = new Map<string, string[]>();
const sourceMigrations = new Map<string, string[]>();

for (const root of contract.roots) {
  const absoluteRoot = resolve(realRepoRoot, root.path);
  const migrationNames: string[] = [];
  for (const entry of readdirSync(absoluteRoot, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    if (entry.name === ".diesel_lock" && entry.isFile()) continue;
    if (entry.isSymbolicLink()) {
      addReason({ code: "symlink_in_active_root", domain: root.id, path: `${root.path}/${entry.name}`, detail: "active migration entries must not be symlinks" });
      continue;
    }
    if (!entry.isDirectory()) {
      addReason({ code: "non_directory_in_active_root", domain: root.id, path: `${root.path}/${entry.name}`, detail: "Diesel active roots may contain migration directories only" });
      continue;
    }
    migrationNames.push(entry.name);
    for (const direction of ["up", "down"] as const) {
      const relativePath = `${root.path}/${entry.name}/${direction}.sql`;
      const absolutePath = resolve(realRepoRoot, relativePath);
      domainByPath.set(relativePath, root.id);
      if (!existsSync(absolutePath)) continue;
      const fileStat = lstatSync(absolutePath);
      if (fileStat.isSymbolicLink() || !fileStat.isFile()) {
        addReason({ code: "unsafe_sql_file", domain: root.id, path: relativePath, detail: "migration SQL must be a regular non-symlink file" });
        continue;
      }
      worktreeSql.set(relativePath, readFileSync(absolutePath));
    }
  }
  currentMigrations.set(root.id, migrationNames.sort());

  const sourcePaths = run("git", ["-C", realRepoRoot, "ls-tree", "-r", "--name-only", contract.source.commit, "--", root.path])
    .toString("utf8")
    .split(/\r?\n/)
    .filter((path) => /\/(?:up|down)\.sql$/.test(path))
    .sort();
  const sourceNames = new Set<string>();
  for (const path of sourcePaths) {
    const suffix = path.slice(root.path.length + 1);
    const migration = suffix.split("/")[0];
    sourceNames.add(migration);
    domainByPath.set(path, root.id);
    sourceSql.set(path, run("git", ["-C", realRepoRoot, "show", `${contract.source.commit}:${path}`]));
  }
  sourceMigrations.set(root.id, [...sourceNames].sort());
}

const findDuplicates = (names: string[]): Array<{ version: string; migrations: string[] }> => {
  const versions = new Map<string, string[]>();
  for (const name of names) {
    const version = normalizedVersion(name);
    versions.set(version, [...(versions.get(version) ?? []), name]);
  }
  return [...versions.entries()]
    .filter(([, migrations]) => migrations.length > 1)
    .map(([version, migrations]) => ({ version, migrations: migrations.sort() }))
    .sort((left, right) => left.version.localeCompare(right.version));
};

const rootReports = contract.roots.map((root) => {
  const current = currentMigrations.get(root.id) ?? [];
  const source = sourceMigrations.get(root.id) ?? [];
  const currentDuplicates = findDuplicates(current);
  const sourceDuplicates = findDuplicates(source);
  for (const duplicate of currentDuplicates) {
    addReason({
      code: "duplicate_normalized_version",
      domain: root.id,
      version: duplicate.version,
      detail: duplicate.migrations.join(", "),
    });
  }
  if (!current.includes(root.canonicalBaseline)) {
    addReason({
      code: "canonical_baseline_missing",
      domain: root.id,
      path: `${root.path}/${root.canonicalBaseline}`,
      version: normalizedVersion(root.canonicalBaseline),
      detail: "canonical source-compatible baseline is absent from the active root",
    });
  }
  for (const alternate of root.alternateBaselines.filter((name) => current.includes(name))) {
    addReason({
      code: "alternate_baseline_active",
      domain: root.id,
      path: `${root.path}/${alternate}`,
      version: normalizedVersion(alternate),
      detail: "alternate baseline must not coexist in an active Diesel root",
    });
  }
  return {
    id: root.id,
    path: root.path,
    canonicalBaseline: root.canonicalBaseline,
    recoveryClass: root.recoveryClass,
    currentMigrations: current.map((name) => ({ name, version: normalizedVersion(name) })),
    sourceMigrations: source.map((name) => ({ name, version: normalizedVersion(name) })),
    currentDuplicates,
    sourceDuplicates,
  };
});

const trustedSource = new Map(contract.trustedSourceFiles.map((entry) => [entry.path, entry.sha256]));
for (const [path, expected] of trustedSource) {
  const actual = sourceSql.get(path);
  if (!actual || sha256(actual) !== expected) {
    addReason({
      code: "trusted_source_hash_mismatch",
      domain: domainByPath.get(path) ?? "all",
      path,
      detail: `pinned source SHA-256 is ${actual ? sha256(actual) : "missing"}; expected ${expected}`,
    });
  }
}

const approvedAdditions = new Map(contract.approvedWorktreeAdditions.map((entry) => [entry.path, entry.sha256]));
const approvedRelocations = new Map(contract.approvedRelocations.map((entry) => [entry.sourcePath, entry]));
const fingerprintPaths = [...new Set([...sourceSql.keys(), ...worktreeSql.keys()])].sort();
const fingerprints: SqlFingerprint[] = [];

for (const path of fingerprintPaths) {
  const domain = domainByPath.get(path) ?? "unknown";
  const source = sourceSql.get(path);
  const worktree = worktreeSql.get(path);
  const sourceHash = source ? sha256(source) : null;
  const worktreeHash = worktree ? sha256(worktree) : null;
  const root = contract.roots.find((item) => item.id === domain);
  const suffix = root ? path.slice(root.path.length + 1) : path;
  const [migration, file] = suffix.split("/");
  const direction = file === "down.sql" ? "down" : "up";
  let classification: SqlFingerprint["classification"];
  let archivePath: string | null = null;

  if (source && worktree) {
    classification = sourceHash === worktreeHash ? "unchanged" : "modified";
    if (classification === "modified") {
      addReason({ code: "historical_sql_modified", domain, path, version: normalizedVersion(migration), detail: "worktree bytes differ from the pinned source commit" });
    }
  } else if (source) {
    const relocation = approvedRelocations.get(path);
    if (relocation) {
      const absoluteArchive = resolve(realRepoRoot, relocation.archivePath);
      const validArchive =
        existsSync(absoluteArchive) &&
        lstatSync(absoluteArchive).isFile() &&
        !lstatSync(absoluteArchive).isSymbolicLink() &&
        sha256(readFileSync(absoluteArchive)) === relocation.sha256 &&
        relocation.sha256 === sourceHash;
      if (validArchive) {
        classification = "relocated";
        archivePath = relocation.archivePath;
      } else {
        classification = "deleted";
        addReason({ code: "approved_relocation_invalid", domain, path, version: normalizedVersion(migration), detail: `approved archive is missing or checksum-invalid: ${relocation.archivePath}` });
      }
    } else {
      classification = "deleted";
      addReason({ code: "historical_sql_deleted", domain, path, version: normalizedVersion(migration), detail: "source SQL is absent and has no checksum-locked archive relocation" });
    }
  } else {
    const approvedHash = approvedAdditions.get(path);
    classification = approvedHash && approvedHash === worktreeHash ? "approved-added" : "added";
    if (classification === "added") {
      addReason({ code: "unreviewed_sql_added", domain, path, version: normalizedVersion(migration), detail: "worktree SQL is absent from the pinned source and not checksum-approved" });
    }
  }

  if (worktree && direction === "up") {
    const uncommented = worktree
      .toString("utf8")
      .replace(/\/\*[\s\S]*?\*\//g, " ")
      .replace(/--[^\r\n]*/g, " ");
    if (/\b(?:DROP\s+(?:TABLE|SCHEMA)|TRUNCATE(?:\s+TABLE)?|DELETE\s+FROM)\b/i.test(uncommented)) {
      addReason({ code: "destructive_forward_sql", domain, path, version: normalizedVersion(migration), detail: "up.sql contains DROP TABLE/SCHEMA, TRUNCATE, or DELETE FROM" });
    }
    if (/\bSET\s+(?:LOCAL\s+)?search_path\b/i.test(uncommented)) {
      addReason({ code: "search_path_mutation", domain, path, version: normalizedVersion(migration), detail: "migration mutates the session search_path used by Diesel history recording" });
    }
    if (/\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM|ALTER\s+TABLE|DROP\s+TABLE|TRUNCATE(?:\s+TABLE)?)\s+(?:[\w".]+\.)?__diesel_schema_migrations\b/i.test(uncommented)) {
      addReason({ code: "migration_table_mutation", domain, path, version: normalizedVersion(migration), detail: "migration SQL must not mutate Diesel history directly" });
    }
  }

  fingerprints.push({
    domain,
    path,
    migration,
    version: normalizedVersion(migration),
    direction,
    classification,
    sourceSha256: sourceHash,
    worktreeSha256: worktreeHash,
    archivePath,
  });
}

for (const [path] of approvedAdditions) {
  if (!worktreeSql.has(path)) {
    addReason({ code: "approved_addition_missing", domain: domainByPath.get(path) ?? "all", path, detail: "checksum-approved worktree addition is absent" });
  }
}

reasons.sort((left, right) =>
  left.code.localeCompare(right.code) ||
  left.domain.localeCompare(right.domain) ||
  (left.path ?? "").localeCompare(right.path ?? "") ||
  (left.version ?? "").localeCompare(right.version ?? "") ||
  left.detail.localeCompare(right.detail),
);

const reportCore = {
  schemaVersion: 1,
  package: "A3.1a",
  purpose: "static-diesel-history-preflight",
  productionReady: false,
  source: { ref: contract.source.ref, pinnedCommit: contract.source.commit, resolvedCommit: resolvedSourceRef },
  dieselVersionRule: "directory prefix before first underscore, with every hyphen removed",
  status: reasons.length === 0 ? "static-clear-database-evidence-still-required" : "stop",
  roots: rootReports,
  sqlFingerprints: fingerprints,
  stopReasons: reasons,
  recoveryMatrix: contract.recoveryMatrix,
};
const report = {
  ...reportCore,
  deterministicSha256: sha256(`${JSON.stringify(reportCore)}\n`),
};
const serialized = `${JSON.stringify(report, null, 2)}\n`;

if (outputPath === "-") {
  process.stdout.write(serialized);
} else {
  if (outputPath.includes("\0") || !isAbsolute(outputPath)) dieUsage("--output must be an absolute path or '-'");
  const outputName = basename(outputPath);
  if (outputName === "." || outputName === "..") dieUsage("--output must name a file");
  const parent = dirname(resolve(outputPath));
  if (!existsSync(parent) || !statSync(parent).isDirectory()) dieUsage("--output parent directory must already exist");
  const resolvedOutput = resolve(realpathSync(parent), outputName);
  if (inside(realRepoRoot, resolvedOutput)) dieUsage("--output must be outside the repository");
  if (existsSync(resolvedOutput)) dieUsage("--output refuses to overwrite an existing path");
  const descriptor = openSync(resolvedOutput, "wx", 0o600);
  try {
    writeFileSync(descriptor, serialized, { encoding: "utf8" });
  } finally {
    closeSync(descriptor);
  }
}

process.exit(reasons.length === 0 ? 0 : 2);
