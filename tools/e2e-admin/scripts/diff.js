// diff.js — compare prod vs dev baselines for one admin slug.
//
// Reads:
//   <prod-dir>/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
//   <dev-dir>/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
//
// Emits:
//   <out-dir>/<slug>.json          structured per-slug diff
//   <out-dir>/<slug>.diff.png      pixel-diff visualization
//
// Stdout:
//   PIXEL_DIFF=<N> DIFF_PCT=<x.xx>  on first line
//   subsequent lines: JSON-encoded issues

const fs = require("fs");
const path = require("path");
const { PNG } = (() => {
  // try to load pngjs from apps-old/admin-frontend (pnpmed)
  const candidates = [
    "/Users/fluke/Desktop/Work/epsx/apps-old/admin-frontend/node_modules/pngjs",
    "/Users/fluke/Desktop/Work/epsx/apps-old/admin-frontend/node_modules/.pnpm/pngjs@5.0.0/node_modules/pngjs",
    "/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules/pngjs",
    "/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules/.pnpm/pngjs@5.0.0/node_modules/pngjs",
  ];
  for (const c of candidates) {
    try {
      return require(c);
    } catch (_) {
      // try next
    }
  }
  return {};
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
  if (PNG && PNG.sync && fs.existsSync(prodPng) && fs.existsSync(devPng)) {
    const prod = PNG.sync.read(fs.readFileSync(prodPng));
    const dev = PNG.sync.read(fs.readFileSync(devPng));
    const w = Math.min(prod.width, dev.width);
    const h = Math.min(prod.height, dev.height);
    const out = new PNG({ width: WIDTH, height: HEIGHT });
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
  if (!prodHtml || !devHtml) return { missing: [], extra: [], missingButtons: [] };
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

// 6) redirect chain differences
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

// 7) status-404: dev is currently a mostly-empty Dioxus admin shell; many
//    routes may return 404. Record this as a structural issue kind so the
//    report can call it out.
const prodPngPath = path.join(PROD, `${SLUG}.png`);
const devPngPath = path.join(DEV, `${SLUG}.png`);
function getStatusFromNetworkLog(networkText) {
  if (!networkText) return null;
  const lines = networkText.split(/\r?\n/).filter(Boolean);
  // first response is the navigation response
  for (const l of lines) {
    try {
      const o = JSON.parse(l);
      if (o.type === "response" && o.url && o.url.includes(SLUG.replace(/-/g, ""))) {
        return o.status;
      }
    } catch (_) {}
  }
  // fall back: first non-redirect 200/4xx
  for (const l of lines) {
    try {
      const o = JSON.parse(l);
      if (o.type === "response") {
        return o.status;
      }
    } catch (_) {}
  }
  return null;
}

// We don't have a network log here (the issue is dev returns 404 quickly),
// so just rely on the redirect chain to detect 404.
const devLastUrl = devChain[devChain.length - 1] || "";
// Try the dev network log for the first response status
let devFirstStatus = null;
const devNetPath = path.join(DEV, `${SLUG}.network.jsonl`);
if (fs.existsSync(devNetPath)) {
  const netLines = readJsonl(devNetPath);
  for (const nl of netLines) {
    if (nl && nl.type === "response" && nl.status) {
      devFirstStatus = nl.status;
      break;
    }
  }
}
const devEndsAt404 = /\/404|\/not-found/i.test(devLastUrl) ||
  devFirstStatus === 404 || devFirstStatus === 500;
if (devEndsAt404) {
  issues.push({
    kind: "dev-returns-404",
    devFinal: devLastUrl,
    devStatus: devFirstStatus,
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
