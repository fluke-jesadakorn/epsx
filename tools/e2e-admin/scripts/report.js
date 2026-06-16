// report.js — assemble tools/e2e-admin/report.md from per-slug JSON diffs.
//
// Reads:
//   <diff-dir>/<slug>.json  (one per slug)
//   <prod-dir>/_summary.json   optional, written by capture-prod-admin.sh
//   <dev-dir>/_summary.json    optional, written by capture-dev-admin.sh
//
// Emits:
//   <out>                    Markdown report
//
// Usage:
//   node report.js --diff-dir D --prod-dir P --dev-dir V --routes <csv> --out R

const fs = require("fs");
const path = require("path");

const args = (() => {
  const a = {};
  for (let i = 2; i < process.argv.length; i += 2) {
    a[process.argv[i].replace(/^--/, "").replace(/-/g, "_")] = process.argv[i + 1];
  }
  return a;
})();

const DIFF = args.diff_dir;
const PROD = args.prod_dir;
const DEV = args.dev_dir;
const ROUTES_CSV = args.routes || "";
const OUT = args.out;
const ROUTES_PATH = path.join(__dirname, "routes.json");

if (!DIFF || !PROD || !DEV || !OUT) {
  console.error("usage: node report.js --diff-dir D --prod-dir P --dev-dir V --routes CSV --out R");
  process.exit(1);
}

const routes = JSON.parse(fs.readFileSync(ROUTES_PATH, "utf8")).routes;
const wantedSlugs = ROUTES_CSV === "all"
  ? new Set(routes.map((r) => r.slug))
  : new Set(ROUTES_CSV.split(",").map((s) => s.trim()).filter(Boolean));

const slugToPath = new Map(routes.map((r) => [r.slug, r.path]));

const rows = [];
const issuesByKind = new Map();
let totalPixelDiff = 0;
let totalPx = 0;
const missingArtifacts = [];

// Load _summary.json (capture status) for prod and dev
function loadSummary(dir) {
  const p = path.join(dir, "_summary.json");
  if (!fs.existsSync(p)) return new Map();
  try {
    const j = JSON.parse(fs.readFileSync(p, "utf8"));
    const m = new Map();
    for (const r of j.routes || []) m.set(r.slug, r);
    return m;
  } catch (_) {
    return new Map();
  }
}
const prodSummary = loadSummary(PROD);
const devSummary = loadSummary(DEV);

for (const route of routes) {
  if (!wantedSlugs.has(route.slug)) continue;
  const jsonPath = path.join(DIFF, `${route.slug}.json`);
  const prodCap = prodSummary.get(route.slug) || {};
  const devCap = devSummary.get(route.slug) || {};
  if (!fs.existsSync(jsonPath)) {
    missingArtifacts.push(route.slug);
    rows.push({
      slug: route.slug,
      path: route.path,
      prodStatus: prodCap.status || "N/A",
      devStatus: devCap.status || "N/A",
      pixelDiffPct: "N/A",
      consoleErrorsDev: "N/A",
      brokenLinksDev: "N/A",
      brokenButtonsDev: "N/A",
      missingComponents: "N/A",
      notes: "no diff (artifacts missing)",
    });
    continue;
  }
  const r = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
  let brokenLinks = 0;
  let brokenButtons = 0;
  let missingComponents = 0;
  let dev404 = false;
  for (const iss of r.issues || []) {
    issuesByKind.set(iss.kind, (issuesByKind.get(iss.kind) || 0) + (iss.count || 1));
    if (iss.kind === "missing-hrefs") brokenLinks = iss.count;
    if (iss.kind === "missing-buttons") brokenButtons = iss.count;
    if (iss.kind === "click-does-not-navigate") brokenButtons += iss.count;
    if (iss.kind === "broken-clicks") brokenButtons += iss.count;
    if (iss.kind === "missing-hrefs") missingComponents += iss.count;
    if (iss.kind === "dev-returns-404") dev404 = true;
  }
  rows.push({
    slug: route.slug,
    path: route.path,
    prodStatus: prodCap.status || "?",
    devStatus: devCap.status || (dev404 ? "404" : "?"),
    pixelDiffPct: r.pixelDiff ? r.pixelDiff.pct : 0,
    consoleErrorsDev: r.devConsoleErrors || 0,
    brokenLinksDev: brokenLinks,
    brokenButtonsDev: brokenButtons,
    missingComponents: missingComponents,
    dev404,
    issues: r.issues || [],
  });
  if (r.pixelDiff) {
    totalPixelDiff += r.pixelDiff.count;
    totalPx += (1280 * 800);
  }
}

