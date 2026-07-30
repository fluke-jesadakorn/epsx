import { describe, expect, test } from 'bun:test';

import { captureStatusPassed } from './report';
import type { CaptureResult } from './types';

type OutcomeCheck = CaptureResult['outcomeChecks'][number];

function statusCheck(options: {
  actual: number;
  expected: number;
  passed: boolean;
  side?: 'source' | 'target' | 'both';
}): OutcomeCheck {
  return {
    outcome: {
      type: 'status',
      value: options.expected,
      side: options.side,
    },
    actual: options.actual,
    passed: options.passed,
  };
}

describe('captureStatusPassed', () => {
  test('accepts ordinary statuses and an exact declared dependency status', () => {
    expect(
      captureStatusPassed({
        side: 'target',
        status: 200,
        outcomeChecks: [],
      })
    ).toBe(true);
    expect(
      captureStatusPassed({
        side: 'target',
        status: 503,
        outcomeChecks: [
          statusCheck({
            actual: 503,
            expected: 503,
            passed: true,
            side: 'target',
          }),
        ],
      })
    ).toBe(true);
  });

  test('rejects missing, unexplained, mismatched, or other-side failures', () => {
    expect(
      captureStatusPassed({
        side: 'target',
        status: null,
        outcomeChecks: [],
      })
    ).toBe(false);
    expect(
      captureStatusPassed({
        side: 'target',
        status: 503,
        outcomeChecks: [],
      })
    ).toBe(false);
    expect(
      captureStatusPassed({
        side: 'target',
        status: 503,
        outcomeChecks: [
          statusCheck({
            actual: 503,
            expected: 503,
            passed: false,
            side: 'target',
          }),
          statusCheck({
            actual: 503,
            expected: 503,
            passed: true,
            side: 'source',
          }),
        ],
      })
    ).toBe(false);
  });
});
