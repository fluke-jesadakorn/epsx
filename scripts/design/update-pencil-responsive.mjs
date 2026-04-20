#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { execFileSync } from 'node:child_process';

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
const HELPER = '/tmp/pencil-mcp-tool.js';
const FILE_PATH = process.env.PENCIL_FILE ?? path.join('webDesign', 'AllDesign.pen');
const CAPTURE_ROOT = path.join(ROOT, '.generated', 'design-captures');
const VIEWPORTS = new Set(
  (process.env.PENCIL_UPDATE_VIEWPORTS ?? 'desktop,tablet,mobile')
    .split(',')
    .map((value) => value.trim().toLowerCase())
);
const SCREENSHOT_DIRS = {
  frontend: path.join(ROOT, 'apps', 'frontend', 'public', 'screenshots'),
  admin: path.join(ROOT, 'apps', 'admin-frontend', 'public', 'screenshots'),
};
const DEFAULT_EXTENSIONS = {
  frontend: ['png', 'webp', 'jpg', 'jpeg'],
  admin: ['png', 'webp', 'jpg', 'jpeg'],
};

function runPencil(command, payload) {
  const raw = execFileSync('node', [HELPER, command, JSON.stringify(payload)], {
    cwd: ROOT,
    encoding: 'utf8',
  });

  const jsonStart = raw.indexOf('{');
  const parsed = JSON.parse(raw.slice(jsonStart));
  const text = parsed.content?.find((item) => item.type === 'text')?.text;
  return text ?? '';
}

function loadBoards() {
  const responseText = runPencil('batch_get', {
    filePath: FILE_PATH,
    nodeIds: ['SAOta', 'ZFtTR', 'H4QsI', 'jG32H'],
    readDepth: 6,
    resolveVariables: true,
  });
  return JSON.parse(responseText);
}

function walkNodes(node, context, records) {
  const nextContext = { ...context };

  if (typeof node?.name === 'string') {
    if (node.name.includes('Frontend')) {
      nextContext.app = 'frontend';
      nextContext.mode = node.name.toLowerCase().includes('dark') ? 'dark' : 'light';
    } else if (node.name.includes('Admin')) {
      nextContext.app = 'admin';
      nextContext.mode = node.name.toLowerCase().includes('dark') ? 'dark' : 'light';
    }

    if (
      node.children !== undefined &&
      Array.isArray(node.children) &&
      node.children.some(
        (child) => typeof child?.name === 'string' && child.name === 'Responsive Frames'
      )
    ) {
      nextContext.state = node.name;
    }
  }

  if (
    nextContext.app !== undefined &&
    nextContext.mode !== undefined &&
    nextContext.state !== undefined &&
    typeof node?.name === 'string' &&
    ['Desktop', 'Tablet', 'Mobile'].includes(node.name) &&
    node.fill?.type === 'image'
  ) {
    records.push({
      nodeId: node.id,
      app: nextContext.app,
      mode: nextContext.mode,
      state: nextContext.state,
      viewport: node.name.toLowerCase(),
      currentUrl: node.fill.url,
    });
  }

  if (Array.isArray(node?.children)) {
    for (const child of node.children) {
      walkNodes(child, nextContext, records);
    }
  }
}

function buildUpdateRecords(boardTree) {
  const records = [];
  for (const board of boardTree) {
    walkNodes(board, {}, records);
  }
  return records;
}

function chunk(items, size) {
  const result = [];
  for (let index = 0; index < items.length; index += size) {
    result.push(items.slice(index, index + size));
  }
  return result;
}

function buildOperation(record, targetPath) {
  return `U(${JSON.stringify(record.nodeId)},{fill:{enabled:true,mode:"fit",type:"image",url:${JSON.stringify(targetPath)}}})`;
}

function resolveScreenshotPath(record) {
  const screenshotDir = SCREENSHOT_DIRS[record.app];
  const candidates = [];
  const basename = typeof record.currentUrl === 'string' ? path.basename(record.currentUrl) : '';

  for (const extension of DEFAULT_EXTENSIONS[record.app] ?? []) {
    candidates.push(path.join(screenshotDir, `${record.state}.${extension}`));
  }

  if (basename !== '') {
    candidates.push(path.join(screenshotDir, basename));
  }

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

function resolveTargetPath(record) {
  if (record.viewport !== 'desktop') {
    const capturePath = path.join(
      CAPTURE_ROOT,
      record.app,
      record.mode,
      record.viewport,
      `${record.state}.jpg`
    );

    if (fs.existsSync(capturePath)) {
      return {
        targetPath: capturePath,
        source: 'responsive-capture',
      };
    }
  }

  const screenshotPath = resolveScreenshotPath(record);
  if (screenshotPath !== null) {
    return {
      targetPath: screenshotPath,
      source: record.viewport === 'desktop' ? 'repo-screenshot' : 'repo-screenshot-fallback',
    };
  }

  return null;
}

function main() {
  const boards = loadBoards();
  const records = buildUpdateRecords(boards).filter((record) => VIEWPORTS.has(record.viewport));

  const updates = [];
  const skipped = [];

  for (const record of records) {
    const resolved = resolveTargetPath(record);

    if (resolved === null) {
      skipped.push({ ...record, reason: 'asset-missing' });
      continue;
    }

    updates.push({ ...record, ...resolved });
  }

  for (const group of chunk(updates, 24)) {
    const operations = group
      .map((item) => buildOperation(item, item.targetPath))
      .join('\n');
    runPencil('batch_design', {
      filePath: FILE_PATH,
      operations,
    });
  }

  const report = {
    updated: updates,
    skipped,
  };

  const reportPath = path.join(CAPTURE_ROOT, 'pencil-update-report.json');
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

  console.log(JSON.stringify({
    updated: updates.length,
    skipped: skipped.length,
    reportPath,
  }, null, 2));
}

main();
