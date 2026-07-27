import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve } from "node:path";

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

const uniqueSection = (source: string, start: string, end: string, label: string): string => {
  const starts = source.split(start).length - 1;
  const ends = source.split(end).length - 1;
  if (starts !== 1 || ends !== 1) die(`${label} boundaries must each occur exactly once`);
  const startOffset = source.indexOf(start);
  const endOffset = source.indexOf(end, startOffset + start.length);
  if (startOffset < 0 || endOffset < 0 || startOffset >= endOffset) die(`${label} boundaries are invalid`);
  return source.slice(startOffset, endOffset);
};

type RustToken = {
  kind: "ident" | "string" | "symbol";
  value: string;
  offset: number;
};

const rustTokens = (source: string, label: string): RustToken[] => {
  const tokens: RustToken[] = [];
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    if (/\s/.test(char)) {
      index += 1;
      continue;
    }
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? source.length : newline + 1;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const commentStart = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      if (depth !== 0) die(`${label} has an unterminated block comment at byte ${commentStart}`);
      continue;
    }
    if (char === "r") {
      let cursor = index + 1;
      while (source[cursor] === "#") cursor += 1;
      if (source[cursor] === '"') {
        const hashes = source.slice(index + 1, cursor);
        const close = `"${hashes}`;
        const valueStart = cursor + 1;
        const closeAt = source.indexOf(close, valueStart);
        if (closeAt < 0) die(`${label} has an unterminated raw string at byte ${index}`);
        tokens.push({ kind: "string", value: source.slice(valueStart, closeAt), offset: index });
        index = closeAt + close.length;
        continue;
      }
    }
    if (char === '"') {
      const stringStart = index;
      index += 1;
      let value = "";
      let closed = false;
      while (index < source.length) {
        const current = source[index];
        if (current === "\\") {
          if (index + 1 >= source.length) break;
          value += source.slice(index, index + 2);
          index += 2;
        } else if (current === '"') {
          index += 1;
          closed = true;
          break;
        } else {
          value += current;
          index += 1;
        }
      }
      if (!closed) die(`${label} has an unterminated string at byte ${stringStart}`);
      tokens.push({ kind: "string", value, offset: stringStart });
      continue;
    }
    if (/[A-Za-z_]/.test(char)) {
      const identStart = index;
      index += 1;
      while (index < source.length && /[A-Za-z0-9_]/.test(source[index])) index += 1;
      tokens.push({ kind: "ident", value: source.slice(identStart, index), offset: identStart });
      continue;
    }
    tokens.push({ kind: "symbol", value: char, offset: index });
    index += 1;
  }
  return tokens;
};

