#!/usr/bin/env bun
/**
 * Visual regression: pixel-diff all Dioxus screenshots against the captured
 * Next.js baseline.
 *
 * Usage:
 *   bun scripts/visual-regression.mjs <baselineDir> <dioxusDir> [reportDir]
 */
import { readdir, mkdir, writeFile, stat, readFile } from 'node:fs/promises';
import { join, basename } from 'node:path';
import { PNG } from 'pngjs';

const [, , baselineDir, dioxusDir, reportDir] = process.argv;

if (!baselineDir || !dioxusDir || !reportDir) {
  console.error('Usage: bun scripts/visual-regression.mjs <baselineDir> <dioxusDir> <reportDir>');
  process.exit(1);
}

const TOLERANCE = 3;
const MINOR_THRESHOLD = 0.5;
const MAJOR_THRESHOLD = 5.0;

function diffPNGs(baselineBytes, dioxusBytes, width, height) {
  const a = PNG.sync.read(baselineBytes);
  const b = PNG.sync.read(dioxusBytes);
  const out = new PNG({ width, height });
  let different = 0;
  let sumSq = 0;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      const aj = (Math.min(y, a.height - 1) * a.width + x) * 4;
      const bj = (Math.min(y, b.height - 1) * b.width + x) * 4;
      const dr = Math.abs(a.data[aj] - b.data[bj]);
      const dg = Math.abs(a.data[aj + 1] - b.data[bj + 1]);
      const db = Math.abs(a.data[aj + 2] - b.data[bj + 2]);
      const max = Math.max(dr, dg, db);
      sumSq += dr * dr + dg * dg + db * db;
      if (max > TOLERANCE) {
        different++;
        out.data[i] = 255;
        out.data[i + 1] = 0;
        out.data[i + 2] = 0;
        out.data[i + 3] = 255;
      } else {
        out.data[i] = a.data[aj];
        out.data[i + 1] = a.data[aj + 1];
        out.data[i + 2] = a.data[aj + 2];
        out.data[i + 3] = 64;
      }
    }
  }
  const total = width * height;
  return { different, rmse: Math.sqrt(sumSq / (total * 3)), pct: (different / total) * 100 };
}

async function diffFile(baselinePath, dioxusPath) {
  const [baselineBytes, dioxusBytes] = await Promise.all([
    readFile(baselinePath),
    readFile(dioxusPath),
  ]);
  const aMeta = PNG.sync.read(baselineBytes);
  const bMeta = PNG.sync.read(dioxusBytes);
  if (aMeta.width !== bMeta.width) {
    return { file: basename(baselinePath), baseline: baselinePath, dioxus: dioxusPath, baselineSize: baselineBytes.length, dioxusSize: dioxusBytes.length, width: aMeta.width, height: Math.min(aMeta.height, bMeta.height), pixelsCompared: 0, pixelsDifferent: 0, pixelDiffPct: 100, rmse: 0, status: 'size-mismatch', threshold: 'major', diffImage: '' };
  }
  const w = aMeta.width;
  const h = Math.min(aMeta.height, bMeta.height);
  const r = diffPNGs(baselineBytes, dioxusBytes, w, h);
  let status, threshold;
  if (r.pct === 0) { status = 'identical'; threshold = 'identical'; }
  else if (r.pct < MINOR_THRESHOLD) { status = 'minor'; threshold = 'minor'; }
  else { status = 'major'; threshold = r.pct < MAJOR_THRESHOLD ? 'minor' : 'major'; }
  return { file: basename(baselinePath), baseline: baselinePath, dioxus: dioxusPath, baselineSize: baselineBytes.length, dioxusSize: dioxusBytes.length, width: w, height: h, pixelsCompared: w * h, pixelsDifferent: r.different, pixelDiffPct: r.pct, rmse: r.rmse, status, threshold, diffImage: '' };
}

