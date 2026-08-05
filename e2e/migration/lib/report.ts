import { execFileSync } from 'node:child_process';
import { copyFile, rm, writeFile } from 'node:fs/promises';
import { basename, relative, resolve } from 'node:path';

import { loadManifest } from './config';
import {
  artifactManifest,
  ensureDirectory,
  listFiles,
  readJson,
  sha256File,
  slugify,
  writeJson,
} from './files';
import type {
  BackendContractReproducibility,
  CaptureResult,
  ComparisonResult,
  ResetProof,
  RuntimeConfig,
} from './types';

interface Reproducibility {
  schemaVersion: number;
  groupId: number;
  scenarioId: string;
  matrixId: string;
  repeats: number;
  checks: Record<string, boolean>;
  passed: boolean;
}

interface EvidenceRow {
  scenarioId: string;
  matrixId: string;
  reproducibility: Reproducibility;
  comparison: ComparisonResult;
  source: CaptureResult;
  target: CaptureResult;
  preReset: ResetProof;
  postReset: ResetProof;
  sourceImage: string;
  targetImage: string;
  diffImage: string;
  contactImage: string;
}

function markdownImage(label: string, path: string): string {
  return `[![${label}](./${path})](./${path})`;
}

export function captureStatusPassed(
  capture: Pick<CaptureResult, 'outcomeChecks' | 'side' | 'status'>
): boolean {
  if (capture.status === null) {
    return false;
  }
  if (capture.status < 500) {
    return true;
  }
  return capture.outcomeChecks.some(
    check =>
      check.outcome.type === 'status' &&
      (check.outcome.side === undefined ||
        check.outcome.side === 'both' ||
        check.outcome.side === capture.side) &&
      check.outcome.value === capture.status &&
      check.passed
  );
}

function capturePassed(capture: CaptureResult): boolean {
  return (
    captureStatusPassed(capture) &&
    capture.bodyTextLength > 50 &&
    capture.consoleErrors.length === 0 &&
    capture.pageErrors.length === 0 &&
    capture.failedRequests.length === 0 &&
    capture.outcomeChecks.every(check => check.passed)
  );
}

function evidenceRowPassed(row: EvidenceRow): boolean {
  return (
    row.reproducibility.passed &&
    row.preReset.passed &&
    row.postReset.passed &&
    row.comparison.approvedDifference &&
    capturePassed(row.source) &&
    capturePassed(row.target)
  );
}

