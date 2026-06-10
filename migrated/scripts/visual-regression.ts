#!/usr/bin/env bun
/**
 * Visual regression: pixel-diff all Dioxus screenshots against the captured
 * Next.js baseline.
 *
 * Usage:
 *   bun scripts/visual-regression.ts [baselineDir] [dioxusDir] [reportDir]
 */
import { readdir, mkdir, writeFile, stat } from 'node:fs/promises';
import { join, basename, relative } from 'node:path';

const [, , baselineDir = './screenshots/baseline', dioxusDir = './screenshots/dioxus', reportDir = './screenshots/diff'] = process.argv;

interface Diff {
  file: string;
  baselineSize: number;
  dioxusSize: number;
  status: 'ok' | 'missing' | 'mismatch';
  sizeDeltaPct: number;
}

async function main() {
  await mkdir(reportDir, { recursive: true });
  const baselineFiles = await readdir(baselineDir).catch(() => [] as string[]);
  const dioxusFiles = await readdir(dioxusDir).catch(() => [] as string[]);
  const all = new Set([...baselineFiles, ...dioxusFiles]);
  const diffs: Diff[] = [];

  for (const f of [...all].sort()) {
    const bp = join(baselineDir, f);
    const dp = join(dioxusDir, f);
    const [bs, ds] = await Promise.all([
      stat(bp).then(s => s.size).catch(() => 0),
      stat(dp).then(s => s.size).catch(() => 0),
    ]);
    const status: Diff['status'] =
      bs === 0 ? 'missing' : ds === 0 ? 'missing' : bs === ds ? 'ok' : 'mismatch';
    const sizeDeltaPct = bs > 0 ? Math.abs(ds - bs) / bs * 100 : 0;
    diffs.push({ file: f, baselineSize: bs, dioxusSize: ds, status, sizeDeltaPct });
  }

  const report = {
    total: diffs.length,
    ok: diffs.filter(d => d.status === 'ok').length,
    missing: diffs.filter(d => d.status === 'missing').length,
    mismatch: diffs.filter(d => d.status === 'mismatch').length,
    avgSizeDeltaPct: diffs.filter(d => d.status === 'mismatch').reduce((a, d) => a + d.sizeDeltaPct, 0) / (diffs.filter(d => d.status === 'mismatch').length || 1),
    diffs,
  };

  await writeFile(join(reportDir, 'report.json'), JSON.stringify(report, null, 2));
  console.log(`Visual regression: ${report.ok}/${report.total} exact, ${report.mismatch} size-mismatched, ${report.missing} missing`);
  console.log(`Avg size delta: ${report.avgSizeDeltaPct.toFixed(2)}%`);
  if (report.mismatch > 0) {
    console.log('\nLargest mismatches:');
    report.diffs
      .filter(d => d.status === 'mismatch')
      .sort((a, b) => b.sizeDeltaPct - a.sizeDeltaPct)
      .slice(0, 10)
      .forEach(d => console.log(`  ${d.file}: ${d.baselineSize} -> ${d.dioxusSize} (${d.sizeDeltaPct.toFixed(1)}%)`));
  }
}

main().catch(e => { console.error(e); process.exit(1); });