async function main() {
  await mkdir(reportDir, { recursive: true });
  const diffImagesDir = join(reportDir, 'images');
  await mkdir(diffImagesDir, { recursive: true });

  const [baselineFiles, dioxusFiles] = await Promise.all([
    readdir(baselineDir).catch(() => []),
    readdir(dioxusDir).catch(() => []),
  ]);
  const baselineSet = new Set(baselineFiles);
  const dioxusSet = new Set(dioxusFiles);
  const all = new Set([...baselineFiles, ...dioxusFiles]);
  const diffs = [];

  for (const f of [...all].sort()) {
    const bp = join(baselineDir, f);
    const dp = join(dioxusDir, f);
    const [bs, ds] = await Promise.all([
      stat(bp).then(s => s.size).catch(() => 0),
      stat(dp).then(s => s.size).catch(() => 0),
    ]);

    if (bs === 0) { diffs.push({ file: f, baseline: bp, dioxus: dp, baselineSize: 0, dioxusSize: ds, width: 0, height: 0, pixelsCompared: 0, pixelsDifferent: 0, pixelDiffPct: 100, rmse: 0, status: 'missing-baseline', threshold: 'major', diffImage: '' }); continue; }
    if (ds === 0) { diffs.push({ file: f, baseline: bp, dioxus: dp, baselineSize: bs, dioxusSize: 0, width: 0, height: 0, pixelsCompared: 0, pixelsDifferent: 0, pixelDiffPct: 100, rmse: 0, status: 'missing-dioxus', threshold: 'major', diffImage: '' }); continue; }

    const result = await diffFile(bp, dp);
    if (!result) continue;
    diffs.push(result);
  }

  const summary = {
    baselineDir, dioxusDir, reportDir,
    tolerance: TOLERANCE, minorThresholdPct: MINOR_THRESHOLD, majorThresholdPct: MAJOR_THRESHOLD,
    total: diffs.length,
    identical: diffs.filter(d => d.status === 'identical').length,
    minor: diffs.filter(d => d.status === 'minor').length,
    major: diffs.filter(d => d.status === 'major').length,
    missingBaseline: diffs.filter(d => d.status === 'missing-baseline').length,
    missingDioxus: diffs.filter(d => d.status === 'missing-dioxus').length,
    sizeMismatch: diffs.filter(d => d.status === 'size-mismatch').length,
    pass: diffs.filter(d => d.status === 'identical' || d.status === 'minor').length,
    fail: diffs.filter(d => d.status === 'major' || d.status === 'missing-baseline' || d.status === 'missing-dioxus' || d.status === 'size-mismatch').length,
    avgDiffPct: diffs.reduce((a, d) => a + d.pixelDiffPct, 0) / diffs.length,
    avgRmse: diffs.reduce((a, d) => a + d.rmse, 0) / diffs.length,
    diffs: diffs.map(d => ({ ...d, baseline: undefined, dioxus: undefined, diffImage: undefined })),
  };

  await writeFile(join(reportDir, 'report.json'), JSON.stringify(summary, null, 2));
  await writeFile(join(reportDir, 'summary.md'), renderMd(summary));

  console.log(`\n=== Visual regression report ===`);
  console.log(`Baseline: ${baselineDir}`);
  console.log(`Dioxus:   ${dioxusDir}`);
  console.log(`Report:   ${reportDir}/report.json`);
  console.log(``);
  console.log(`Total:       ${summary.total}`);
  console.log(`Identical:   ${summary.identical}`);
  console.log(`Minor (<${MINOR_THRESHOLD}%):  ${summary.minor}`);
  console.log(`Major (>=${MAJOR_THRESHOLD}%): ${summary.major}`);
  console.log(`Missing:     ${summary.missingBaseline + summary.missingDioxus}`);
  console.log(`Size mismatch: ${summary.sizeMismatch}`);
  console.log(``);
  console.log(`PASS: ${summary.pass}/${summary.total}  FAIL: ${summary.fail}`);
  console.log(`Avg diff:  ${summary.avgDiffPct.toFixed(2)}%`);
  console.log(`Avg RMSE:  ${summary.avgRmse.toFixed(2)}`);

  if (summary.major > 0 || summary.sizeMismatch > 0 || summary.missingBaseline > 0 || summary.missingDioxus > 0) {
    console.log(`\nFailing routes:`);
    diffs
      .filter(d => d.status === 'major' || d.status === 'size-mismatch' || d.status === 'missing-baseline' || d.status === 'missing-dioxus')
      .forEach(d => console.log(`  [${d.status}] ${d.file}  diff=${d.pixelDiffPct.toFixed(2)}%  rmse=${d.rmse.toFixed(2)}`));
  }
}

function renderMd(s) {
  const pct = (n) => n.toFixed(2);
  let md = `# Visual Regression Report\n\n`;
  md += `- **Baseline**: \`${s.baselineDir}\`\n`;
  md += `- **Dioxus**:   \`${s.dioxusDir}\`\n`;
  md += `- **Report**:   \`${s.reportDir}\`\n`;
  md += `- **Tolerance**: ${s.tolerance} (per channel)\n`;
  md += `- **Thresholds**: minor < ${s.minorThresholdPct}%, major >= ${s.majorThresholdPct}%\n\n`;
  md += `## Summary\n\n`;
  md += `| Metric | Count |\n|---|---|\n`;
  md += `| Total | ${s.total} |\n`;
  md += `| Identical | ${s.identical} |\n`;
  md += `| Minor (< ${s.minorThresholdPct}%) | ${s.minor} |\n`;
  md += `| Major (>= ${s.majorThresholdPct}%) | ${s.major} |\n`;
  md += `| Size mismatch | ${s.sizeMismatch} |\n`;
  md += `| Missing baseline | ${s.missingBaseline} |\n`;
  md += `| Missing dioxus | ${s.missingDioxus} |\n`;
  md += `| **PASS** | **${s.pass}** |\n`;
  md += `| **FAIL** | **${s.fail}** |\n\n`;
  md += `Average pixel diff: **${pct(s.avgDiffPct)}%**\n`;
  md += `Average RMSE: **${pct(s.avgRmse)}**\n\n`;
  md += `## Per-route results\n\n`;
  md += `| Status | Route | Diff % | RMSE | Threshold |\n|---|---|---:|---:|---|\n`;
  for (const d of s.diffs) {
    md += `| ${d.status} | \`${d.file}\` | ${pct(d.pixelDiffPct)} | ${pct(d.rmse)} | ${d.threshold} |\n`;
  }
  return md;
}

main().catch(e => { console.error(e); process.exit(1); });