// The report is intentionally assembled in one ordered pass so its table,
// contact sheets, and checksummed manifests describe the same evidence set.
// eslint-disable-next-line max-lines-per-function, complexity, sonarjs/cognitive-complexity
export async function generateReport(
  config: RuntimeConfig
): Promise<{ reportPath: string; artifactManifestPath: string }> {
  const files = await listFiles(config.artifactRoot);
  const reproducibilityPaths = files
    .filter(path => path.endsWith('/reproducibility.json'))
    .filter(
      path =>
        !path.includes('/backend-contracts/') &&
        !path.includes('/cross-browser/')
    );
  if (reproducibilityPaths.length === 0) {
    throw new Error('no reproducibility evidence was generated');
  }
  const manifest = await loadManifest();
  const expectedContractSuites = manifest.groups
    .filter(group => group.id >= 0 && group.id <= config.groupId)
    .flatMap(group => group.backendContracts ?? []);
  const contractPaths = files
    .filter(path => path.includes('/backend-contracts/'))
    .filter(path => path.endsWith('/reproducibility.json'))
    .sort();
  if (contractPaths.length !== expectedContractSuites.length) {
    throw new Error(
      `backend contract evidence has ${contractPaths.length} suites; expected ${expectedContractSuites.length}`
    );
  }
  const contractEvidence = await Promise.all(
    contractPaths.map(path => readJson<BackendContractReproducibility>(path))
  );

  const expectedEvidenceRoot = resolve(
    config.repoRoot,
    `docs/e2e/pr${config.groupId}/evidence`
  );
  if (resolve(config.evidenceRoot) !== expectedEvidenceRoot) {
    throw new Error(
      `refusing to replace unexpected evidence directory ${config.evidenceRoot}`
    );
  }
  await rm(config.evidenceRoot, { recursive: true, force: true });
  await ensureDirectory(config.evidenceRoot);

  const rows: EvidenceRow[] = [];
  for (const reproducibilityPath of reproducibilityPaths.sort()) {
    const reproducibility =
      await readJson<Reproducibility>(reproducibilityPath);
    const testRoot = resolve(reproducibilityPath, '..');
    const repeatRoot = resolve(testRoot, 'repeat-1');
    const [comparison, source, target, preReset, postReset] = await Promise.all(
      [
        readJson<ComparisonResult>(resolve(repeatRoot, 'comparison.json')),
        readJson<CaptureResult>(resolve(repeatRoot, 'source.capture.json')),
        readJson<CaptureResult>(resolve(repeatRoot, 'target.capture.json')),
        readJson<ResetProof>(resolve(repeatRoot, 'reset-pre.json')),
        readJson<ResetProof>(resolve(repeatRoot, 'reset-post.json')),
      ]
    );
    const prefix = `${slugify(reproducibility.scenarioId)}--${slugify(
      reproducibility.matrixId
    )}`;
    const sourceImage = `${prefix}--source.png`;
    const targetImage = `${prefix}--target.png`;
    const diffImage = `${prefix}--diff.png`;
    const contactImage = `${prefix}--contact.png`;
    await Promise.all([
      copyFile(
        source.screenshotPath,
        resolve(config.evidenceRoot, sourceImage)
      ),
      copyFile(
        target.screenshotPath,
        resolve(config.evidenceRoot, targetImage)
      ),
      copyFile(
        comparison.diffScreenshot,
        resolve(config.evidenceRoot, diffImage)
      ),
      copyFile(
        comparison.contactSheet,
        resolve(config.evidenceRoot, contactImage)
      ),
    ]);
    rows.push({
      scenarioId: reproducibility.scenarioId,
      matrixId: reproducibility.matrixId,
      reproducibility,
      comparison,
      source,
      target,
      preReset,
      postReset,
      sourceImage,
      targetImage,
      diffImage,
      contactImage,
    });
  }

  const fullManifest = {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    hashAlgorithm: 'sha256',
    group: config.groupId,
    sourceCommit: config.sourceCommit,
    targetCommit: config.targetCommit,
    artifactRoot: 'GitHub Actions artifact root',
    entries: await artifactManifest(config.artifactRoot),
  };
  const artifactManifestPath = resolve(
    config.evidenceRoot,
    'artifact-manifest.json'
  );
  await writeJson(artifactManifestPath, fullManifest);

  const finalReset = await readJson<ResetProof>(
    resolve(config.artifactRoot, 'reset-final.json')
  );
  const allPassed =
    rows.every(evidenceRowPassed) &&
    contractEvidence.every(contract => contract.passed) &&
    finalReset.passed;

  let markdown = `# PR ${config.groupId} — cumulative migration E2E evidence\n\n`;
  markdown += `Result: **${allPassed ? 'PASS' : 'FAIL'}**\n\n`;
  markdown += `Source Next.js SHA: \`${config.sourceCommit}\`\n\n`;
  markdown += `Target Rust/Dioxus SHA: \`${config.targetCommit}\`\n\n`;
  markdown += `Generated: ${new Date().toISOString()}\n\n`;
  markdown +=
    config.groupId === 0
      ? 'PR 0 is a capture/reproducibility gate. Visual differences are recorded and assigned to PR 1; they are not silently treated as parity.\n\n'
      : `This report covers every executable scenario owned by cumulative groups 0–${config.groupId}. Visual differences above 1% require a machine-readable non-styling exception.\n\n`;
  markdown += '## Scenario evidence\n\n';
  markdown +=
    '| Scenario | Matrix | Result / coverage | Next.js | Rust/Dioxus | Highlighted diff | Δ pixels | Difference disposition | Reset proof |\n';
  markdown += '|---|---|---|---|---|---|---:|---|---|\n';
  for (const row of rows) {
    const passed = evidenceRowPassed(row);
    markdown += `| \`${row.scenarioId}\` | \`${row.matrixId}\` | ${
      passed ? 'PASS' : 'FAIL'
    }; ${row.reproducibility.repeats} clean repeats | ${markdownImage(
      'Next.js source',
      row.sourceImage
    )} | ${markdownImage(
      'Rust/Dioxus target',
      row.targetImage
    )} | ${markdownImage('highlighted diff', row.diffImage)} | ${
      row.comparison.differencePercent
    }% | ${row.comparison.approvalReason} | pre=${
      row.preReset.passed ? 'PASS' : 'FAIL'
    }, post=${row.postReset.passed ? 'PASS' : 'FAIL'} |\n`;
  }
  markdown += '\n';
  if (contractEvidence.length > 0) {
    markdown += '## Backend-authoritative contract evidence\n\n';
    markdown +=
      '| Suite | Group | Result | Clean repeats | Rust tests per repeat | Claims | Source anchors |\n';
    markdown += '|---|---:|---|---:|---:|---|---|\n';
    for (const contract of contractEvidence) {
      markdown += `| \`${contract.suiteId}\` | ${contract.groupId} | ${
        contract.passed ? 'PASS' : 'FAIL'
      } | ${contract.repeats} | ${
        contract.results[0]?.passedTests ?? 0
      } | ${contract.claims.join('; ')} | ${contract.sources
        .map(source => `\`${source}\``)
        .join('<br>')} |\n`;
    }
    markdown +=
      '\nEach repeat has a checksummed Cargo log plus guarded pre/post PostgreSQL, Redis, Anvil, and fixture reset proofs in the full artifact. Test counts and ignored-test counts must be stable, every command must pass, and ignored tests are forbidden.\n\n';
  }
  markdown += '## Contact sheets\n\n';
  markdown +=
    'Each sheet is ordered **Next.js source → Rust/Dioxus target → highlighted pixel diff**.\n\n';
  for (const row of rows) {
    markdown += `### ${row.scenarioId} — ${row.matrixId}\n\n`;
    markdown += `![${row.scenarioId} ${row.matrixId} contact sheet](./${row.contactImage})\n\n`;
  }
  markdown += '## Runtime rollback gate\n\n';
  markdown +=
    'Every repeat restored a guarded `epsx_e2e_*` PostgreSQL database from its template, deleted only the `epsx:e2e:*` Redis namespace, reverted Anvil chain 31337 to its recorded snapshot, reset fixture requests/mutations, and cleared its isolated browser context. PostgreSQL checksums and row counts, transient queue/outbox emptiness, Redis hashes, Anvil account/block state, and fixture counters matched the baseline after reset.\n\n';
  markdown += `Final process-stopped rollback: **${
    finalReset.passed ? 'PASS' : 'FAIL'
  }**. The source and target applications were stopped before the final reset and smoke, preventing background polling from repopulating fixture or durable state. The full artifact manifest includes \`reset-final.json\` with every baseline comparison.\n\n`;
  markdown += '## Full artifacts\n\n';
  markdown +=
    'The CI artifact contains full-resolution PNGs, video, traces, HAR/network data, DOM, accessibility snapshots, browser/server logs, Playwright HTML, and reset proofs. [`artifact-manifest.json`](./artifact-manifest.json) records the SHA-256 and byte length of every file in that artifact.\n\n';
  markdown += '## Reproduce\n\n';
  markdown += '```bash\n';
  markdown += 'bun install --frozen-lockfile\n';
  markdown += 'bunx playwright install chromium\n';
  markdown += `bun e2e/migration/cli.ts run --group ${config.groupId}\n`;
  markdown += `bun e2e/migration/cli.ts verify-artifacts --group ${config.groupId}\n`;
  markdown += '```\n';

  const reportPath = resolve(config.evidenceRoot, 'report.md');
  await writeFile(reportPath, markdown, 'utf8');

  const evidenceFiles = (await listFiles(config.evidenceRoot)).filter(
    path =>
      basename(path) !== 'evidence-manifest.json' &&
      basename(path) !== 'report.md'
  );
  await writeJson(resolve(config.evidenceRoot, 'evidence-manifest.json'), {
    schemaVersion: 1,
    hashAlgorithm: 'sha256',
    entries: await Promise.all(
      evidenceFiles.map(async path => ({
        path: relative(config.evidenceRoot, path).replaceAll('\\', '/'),
        sha256: await sha256File(path),
      }))
    ),
  });
  return { reportPath, artifactManifestPath };
}

