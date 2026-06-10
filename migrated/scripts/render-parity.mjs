import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const [, , nextjsBase, dioxusBase, ...rest] = process.argv;
const isAdmin = rest.includes('--admin');
const reportDir = rest.find(a => !a.startsWith('--')) || './screenshots/render-parity';

if (!nextjsBase || !dioxusBase) {
  console.error('Usage: node render-parity.mjs <nextjsBase> <dioxusBase> [--admin] [reportDir]');
  process.exit(1);
}

const FRONTEND_ROUTES = [
  '/', '/auth', '/dashboard', '/profile', '/account', '/account/credits',
  '/analytics', '/chat', '/chat/history', '/contact', '/about', '/news',
  '/notifications', '/payment', '/payment/subscription/1', '/permissions',
  '/plans', '/portfolio', '/developer', '/developer/usage', '/developer/docs',
  '/manual', '/access-denied', '/privacy', '/terms',
  '/not-a-real-page',
];

const ADMIN_ROUTES = [
  '/', '/analytics', '/audit-log', '/chat',
  '/developer-portal', '/developer-portal/api-keys/create',
  '/media', '/news', '/news/create', '/news/1/edit',
  '/notifications', '/notifications/create', '/notifications/manage',
  '/payments', '/policies', '/settings', '/unauthorized',
  '/wallet-management', '/wallet-management/wallets',
  '/wallet-management/0x1234', '/wallet-management/0x1234/disable',
  '/wallet-management/credits', '/wallet-management/access',
  '/wallet-management/access/plans', '/wallet-management/access/plans/1',
  '/access-denied', '/auth',
  '/not-a-real-page',
];

const routes = isAdmin ? ADMIN_ROUTES : FRONTEND_ROUTES;

async function extractAfterRender(page, url) {
  try {
    const resp = await page.goto(url, { waitUntil: 'networkidle', timeout: 25000 });
    await page.waitForTimeout(1200);
    return await page.evaluate(() => {
      const strip = (s) => (s || '').replace(/\s+/g, ' ').trim();
      const t = document.querySelector('title')?.textContent || '';
      const h1 = Array.from(document.querySelectorAll('h1')).map(e => strip(e.textContent || '')).filter(Boolean);
      const h2 = Array.from(document.querySelectorAll('h2')).map(e => strip(e.textContent || '')).filter(Boolean);
      const h3 = Array.from(document.querySelectorAll('h3')).map(e => strip(e.textContent || '')).filter(Boolean);
      const body = document.body?.innerText || '';
      const buttons = document.querySelectorAll('button').length;
      const links = document.querySelectorAll('a').length;
      const inputs = document.querySelectorAll('input, select, textarea').length;
      const forms = document.querySelectorAll('form').length;
      const hasNavbar = !!document.querySelector('nav, .navbar, [class*="navbar" i], header');
      const hasFooter = !!document.querySelector('footer, .footer, [class*="footer" i]');
      return { title: strip(t), h1, h2, h3, text: body, buttons, links, inputs, forms, hasNavbar, hasFooter, bodyTextLen: body.length };
    });
  } catch (e) {
    return { url, title: '', h1: [], h2: [], h3: [], text: '', buttons: 0, links: 0, inputs: 0, forms: 0, hasNavbar: false, hasFooter: false, bodyTextLen: 0 };
  }
}

function overlap(a, b) {
  if (!a.length && !b.length) return 100;
  if (!a.length || !b.length) return 0;
  const setA = new Set(a.map(s => s.toLowerCase()));
  const setB = new Set(b.map(s => s.toLowerCase()));
  let matches = 0;
  for (const s of setA) if (setB.has(s)) matches++;
  return (matches * 2) / (setA.size + setB.size) * 100;
}

function textOverlap(a, b) {
  const wordsA = new Set(a.toLowerCase().split(/\s+/).filter(w => w.length > 3));
  const wordsB = new Set(b.toLowerCase().split(/\s+/).filter(w => w.length > 3));
  if (!wordsA.size || !wordsB.size) return 0;
  let matches = 0;
  for (const w of wordsA) if (wordsB.has(w)) matches++;
  return (matches * 2) / (wordsA.size + wordsB.size) * 100;
}