// ----- Build Markdown -----
let md = "";
md += "# Wave 24 T1' — Admin E2E Component Interaction Report\n\n";
md += `Generated: ${new Date().toISOString()}\n\n`;
md += `Prod: \`https://admin.epsx.io\`  |  Dev: BFF port-forward (default \`localhost:3001\`)\n\n`;
md += "## Per-route summary\n\n";
md += "| # | Slug | Path | prod_status | dev_status | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components | notes |\n";
md += "|---|------|------|------------:|-----------:|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|-------|\n";
let i = 1;
for (const row of rows) {
  const note = row.dev404 ? "dev returns 404 (admin Dioxus shell mostly empty — expected)"
    : (row.notes || "");
  md += `| ${i++} | \`${row.slug}\` | \`${row.path}\` | ${row.prodStatus} | ${row.devStatus} | ${row.pixelDiffPct} | ${row.consoleErrorsDev} | ${row.brokenLinksDev} | ${row.brokenButtonsDev} | ${row.missingComponents} | ${note} |\n`;
}
md += "\n";

md += "## Issue digest (aggregated across routes)\n\n";
if (issuesByKind.size === 0) {
  md += "_no cross-route issues detected_\n\n";
} else {
  md += "| Kind | Total occurrences | Routes affected |\n";
  md += "|------|------------------:|----------------:|\n";
  for (const [kind, count] of [...issuesByKind.entries()].sort((a, b) => b[1] - a[1])) {
    let routesAffected = 0;
    for (const row of rows) {
      if (row.issues && row.issues.find((i) => i.kind === kind)) routesAffected += 1;
    }
    md += `| \`${kind}\` | ${count} | ${routesAffected} |\n`;
  }
  md += "\n";
}

md += "## Top 5 issues (by occurrence)\n\n";
const top = [...issuesByKind.entries()].sort((a, b) => b[1] - a[1]).slice(0, 5);
if (top.length === 0) {
  md += "_no issues_\n\n";
} else {
  for (const [kind, count] of top) {
    md += `### \`${kind}\` — ${count} occurrences\n\n`;
    const samples = [];
    for (const row of rows) {
      if (!row.issues) continue;
      for (const iss of row.issues) {
        if (iss.kind === kind) {
          samples.push({ slug: row.slug, ...iss });
        }
      }
    }
    md += "Affected routes (first 10):\n\n";
    for (const s of samples.slice(0, 10)) {
      md += `- \`${s.slug}\``;
      if (s.samples && s.samples.length) {
        md += ` — sample: \`${JSON.stringify(s.samples[0]).slice(0, 200)}\``;
      } else if (s.prodFinal || s.devFinal) {
        md += ` — prod→\`${s.prodFinal}\` dev→\`${s.devFinal}\``;
      }
      md += "\n";
    }
    md += "\n";
  }
}

md += "## Skipped / missing artifacts\n\n";
if (missingArtifacts.length === 0) {
  md += "_none_\n\n";
} else {
  for (const s of missingArtifacts) md += `- \`${s}\`\n`;
  md += "\n";
}

md += "## How to reproduce\n\n";
md += "```bash\n";
md += "# 1. Capture prod (10-15 min)\n";
md += "bash tools/e2e-admin/capture-prod-admin.sh\n\n";
md += "# 2. Capture dev (10-15 min, requires port-forward 3001)\n";
md += "kubectl port-forward -n epsx-dev svc/epsx-admin 3001:3000 &\n";
md += "EPSX_DEV_AUTH_BYPASS=1 bash tools/e2e-admin/capture-dev-admin.sh\n\n";
md += "# 3. Diff + report (~5 min)\n";
md += "bash tools/e2e-admin/diff-admin.sh\n\n";
md += "# 4. Subset (e.g. smoke 3 routes)\n";
md += "bash tools/e2e-admin/capture-prod-admin.sh admin-dashboard,admin-settings,admin-policies\n";
md += "bash tools/e2e-admin/capture-dev-admin.sh admin-dashboard,admin-settings,admin-policies\n";
md += "bash tools/e2e-admin/diff-admin.sh admin-dashboard,admin-settings,admin-policies\n";
md += "```\n";

fs.writeFileSync(OUT, md);
console.log(`wrote ${OUT} (${rows.length} rows, ${issuesByKind.size} issue kinds)`);
