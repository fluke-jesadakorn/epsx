#!/usr/bin/env bun
/**
 * Content parity check: extract semantic text content from Next.js and Dioxus
 * HTML responses, then compute text overlap.
 *
 * Compares:
 *   - Page title (<title>)
 *   - H1, H2, H3 headings
 *   - Major text blocks (>20 chars)
 *   - Number of interactive elements (button, a, input)
 *   - Number of form fields
 *
 * A high text overlap means functional parity even when the pixel diff is
 * 99% (different CSS approach).
 *
 * Usage:
 *   bun scripts/content-parity.ts <nextjsBaseUrl> <dioxusBaseUrl> [--admin] [reportDir]
 */
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join } from 'node:path';

const [, , nextjsBase, dioxusBase, ...rest] = process.argv;
const isAdmin = rest.includes('--admin');
const reportDir = rest.find(a => !a.startsWith('--')) || './screenshots/content-parity';

if (!nextjsBase || !dioxusBase) {
  console.error('Usage: bun scripts/content-parity.ts <nextjsBaseUrl> <dioxusBaseUrl> [--admin] [reportDir]');
  process.exit(1);
}

const FRONTEND_ROUTES = [
  '/', '/auth', '/dashboard', '/profile', '/account', '/account/credits',
  '/analytics', '/chat', '/chat/history', '/chat/sample-1',
  '/contact', '/about', '/news', '/news/welcome-to-epsx',
  '/notifications', '/payment', '/payment/subscription/1', '/permissions',
  '/plans', '/portfolio', '/developer', '/developer/usage', '/developer/docs',
  '/manual', '/access-denied', '/offline', '/privacy', '/terms',
  '/not-a-real-page',
];

