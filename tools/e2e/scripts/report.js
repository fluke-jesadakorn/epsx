// report.js — assemble tools/e2e/report.md from per-slug JSON diffs.
//
// Reads:
//   <diff-dir>/<slug>.json  (one per slug)
//   <prod-dir>/_meta.json   optional, written by capture-prod.sh
//   <dev-dir>/_meta.json    optional, written by capture-dev.sh
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
let missingArtifacts = [];

for (const route of routes) {
  if (!wantedSlugs.has(route.slug)) continue;
  const jsonPath = path.join(DIFF, `${route.slug}.json`);
  if (!fs.existsSync(jsonPath)) {
    missingArtifacts.push(route.slug);
    rows.push({
      slug: route.slug,
      path: route.path,
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
  // count issue kinds
  let brokenLinks = 0;
  let brokenButtons = 0;
  let missingComponents = 0;
  for (const iss of r.issues || []) {
    issuesByKind.set(iss.kind, (issuesByKind.get(iss.kind) || 0) + (iss.count || 1));
    if (iss.kind === "missing-hrefs") brokenLinks = iss.count;
    if (iss.kind === "missing-buttons") brokenButtons = iss.count;
    if (iss.kind === "click-does-not-navigate") brokenButtons += iss.count;
    if (iss.kind === "broken-clicks") brokenButtons += iss.count;
    if (iss.kind === "missing-hrefs") missingComponents += iss.count;
  }
  rows.push({
    slug: route.slug,
    path: route.path,
    pixelDiffPct: r.pixelDiff ? r.pixelDiff.pct : 0,
    consoleErrorsDev: r.devConsoleErrors || 0,
    brokenLinksDev: brokenLinks,
    brokenButtonsDev: brokenButtons,
    missingComponents: missingComponents,
    issues: r.issues || [],
    skipped: r.skipped || false,
    skipReason: r.skipReason || null,
  });
  if (r.skipped) {
    // Don't include skipped routes in the "structural divergence" roll-up;
    // the report's bottom table lists them with a `SKIP` badge.
    continue;
  }
  if (r.pixelDiff) {
    totalPixelDiff += r.pixelDiff.count;
    totalPx += (1280 * 800); // 1 PNG
  }
}

// ----- Build Markdown -----
let md = "";
md += "# Wave 23 T1 — E2E Component Interaction Report\n\n";
md += `Generated: ${new Date().toISOString()}\n\n`;
md += `Prod: \`https://epsx.io\`  |  Dev: BFF port-forward (default \`localhost:30101\`)\n\n`;
md += "## Per-route summary\n\n";
md += "| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |\n";
md += "|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|\n";
let i = 1;
for (const row of rows) {
  const skipTag = row.skipped ? " *(SKIP)*" : "";
  md += `| ${i++} | \`${row.slug}\`${skipTag} | \`${row.path}\` | ${row.pixelDiffPct} | ${row.consoleErrorsDev} | ${row.brokenLinksDev} | ${row.brokenButtonsDev} | ${row.missingComponents} |\n`;
}
md += "\n";

const skippedRows = rows.filter((r) => r.skipped);
if (skippedRows.length > 0) {
  md += "## Skipped routes (intentional — see `routes-skip.json`)\n\n";
  md += "| Slug | Reason |\n";
  md += "|------|--------|\n";
  for (const r of skippedRows) {
    md += `| \`${r.slug}\` | ${r.skipReason} |\n`;
  }
  md += "\n";
}

md += "## Issue digest (aggregated across routes)\n\n";
if (issuesByKind.size === 0) {
  md += "_no cross-route issues detected_\n\n";
} else {
  md += "| Kind | Total occurrences | Routes affected |\n";
  md += "|------|------------------:|----------------:|\n";
  for (const [kind, count] of [...issuesByKind.entries()].sort((a, b) => b[1] - a[1])) {
    // count affected routes
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
md += "bash tools/e2e/capture-prod.sh\n\n";
md += "# 2. Capture dev (10-15 min, requires port-forward 30101)\n";
md += "bash tools/e2e/capture-dev.sh\n\n";
md += "# 3. Diff + report (~5 min)\n";
md += "bash tools/e2e/diff.sh\n\n";
md += "# 4. Subset (e.g. smoke 3 routes)\n";
md += "bash tools/e2e/capture-prod.sh home,about,auth\n";
md += "bash tools/e2e/capture-dev.sh  home,about,auth\n";
md += "bash tools/e2e/diff.sh        home,about,auth\n";
md += "```\n";

fs.writeFileSync(OUT, md);
console.log(`wrote ${OUT} (${rows.length} rows, ${issuesByKind.size} issue kinds)`);
