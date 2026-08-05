import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import pixelmatch from 'pixelmatch';
import { PNG } from 'pngjs';

import { writeJson } from './files';
import type {
  ApprovedDifferenceRegistry,
  CaptureResult,
  ComparisonResult,
} from './types';

interface ScreenshotReproducibility {
  equivalent: boolean;
  exactHashes: boolean;
  threshold: number;
  allowedDifferencePercent: number;
  comparisons: Array<{
    repeat: number;
    differingPixels: number;
    totalPixels: number;
    differencePercent: number;
    diffScreenshot: string;
  }>;
}

function scaledCopy(options: {
  source: PNG;
  target: PNG;
  targetX: number;
  width: number;
  height: number;
}): void {
  const { height, source, target, targetX, width } = options;
  for (let y = 0; y < height; y += 1) {
    const sourceY = Math.min(
      source.height - 1,
      Math.floor((y * source.height) / height)
    );
    for (let x = 0; x < width; x += 1) {
      const sourceX = Math.min(
        source.width - 1,
        Math.floor((x * source.width) / width)
      );
      const sourceOffset = (sourceY * source.width + sourceX) * 4;
      const targetOffset = (y * target.width + targetX + x) * 4;
      source.data.copy(
        target.data,
        targetOffset,
        sourceOffset,
        sourceOffset + 4
      );
    }
  }
}

// eslint-disable-next-line complexity
export async function compareCaptures(options: {
  source: CaptureResult;
  target: CaptureResult;
  artifactDirectory: string;
  captureOnly: boolean;
  approvedDifferences: ApprovedDifferenceRegistry;
}): Promise<ComparisonResult> {
  const {
    approvedDifferences,
    artifactDirectory,
    captureOnly,
    source,
    target,
  } = options;
  const sourcePng = PNG.sync.read(await readFile(source.screenshotPath));
  const targetPng = PNG.sync.read(await readFile(target.screenshotPath));
  if (
    sourcePng.width !== targetPng.width ||
    sourcePng.height !== targetPng.height
  ) {
    throw new Error(
      `screenshot dimensions differ: source=${sourcePng.width}x${sourcePng.height}, target=${targetPng.width}x${targetPng.height}`
    );
  }
  const diffPng = new PNG({
    width: sourcePng.width,
    height: sourcePng.height,
  });
  const differingPixels = pixelmatch(
    sourcePng.data,
    targetPng.data,
    diffPng.data,
    sourcePng.width,
    sourcePng.height,
    {
      threshold: 0.1,
      includeAA: false,
      alpha: 0.7,
      diffColor: [255, 0, 128],
      aaColor: [255, 196, 0],
    }
  );
  const diffScreenshot = resolve(artifactDirectory, 'diff.png');
  await writeFile(diffScreenshot, PNG.sync.write(diffPng));

  const thumbnailWidth = Math.min(480, sourcePng.width);
  const thumbnailHeight = Math.min(300, sourcePng.height);
  const contactPng = new PNG({
    width: thumbnailWidth * 3,
    height: thumbnailHeight,
  });
  scaledCopy({
    source: sourcePng,
    target: contactPng,
    targetX: 0,
    width: thumbnailWidth,
    height: thumbnailHeight,
  });
  scaledCopy({
    source: targetPng,
    target: contactPng,
    targetX: thumbnailWidth,
    width: thumbnailWidth,
    height: thumbnailHeight,
  });
  scaledCopy({
    source: diffPng,
    target: contactPng,
    targetX: thumbnailWidth * 2,
    width: thumbnailWidth,
    height: thumbnailHeight,
  });
  const contactSheet = resolve(artifactDirectory, 'contact-sheet.png');
  await writeFile(contactSheet, PNG.sync.write(contactPng));

  const totalPixels = sourcePng.width * sourcePng.height;
  const differencePercent = Number(
    ((differingPixels / totalPixels) * 100).toFixed(4)
  );
  const approval = approvedDifferences.items.find(
    item =>
      item.scenarioId === source.scenarioId &&
      item.matrixIds.includes(source.matrixId)
  );
  const withinDefault =
    differencePercent <=
    approvedDifferences.maximumUnapprovedDifferencePercent;
  const withinApprovedException =
    approval !== undefined &&
    approvedDifferences.allowedCategories.includes(approval.category) &&
    differencePercent <= approval.maximumDifferencePercent;
  const approvedDifference =
    captureOnly || withinDefault || withinApprovedException;
  const approvalReason = captureOnly
    ? 'PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1.'
    : withinDefault
      ? `Within the ${approvedDifferences.maximumUnapprovedDifferencePercent}% campaign visual threshold.`
      : withinApprovedException
        ? approval.reason
        : `Unapproved visual difference exceeds ${approvedDifferences.maximumUnapprovedDifferencePercent}%.`;
  const comparison: ComparisonResult = {
    schemaVersion: 1,
    scenarioId: source.scenarioId,
    matrixId: source.matrixId,
    repeat: source.repeat,
    sourceScreenshot: source.screenshotPath,
    targetScreenshot: target.screenshotPath,
    diffScreenshot,
    contactSheet,
    differingPixels,
    totalPixels,
    differencePercent,
    approvedDifference,
    approvalReason,
    approvalCategory: approval?.category,
    maximumAllowedDifferencePercent:
      approval?.maximumDifferencePercent ??
      approvedDifferences.maximumUnapprovedDifferencePercent,
  };
  await writeJson(resolve(artifactDirectory, 'comparison.json'), comparison);
  return comparison;
}

