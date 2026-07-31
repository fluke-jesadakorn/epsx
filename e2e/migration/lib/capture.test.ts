import { describe, expect, test } from 'bun:test';

import {
  blockingFailedRequests,
  expectedDocumentConsoleError,
  inputFilePayload,
  MIGRATION_CAPTURE_TIME,
  requiresDeterministicWallClock,
} from './capture';
import type { NetworkEntry, Scenario } from './types';

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

  test('accepts only the exact canceled harness-stubbed wallet config fetch', () => {
    const exactStubAbort: NetworkEntry = {
      kind: 'failed',
      method: 'GET',
      resourceType: 'fetch',
      url: 'https://api.web3modal.org/appkit/v1/config?projectId=00000000000000000000000000000000&st=appkit&sv=html-core-1.7.8',
      failure: 'net::ERR_ABORTED',
    };
    expect(blockingFailedRequests([exactStubAbort])).toEqual([]);

    const realProjectFailure = {
      ...exactStubAbort,
      url: 'https://api.web3modal.org/appkit/v1/config?projectId=real-project&st=appkit&sv=html-core-1.7.8',
    };
    expect(blockingFailedRequests([realProjectFailure])).toEqual([
      realProjectFailure,
    ]);

    const unexplainedFailure = {
      ...exactStubAbort,
      failure: 'net::ERR_FAILED',
    };
    expect(blockingFailedRequests([unexplainedFailure])).toEqual([
      unexplainedFailure,
    ]);
  });

  test('accepts an image canceled by a redirect when the exact image retry succeeds', () => {
    const url = 'http://127.0.0.1:4201/public/logos/epsx-icon.svg';
    const failure: NetworkEntry = {
      kind: 'failed',
      method: 'GET',
      resourceType: 'image',
      url,
      failure: 'net::ERR_ABORTED',
    };

    expect(
      blockingFailedRequests([
        failure,
        {
          kind: 'response',
          method: 'GET',
          resourceType: 'image',
          url,
          status: 200,
        },
      ])
    ).toEqual([]);
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

test('only the exact pinned admin chat hydration error is explained', () => {
  const scenario: Scenario = {
    id: 'pr6.admin.chat-detail',
    surface: 'admin',
    path: '/chat/550e8400-e29b-41d4-a716-446655440000',
    title: 'Pinned chat detail',
    state: { id: 'chat-admin', session: 'authenticated' },
    actions: [],
    outcomes: [],
    fixtureRequirements: [],
  };
  const entry = {
    type: 'error',
    location:
      'http://127.0.0.1:4101/_next/static/chunks/c6277_next_dist_client_45827b1f._.js:1375:24',
    text: [
      'Error: Event handlers cannot be passed to Client Component props.',
      'onUpdate={function onUpdate}',
      'The above error occurred in the <ChatConversationView> component.',
      'It was handled by the <ErrorBoundaryHandler> error boundary.',
    ].join('\n'),
  };
  const options = {
    entry,
    finalUrl: 'http://127.0.0.1:4101/chat/550e8400-e29b-41d4-a716-446655440000',
    finalStatus: 200,
    scenario,
    side: 'source' as const,
    networkEntries: [] as NetworkEntry[],
  };

  expect(expectedDocumentConsoleError(options)).toBe(true);
  expect(
    expectedDocumentConsoleError({
      ...options,
      entry: {
        ...entry,
        text: entry.text.replace(
          '<ChatConversationView>',
          '<Unknown>'
        ),
      },
    })
  ).toBe(true);
  expect(expectedDocumentConsoleError({ ...options, side: 'target' })).toBe(
    false
  );
  expect(
    expectedDocumentConsoleError({
      ...options,
      entry: { ...entry, text: 'different error' },
    })
  ).toBe(false);
  expect(
    expectedDocumentConsoleError({
      ...options,
      scenario: { ...scenario, path: '/chat/not-a-uuid' },
    })
  ).toBe(false);
});

test('deterministic multipart actions decode an exact in-memory file', () => {
  const payload = inputFilePayload({
    type: 'set-input-files',
    selector: 'input[type=file]',
    name: 'migration-proof.txt',
    mimeType: 'text/plain',
    contentBase64: 'RVBTWCBtaWdyYXRpb24gcHJvb2YK',
  });

  expect(payload.name).toBe('migration-proof.txt');
  expect(payload.mimeType).toBe('text/plain');
  expect(payload.buffer.toString('utf8')).toBe('EPSX migration proof\n');
});

test('deterministic multipart actions reject ambiguous metadata and bytes', () => {
  expect(() =>
    inputFilePayload({
      type: 'set-input-files',
      selector: 'input[type=file]',
      name: '../escape.txt',
      mimeType: 'text/plain',
      contentBase64: 'RVBTWA==',
    })
  ).toThrow();
  expect(() =>
    inputFilePayload({
      type: 'set-input-files',
      selector: 'input[type=file]',
      name: 'empty.txt',
      mimeType: 'text/plain',
      contentBase64: '',
    })
  ).toThrow();
});
