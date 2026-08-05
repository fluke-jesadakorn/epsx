import { describe, expect, test } from 'bun:test';

import {
  blockingFailedRequests,
  MIGRATION_CAPTURE_TIME,
  requiresDeterministicWallClock,
} from './capture';
import type { NetworkEntry } from './types';

const nextStylesheet =
  'http://127.0.0.1:4100/_next/static/chunks/%5Broot-of-the-server%5D__9f26c86c._.css';

function failedStylesheet(url = nextStylesheet): NetworkEntry {
  return {
    kind: 'failed',
    method: 'GET',
    resourceType: 'stylesheet',
    url,
    failure: 'net::ERR_ABORTED',
  };
}

function successfulStylesheet(url = nextStylesheet): NetworkEntry {
  return {
    kind: 'response',
    method: 'GET',
    resourceType: 'stylesheet',
    url,
    status: 200,
  };
}

describe('blockingFailedRequests', () => {
  test('accepts an exact completed Next.js chunk stylesheet abort', () => {
    const entries = [successfulStylesheet(), failedStylesheet()];

    expect(blockingFailedRequests(entries)).toEqual([]);
  });

  test('blocks a generated stylesheet abort without a successful response', () => {
    const failure = failedStylesheet();

    expect(blockingFailedRequests([failure])).toEqual([failure]);
  });

  test('blocks successful arbitrary stylesheet aborts', () => {
    const url = 'http://127.0.0.1:4100/styles/custom.css';
    const failure = failedStylesheet(url);

    expect(
      blockingFailedRequests([successfulStylesheet(url), failure])
    ).toEqual([failure]);
  });

  test('blocks a generated stylesheet abort when only another URL succeeds', () => {
    const failure = failedStylesheet();

    expect(
      blockingFailedRequests([
        successfulStylesheet(`${nextStylesheet}?different=1`),
        failure,
      ])
    ).toEqual([failure]);
  });
});

test('migration capture wall clock is an exact canonical instant', () => {
  expect(new Date(MIGRATION_CAPTURE_TIME).toISOString()).toBe(
    MIGRATION_CAPTURE_TIME
  );
});

test('wall-clock control is limited to the admin root dashboard', () => {
  expect(requiresDeterministicWallClock({ surface: 'admin', path: '/' })).toBe(
    true
  );
  expect(
    requiresDeterministicWallClock({
      surface: 'admin',
      path: '/?from=admin-alias',
    })
  ).toBe(true);
  expect(
    requiresDeterministicWallClock({
      surface: 'admin',
      path: '/wallet-management/access/plans/example',
    })
  ).toBe(false);
  expect(
    requiresDeterministicWallClock({ surface: 'frontend', path: '/' })
  ).toBe(false);
});