const parseManualFeatureRoutes = (source: string): string[] => {
  const tokens = rustTokens(source, "/manual");
  const declarations: number[] = [];
  for (let index = 0; index + 1 < tokens.length; index += 1) {
    if (tokens[index].kind === "ident" && tokens[index].value === "const" && tokens[index + 1].value === "FEATURES") {
      declarations.push(index);
    }
  }
  if (declarations.length !== 1) {
    const constNames = tokens
      .map((token, index) => token.value === "const" ? tokens[index + 1]?.value : undefined)
      .filter(Boolean);
    die(`/manual must declare exactly one tokenized const FEATURES table, found ${declarations.length}; consts=${JSON.stringify(constNames)}`);
  }
  let cursor = declarations[0];
  const expect = (kind: RustToken["kind"], value: string): RustToken => {
    const token = tokens[cursor];
    if (!token || token.kind !== kind || token.value !== value) {
      die(`/manual FEATURES expected ${kind} ${JSON.stringify(value)} at token ${cursor}, found ${JSON.stringify(token?.value)}`);
    }
    cursor += 1;
    return token;
  };
  expect("ident", "const");
  expect("ident", "FEATURES");
  expect("symbol", ":");
  expect("symbol", "&");
  expect("symbol", "[");
  expect("ident", "ManualFeature");
  expect("symbol", "]");
  expect("symbol", "=");
  expect("symbol", "&");
  expect("symbol", "[");

  const routes: string[] = [];
  const exactFields = ["id", "name", "desc", "route", "screenshots", "category"];
  while (tokens[cursor]?.value !== "]") {
    expect("ident", "ManualFeature");
    expect("symbol", "{");
    let route = "";
    for (const [fieldIndex, field] of exactFields.entries()) {
      expect("ident", field);
      expect("symbol", ":");
      if (field === "screenshots") {
        expect("symbol", "&");
        expect("symbol", "[");
        expect("string", tokens[cursor]?.value ?? "");
        while (tokens[cursor]?.value === ",") {
          cursor += 1;
          if (tokens[cursor]?.value === "]") break;
          if (tokens[cursor]?.kind !== "string") {
            die(`/manual FEATURES screenshots must contain only literal strings at token ${cursor}`);
          }
          cursor += 1;
        }
        expect("symbol", "]");
      } else {
        const value = tokens[cursor];
        if (!value || value.kind !== "string") {
          die(`/manual FEATURES field ${field} must be a literal string at token ${cursor}`);
        }
        cursor += 1;
        if (field === "route") route = value.value;
      }
      if (fieldIndex < exactFields.length - 1) expect("symbol", ",");
      else if (tokens[cursor]?.value === ",") cursor += 1;
    }
    expect("symbol", "}");
    expect("symbol", ",");
    if (route.length === 0) die("/manual FEATURES route must be a nonempty literal string");
    routes.push(route);
  }
  expect("symbol", "]");
  expect("symbol", ";");
  return routes;
};

