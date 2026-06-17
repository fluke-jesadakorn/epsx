// capture.js — Playwright runner: visit an admin route, click every interactive
// element, record what happens. Emits per-route artifacts:
//   <out-dir>/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
//
// Usage:
//   node capture.js --base <url> --out <dir> --routes "<slug>,<slug>..."
//                                            [--cookie "<name>=<value>"]
//                                            [--viewport 1280x800]
//                                            [--wait-ms 4000]
//                                            [--click-budget 30]
//
// Environment overrides:
//   EPSX_ADMIN_PROD_COOKIE   full Cookie header for prod admin
//   EPSX_DEV_AUTH_BYPASS     set to "1" to add 0x...d3v1 dev-bypass cookie
//
// Exit codes:
//   0  All requested routes captured (some may have non-fatal errors).
//   1  Bad usage.
//   2  No routes matched.

const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

// ---------- helpers (Wave 25 T1) ----------
function stripReturnUrl(rawUrl) {
  if (!rawUrl || typeof rawUrl !== "string") return rawUrl || "";
  try {
    const u = new URL(rawUrl);
    u.searchParams.delete("return_url");
    let out = u.toString();
    if (out.endsWith("?")) out = out.slice(0, -1);
    return out;
  } catch (_) {
    return rawUrl;
  }
}

// ---------- args ----------
const args = (() => {
  const a = {};
  for (let i = 2; i < process.argv.length; i += 2) {
    a[process.argv[i].replace(/^--/, "").replace(/-/g, "_")] = process.argv[i + 1];
  }
  return a;
})();

const BASE = args.base;
const OUT = args.out;
const ROUTES = (args.routes || "").split(",").map((s) => s.trim()).filter(Boolean);
const VIEWPORT = (args.viewport || "1280x800").split("x").map(Number);
const WAIT_MS = parseInt(args.wait_ms || "4000", 10);
const CLICK_BUDGET = parseInt(args.click_budget || "30", 10);
const EXTRA_COOKIE = args.cookie ||
  process.env.EPSX_ADMIN_PROD_COOKIE ||
  process.env.EPSX_AUTH_COOKIE ||
  "";
const DEV_BYPASS = process.env.EPSX_DEV_AUTH_BYPASS === "1" ||
  process.env.EPSX_AUTH_BYPASS === "1" ||
  process.env.EPSX_AUTH_BYPASS_DEV === "1";

if (!BASE || !OUT || ROUTES.length === 0) {
  console.error("usage: node capture.js --base <url> --out <dir> --routes <csv>");
  process.exit(1);
}

fs.mkdirSync(OUT, { recursive: true });

// load routes.json
const routesPath = path.join(__dirname, "routes.json");
const routesData = JSON.parse(fs.readFileSync(routesPath, "utf8"));
const routeMap = new Map(routesData.routes.map((r) => [r.slug, r.path]));

// Wave 25 T1: load routes-skip.json (auth_redirect_routes subset). Admin
// has none today, but the harness reads the same schema in case it's
// added later.
const skipPath = path.join(__dirname, "routes-skip.json");
let authRedirectRoutes = new Set();
if (fs.existsSync(skipPath)) {
  try {
    const skipCfg = JSON.parse(fs.readFileSync(skipPath, "utf8"));
    for (const slug of skipCfg.auth_redirect_routes || []) {
      authRedirectRoutes.add(slug);
    }
  } catch (e) {
    console.error(`[warn] routes-skip.json parse error: ${e.message}`);
  }
}

