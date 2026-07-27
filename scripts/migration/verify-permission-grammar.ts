#!/usr/bin/env bun

import { existsSync, readFileSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const READINESS_STOP_EXIT = 3;
const USAGE_EXIT = 64;
const INVENTORY_FILE = "docs/migration/contracts/permission-grammar.json";
const SERVICE_AUTH_FILE = "docs/migration/contracts/service-authorization.json";
const UI_GLOB = "shared/rust/dioxus_ui/src/pages/**/*.rs";

const classifications = [
  "canonical-3-segment",
  "wildcard-aligned",
  "legacy-2-segment",
  "unknown",
  "impossible/cross-grammar",
] as const;

type Classification = (typeof classifications)[number];
type SourceType =
  | "dioxus-security-gate"
  | "dioxus-presentation-literal"
  | "dioxus-presentation-dynamic"
  | "service-authorization";

type InventoryProjection = {
  sourceType: SourceType;
  file: string;
  line: number;
  permission: string;
  surface: string;
  classification: Classification;
};

type Evidence = { file: string; anchor: string; purpose?: string };
type Remediation = {
  package: "A7" | "A8";
  candidates: string[];
  requiresSplit: boolean;
  evidence: string[];
  decision: string;
};

type InventoryRecord = InventoryProjection & {
  remediation: Remediation | null;
};

type Fixture = {
  schemaVersion: number;
  artifact: string;
  sourceBaseline: { ref: string; commit: string };
  readinessStopExit: number;
  authorityBoundary: {
    policyAuthority: string;
    uiPolicyAuthority: boolean;
    canonicalGrammar: string;
    canonicalOwnerPackage: string;
  };
  scanScope: {
    uiGlob: string;
    uiLiteralPattern: string;
    dynamicPassThroughPattern: string;
    securityGateComponents: string[];
    presentationComponent: string;
    serviceAuthorizationFile: string;
  };
  evidenceCatalog: Record<string, Evidence>;
  inventory: InventoryRecord[];
  summary: {
    total: number;
    sourceCounts: Record<string, number>;
    classificationCounts: Record<Classification, number>;
  };
};

type CliOptions = {
  mode: "integrity" | "readiness";
  fixturePath: string;
  repoRoot: string;
  emitInventory: boolean;
  checkRef: boolean;
};

function usage(message?: string): never {
  if (message) console.error(`permission-grammar: ${message}`);
  console.error(
    "usage: verify-permission-grammar.ts [--mode integrity|readiness] [--fixture PATH] [--root PATH] [--emit-inventory] [--skip-ref-check]",
  );
  process.exit(USAGE_EXIT);
}

function parseArgs(): CliOptions {
  const scriptDir = dirname(fileURLToPath(import.meta.url));
  const defaultRoot = resolve(scriptDir, "../..");
  let mode: CliOptions["mode"] = "integrity";
  let fixturePath = resolve(defaultRoot, INVENTORY_FILE);
  let repoRoot = defaultRoot;
  let emitInventory = false;
  let checkRef = true;

  for (let index = 2; index < process.argv.length; index += 1) {
    const arg = process.argv[index];
    if (arg === "--mode") {
      const value = process.argv[++index];
      if (value !== "integrity" && value !== "readiness") usage(`invalid mode: ${value ?? "<missing>"}`);
      mode = value;
    } else if (arg === "--fixture") {
      const value = process.argv[++index];
      if (!value) usage("--fixture requires a path");
      fixturePath = resolve(value);
    } else if (arg === "--root") {
      const value = process.argv[++index];
      if (!value) usage("--root requires a path");
      repoRoot = resolve(value);
    } else if (arg === "--emit-inventory") {
      emitInventory = true;
    } else if (arg === "--skip-ref-check") {
      checkRef = false;
    } else {
      usage(`unknown argument: ${arg}`);
    }
  }

  if (!isAbsolute(fixturePath) || !isAbsolute(repoRoot)) usage("paths must resolve to absolute paths");
  return { mode, fixturePath, repoRoot, emitInventory, checkRef };
}

function classify(permission: string): Classification {
  if (permission === "<dynamic-pass-through>") return "unknown";
  const segments = permission.split(":");
  if (permission === "*:*" || permission === "*:*:*") return "wildcard-aligned";
  if (segments.length === 2 && segments.includes("*")) return "impossible/cross-grammar";
  if (segments.length === 2 && segments.every((segment) => segment.length > 0 && segment !== "*")) {
    return "legacy-2-segment";
  }
  if (segments.length === 3 && segments.every((segment) => segment.length > 0)) {
    if (!segments.includes("*")) return "canonical-3-segment";
    const platformWildcard = segments[0] !== "*" && segments[1] === "*" && segments[2] === "*";
    const resourceWildcard = segments[0] !== "*" && segments[1] !== "*" && segments[2] === "*";
    return platformWildcard || resourceWildcard ? "wildcard-aligned" : "impossible/cross-grammar";
  }
  return "unknown";
}

function uiSurface(file: string): string {
  const stem = file.slice(file.lastIndexOf("/") + 1).replace(/\.rs$/, "");
  return `${file.includes("/admin_pages/") ? "admin" : "frontend"}:${stem}`;
}

function lineNumber(text: string, offset: number): number {
  return text.slice(0, offset).split("\n").length;
}

function scanUi(repoRoot: string): InventoryProjection[] {
  const paths = [...new Bun.Glob(UI_GLOB).scanSync({ cwd: repoRoot })].sort();
  const records: InventoryProjection[] = [];

  for (const file of paths) {
    const absolute = resolve(repoRoot, file);
    const text = readFileSync(absolute, "utf8");
    const recognizedOccurrences = new Set<number>();
    const literalPattern = /required_permissions:\s*Some\s*\(\s*vec!\s*\[([^\]]*)\]\s*\)/gms;
    for (const match of text.matchAll(literalPattern)) {
      recognizedOccurrences.add(match.index ?? 0);
      const body = match[1] ?? "";
      const bodyOffset = (match.index ?? 0) + match[0].indexOf(body);
      const context = text.slice(Math.max(0, (match.index ?? 0) - 500), match.index ?? 0);
      const accessDeniedOffset = context.lastIndexOf("AccessDenied {");
      const authGateOffset = Math.max(context.lastIndexOf("AuthGate {"), context.lastIndexOf("AdminAuthGate {"));
      const sourceType: SourceType =
        accessDeniedOffset > authGateOffset ? "dioxus-presentation-literal" : "dioxus-security-gate";
      const permissionPattern = /"([^"]+)"/g;
      for (const permissionMatch of body.matchAll(permissionPattern)) {
        const permission = permissionMatch[1];
        records.push({
          sourceType,
          file,
          line: lineNumber(text, bodyOffset + (permissionMatch.index ?? 0)),
          permission,
          surface: uiSurface(file),
          classification: classify(permission),
        });
      }
    }

    const dynamicPattern = /required_permissions:\s*required_permissions\s*,/g;
    for (const match of text.matchAll(dynamicPattern)) {
      recognizedOccurrences.add(match.index ?? 0);
      const context = text.slice(Math.max(0, (match.index ?? 0) - 500), match.index ?? 0);
      const accessDeniedOffset = context.lastIndexOf("AccessDenied {");
      const authGateOffset = Math.max(context.lastIndexOf("AuthGate {"), context.lastIndexOf("AdminAuthGate {"));
      records.push({
        sourceType: accessDeniedOffset > authGateOffset ? "dioxus-presentation-dynamic" : "dioxus-security-gate",
        file,
        line: lineNumber(text, match.index ?? 0),
        permission: "<dynamic-pass-through>",
        surface: uiSurface(file),
        classification: "unknown",
      });
    }

    const occurrencePattern = /required_permissions\s*:/g;
    for (const match of text.matchAll(occurrencePattern)) {
      const offset = match.index ?? 0;
      if (recognizedOccurrences.has(offset)) continue;
      const tail = text.slice(offset, offset + 160);
      if (/^required_permissions\s*:\s*None\b/.test(tail)) continue;
      throw new Error(`${file}:${lineNumber(text, offset)} has an unclassified required_permissions occurrence`);
    }
  }

  return records;
}