export async function compareRepeatScreenshots(options: {
  captures: CaptureResult[];
  artifactDirectory: string;
  side: 'source' | 'target';
}): Promise<ScreenshotReproducibility> {
  const { artifactDirectory, captures, side } = options;
  if (captures.length === 0) {
    throw new Error(`${side} repeat capture list is empty`);
  }
  const threshold = 0.1;
  const allowedDifferencePercent = 0.001;
  const reference = PNG.sync.read(
    await readFile(captures[0].screenshotPath)
  );
  const comparisons: ScreenshotReproducibility['comparisons'] = [];
  for (const capture of captures.slice(1)) {
    const candidate = PNG.sync.read(await readFile(capture.screenshotPath));
    if (
      candidate.width !== reference.width ||
      candidate.height !== reference.height
    ) {
      throw new Error(
        `${side} repeat screenshot dimensions differ: repeat-1=${reference.width}x${reference.height}, repeat-${capture.repeat}=${candidate.width}x${candidate.height}`
      );
    }
    const diff = new PNG({
      width: reference.width,
      height: reference.height,
    });
    const differingPixels = pixelmatch(
      reference.data,
      candidate.data,
      diff.data,
      reference.width,
      reference.height,
      {
        threshold,
        includeAA: false,
        alpha: 0.7,
        diffColor: [255, 0, 128],
        aaColor: [255, 196, 0],
      }
    );
    const totalPixels = reference.width * reference.height;
    const differencePercent = Number(
      ((differingPixels / totalPixels) * 100).toFixed(6)
    );
    const diffScreenshot = resolve(
      artifactDirectory,
      `${side}-repeat-1-vs-${capture.repeat}.png`
    );
    await writeFile(diffScreenshot, PNG.sync.write(diff));
    comparisons.push({
      repeat: capture.repeat,
      differingPixels,
      totalPixels,
      differencePercent,
      diffScreenshot,
    });
  }
  return {
    equivalent: comparisons.every(
      comparison =>
        comparison.differencePercent <= allowedDifferencePercent
    ),
    exactHashes:
      new Set(captures.map(capture => capture.screenshotSha256)).size === 1,
    threshold,
    allowedDifferencePercent,
    comparisons,
  };
}