// ---------- run ----------
(async () => {
  // Resolve chromium binary: prefer the env var EPSX_CHROME, then the wave 22
  // pinned 1208 cache.
  const CHROME_BIN = process.env.EPSX_CHROME ||
    "/Users/fluke/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell";
  const launchOpts = {
    headless: true,
    args: ["--no-sandbox", "--disable-gpu"],
  };
  if (fs.existsSync(CHROME_BIN)) {
    launchOpts.executablePath = CHROME_BIN;
  }
  const browser = await chromium.launch(launchOpts);

  let totalOk = 0;
  let totalFail = 0;
  const summary = [];

  for (const slug of ROUTES) {
    const routePath = routeMap.get(slug);
    if (!routePath) {
      console.error(`[skip] unknown slug: ${slug}`);
      totalFail += 1;
      continue;
    }
    const url = `${BASE}${routePath}`;
    const outPng = path.join(OUT, `${slug}.png`);
    const outHtml = path.join(OUT, `${slug}.html`);
    const outConsole = path.join(OUT, `${slug}.console.log`);
    const outInteractions = path.join(OUT, `${slug}.interactions.jsonl`);
    const outNetwork = path.join(OUT, `${slug}.network.jsonl`);
    const outRedirects = path.join(OUT, `${slug}.redirects.log`);

    console.log(`[${slug}] ${url}`);

    // each route gets a fresh context to avoid state carry-over (esp. for
    // 307 redirects that would otherwise accumulate auth cookies).
    // Wave 24 T5' — force `colorScheme: 'dark'` so the BFF's FOUC script
    // resolves to dark mode (matches prod's `admin.epsx.io` which is
    // dark by default). Without this, headless Chromium reports
    // `prefers-color-scheme: light` and the BFF renders light mode
    // while prod ships dark mode, producing ~100% pixel diff on every
    // admin page.
    const COLOR_SCHEME = process.env.EPSX_COLOR_SCHEME || "dark";
    const context = await browser.newContext({
      viewport: { width: VIEWPORT[0], height: VIEWPORT[1] },
      ignoreHTTPSErrors: true,
      colorScheme: COLOR_SCHEME,
    });

    // add cookies for the BASE origin
    if (EXTRA_COOKIE) {
      const cookies = EXTRA_COOKIE.split(";").map((kv) => {
        const [name, ...rest] = kv.trim().split("=");
        return {
          name: name.trim(),
          value: rest.join("=").trim(),
          domain: new URL(BASE).hostname,
          path: "/",
        };
      });
      await context.addCookies(cookies);
    }
    if (DEV_BYPASS) {
      await context.addCookies([
        {
          name: "0x",
          value: "0x000000000000000000000000000000000000d3v1",
          domain: new URL(BASE).hostname,
          path: "/",
        },
      ]);
    }

    const page = await context.newPage();

    // capture console
    const consoleLines = [];
    page.on("console", (msg) => {
      consoleLines.push(`[${msg.type()}] ${msg.text()}`);
    });
    page.on("pageerror", (err) => {
      consoleLines.push(`[pageerror] ${err.message}`);
    });

    // capture network
    const networkLines = [];
    page.on("request", (req) => {
      networkLines.push(JSON.stringify({
        type: "request",
        method: req.method(),
        url: req.url(),
        resourceType: req.resourceType(),
      }));
    });
    page.on("response", (resp) => {
      networkLines.push(JSON.stringify({
        type: "response",
        status: resp.status(),
        url: resp.url(),
        contentType: resp.headers()["content-type"] || "",
      }));
    });
    page.on("requestfailed", (req) => {
      networkLines.push(JSON.stringify({
        type: "failed",
        url: req.url(),
        failure: req.failure()?.errorText || "unknown",
      }));
    });

    // capture navigation chain
    const redirects = [];
    page.on("framenavigated", (frame) => {
      if (frame === page.mainFrame()) {
        redirects.push(frame.url());
      }
    });

    let httpStatus = null;
    let navigatedTo = null;
    let interactions = [];

    try {
      // Visit
      const resp = await page.goto(url, {
        waitUntil: "domcontentloaded",
        timeout: 30000,
      });
      httpStatus = resp ? resp.status() : null;
      navigatedTo = page.url();

      // wait for hydration
      await page.waitForTimeout(WAIT_MS);

      // Wave 25 T1: strip ?return_url=… artifact on auth-redirect routes.
      if (authRedirectRoutes.has(slug)) {
        const cleanUrl = stripReturnUrl(page.url());
        if (cleanUrl !== page.url()) {
          console.log(`  [strip-return-url] ${page.url()} -> ${cleanUrl}`);
          await page.goto(cleanUrl, {
            waitUntil: "domcontentloaded",
            timeout: 30000,
          });
          await page.waitForTimeout(WAIT_MS);
        }
      }

      // Take initial screenshot
      await page.screenshot({ path: outPng, fullPage: false });

      // Save post-hydration HTML
      const html = await page.content();
      fs.writeFileSync(outHtml, html);

      // Save console
      fs.writeFileSync(outConsole, consoleLines.join("\n") + "\n");

      // Save network
      fs.writeFileSync(outNetwork, networkLines.join("\n") + "\n");

      // Save redirects chain
      fs.writeFileSync(
        outRedirects,
        redirects.map((u, i) => `${i}: ${u}`).join("\n") + "\n",
      );

      // ----- Discover interactive elements -----
      const interactive = await page.evaluate((budget) => {
        const sel = [
          'a[href]',
          'button:not([disabled])',
          '[role="button"]:not([disabled])',
          'details > summary',
          'select',
          'input:not([type="hidden"])',
          'textarea',
          '[tabindex]:not([tabindex="-1"])',
        ].join(",");

        const els = Array.from(document.querySelectorAll(sel));
        const seen = new Set();
        const out = [];
        for (const el of els.slice(0, budget)) {
          if (seen.has(el)) continue;
          seen.add(el);
          // skip if not in viewport
          const r = el.getBoundingClientRect();
          const visible =
            r.width > 0 && r.height > 0 &&
            r.top < window.innerHeight && r.bottom > 0 &&
            r.left < window.innerWidth && r.right > 0;
          if (!visible) continue;
          out.push({
            tag: el.tagName.toLowerCase(),
            role: el.getAttribute("role"),
            text: (el.textContent || "").trim().slice(0, 100),
            href: el.getAttribute("href") || null,
            type: el.getAttribute("type") || null,
            ariaLabel: el.getAttribute("aria-label") || null,
            selector:
              el.tagName.toLowerCase() +
              (el.id ? `#${el.id}` : "") +
              (el.className && typeof el.className === "string"
                ? `.${el.className.trim().split(/\s+/).slice(0, 2).join(".")}`
                : ""),
            rect: { x: r.x, y: r.y, w: r.width, h: r.height },
          });
        }
        return out;
      }, CLICK_BUDGET);

      // ----- Click them in viewport order; record before/after URL + network -----
      const urlBefore = page.url();
      for (const el of interactive) {
        try {
          // record requests during this click
          const reqsBefore = networkLines.length;
          const urlBeforeClick = page.url();
          // re-query fresh handle
          const handle = await page.evaluateHandle((selector) => {
            return document.querySelector(selector);
          }, el.selector);
          const elem = handle.asElement();
          if (!elem) {
            interactions.push({ ...el, action: "click", result: "stale-handle" });
            continue;
          }
          await elem.click({ timeout: 2000, force: false });
          await page.waitForTimeout(400);
          const urlAfterClick = page.url();
          interactions.push({
            ...el,
            action: "click",
            url_before: urlBeforeClick,
            url_after: urlAfterClick,
            url_changed: urlBeforeClick !== urlAfterClick,
            new_requests: networkLines.length - reqsBefore,
            result: "ok",
          });
          // If clicking caused a navigation, break — we're on a new page
          if (urlBeforeClick !== urlAfterClick) break;
        } catch (e) {
          interactions.push({ ...el, action: "click", result: "error", error: e.message });
        }
      }
      const urlAfter = page.url();

      fs.writeFileSync(
        outInteractions,
        interactions.map((i) => JSON.stringify(i)).join("\n") + "\n",
      );

      console.log(
        `  ok status=${httpStatus} nav=${urlBefore} -> ${urlAfter} interactions=${interactions.length}`,
      );
      summary.push({ slug, status: httpStatus, ok: true, interactions: interactions.length });
      totalOk += 1;
    } catch (e) {
      console.error(`  FAIL ${slug}: ${e.message}`);
      // still write whatever artifacts we have
      fs.writeFileSync(outConsole, consoleLines.join("\n") + "\n");
      fs.writeFileSync(outNetwork, networkLines.join("\n") + "\n");
      fs.writeFileSync(
        outRedirects,
        redirects.map((u, i) => `${i}: ${u}`).join("\n") + "\n",
      );
      fs.writeFileSync(
        outInteractions,
        interactions.map((i) => JSON.stringify(i)).join("\n") + "\n",
      );
      summary.push({ slug, status: httpStatus, ok: false, error: e.message });
      totalFail += 1;
    } finally {
      await context.close();
    }
  }

  // write summary
  fs.writeFileSync(
    path.join(OUT, "_summary.json"),
    JSON.stringify({ generated: new Date().toISOString(), routes: summary }, null, 2),
  );

  await browser.close();
  console.log(`\n=== capture summary: ok=${totalOk} fail=${totalFail} ===`);
  process.exit(totalFail > 0 ? 0 : 0); // don't hard-fail the whole run
})();