function findJsonStringLine(text: string, value: string, startOffset: number): { line: number; nextOffset: number } {
  const needle = JSON.stringify(value);
  const offset = text.indexOf(needle, startOffset);
  if (offset < 0) throw new Error(`unable to locate JSON string ${needle} after offset ${startOffset}`);
  return { line: lineNumber(text, offset), nextOffset: offset + needle.length };
}

function scanServiceAuthorization(repoRoot: string): InventoryProjection[] {
  const file = SERVICE_AUTH_FILE;
  const absolute = resolve(repoRoot, file);
  const text = readFileSync(absolute, "utf8");
  const fixture = JSON.parse(text) as {
    services?: Array<{
      name: string;
      routes?: Array<{ id: string; requiredPermission?: string | null }>;
    }>;
  };
  if (!Array.isArray(fixture.services)) throw new Error(`${file} does not contain a services array`);

  const records: InventoryProjection[] = [];
  let cursor = 0;
  for (const service of fixture.services) {
    if (!Array.isArray(service.routes)) throw new Error(`${file} service ${service.name} does not contain a routes array`);
    for (const route of service.routes) {
      if (!route.requiredPermission) continue;
      const routeLocation = findJsonStringLine(text, route.id, cursor);
      const permissionLocation = findJsonStringLine(text, route.requiredPermission, routeLocation.nextOffset);
      cursor = permissionLocation.nextOffset;
      records.push({
        sourceType: "service-authorization",
        file,
        line: permissionLocation.line,
        permission: route.requiredPermission,
        surface: `service:${service.name}:${route.id}`,
        classification: classify(route.requiredPermission),
      });
    }
  }
  return records;
}

