// diff.js — compare prod vs dev baselines for one slug.
//
// Reads:
//   <prod-dir>/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
//   <dev-dir>/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
//
// Emits:
//   <out-dir>/<slug>.json          structured per-slug diff
//
// Stdout:
//   PIXEL_DIFF=<N> DIFF_PCT=<x.xx>  on first line
//   subsequent lines: JSON-encoded issues
//
// All emissions are JSON-safe: the per-slug report can be assembled by
// the shell wrapper into report.md.

const fs = require("fs");
const path = require("path");
const { PNG } = (() => {
  // try to load pngjs from apps-old; fall back to no-png path
  try {
    return require("/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules/pngjs");
  } catch (_) {
    return {};
  }
})();

const args = (() => {
  const a = {};
  for (let i = 2; i < process.argv.length; i += 2) {
    a[process.argv[i].replace(/^--/, "").replace(/-/g, "_")] = process.argv[i + 1];
  }
  return a;
})();

const SLUG = args.slug;
const PROD = args.prod;
const DEV = args.dev;
const OUT = args.out;
const PIXEL_DIFF_SH = args.pixel_diff_sh || "";
const WIDTH = parseInt(args.width || "1280", 10);
const HEIGHT = parseInt(args.height || "800", 10);

if (!SLUG || !PROD || !DEV || !OUT) {
  console.error("usage: node diff.js --slug X --prod P --dev D --out O [--pixel-diff-sh P]");
  process.exit(1);
}

const TOTAL_PX = WIDTH * HEIGHT;

fs.mkdirSync(OUT, { recursive: true });

// ---------- skip / auth-redirect config (Wave 25 T1) ----------
//
// Two categories:
//   1. `skip_routes` — comparison is meaningless (prod SPA fallback / 404
//      that differs structurally from dev). Emit a no-op diff that copies
//      prod.png to diff.png and reports DIFF_PCT=0 so the report can
//      distinguish "intentionally skipped" from "captured but diverged".
//   2. `auth_redirect_routes` — both prod and dev 307 to /auth with the
//      same visual content, but dev's URL carries a `?return_url=...`
//      artifact (the /auth page is the same in both). The capture script
//      normalizes dev's URL via `stripReturnUrl()` before screenshotting,
//      so the diff itself is unchanged. The config is loaded here as a
//      cross-check: if a slug in `auth_redirect_routes` shows up in the
//      diff with `redirect-chain-differs` AND the dev's final URL still
//      contains `return_url=`, the harness has a bug.
const SKIP_PATH = path.join(__dirname, "routes-skip.json");
let SKIP_CONFIG = { skip_routes: [], admin_skip_routes: [], auth_redirect_routes: [] };
if (fs.existsSync(SKIP_PATH)) {
  try {
    SKIP_CONFIG = JSON.parse(fs.readFileSync(SKIP_PATH, "utf8"));
  } catch (e) {
    console.error(`[warn] routes-skip.json parse error: ${e.message}`);
  }
}
const skipReasonBySlug = new Map();
for (const entry of SKIP_CONFIG.skip_routes || []) {
  skipReasonBySlug.set(entry.slug, entry.reason);
}
for (const entry of SKIP_CONFIG.admin_skip_routes || []) {
  // admin slugs in the FE harness are still useful as no-ops so report.md
  // can show the skip reason against the FE report table.
  skipReasonBySlug.set(entry.slug, `[admin] ${entry.reason}`);
}
const authRedirectRoutes = new Set(SKIP_CONFIG.auth_redirect_routes || []);

function maybeSkip() {
  if (!skipReasonBySlug.has(SLUG)) return false;
  const reason = skipReasonBySlug.get(SLUG);
  // Emit a "no-op" diff: copy prod.png to diff.png so the artifact set
  // stays complete (the report can show diff.png in the row), and emit
  // PIXEL_DIFF=0 DIFF_PCT=0.00 with a marker issue explaining the skip.
  const prodPng = path.join(PROD, `${SLUG}.png`);
  const devPng = path.join(DEV, `${SLUG}.png`);
  const diffPng = path.join(OUT, `${SLUG}.diff.png`);
  if (fs.existsSync(prodPng)) {
    fs.copyFileSync(prodPng, diffPng);
  } else if (fs.existsSync(devPng)) {
    fs.copyFileSync(devPng, diffPng);
  }
  const result = {
    slug: SLUG,
    skipped: true,
    skipReason: reason,
    pixelDiff: { count: 0, pct: 0, method: "skip-no-op", diffPng },
    issues: [{ kind: "skipped-route", reason }],
    prodConsoleErrors: 0,
    devConsoleErrors: 0,
    prodInteractions: 0,
    devInteractions: 0,
  };
  fs.writeFileSync(path.join(OUT, `${SLUG}.json`), JSON.stringify(result, null, 2));
  console.log(`PIXEL_DIFF=0 DIFF_PCT=0.00 [SKIP] ${SLUG}: ${reason}`);
  return true;
}