async function main() {
  await mkdir(reportDir, { recursive: true });
  console.log(`\n=== Render-based Content Parity ===`);
  console.log(`Next.js: ${nextjsBase}`);
  console.log(`Dioxus:  ${dioxusBase}`);
  console.log(`Mode:    ${isAdmin ? 'admin' : 'frontend'}`);
  console.log(`Routes:  ${routes.length}\n`);

  const browser = await chromium.launch();

  // Try to sign in via demo login to get an authenticated session
  let authCookies = [];
  try {
    const authCtx = await browser.newContext();
    const authPage = await authCtx.newPage();
    const resp = await authPage.evaluate(async (base) => {
      const r = await fetch(`${base}/api/v1/auth/demo`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: '{}',
      });
      const setCookies = r.headers.getSetCookie ? r.headers.getSetCookie() : [];
      return { ok: r.ok, cookies: setCookies };
    }, nextjsBase);
    if (resp.ok && resp.cookies) {
      for (const sc of resp.cookies) {
        const [pair] = sc.split(';');
        const [name, ...rest] = pair.split('=');
        authCookies.push({ name: name.trim(), value: rest.join('=').trim(), domain: 'localhost', path: '/' });
      }
    }
    await authCtx.close();
    console.log(`Got ${authCookies.length} auth cookies from demo login`);
  } catch (e) {
    console.log('Demo login failed:', e.message);
  }

  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  if (authCookies.length) {
    await context.addCookies(authCookies);
    console.log('Auth cookies set on context');
  }
  const page = await context.newPage();

  const results = [];
  for (const route of routes) {
    console.log(`  Rendering ${route}...`);
    const n = await extractAfterRender(page, `${nextjsBase}${route}`);
    const d = await extractAfterRender(page, `${dioxusBase}${route}`);
    const tOv = textOverlap(n.text, d.text);
    const h1Ov = overlap(n.h1, d.h1);
    const h2Ov = overlap(n.h2, d.h2);
    const h3Ov = overlap(n.h3, d.h3);
    const status = tOv > 50 ? 'high' : tOv > 25 ? 'medium' : tOv > 10 ? 'low' : 'poor';
    const diff = {
      route, status,
      textOverlap: tOv,
      h1Overlap: h1Ov, h2Overlap: h2Ov, h3Overlap: h3Ov,
      nTitle: n.title, dTitle: d.title,
      nBodyLen: n.bodyTextLen, dBodyLen: d.bodyTextLen,
      nH1: n.h1, dH1: d.h1,
      nH2: n.h2.slice(0, 5), dH2: d.h2.slice(0, 5),
      nButtons: n.buttons, dButtons: d.buttons,
      nLinks: n.links, dLinks: d.links,
      nInputs: n.inputs, dInputs: d.inputs,
      nForms: n.forms, dForms: d.forms,
      nHasNavbar: n.hasNavbar, dHasNavbar: d.hasNavbar,
      nHasFooter: n.hasFooter, dHasFooter: d.hasFooter,
    };
    results.push(diff);
    console.log(`    [${status.padEnd(6)}] text=${tOv.toFixed(0).padStart(3)}% h1=${h1Ov.toFixed(0).padStart(3)}% h2=${h2Ov.toFixed(0).padStart(3)}% h3=${h3Ov.toFixed(0).padStart(3)}%  bodyN=${n.bodyTextLen} bodyD=${d.bodyTextLen}`);
  }

  const summary = {
    nextjsBase, dioxusBase, mode: isAdmin ? 'admin' : 'frontend',
    total: results.length,
    high: results.filter(r => r.status === 'high').length,
    medium: results.filter(r => r.status === 'medium').length,
    low: results.filter(r => r.status === 'low').length,
    poor: results.filter(r => r.status === 'poor').length,
    avgTextOverlap: results.reduce((a, r) => a + r.textOverlap, 0) / results.length,
    avgH1Overlap: results.reduce((a, r) => a + r.h1Overlap, 0) / results.length,
    avgH2Overlap: results.reduce((a, r) => a + r.h2Overlap, 0) / results.length,
    avgBodyLenN: results.reduce((a, r) => a + r.nBodyLen, 0) / results.length,
    avgBodyLenD: results.reduce((a, r) => a + r.dBodyLen, 0) / results.length,
    results,
  };

  await writeFile(join(reportDir, 'render-report.json'), JSON.stringify(summary, null, 2));
  await writeFile(join(reportDir, 'render-summary.md'), renderMd(summary));

  console.log(`\n=== Summary ===`);
  console.log(`Total:        ${summary.total}`);
  console.log(`High (>50%):  ${summary.high}`);
  console.log(`Medium:       ${summary.medium}`);
  console.log(`Low:          ${summary.low}`);
  console.log(`Poor (<10%):  ${summary.poor}`);
  console.log(`Avg text:     ${summary.avgTextOverlap.toFixed(1)}%`);
  console.log(`Avg H1:       ${summary.avgH1Overlap.toFixed(1)}%`);
  console.log(`Avg body len: Next.js=${summary.avgBodyLenN.toFixed(0)}  Dioxus=${summary.avgBodyLenD.toFixed(0)}`);

  await context.close();
  await browser.close();
}

function renderMd(s) {
  const pct = (n) => n.toFixed(1);
  let md = `# Render-based Content Parity Report\n\n`;
  md += `- **Next.js**: \`${s.nextjsBase}\`\n`;
  md += `- **Dioxus**:  \`${s.dioxusBase}\`\n`;
  md += `- **Mode**:    ${s.mode}\n\n`;
  md += `## Summary\n\n`;
  md += `| Bucket | Count |\n|---|---:|\n`;
  md += `| High overlap (>50%) | ${s.high} |\n`;
  md += `| Medium | ${s.medium} |\n`;
  md += `| Low | ${s.low} |\n`;
  md += `| Poor (<10%) | ${s.poor} |\n\n`;
  md += `Average text overlap: **${pct(s.avgTextOverlap)}%**\n`;
  md += `Average H1 overlap:   **${pct(s.avgH1Overlap)}%**\n`;
  md += `Average H2 overlap:   **${pct(s.avgH2Overlap)}%**\n\n`;
  md += `Average body length: Next.js=${s.avgBodyLenN.toFixed(0)}  Dioxus=${s.avgBodyLenD.toFixed(0)}\n\n`;
  md += `## Per-route\n\n`;
  md += `| Status | Route | Text % | H1 % | H2 % | Body N/D |\n|---|---|---:|---:|---:|---:|\n`;
  for (const r of s.results) {
    md += `| ${r.status} | \`${r.route}\` | ${pct(r.textOverlap)} | ${pct(r.h1Overlap)} | ${pct(r.h2Overlap)} | ${r.nBodyLen}/${r.dBodyLen} |\n`;
  }
  return md;
}

main().catch(e => { console.error(e); process.exit(1); });