function recordKey(record: InventoryProjection): string {
  return [record.sourceType, record.file, String(record.line).padStart(8, "0"), record.permission, record.surface].join("\u0000");
}

function sorted(records: InventoryProjection[]): InventoryProjection[] {
  return [...records].sort((left, right) => recordKey(left).localeCompare(recordKey(right)));
}

function project(record: InventoryRecord): InventoryProjection {
  return {
    sourceType: record.sourceType,
    file: record.file,
    line: record.line,
    permission: record.permission,
    surface: record.surface,
    classification: record.classification,
  };
}

function safeSourcePath(repoRoot: string, file: string): string | null {
  if (isAbsolute(file)) return null;
  const absolute = resolve(repoRoot, file);
  const relation = relative(repoRoot, absolute);
  if (relation === ".." || relation.startsWith(`..${process.platform === "win32" ? "\\" : "/"}`)) return null;
  return absolute;
}

function verifyEvidence(repoRoot: string, catalog: Record<string, Evidence>, errors: string[]): Map<string, string> {
  const sourceById = new Map<string, string>();
  for (const [id, evidence] of Object.entries(catalog).sort(([left], [right]) => left.localeCompare(right))) {
    if (!evidence || typeof evidence.file !== "string" || typeof evidence.anchor !== "string" || !evidence.anchor) {
      errors.push(`evidence ${id} must have non-empty file and anchor fields`);
      continue;
    }
    const sourcePath = safeSourcePath(repoRoot, evidence.file);
    if (!sourcePath || !existsSync(sourcePath)) {
      errors.push(`evidence ${id} has missing or unsafe source path: ${evidence.file}`);
      continue;
    }
    const source = readFileSync(sourcePath, "utf8");
    if (!source.includes(evidence.anchor)) errors.push(`evidence ${id} anchor not found in ${evidence.file}`);
    sourceById.set(id, evidence.anchor);
  }
  return sourceById;
}

function expectedCounts(records: InventoryProjection[]) {
  const sourceCounts: Record<string, number> = {
    "dioxus-security-gate": 0,
    "dioxus-presentation-literal": 0,
    "dioxus-presentation-dynamic": 0,
    "service-authorization": 0,
  };
  const classificationCounts = Object.fromEntries(classifications.map((value) => [value, 0])) as Record<Classification, number>;
  for (const record of records) {
    sourceCounts[record.sourceType] = (sourceCounts[record.sourceType] ?? 0) + 1;
    classificationCounts[record.classification] += 1;
  }
  return { total: records.length, sourceCounts, classificationCounts };
}