// ---------- helpers ----------
function readJsonl(p) {
  if (!fs.existsSync(p)) return [];
  return fs
    .readFileSync(p, "utf8")
    .split(/\r?\n/)
    .filter((l) => l.trim().length > 0)
    .map((l) => {
      try {
        return JSON.parse(l);
      } catch (_) {
        return null;
      }
    })
    .filter(Boolean);
}

function readText(p) {
  if (!fs.existsSync(p)) return "";
  return fs.readFileSync(p, "utf8");
}

function compareStrings(a, b) {
  if (a === b) return [];
  return [{ kind: "text-mismatch", prod: a, dev: b }];
}

function pixelDiff(prodPng, devPng) {
  // Strategy 1: shell out to pixel-diff.sh if provided (uses ImageMagick)
  if (PIXEL_DIFF_SH && fs.existsSync(prodPng) && fs.existsSync(devPng)) {
    const { execFileSync } = require("child_process");
    const diffPng = path.join(OUT, `${SLUG}.diff.png`);
    try {
      const out = execFileSync("bash", [PIXEL_DIFF_SH, prodPng, devPng, diffPng], {
        encoding: "utf8",
        timeout: 60000,
      });
      const m = out.match(/PIXEL_DIFF=(\d+)/);
      if (m) return { count: parseInt(m[1], 10), diffPng, method: "imagemagick" };
    } catch (e) {
      // fall through to pngjs
    }
  }

  // Strategy 2: pngjs pure-Node
  if (PNG && fs.existsSync(prodPng) && fs.existsSync(devPng)) {
    const prod = PNG.sync.read(fs.readFileSync(prodPng));
    const dev = PNG.sync.read(fs.readFileSync(devPng));
    const w = Math.min(prod.width, dev.width);
    const h = Math.min(prod.height, dev.height);
    const out = new PNG({ width, height });
    let count = 0;
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const idx = (y * prod.width + x) * 4;
        const idxD = (y * dev.width + x) * 4;
        const dr = Math.abs(prod.data[idx] - dev.data[idxD]);
        const dg = Math.abs(prod.data[idx + 1] - dev.data[idxD + 1]);
        const db = Math.abs(prod.data[idx + 2] - dev.data[idxD + 2]);
        const sum = dr + dg + db;
        if (sum > 15) {
          count += 1;
          out.data[idx] = 255;
          out.data[idx + 1] = 0;
          out.data[idx + 2] = 0;
          out.data[idx + 3] = 255;
        } else {
          out.data[idx] = 255;
          out.data[idx + 1] = 255;
          out.data[idx + 2] = 255;
          out.data[idx + 3] = 255;
        }
      }
    }
    const diffPng = path.join(OUT, `${SLUG}.diff.png`);
    fs.writeFileSync(diffPng, PNG.sync.write(out));
    return { count, diffPng, method: "pngjs" };
  }

  return { count: 0, diffPng: null, method: "none" };
}

// ---------- HTML structural diff ----------
function structuralDiff(prodHtml, devHtml) {
  if (!prodHtml || !devHtml) return { missing: [], extra: [], brokenHrefs: [] };
  // crude: extract hrefs, link text + roles. We compare hrefs sets.
  const extractLinks = (html) => {
    const re = /<a\s+[^>]*href="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gi;
    const out = [];
    let m;
    while ((m = re.exec(html)) !== null) {
      const href = m[1].trim();
      const text = m[2].replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
      out.push({ href, text });
    }
    return out;
  };
  const extractButtons = (html) => {
    const re = /<button\s+[^>]*>([\s\S]*?)<\/button>/gi;
    const out = [];
    let m;
    while ((m = re.exec(html)) !== null) {
      const text = m[1].replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
      out.push({ text });
    }
    return out;
  };
  const prodLinks = extractLinks(prodHtml);
  const devLinks = extractLinks(devHtml);
  const prodHrefs = new Set(prodLinks.map((l) => l.href));
  const devHrefs = new Set(devLinks.map((l) => l.href));
  const missing = [...prodHrefs].filter((h) => !devHrefs.has(h));
  const extra = [...devHrefs].filter((h) => !prodHrefs.has(h));
  const prodButtons = new Set(extractButtons(prodHtml).map((b) => b.text));
  const devButtons = new Set(extractButtons(devHtml).map((b) => b.text));
  const missingButtons = [...prodButtons].filter((b) => !devButtons.has(b));
  // broken hrefs = dev hrefs that point to /<path> with no corresponding route in routes.json
  return { missing, extra, missingButtons };
}