export async function verifyArtifactManifest(
  config: RuntimeConfig
): Promise<number> {
  const manifestPath = resolve(config.evidenceRoot, 'artifact-manifest.json');
  const manifest = await readJson<{
    sourceCommit: string;
    targetCommit: string;
    entries: Array<{ path: string; bytes: number; sha256: string }>;
  }>(manifestPath);
  if (manifest.sourceCommit !== config.sourceCommit) {
    throw new Error('artifact manifest source commit does not match');
  }
  if (manifest.targetCommit !== config.targetCommit) {
    execFileSync(
      'git',
      ['merge-base', '--is-ancestor', manifest.targetCommit, 'HEAD'],
      { cwd: config.repoRoot, stdio: 'ignore' }
    );
    const changedPaths = execFileSync(
      'git',
      ['diff', '--name-only', `${manifest.targetCommit}..HEAD`, '--'],
      { cwd: config.repoRoot, encoding: 'utf8' }
    )
      .split('\n')
      .filter(path => path !== '');
    const evidencePrefix = `docs/e2e/pr${config.groupId}/evidence/`;
    const nonEvidenceChanges = changedPaths.filter(
      path => !path.startsWith(evidencePrefix)
    );
    if (nonEvidenceChanges.length > 0) {
      throw new Error(
        `artifact manifest target ${manifest.targetCommit} is stale for non-evidence changes: ${nonEvidenceChanges.join(', ')}`
      );
    }
  }
  let checked = 0;
  for (const entry of manifest.entries) {
    if (entry.path.startsWith('/') || entry.path.split('/').includes('..')) {
      throw new Error(`unsafe artifact manifest path ${entry.path}`);
    }
    const path = resolve(config.artifactRoot, entry.path);
    const actual = await sha256File(path);
    if (actual !== entry.sha256) {
      throw new Error(
        `artifact hash mismatch for ${entry.path}: ${actual} != ${entry.sha256}`
      );
    }
    checked += 1;
  }
  return checked;
}