function sameJson(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function validateFixture(
  fixture: Fixture,
  actual: InventoryProjection[],
  repoRoot: string,
  checkRef: boolean,
): string[] {
  const errors: string[] = [];
  const classifierContract: Array<[string, Classification]> = [
    ["*:*", "wildcard-aligned"],
    ["*:*:*", "wildcard-aligned"],
    ["admin:*:*", "wildcard-aligned"],
    ["admin:users:*", "wildcard-aligned"],
    ["admin:*:read", "impossible/cross-grammar"],
    ["*:users:read", "impossible/cross-grammar"],
    ["admin:*", "impossible/cross-grammar"],
  ];
  for (const [permission, expected] of classifierContract) {
    if (classify(permission) !== expected) {
      errors.push(`internal classifier contract failed for ${permission}: expected ${expected}`);
    }
  }
  if (fixture.schemaVersion !== 1) errors.push("schemaVersion must be 1");
  if (fixture.artifact !== "permission-grammar-audit") errors.push("artifact must be permission-grammar-audit");
  if (fixture.readinessStopExit !== READINESS_STOP_EXIT) {
    errors.push(`readinessStopExit must reserve ${READINESS_STOP_EXIT}`);
  }
  if (fixture.sourceBaseline?.ref !== "development" || !/^[0-9a-f]{40}$/.test(fixture.sourceBaseline?.commit ?? "")) {
    errors.push("sourceBaseline must pin development to a 40-character commit");
  }
  if (
    fixture.authorityBoundary?.policyAuthority !== "rust-backend-only" ||
    fixture.authorityBoundary?.uiPolicyAuthority !== false ||
    fixture.authorityBoundary?.canonicalGrammar !== "platform:resource:action" ||
    fixture.authorityBoundary?.canonicalOwnerPackage !== "A4"
  ) {
    errors.push("authorityBoundary must keep canonical permission policy in the Rust backend under A4");
  }
  if (
    fixture.scanScope?.uiGlob !== UI_GLOB ||
    fixture.scanScope?.uiLiteralPattern !== "required_permissions: Some(vec![...])" ||
    fixture.scanScope?.dynamicPassThroughPattern !== "required_permissions: required_permissions" ||
    !sameJson(fixture.scanScope?.securityGateComponents, ["AuthGate", "AdminAuthGate"]) ||
    fixture.scanScope?.presentationComponent !== "AccessDenied" ||
    fixture.scanScope?.serviceAuthorizationFile !== SERVICE_AUTH_FILE
  ) {
    errors.push("scanScope does not match the executable Dioxus literals and service authorization fixture");
  }

  if (checkRef) {
    const result = Bun.spawnSync(["git", "-C", repoRoot, "rev-parse", "development"], {
      stdout: "pipe",
      stderr: "pipe",
    });
    if (result.exitCode !== 0) {
      errors.push("unable to resolve development for sourceBaseline verification");
    } else {
      const resolvedRef = new TextDecoder().decode(result.stdout).trim();
      if (resolvedRef !== fixture.sourceBaseline.commit) {
        errors.push(`development moved: fixture=${fixture.sourceBaseline.commit} actual=${resolvedRef}`);
      }
    }
  }

  const fixtureRecords = Array.isArray(fixture.inventory) ? sorted(fixture.inventory.map(project)) : [];
  const actualRecords = sorted(actual);
  if (!sameJson(fixtureRecords, actualRecords)) {
    const fixtureKeys = new Set(fixtureRecords.map(recordKey));
    const actualKeys = new Set(actualRecords.map(recordKey));
    for (const key of [...actualKeys].filter((value) => !fixtureKeys.has(value)).sort()) {
      errors.push(`inventory missing current source record: ${key.replaceAll("\u0000", " | ")}`);
    }
    for (const key of [...fixtureKeys].filter((value) => !actualKeys.has(value)).sort()) {
      errors.push(`inventory contains stale source record: ${key.replaceAll("\u0000", " | ")}`);
    }
    if (errors.length === 0) errors.push("inventory projections differ");
  }

  const keys = fixtureRecords.map(recordKey);
  if (new Set(keys).size !== keys.length) errors.push("inventory contains duplicate source records");
  const counts = expectedCounts(actualRecords);
  if (!sameJson(fixture.summary, counts)) errors.push("summary does not match the scanned inventory counts");

  const evidenceSource = verifyEvidence(repoRoot, fixture.evidenceCatalog ?? {}, errors);
  for (const requiredEvidence of ["grammar-format", "grammar-wildcards"]) {
    if (!fixture.evidenceCatalog?.[requiredEvidence]) {
      errors.push(`missing canonical grammar evidence: ${requiredEvidence}`);
    }
  }
  for (const record of fixture.inventory ?? []) {
    const expectedClass = classify(record.permission);
    if (record.classification !== expectedClass) {
      errors.push(`${record.surface} ${record.permission} classification must be ${expectedClass}`);
    }
    if (record.sourceType === "service-authorization") {
      if (record.remediation !== null) errors.push(`${record.surface} is canonical service evidence and must not prescribe UI remediation`);
      if (record.classification !== "canonical-3-segment" && record.classification !== "wildcard-aligned") {
        errors.push(`${record.surface} has a non-canonical service authorization permission`);
      }
      continue;
    }

    const remediation = record.remediation;
    if (!remediation || !["A7", "A8"].includes(remediation.package)) {
      errors.push(`${record.surface} must assign remediation to A7 or A8`);
      continue;
    }
    const expectedPackage = record.surface.startsWith("admin:") ? "A8" : "A7";
    if (remediation.package !== expectedPackage) {
      errors.push(`${record.surface} remediation belongs to ${expectedPackage}, not ${remediation.package}`);
    }
    if (!Array.isArray(remediation.candidates) || !Array.isArray(remediation.evidence) || !remediation.decision) {
      errors.push(`${record.surface} remediation fields are incomplete`);
      continue;
    }
    if (remediation.requiresSplit !== (remediation.candidates.length > 1)) {
      errors.push(`${record.surface} requiresSplit must be true only for multiple source-backed candidates`);
    }
    for (const evidenceId of remediation.evidence) {
      if (!fixture.evidenceCatalog?.[evidenceId]) errors.push(`${record.surface} references missing evidence ${evidenceId}`);
    }
    for (const candidate of remediation.candidates) {
      const candidateClass = classify(candidate);
      if (candidateClass !== "canonical-3-segment" && candidateClass !== "wildcard-aligned") {
        errors.push(`${record.surface} candidate is not canonical: ${candidate}`);
        continue;
      }
      const supported = remediation.evidence.some((id) => evidenceSource.get(id)?.includes(candidate));
      if (!supported) errors.push(`${record.surface} candidate lacks a literal source guard/token anchor: ${candidate}`);
    }
    if (
      record.classification === "canonical-3-segment" &&
      remediation.candidates.length === 1 &&
      record.permission !== remediation.candidates[0]
    ) {
      errors.push(`${record.surface} canonical UI gate does not consume its sole source-backed candidate`);
    }
  }
  return errors;
}

const options = parseArgs();
let actual: InventoryProjection[];
try {
  actual = sorted([...scanUi(options.repoRoot), ...scanServiceAuthorization(options.repoRoot)]);
} catch (error) {
  console.error(`permission-grammar: scan failed: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}

if (options.emitInventory) {
  console.log(JSON.stringify({ inventory: actual, summary: expectedCounts(actual) }, null, 2));
  process.exit(0);
}

let fixture: Fixture;
try {
  fixture = JSON.parse(readFileSync(options.fixturePath, "utf8")) as Fixture;
} catch (error) {
  console.error(`permission-grammar: unable to read fixture ${options.fixturePath}: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}

const errors = validateFixture(fixture, actual, options.repoRoot, options.checkRef);
if (errors.length > 0) {
  console.error("permission-grammar integrity: FAIL");
  for (const error of errors) console.error(`- ${error}`);
  process.exit(1);
}

if (options.mode === "readiness") {
  const blockers = actual.filter(
    (record) =>
      record.sourceType === "dioxus-security-gate" &&
      record.classification !== "canonical-3-segment" &&
      record.classification !== "wildcard-aligned",
  );
  const presentationDrift = actual.filter(
    (record) =>
      (record.sourceType === "dioxus-presentation-literal" || record.sourceType === "dioxus-presentation-dynamic") &&
      record.classification !== "canonical-3-segment" &&
      record.classification !== "wildcard-aligned",
  );
  if (blockers.length > 0) {
    const blockerCounts = expectedCounts(blockers).classificationCounts;
    console.error(
      `permission-grammar readiness: STOP (${blockers.length} security-gate blockers; legacy=${blockerCounts["legacy-2-segment"]}, unknown=${blockerCounts.unknown}, impossible=${blockerCounts["impossible/cross-grammar"]}; presentation-drift=${presentationDrift.length})`,
    );
    process.exit(READINESS_STOP_EXIT);
  }
  console.log("permission-grammar readiness: PASS");
  process.exit(0);
}

console.log(
  `permission-grammar integrity: PASS (${actual.length} records; source baseline ${fixture.sourceBaseline.ref}@${fixture.sourceBaseline.commit})`,
);