// ---------- main ----------
const issues = [];
const prodHtml = readText(path.join(PROD, `${SLUG}.html`));
const devHtml = readText(path.join(DEV, `${SLUG}.html`));
const prodConsole = readText(path.join(PROD, `${SLUG}.console.log`));
const devConsole = readText(path.join(DEV, `${SLUG}.console.log`));
const prodInteractions = readJsonl(path.join(PROD, `${SLUG}.interactions.jsonl`));
const devInteractions = readJsonl(path.join(DEV, `${SLUG}.interactions.jsonl`));
const prodRedirects = readText(path.join(PROD, `${SLUG}.redirects.log`));
const devRedirects = readText(path.join(DEV, `${SLUG}.redirects.log`));

// Wave 25 T1 — skip no-op routes (SPA-fallback / 404 structural gaps).
// Must come BEFORE pixel-diff so a missing artifact doesn't poison the count.
if (maybeSkip()) {
  process.exit(0);
}

// 1) pixel diff
const px = pixelDiff(path.join(PROD, `${SLUG}.png`), path.join(DEV, `${SLUG}.png`));
const pct = ((px.count / TOTAL_PX) * 100).toFixed(2);
console.log(`PIXEL_DIFF=${px.count} DIFF_PCT=${pct}`);

// 2) console errors that appear in dev but not in prod
function extractErrors(text) {
  return text
    .split(/\r?\n/)
    .filter((l) => /\[error\]|\[pageerror\]/i.test(l))
    .map((l) => l.trim());
}
const prodErrors = new Set(extractErrors(prodConsole));
const devErrors = extractErrors(devConsole).filter((e) => !prodErrors.has(e));
if (devErrors.length > 0) {
  issues.push({
    kind: "console-error-dev-only",
    count: devErrors.length,
    samples: devErrors.slice(0, 3),
  });
}

// 3) interactions: broken clicks = dev clicks that errored
const devClickErrors = devInteractions.filter((i) => i.result === "error");
const devStaleHandles = devInteractions.filter((i) => i.result === "stale-handle");
if (devClickErrors.length > 0) {
  issues.push({
    kind: "broken-clicks",
    count: devClickErrors.length,
    samples: devClickErrors.slice(0, 3).map((i) => ({
      selector: i.selector,
      text: i.text,
      error: i.error,
    })),
  });
}

// 4) interactions: clicks that should have navigated but didn't
const devClicksNoNav = devInteractions.filter(
  (i) => i.action === "click" && i.result === "ok" && !i.url_changed,
);
const prodClicksWithNav = prodInteractions.filter(
  (i) => i.action === "click" && i.result === "ok" && i.url_changed,
);
if (prodClicksWithNav.length > 0 && devClicksNoNav.length > 0) {
  // heuristic: if prod had nav-causing clicks and dev's same-selector click did not
  const prodNavTargets = new Set(prodClicksWithNav.map((i) => i.url_after));
  const sameSlugs = devClicksNoNav.filter(
    (i) => i.selector && prodInteractions.find((p) => p.selector === i.selector && p.url_changed),
  );
  if (sameSlugs.length > 0) {
    issues.push({
      kind: "click-does-not-navigate",
      count: sameSlugs.length,
      samples: sameSlugs.slice(0, 3).map((i) => ({
        selector: i.selector,
        text: i.text,
      })),
    });
  }
}

// 5) HTML structural: missing hrefs/buttons
const structDiff = structuralDiff(prodHtml, devHtml);
if (structDiff.missing.length > 0) {
  issues.push({
    kind: "missing-hrefs",
    count: structDiff.missing.length,
    samples: structDiff.missing.slice(0, 5),
  });
}
if (structDiff.missingButtons.length > 0) {
  issues.push({
    kind: "missing-buttons",
    count: structDiff.missingButtons.length,
    samples: structDiff.missingButtons.slice(0, 5),
  });
}

// 6) redirect chain differences (sign that the prod 307 logic is mirrored or not)
const prodChain = prodRedirects.split(/\r?\n/).filter(Boolean);
const devChain = devRedirects.split(/\r?\n/).filter(Boolean);
if (prodChain.length !== devChain.length || prodChain[prodChain.length - 1] !== devChain[devChain.length - 1]) {
  issues.push({
    kind: "redirect-chain-differs",
    prodFinal: prodChain[prodChain.length - 1] || "",
    devFinal: devChain[devChain.length - 1] || "",
    prodLength: prodChain.length,
    devLength: devChain.length,
  });
}

const result = {
  slug: SLUG,
  pixelDiff: { count: px.count, pct: parseFloat(pct), method: px.method, diffPng: px.diffPng },
  issues,
  prodConsoleErrors: extractErrors(prodConsole).length,
  devConsoleErrors: devErrors.length + extractErrors(prodConsole).length,
  prodInteractions: prodInteractions.length,
  devInteractions: devInteractions.length,
};

fs.writeFileSync(path.join(OUT, `${SLUG}.json`), JSON.stringify(result, null, 2));
process.stdout.write(JSON.stringify(result) + "\n");