const ADMIN_ROUTES = [
  '/', '/analytics', '/audit-log', '/chat', '/chat/sample-1',
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

function extractContent(html: string): {
  title: string;
  h1: string[];
  h2: string[];
  h3: string[];
  text: string[];
  buttons: number;
  links: number;
  inputs: number;
  forms: number;
  dataSignals: string[];
} {
  const titleMatch = html.match(/<title[^>]*>([^<]*)<\/title>/i);
  const h1Matches = [...html.matchAll(/<h1[^>]*>([\s\S]*?)<\/h1>/gi)].map(m => stripTags(m[1] || '').trim()).filter(Boolean);
  const h2Matches = [...html.matchAll(/<h2[^>]*>([\s\S]*?)<\/h2>/gi)].map(m => stripTags(m[1] || '').trim()).filter(Boolean);
  const h3Matches = [...html.matchAll(/<h3[^>]*>([\s\S]*?)<\/h3>/gi)].map(m => stripTags(m[1] || '').trim()).filter(Boolean);
  const allText = stripTags(html).replace(/\s+/g, ' ');
  const textBlocks = allText.split(/[.!?]\s/).map(s => s.trim()).filter(s => s.length > 20 && s.length < 200);
  const buttons = (html.match(/<button\b/gi) || []).length;
  const links = (html.match(/<a\b/gi) || []).length;
  const inputs = (html.match(/<input\b/gi) || []).length;
  const forms = (html.match(/<form\b/gi) || []).length;
  const dataSignals = [
    /data-(?:nextjs|dioxus|epsx)/i,
    /id="[a-z0-9_-]*[a-z][a-z0-9_-]*"/i,
  ].map(r => (html.match(r) || []).length);

  return {
    title: (titleMatch?.[1] || '').trim(),
    h1: h1Matches,
    h2: h2Matches,
    h3: h3Matches,
    text: textBlocks,
    buttons, links, inputs, forms,
    dataSignals: dataSignals.map(String),
  };
}

function stripTags(s: string): string {
  return s.replace(/<[^>]+>/g, ' ').replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'");
}

function textOverlap(a: string[], b: string[]): number {
  if (a.length === 0 && b.length === 0) return 100;
  if (a.length === 0 || b.length === 0) return 0;
  const setA = new Set(a.map(s => s.toLowerCase()));
  const setB = new Set(b.map(s => s.toLowerCase()));
  let matches = 0;
  for (const s of setA) if (setB.has(s)) matches++;
  return (matches * 2) / (setA.size + setB.size) * 100;
}

async function fetchHtml(base: string, path: string): Promise<string> {
  try {
    const res = await fetch(`${base}${path}`);
    if (!res.ok) return '';
    return await res.text();
  } catch { return ''; }
}

async function main() {
  await mkdir(reportDir, { recursive: true });
  console.log(`\n=== Content Parity Check ===`);
  console.log(`Next.js:  ${nextjsBase}`);
  console.log(`Dioxus:   ${dioxusBase}`);
  console.log(`Mode:     ${isAdmin ? 'admin' : 'frontend'}`);
  console.log(`Routes:   ${routes.length}`);
  console.log(``);

  const results: any[] = [];

  for (const route of routes) {
    const [nHtml, dHtml] = await Promise.all([
      fetchHtml(nextjsBase, route),
      fetchHtml(dioxusBase, route),
    ]);

    if (!nHtml || !dHtml) {
      results.push({ route, status: nHtml && dHtml ? 'ok' : 'missing', textOverlap: 0, h1Overlap: 0, h2Overlap: 0, h3Overlap: 0, nTitle: '', dTitle: '', nBlocks: 0, dBlocks: 0 });
      console.log(`  [${nHtml ? 'd' : 'n'}:miss] ${route}`);
      continue;
    }

    const nC = extractContent(nHtml);
    const dC = extractContent(dHtml);

    const tOv = textOverlap(nC.text, dC.text);
    const h1Ov = textOverlap(nC.h1, dC.h1);
    const h2Ov = textOverlap(nC.h2, dC.h2);
    const h3Ov = textOverlap(nC.h3, dC.h3);

    const status = tOv > 50 ? 'high' : tOv > 25 ? 'medium' : tOv > 10 ? 'low' : 'poor';

    results.push({
      route,
      status,
      textOverlap: tOv,
      h1Overlap: h1Ov,
      h2Overlap: h2Ov,
      h3Overlap: h3Ov,
      nTitle: nC.title,
      dTitle: dC.title,
      nBlocks: nC.text.length,
      dBlocks: dC.text.length,
      nH1: nC.h1.length, dH1: dC.h1.length,
      nH2: nC.h2.length, dH2: dC.h2.length,
      nH3: nC.h3.length, dH3: dC.h3.length,
      nButtons: nC.buttons, dButtons: dC.buttons,
      nLinks: nC.links, dLinks: dC.links,
      nInputs: nC.inputs, dInputs: dC.inputs,
      nForms: nC.forms, dForms: dC.forms,
    });
    console.log(`  [${status.padEnd(6)}] ${route.padEnd(40)}  text=${tOv.toFixed(0).padStart(3)}%  h1=${h1Ov.toFixed(0).padStart(3)}%  h2=${h2Ov.toFixed(0).padStart(3)}%  h3=${h3Ov.toFixed(0).padStart(3)}%`);
  }

  const summary = {
    nextjsBase, dioxusBase, mode: isAdmin ? 'admin' : 'frontend',
    total: results.length,
    high: results.filter(r => r.status === 'high').length,
    medium: results.filter(r => r.status === 'medium').length,
    low: results.filter(r => r.status === 'low').length,
    poor: results.filter(r => r.status === 'poor').length,
    missing: results.filter(r => r.status === 'missing').length,
    avgTextOverlap: results.reduce((a, r) => a + r.textOverlap, 0) / results.length,
    avgH1Overlap: results.reduce((a, r) => a + r.h1Overlap, 0) / results.length,
    results,
  };

  await writeFile(join(reportDir, 'content-report.json'), JSON.stringify(summary, null, 2));
  await writeFile(join(reportDir, 'content-summary.md'), renderMd(summary));

  console.log(``);
  console.log(`Total:        ${summary.total}`);
  console.log(`High (>50%):  ${summary.high}`);
  console.log(`Medium:       ${summary.medium}`);
  console.log(`Low:          ${summary.low}`);
  console.log(`Poor (<10%):  ${summary.poor}`);
  console.log(`Missing:      ${summary.missing}`);
  console.log(``);
  console.log(`Avg text overlap: ${summary.avgTextOverlap.toFixed(1)}%`);
  console.log(`Avg H1 overlap:   ${summary.avgH1Overlap.toFixed(1)}%`);
}

function renderMd(s: any): string {
  const pct = (n: number) => n.toFixed(1);
  let md = `# Content Parity Report\n\n`;
  md += `- **Next.js**: \`${s.nextjsBase}\`\n`;
  md += `- **Dioxus**:  \`${s.dioxusBase}\`\n`;
  md += `- **Mode**:    ${s.mode}\n\n`;
  md += `## Summary\n\n`;
  md += `| Bucket | Count |\n|---|---:|\n`;
  md += `| High overlap (>50%) | ${s.high} |\n`;
  md += `| Medium overlap | ${s.medium} |\n`;
  md += `| Low overlap | ${s.low} |\n`;
  md += `| Poor overlap (<10%) | ${s.poor} |\n`;
  md += `| Missing | ${s.missing} |\n\n`;
  md += `Average text overlap: **${pct(s.avgTextOverlap)}%**\n`;
  md += `Average H1 overlap: **${pct(s.avgH1Overlap)}%**\n\n`;
  md += `## Per-route\n\n`;
  md += `| Status | Route | Text % | H1 % | H2 % | H3 % |\n|---|---|---:|---:|---:|---:|\n`;
  for (const r of s.results) {
    md += `| ${r.status} | \`${r.route}\` | ${pct(r.textOverlap)} | ${pct(r.h1Overlap)} | ${pct(r.h2Overlap)} | ${pct(r.h3Overlap)} |\n`;
  }
  return md;
}

main().catch(e => { console.error(e); process.exit(1); });