export const generateManualRouteStatuses = (routes: Json[]): string => {
  const variants: Record<string, string> = {
    aligned: "Aligned",
    partial: "Partial",
    blocked: "Blocked",
  };
  const rows = routes.map((route) => {
    const variant = variants[route.status];
    if (!variant || typeof route.path !== "string") die("cannot generate /manual status module from invalid contract route");
    return `    ManualRouteStatus {
        target_route: "${route.path}",
        status: RouteMigrationStatus::${variant},
    },`;
  });
  return `// @generated from docs/migration/contracts/frontend-live-data.json
// Checked source: keep this deterministic table byte-comparable with the contract.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RouteMigrationStatus {
    Aligned,
    Partial,
    Blocked,
}

impl RouteMigrationStatus {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Aligned => "Migration aligned",
            Self::Partial => "Migration partial",
            Self::Blocked => "Migration blocked",
        }
    }

    pub(super) const fn token(self) -> &'static str {
        match self {
            Self::Aligned => "aligned",
            Self::Partial => "partial",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ManualRouteStatus {
    pub(super) target_route: &'static str,
    pub(super) status: RouteMigrationStatus,
}

pub(super) const MANUAL_ROUTE_STATUSES: &[ManualRouteStatus] = &[
${rows.join("\n")}
];
`;
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

const manual = contract.routes.find((route: Json) => route.path === "/manual");
if (!manual) die("/manual contract record is required");
const manualUi = currentFile(manual.target?.file, "/manual status presentation target");
const manualTestBoundary = "// === wave5-page-depth-track-b tests + Wave 25 T2 prod markers ===";
if (manualUi.split(manualTestBoundary).length !== 2) die("/manual runtime/test boundary must occur exactly once");
const manualRuntime = manualUi.slice(0, manualUi.indexOf(manualTestBoundary));
const rawFeatureRoutes = parseManualFeatureRoutes(manualRuntime);
if (rawFeatureRoutes.length !== 35) {
  die(`/manual FEATURES must contain exactly 35 literal ManualFeature struct entries, found ${rawFeatureRoutes.length}`);
}
if (
  rawFeatureRoutes.filter((path) => path === "/payment/[type]/[id]").length !== 1 ||
  rawFeatureRoutes.filter((path) => path.includes("[")).length !== 1
) {
  die("/manual must contain exactly one dynamic route, the pinned /payment/[type]/[id] template");
}
const normalizeManualRoute = (path: string): string =>
  path === "/payment/[type]/[id]" ? "/payment/:type/:id" : path;
const manualNormalization = uniqueSection(
  manualRuntime,
  "fn normalize_manual_route(route: &str) -> &str {",
  "fn lookup_manual_route_status(route: &str)",
  "/manual route normalization",
);
if (
  !manualNormalization.includes('"/payment/[type]/[id]" => "/payment/:type/:id"') ||
  !manualNormalization.includes("_ => route") ||
  (manualNormalization.match(/=>/g) ?? []).length !== 2
) {
  die("/manual may normalize only the exact /payment/[type]/[id] route template");
}
const featureRouteCounts = new Map<string, number>();
for (const rawRoute of rawFeatureRoutes) {
  const normalized = normalizeManualRoute(rawRoute);
  featureRouteCounts.set(normalized, (featureRouteCounts.get(normalized) ?? 0) + 1);
}
if (featureRouteCounts.size !== 21) {
  die(`/manual FEATURES must contain exactly 21 unique normalized routes, found ${featureRouteCounts.size}`);
}
const exactDuplicateFeatureRoutes = new Map([
  ["/account", 3],
  ["/profile", 2],
  ["/analytics", 6],
  ["/portfolio", 2],
  ["/notifications", 5],
  ["/developer", 2],
]);
for (const [path, count] of featureRouteCounts) {
  const expectedCount = exactDuplicateFeatureRoutes.get(path) ?? 1;
  if (count !== expectedCount) {
    die(`/manual feature route ${path} must occur ${expectedCount} time(s), found ${count}`);
  }
  if (!expectedByPath.has(path)) die(`/manual feature route is unmapped after exact normalization: ${path}`);
}
for (const [path, expectedCount] of exactDuplicateFeatureRoutes) {
  if (featureRouteCounts.get(path) !== expectedCount) {
    die(`/manual duplicate feature route ${path} must occur exactly ${expectedCount} times`);
  }
}

const manualTargetRel = safeRelative(manual.target?.file, "/manual target file");
const generatedStatusesRel = `${dirname(manualTargetRel)}/manual_route_statuses.rs`;
const actualGeneratedStatuses = currentFile(generatedStatusesRel, "/manual generated route statuses");
const expectedGeneratedStatuses = generateManualRouteStatuses(contract.routes);
if (actualGeneratedStatuses !== expectedGeneratedStatuses) {
  die("/manual generated route status module differs byte-for-byte from the checked contract");
}

const statusByPath = new Map(contract.routes.map((route: Json) => [route.path, route.status]));
const manualBadgeCounts = { aligned: 0, partial: 0, blocked: 0 };
for (const rawRoute of rawFeatureRoutes) {
  const path = normalizeManualRoute(rawRoute);
  const status = statusByPath.get(path);
  if (!status || !(status in manualBadgeCounts)) die(`/manual feature route lacks a checked migration status: ${path}`);
  manualBadgeCounts[status as keyof typeof manualBadgeCounts] += 1;
}
if (
  JSON.stringify(manualBadgeCounts) !==
  JSON.stringify({ aligned: 1, partial: 10, blocked: 24 })
) {
  die(`/manual route-migration badge totals drifted: ${JSON.stringify(manualBadgeCounts)}`);
}
for (const anchor of [
  '#[path = "manual_route_statuses.rs"]',
  "use manual_route_statuses::{ManualRouteStatus, MANUAL_ROUTE_STATUSES};",
  'let dynamic_route = feature.route == "/payment/[type]/[id]";',
  ".map(|status| status.status.label())",
  '.unwrap_or("Migration status unavailable")',
  ".map(|status| status.status.token())",
  '.unwrap_or("unavailable")',
  '"Route template only"',
  '} else if route_status.is_some() {\n        "View route"',
  '"Route status unavailable"',
  "if dynamic_route || route_status.is_none()",
  '"aria-disabled": "true"',
  '"data-manual-route-action": "{status_token}"',
  "} else if let Some(status) = route_status {",
  'href: "{status.target_route}"',
]) {
  if (!manualRuntime.includes(anchor)) die(`/manual route action policy drifted: ${anchor}`);
}
for (const forbidden of [
  "feature.route.contains('[')",
  "ManualRouteAvailability",
  "Reference only",
  "Open page",
]) {
  if (manualRuntime.includes(forbidden)) die(`/manual reintroduced feature-availability or non-neutral action policy: ${forbidden}`);
}

const notifications = contract.routes.find((route: Json) => route.path === "/notifications");
const exactNotificationTargetAnchors = [
  "pub fn render(ctx: &PageContext)",
  "enum NotificationLoad {",
  "fn malformed_and_upstream_states_are_truthful_and_sample_free()",
];
const exactNotificationLoaderEvidence = [
  { file: "apps/frontend/src/ssr.rs", anchor: "let outcome = crate::api::load_owner_notifications(" },
  { file: "apps/frontend/src/ssr.rs", anchor: "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"ok\".into());" },
  { file: "apps/frontend/src/ssr.rs", anchor: "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"error\".into());" },
  { file: "apps/frontend/src/ssr.rs", anchor: "let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);" },
  { file: "apps/frontend/src/ssr.rs", anchor: "var endpoint = '/api/v1/notifications/unread-count';" },
  { file: "shared/rust/templates/src/lib.rs", anchor: "data-epsx-notification-unread-badge=\"true\" data-state=\"unavailable\" aria-hidden=\"true\" hidden></span>" },
];
if (
  !notifications ||
  notifications.status !== "partial" ||
  JSON.stringify(notifications.target?.anchors) !== JSON.stringify(exactNotificationTargetAnchors) ||
  notifications.loader?.kind !== "owner-gateway-explicit-outcome-plus-authenticated-shared-header" ||
  JSON.stringify(notifications.loader?.endpoints) !== JSON.stringify(["GET /api/v1/notification/list", "GET /api/v1/notifications/unread-count"]) ||
  JSON.stringify(notifications.loader?.evidence) !== JSON.stringify(exactNotificationLoaderEvidence) ||
  notifications.payloads?.staticOrSample?.length !== 0 ||
  notifications.states?.loading !== "missing" ||
  notifications.states?.empty !== "present" ||
  notifications.states?.error !== "present" ||
  notifications.states?.retry !== "present" ||
  notifications.authOwner?.auth !== "required" ||
  notifications.hydration?.need !== "browser" ||
  notifications.hydration?.status !== "partial" ||
  notifications.blockers?.length !== 2
) {
  die("/notifications truthful read-only semantic contract drifted");
}
const notificationUi = currentFile(notifications.target.file, "/notifications semantic target");
const notificationUiRuntime = notificationUi.split("#[cfg(test)]", 1)[0];
for (const anchor of [
  "enum RequiredNullable<T> {",
  "fn require(self) -> Result<Option<T>, ()>",
  "created_at: DateTime<Utc>,",
  "read_at: RequiredNullable<DateTime<Utc>>",
  "impl TryFrom<ServiceNotification> for Notification {",
  "let _action_url = value._action_url.require()?;",
  ".unwrap_or_else(|| \"Notification\".to_string());",
  "Some(\"error\") | None => NotificationLoad::UpstreamError",
  "let unread_label = format!(\"{unread_count} unread on this page\");",
]) {
  if (!notificationUiRuntime.includes(anchor)) die(`/notifications missing semantic runtime anchor: ${anchor}`);
}
for (const forbidden of [
  "notifications:read", "sample_notifications", "use_signal(", "onclick:", "notifications-filters",
  "Mark all read", "Clear all", "SwitchInput", "BrowserNotificationsPrompt", "NotificationSettingsSection",
]) {
  if (notificationUiRuntime.includes(forbidden)) die(`/notifications reintroduced blocked UI behavior: ${forbidden}`);
}

const notificationSsr = currentFile("apps/frontend/src/ssr.rs", "/notifications shared-header runtime");
const authDerivation = notificationSsr.indexOf("let is_authenticated = user.is_some();");
const runtimeInjection = notificationSsr.indexOf("let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);");
const bodyInjection = notificationSsr.indexOf("{route_runtime}{authenticated_header_runtime}</body>", runtimeInjection);
const badgeRuntimeStart = notificationSsr.indexOf("fn notification_badge_runtime(is_authenticated: bool, path: &str) -> &'static str {");
const badgeRuntimeEnd = notificationSsr.indexOf("/// Minimal URL-encoder for the `next=` query parameter.", badgeRuntimeStart);
if ([authDerivation, runtimeInjection, bodyInjection, badgeRuntimeStart, badgeRuntimeEnd].some((offset) => offset < 0) || !(authDerivation < runtimeInjection && runtimeInjection < bodyInjection && bodyInjection < badgeRuntimeStart && badgeRuntimeStart < badgeRuntimeEnd)) {
  die("/notifications authenticated shared-header injection boundary drifted");
}
const badgeRuntime = notificationSsr.slice(badgeRuntimeStart, badgeRuntimeEnd);
const authGate = badgeRuntime.indexOf("if !is_authenticated || path == \"/offline\" {");
const emptyReturn = badgeRuntime.indexOf("return \"\";", authGate);
const scriptStart = badgeRuntime.indexOf("data-epsx-notification-badge-runtime", emptyReturn);
if (authGate < 0 || emptyReturn < 0 || scriptStart < 0 || !(authGate < emptyReturn && emptyReturn < scriptStart)) {
  die("/notifications signed-out or public-offline badge exclusion drifted");
}
for (const anchor of [
  "var endpoint = '/api/v1/notifications/unread-count';",
  "cache: 'no-store'",
  "credentials: 'include'",
  "method: 'GET'",
  "Object.getPrototypeOf(payload) !== Object.prototype",
  "keys.length !== 1 || keys[0] !== 'count'",
  "Number.isSafeInteger(payload.count)",
  "if (generation !== requestGeneration || document.hidden || !response.ok) return;",
  "if (generation !== requestGeneration || document.hidden) return;",
  "if (generation === requestGeneration && !document.hidden) setUnavailable();",
  "if (count === 0)",
  "badge.textContent = count > 99 ? '99+' : String(count);",
  "target.setAttribute('aria-label', 'Notifications, ' + String(count) + ' unread');",
  "badge.hidden = true;",
]) {
  if (!badgeRuntime.includes(anchor)) die(`/notifications shared-header badge runtime drifted: ${anchor}`);
}
if ((badgeRuntime.match(/fetch\(/g) ?? []).length !== 1) die("/notifications shared-header badge must use exactly one read fetch");
for (const forbidden of ["innerHTML", "insertAdjacentHTML", "document.write", "method: 'POST'", "method: 'PUT'", "method: 'PATCH'", "method: 'DELETE'", "limit=1", "items.filter"]) {
  if (badgeRuntime.includes(forbidden)) die(`/notifications shared-header badge reintroduced mutation, injection, or fabricated-count behavior: ${forbidden}`);
}
const notificationTemplates = currentFile("shared/rust/templates/src/lib.rs", "/notifications shared-header DOM");
const sharedHeaderStart = notificationTemplates.indexOf("pub fn epsx_header() -> String {");
const sharedHeaderEnd = notificationTemplates.indexOf("/// A standard page shell.", sharedHeaderStart);
if (sharedHeaderStart < 0 || sharedHeaderEnd < 0) die("/notifications shared-header DOM boundaries drifted");
const sharedHeader = notificationTemplates.slice(sharedHeaderStart, sharedHeaderEnd);
for (const anchor of [
  "href=\"/notifications\"",
  "aria-label=\"Notifications\"",
  "data-epsx-notification-badge-target=\"true\"",
  "data-epsx-notification-unread-badge=\"true\" data-state=\"unavailable\" aria-hidden=\"true\" hidden></span>",
]) {
  if (!sharedHeader.includes(anchor)) die(`/notifications shared-header initial DOM drifted: ${anchor}`);
}
for (const forbidden of [">0</span>", "fetch(", "innerHTML", "/api/v1/notifications/unread-count"]) {
  if (sharedHeader.includes(forbidden)) die(`/notifications shared header must start inert and unavailable: ${forbidden}`);
}
const badgeCssStart = notificationTemplates.indexOf(".epsx-notification-badge {{");
const badgeCssEnd = notificationTemplates.indexOf(".epsx-notification-badge[hidden]", badgeCssStart);
if (badgeCssStart < 0 || badgeCssEnd < 0 || badgeCssStart >= badgeCssEnd) {
  die("/notifications shared-header badge CSS boundary drifted");
}
const badgeCss = notificationTemplates.slice(badgeCssStart, badgeCssEnd);
if (!badgeCss.includes("background: #dc2626; color: white;")) {
  die("/notifications shared-header badge lost its reviewed AA text contrast color");
}
if (badgeCss.includes("background: #ef4444; color: white;")) {
  die("/notifications shared-header badge restored the sub-AA text contrast color");
}

const cachePolicyStart = notificationSsr.indexOf("fn apply_ssr_cache_policy(\n    response: &mut Response,\n    is_authenticated: bool,\n    recover_session: bool,\n    auth_page_verifier_unavailable: bool,\n    path: &str,\n) {");
const cachePolicyEnd = notificationSsr.indexOf("/// Fetch page-specific data", cachePolicyStart);
if (cachePolicyStart < 0 || cachePolicyEnd < 0 || cachePolicyStart >= cachePolicyEnd) {
  die("/notifications authenticated SSR cache policy boundary drifted");
}
const cachePolicy = notificationSsr.slice(cachePolicyStart, cachePolicyEnd);
for (const anchor of [
  "if path == \"/offline\" {",
  "HeaderValue::from_static(\"public, max-age=0, must-revalidate\")",
  "} else if is_authenticated || recover_session || auth_page_verifier_unavailable {",
  "HeaderValue::from_static(\"private, no-store\")",
  "HeaderValue::from_static(\"Cookie, Authorization\")",
]) {
  if (!cachePolicy.includes(anchor)) die(`/notifications authenticated SSR cache policy drifted: ${anchor}`);
}
if (!notificationSsr.includes("apply_ssr_cache_policy(\n        &mut response,\n        is_authenticated,\n        recover_session,\n        auth_page_verifier_unavailable,\n        &path,\n    );")) {
  die("/notifications SSR response no longer applies the reviewed private/public cache split");
}

const notificationApi = currentFile("apps/frontend/src/api.rs", "/notifications private BFF responses");
const privateResponseStart = notificationApi.indexOf("fn private_notification_response(");
const privateResponseEnd = notificationApi.indexOf("async fn read_notification_body_limited(", privateResponseStart);
if (privateResponseStart < 0 || privateResponseEnd < 0 || privateResponseStart >= privateResponseEnd) {
  die("/notifications private BFF response boundary drifted");
}
const privateResponse = notificationApi.slice(privateResponseStart, privateResponseEnd);
for (const anchor of ["header::CACHE_CONTROL", "HeaderValue::from_static(\"private, no-store\")"]) {
  if (!privateResponse.includes(anchor)) die(`/notifications private BFF response policy drifted: ${anchor}`);
}
for (const anchor of [
  "private_notification_response(notifications_api_inner(state, headers, raw_query).await)",
  "private_notification_response(notification_unread_count_inner(state, headers).await)",
]) {
  if (!notificationApi.includes(anchor)) die(`/notifications private BFF wrapper drifted: ${anchor}`);
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
