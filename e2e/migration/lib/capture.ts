import { rename, writeFile } from 'node:fs/promises';
import { Buffer } from 'node:buffer';
import { basename, resolve } from 'node:path';

import type { Browser, BrowserContext, Page, Route } from '@playwright/test';

import { ensureDirectory, sha256, sha256File, writeJson } from './files';
import type {
  BrowserLogEntry,
  CaptureResult,
  ColorScheme,
  NetworkEntry,
  Scenario,
  ScenarioAction,
  ScenarioOutcome,
  Viewport,
} from './types';

interface CaptureOptions {
  browser: Browser;
  side: 'source' | 'target';
  scenario: Scenario;
  matrixId: string;
  repeat: number;
  baseUrl: string;
  artifactDirectory: string;
  viewport: Viewport;
  colorScheme: ColorScheme;
  fixtureUrl: string;
  fixtureToken: string;
}

interface BrowserStorageState {
  cookies: number;
  localStorage: number;
  sessionStorage: number;
  indexedDatabases: number;
  cacheStorage: number;
  serviceWorkers: number;
}

export const MIGRATION_CAPTURE_TIME = '2026-07-29T05:00:00.000Z';

type InputFileAction = Extract<ScenarioAction, { type: 'set-input-files' }>;

export function inputFilePayload(action: InputFileAction): {
  name: string;
  mimeType: string;
  buffer: Buffer;
} {
  if (
    action.name.trim() !== action.name ||
    action.name === '' ||
    action.name.includes('/') ||
    action.name.includes('\\') ||
    !/^[a-zA-Z0-9!#$&^_.+-]+\/[a-zA-Z0-9!#$&^_.+-]+$/.test(action.mimeType)
  ) {
    throw new Error('invalid deterministic input-file metadata');
  }
  const normalized = action.contentBase64.replaceAll(/\s+/g, '');
  const buffer = Buffer.from(normalized, 'base64');
  if (
    buffer.length === 0 ||
    buffer.toString('base64').replaceAll(/=+$/g, '') !==
      normalized.replaceAll(/=+$/g, '')
  ) {
    throw new Error('invalid deterministic input-file content');
  }
  return {
    name: action.name,
    mimeType: action.mimeType,
    buffer,
  };
}

export function requiresDeterministicWallClock(
  scenario: Pick<Scenario, 'path' | 'surface'>
): boolean {
  return (
    scenario.surface === 'admin' &&
    new URL(scenario.path, 'http://epsx-e2e.invalid').pathname === '/'
  );
}

/**
 * The pinned Next.js admin access surfaces can run their client effects more
 * than once in development. Sonner keeps duplicate, hidden notification
 * nodes in the DOM, so the immutable source can otherwise produce different
 * semantic and accessibility hashes while rendering identical pixels. Keep
 * this normalization scoped to the affected source scenarios; the first
 * toast still proves the declared outcome.
 */
export function canonicalizeSourceTransientToasts(
  side: 'source' | 'target',
  scenarioId: string
): boolean {
  return (
    side === 'source' &&
    ['pr3.admin.access', 'pr3.admin.plan-conflict'].includes(scenarioId)
  );
}

/**
 * The pinned source wallet detail mounts Next's route announcer and dnd-kit
 * live-region helpers asynchronously. They are framework-owned transient
 * nodes, not application state, and can be present in only one clean repeat.
 * Keep their normalization limited to the exact source denial scenario that
 * demonstrated the race; screenshots and raw HTML remain untouched.
 */
export function canonicalizeSourceTransientFrameworkNodes(
  side: 'source' | 'target',
  scenarioId: string
): boolean {
  return (
    side === 'source' && scenarioId === 'pr3.admin.wallet-detail-forbidden'
  );
}

/**
 * The pinned source news editor hydrates its rich-text projection after the
 * textarea value is available. A clean repeat can therefore capture either
 * the rendered paragraph or the still-empty contenteditable shell while
 * pixels and accessibility remain identical. Normalize only that exact source
 * scenario in the semantic clone; raw HTML and visual evidence stay intact.
 */
export function canonicalizeSourceTransientEditorProjection(
  side: 'source' | 'target',
  scenarioId: string
): boolean {
  return side === 'source' && scenarioId === 'pr5.admin.news-unpublish';
}

async function storageState(
  context: BrowserContext,
  page: Page
): Promise<BrowserStorageState> {
  const browserState = await page.evaluate(async () => {
    const databases =
      typeof indexedDB.databases === 'function'
        ? await indexedDB.databases()
        : [];
    const cacheKeys = 'caches' in globalThis ? await caches.keys() : [];
    const registrations =
      'serviceWorker' in navigator
        ? await navigator.serviceWorker.getRegistrations()
        : [];
    return {
      localStorage: localStorage.length,
      sessionStorage: sessionStorage.length,
      indexedDatabases: databases.length,
      cacheStorage: cacheKeys.length,
      serviceWorkers: registrations.length,
    };
  });
  return {
    cookies: (await context.cookies()).length,
    ...browserState,
  };
}

async function clearBrowserStorage(
  context: BrowserContext,
  page: Page,
  proofPath: string
): Promise<void> {
  const before = await storageState(context, page);
  const origin = new URL(page.url()).origin;
  const probeUrl = `${origin}/__epsx_e2e_storage_probe__`;
  await context.route(probeUrl, route =>
    route.fulfill({
      status: 200,
      contentType: 'text/html',
      body: '<!doctype html><html><body>storage reset probe</body></html>',
    })
  );
  // Navigating to a same-origin inert document closes any IndexedDB handles
  // retained by the application while preserving access to that origin's
  // storage for deletion and verification.
  await page.goto(probeUrl, { waitUntil: 'domcontentloaded' });
  await page.evaluate(async () => {
    localStorage.clear();
    sessionStorage.clear();
    if ('serviceWorker' in navigator) {
      await Promise.all(
        (await navigator.serviceWorker.getRegistrations()).map(registration =>
          registration.unregister()
        )
      );
    }
    if ('caches' in globalThis) {
      await Promise.all((await caches.keys()).map(key => caches.delete(key)));
    }
    if (typeof indexedDB.databases === 'function') {
      await Promise.all(
        (await indexedDB.databases())
          .filter(
            (database): database is IDBDatabaseInfo & { name: string } =>
              database.name !== undefined && database.name !== ''
          )
          .map(
            database =>
              new Promise<void>((resolveDelete, rejectDelete) => {
                const request = indexedDB.deleteDatabase(database.name);
                request.onsuccess = () => resolveDelete();
                request.onerror = () =>
                  rejectDelete(
                    request.error ??
                      new Error(`failed to delete IndexedDB ${database.name}`)
                  );
                request.onblocked = () =>
                  rejectDelete(
                    new Error(`IndexedDB deletion blocked for ${database.name}`)
                  );
              })
          )
      );
    }
  });
  await context.clearCookies();
  const after = await storageState(context, page);
  await context.unroute(probeUrl);
  const passed = Object.values(after).every(value => value === 0);
  await writeJson(proofPath, {
    schemaVersion: 1,
    before,
    after,
    passed,
  });
  if (!passed) {
    throw new Error(`browser state reset failed: ${JSON.stringify(after)}`);
  }
}

function normalizedDom(semanticHtml: string): string {
  return (
    semanticHtml
      .replaceAll(/\bnonce="[^"]*"/g, 'nonce="<normalized>"')
      // React's useId allocation can shift between otherwise identical source
      // captures when Radix mounts a different set of client-only primitives.
      // Preserve every ID relationship while canonicalizing only Radix's
      // generated identifier payload. Accessibility snapshots are gated
      // separately and remain byte-exact.
      .replaceAll(/\bradix-_r_[0-9a-z]+_/g, 'radix-<normalized>')
      // dnd-kit's provider counter is process-global in the pinned source,
      // so otherwise identical captures can receive different numeric
      // suffixes. Canonicalize the two linked live-region IDs while keeping
      // their role-specific prefixes and relationships intact.
      .replaceAll(
        /\bDnd(DescribedBy|LiveRegion)-[0-9]+\b/g,
        'Dnd$1-<normalized>'
      )
      .replaceAll(/\sdata-nextjs-router-state-tree="[^"]*"/g, '')
      .replaceAll(/\s+/g, ' ')
      .trim()
  );
}

async function accessibilitySnapshot(
  page: Page,
  collapseDuplicateSourceToasts: boolean,
  hideSourceFrameworkTransients: boolean
): Promise<string> {
  const announcers = page.locator('next-route-announcer [role="alert"]');
  const priorAriaHidden = await announcers.evaluateAll(elements =>
    elements.map(element => element.getAttribute('aria-hidden'))
  );
  await announcers.evaluateAll(elements => {
    for (const element of elements) {
      // Next.js writes route announcements asynchronously. The announcement
      // has already been delivered by the time the stable page is sampled,
      // but its retained text can race between an empty alert and the last
      // route title. Exclude only that framework-owned transient node from
      // the static accessibility proof; application alerts remain exact.
      element.setAttribute('aria-hidden', 'true');
    }
  });
  const transientToasts = page.locator('[data-sonner-toast]');
  const priorToastAriaHidden = collapseDuplicateSourceToasts
    ? await transientToasts.evaluateAll(elements =>
        elements.map(element => element.getAttribute('aria-hidden'))
      )
    : [];
  if (collapseDuplicateSourceToasts) {
    await transientToasts.evaluateAll(elements => {
      elements.slice(1).forEach(element => {
        element.setAttribute('aria-hidden', 'true');
      });
    });
  }
  const frameworkTransients = hideSourceFrameworkTransients
    ? page.locator(
        'next-route-announcer, [id^="DndDescribedBy-"], [id^="DndLiveRegion-"]'
      )
    : null;
  const priorFrameworkTransientAriaHidden = frameworkTransients
    ? await frameworkTransients.evaluateAll(elements =>
        elements.map(element => element.getAttribute('aria-hidden'))
      )
    : [];
  if (frameworkTransients) {
    await frameworkTransients.evaluateAll(elements => {
      elements.forEach(element => element.setAttribute('aria-hidden', 'true'));
    });
  }
  try {
    return await page.locator('body').ariaSnapshot();
  } finally {
    if (frameworkTransients) {
      await frameworkTransients.evaluateAll((elements, priorValues) => {
        elements.forEach((element, index) => {
          const prior = priorValues[index];
          if (prior === null || prior === undefined) {
            element.removeAttribute('aria-hidden');
          } else {
            element.setAttribute('aria-hidden', prior);
          }
        });
      }, priorFrameworkTransientAriaHidden);
    }
    if (collapseDuplicateSourceToasts) {
      await transientToasts.evaluateAll((elements, priorValues) => {
        elements.forEach((element, index) => {
          const prior = priorValues[index];
          if (prior === null || prior === undefined) {
            element.removeAttribute('aria-hidden');
          } else {
            element.setAttribute('aria-hidden', prior);
          }
        });
      }, priorToastAriaHidden);
    }
    await announcers.evaluateAll((elements, priorValues) => {
      elements.forEach((element, index) => {
        const prior = priorValues[index];
        if (prior === null || prior === undefined) {
          element.removeAttribute('aria-hidden');
        } else {
          element.setAttribute('aria-hidden', prior);
        }
      });
    }, priorAriaHidden);
  }
}

function errorLocation(message: {
  location(): { url?: string; lineNumber?: number; columnNumber?: number };
}): string | undefined {
  const location = message.location();
  if (location.url === undefined || location.url === '') {
    return undefined;
  }
  return `${location.url}:${location.lineNumber ?? 0}:${location.columnNumber ?? 0}`;
}

export function blockingFailedRequests(
  entries: NetworkEntry[]
): NetworkEntry[] {
  return entries
    .filter(entry => entry.kind === 'failed')
    .filter(failure => {
      const failureUrl = new URL(failure.url);
      const isCompletedNextStylesheet =
        failure.method === 'GET' &&
        failure.resourceType === 'stylesheet' &&
        failureUrl.pathname.startsWith('/_next/static/chunks/') &&
        failureUrl.pathname.endsWith('.css');
      const isSuccessfulStreamAbort =
        failure.method !== undefined &&
        (['HEAD', 'POST'].includes(failure.method) ||
          (failure.method === 'GET' &&
            failure.resourceType === 'fetch' &&
            new URL(failure.url).searchParams.has('_rsc')));
      const isCanceledStubbedWalletConfig =
        failure.method === 'GET' &&
        failure.resourceType === 'fetch' &&
        failure.failure === 'net::ERR_ABORTED' &&
        failureUrl.origin === 'https://api.web3modal.org' &&
        failureUrl.pathname === '/appkit/v1/config' &&
        failureUrl.searchParams.get('projectId') ===
          '00000000000000000000000000000000' &&
        failureUrl.searchParams.get('st') === 'appkit' &&
        failureUrl.searchParams.get('sv') === 'html-core-1.7.8' &&
        [...failureUrl.searchParams.keys()].length === 3;
      const isCanceledDeveloperReloadAsset =
        failure.method === 'GET' &&
        failure.failure === 'net::ERR_ABORTED' &&
        failureUrl.origin === 'http://127.0.0.1:4201' &&
        failureUrl.pathname === '/public/dist/tailwind.css';
      const isCanceledDeveloperIconScript =
        failure.method === 'GET' &&
        failure.failure === 'net::ERR_ABORTED' &&
        failureUrl.origin === 'https://unpkg.com' &&
        failureUrl.pathname === '/lucide@latest';
      const isRetriedSubresourceAbort =
        failure.method === 'GET' &&
        failure.resourceType === 'image' &&
        entries.some(
          entry =>
            entry.kind === 'response' &&
            entry.method === failure.method &&
            entry.url === failure.url &&
            entry.status !== undefined &&
            entry.status >= 200 &&
            entry.status < 400
        );
      const successfulResponseAbort =
        (isSuccessfulStreamAbort ||
          isCompletedNextStylesheet ||
          isRetriedSubresourceAbort) &&
        failure.failure === 'net::ERR_ABORTED' &&
        entries.some(
          entry =>
            entry.kind === 'response' &&
            entry.method === failure.method &&
            entry.url === failure.url &&
            entry.status !== undefined &&
            entry.status >= 200 &&
            entry.status < 400
        );
      // Chromium can report an abort after a successful HEAD response, a
      // completed Next.js RSC stream, a generated Next.js stylesheet, or a
      // subresource canceled by an immediate canonical redirect and then
      // fetched successfully on the destination. Only exact 2xx/3xx
      // URL/method matches qualify; both raw entries remain in network.json.
      // The immutable source can also cancel its exact harness-stubbed wallet
      // config fetch during a redirect before Playwright emits the synthetic
      // response. The full URL, dummy project, client version, and abort remain
      // in network.json; every other external or wallet request still blocks.
      return (
        !successfulResponseAbort &&
        !isCanceledStubbedWalletConfig &&
        !isCanceledDeveloperReloadAsset &&
        !isCanceledDeveloperIconScript
      );
    });
}

export function expectedDocumentConsoleError(options: {
  entry: BrowserLogEntry;
  finalUrl: string;
  finalStatus: number | null;
  scenario: Scenario;
  side: 'source' | 'target';
  networkEntries: NetworkEntry[];
}): boolean {
  const { entry, finalStatus, finalUrl, networkEntries, scenario, side } =
    options;
  const scenarioPath = new URL(scenario.path, 'http://epsx-e2e.invalid')
    .pathname;
  const pinnedAdminChatHydrationError =
    side === 'source' &&
    scenario.surface === 'admin' &&
    /^\/chat\/[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(
      scenarioPath
    ) &&
    entry.type === 'error' &&
    entry.location?.startsWith(
      `${new URL(finalUrl).origin}/_next/static/chunks/c6277_next_dist_client_`
    ) === true &&
    entry.text.includes(
      'Error: Event handlers cannot be passed to Client Component props.'
    ) &&
    entry.text.includes('onUpdate={function onUpdate}') &&
    entry.text.includes(
      'It was handled by the <ErrorBoundaryHandler> error boundary.'
    );
  if (pinnedAdminChatHydrationError) {
    return true;
  }
  const pinnedDeveloperApiNotFound =
    side === 'source' &&
    (scenario.id.startsWith('pr7.') ||
      scenario.id === 'pr9.frontend.developer') &&
    entry.type === 'error' &&
    entry.text.includes(
      'Failed to load resource: the server responded with a status of 404 (Not Found)'
    ) &&
    entry.location !== undefined &&
    (() => {
      const location = new URL(entry.location.replace(/:\d+:\d+$/, ''));
      if (location.hostname !== 'localhost' || location.port !== '8080') {
        return false;
      }
      return [
        '/api/developer-portal/my-keys',
        '/api/admin/developer/keys',
        '/api/admin/developer/modules',
        '/api/admin/plans',
      ].includes(location.pathname);
    })();
  if (pinnedDeveloperApiNotFound) {
    return true;
  }
  const pinnedViemBigIntMathError =
    side === 'source' &&
    [
      'pr2.permissions.verified',
      'pr4.frontend.home-rankings',
      'pr4.frontend.analytics-malformed',
    ].includes(scenario.id) &&
    entry.type === 'error' &&
    entry.location?.includes('/_next/static/chunks/6063a_next_dist_client_') ===
      true &&
    entry.text.includes(
      'TypeError: Cannot convert a BigInt value to a number'
    ) &&
    entry.text.includes('Math.pow') &&
    entry.text.includes('node_modules_viem__esm_') &&
    entry.text.includes('GlobalErrorBoundary');
  if (pinnedViemBigIntMathError) {
    return true;
  }
  const expectedStatus = scenario.outcomes.find(
    outcome =>
      outcome.type === 'status' &&
      (outcome.side === undefined ||
        outcome.side === 'both' ||
        outcome.side === side)
  );
  if (
    expectedStatus?.type !== 'status' ||
    finalStatus !== expectedStatus.value ||
    entry.location?.startsWith(`${finalUrl}:`) !== true ||
    !entry.text.includes(`status of ${expectedStatus.value}`)
  ) {
    return false;
  }
  return networkEntries.some(
    networkEntry =>
      networkEntry.kind === 'response' &&
      networkEntry.resourceType === 'document' &&
      networkEntry.url === finalUrl &&
      networkEntry.status === expectedStatus.value
  );
}

async function waitForStableMeaningfulBody(page: Page): Promise<void> {
  await page
    .waitForFunction(
      ({ minimumLength, stableForMs }) => {
        const bodyTextLength = document.body?.innerText.trim().length ?? 0;
        const stateContainer = globalThis as typeof globalThis & {
          __epsxE2eBodyTextState?: {
            length: number;
            observedAt: number;
          };
        };
        const previous = stateContainer.__epsxE2eBodyTextState;
        if (previous?.length !== bodyTextLength) {
          stateContainer.__epsxE2eBodyTextState = {
            length: bodyTextLength,
            // Wall time is fixed later in the capture so application clocks
            // remain deterministic. A monotonic duration keeps this stability
            // sampler independent from that deliberate wall-clock control.
            observedAt: performance.now(),
          };
          return false;
        }
        return (
          bodyTextLength > minimumLength &&
          performance.now() - previous.observedAt >= stableForMs
        );
      },
      { minimumLength: 50, stableForMs: 500 },
      { polling: 100, timeout: 15_000 }
    )
    // A timeout is captured as evidence and rejected by the scenario's
    // bodyTextLength assertion. Continuing here preserves the DOM, screenshot,
    // trace, network log, and reset proof for that failure.
    .catch(() => undefined);
}

async function waitForVisibleImages(page: Page): Promise<void> {
  await page.waitForFunction(
    () => {
      const visibleImages = Array.from(document.images).filter(image => {
        const rect = image.getBoundingClientRect();
        const style = getComputedStyle(image);
        return (
          rect.width > 0 &&
          rect.height > 0 &&
          rect.bottom > 0 &&
          rect.right > 0 &&
          rect.top < window.innerHeight &&
          rect.left < window.innerWidth &&
          style.display !== 'none' &&
          style.visibility !== 'hidden'
        );
      });
      return visibleImages.every(image => image.complete);
    },
    undefined,
    // The immutable Next.js source dev server compiles static image routes on
    // demand. Under a cumulative Linux gate that response can exceed 15
    // seconds even though it completes successfully (and remains recorded in
    // the HAR). Keep the readiness requirement strict, but give the pinned
    // source enough bounded time to finish serving the visible asset.
    { polling: 100, timeout: 60_000 }
  );
  await page.evaluate(async () => {
    const visibleImages = Array.from(document.images).filter(image => {
      const rect = image.getBoundingClientRect();
      const style = getComputedStyle(image);
      return (
        rect.width > 0 &&
        rect.height > 0 &&
        rect.bottom > 0 &&
        rect.right > 0 &&
        rect.top < window.innerHeight &&
        rect.left < window.innerWidth &&
        style.display !== 'none' &&
        style.visibility !== 'hidden'
      );
    });
    await Promise.all(
      visibleImages
        .filter(image => image.naturalWidth > 0)
        .map(image => image.decode())
    );
    await new Promise<void>(resolveFrame =>
      requestAnimationFrame(() => resolveFrame())
    );
  });
}

// eslint-disable-next-line max-params
async function fixtureControl<T>(
  fixtureUrl: string,
  fixtureToken: string,
  path: string,
  init?: RequestInit
): Promise<T> {
  const headers = new Headers(init?.headers);
  headers.set('x-epsx-e2e-token', fixtureToken);
  const response = await fetch(new URL(path, fixtureUrl), {
    ...init,
    headers,
  });
  if (!response.ok) {
    throw new Error(
      `fixture control ${path} failed with HTTP ${response.status}`
    );
  }
  return (await response.json()) as T;
}

async function proxySourceDependency(
  route: Route,
  fixtureUrl: string,
  accessToken: string | undefined
): Promise<void> {
  const request = route.request();
  const originalUrl = new URL(request.url());
  const fixtureRequestUrl = new URL(
    `${originalUrl.pathname}${originalUrl.search}`,
    fixtureUrl
  );
  const headers = new Headers(request.headers());
  headers.delete('content-length');
  headers.delete('host');
  if (!headers.has('authorization') && accessToken !== undefined) {
    headers.set('authorization', `Bearer ${accessToken}`);
  }
  const method = request.method();
  const response = await fetch(fixtureRequestUrl, {
    method,
    headers,
    body: ['GET', 'HEAD'].includes(method) ? undefined : request.postData(),
    redirect: 'manual',
  });
  const responseHeaders = new Headers(response.headers);
  responseHeaders.delete('content-encoding');
  responseHeaders.delete('content-length');
  responseHeaders.delete('transfer-encoding');
  await route.fulfill({
    status: response.status,
    headers: Object.fromEntries(responseHeaders.entries()),
    body: Buffer.from(await response.arrayBuffer()),
  });
}

// eslint-disable-next-line complexity
async function configureScenarioState(options: {
  context: BrowserContext;
  baseUrl: string;
  fixtureUrl: string;
  fixtureToken: string;
  scenario: Scenario;
  side: 'source' | 'target';
}): Promise<string | undefined> {
  const { baseUrl, context, fixtureToken, fixtureUrl, scenario, side } =
    options;
  const fixtureModeSide = scenario.state.fixtureModeSide ?? 'both';
  if (
    scenario.state.fixtureMode !== undefined &&
    (fixtureModeSide === 'both' || fixtureModeSide === side)
  ) {
    await fixtureControl(fixtureUrl, fixtureToken, '/__e2e/mode', {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ mode: scenario.state.fixtureMode }),
    });
  }
  if (scenario.state.session !== 'authenticated') {
    return undefined;
  }
  const audience =
    side === 'source'
      ? (scenario.state.sourceAudience ?? scenario.state.audience)
      : scenario.state.audience;
  if (audience === undefined) {
    throw new Error(
      `authenticated scenario ${scenario.id} must declare an audience`
    );
  }
  const permissions = (scenario.state.permissions ?? []).join(' ');
  const session = await fixtureControl<{ accessToken: string }>(
    fixtureUrl,
    fixtureToken,
    `/__e2e/session?audience=${encodeURIComponent(audience)}&permissions=${encodeURIComponent(
      permissions
    )}&key_id=${encodeURIComponent(
      scenario.state.tokenKeyId ?? 'epsx-e2e-rs256-v1'
    )}`
  );
  const targetName =
    scenario.surface === 'admin'
      ? 'epsx.admin.access_token'
      : 'epsx.frontend.access_token';
  await context.addCookies([
    {
      name: side === 'source' ? 'epsx.access_token' : targetName,
      value: session.accessToken,
      url: baseUrl,
      httpOnly: true,
      sameSite: 'Lax',
    },
  ]);
  return session.accessToken;
}

// eslint-disable-next-line complexity
async function applyActions(options: {
  context: BrowserContext;
  page: Page;
  actions: ScenarioAction[];
  side: 'source' | 'target';
  matrixId: string;
}): Promise<number | null> {
  const { actions, context, matrixId, page, side } = options;
  let status: number | null = null;
  for (const action of actions) {
    if (
      action.side !== undefined &&
      action.side !== 'both' &&
      action.side !== side
    ) {
      continue;
    }
    if (
      action.matrixIds !== undefined &&
      !action.matrixIds.includes(matrixId)
    ) {
      continue;
    }
    if (action.type === 'click') {
      await page.locator(action.selector).click();
    } else if (action.type === 'fill') {
      await page.locator(action.selector).fill(action.value);
    } else if (action.type === 'press') {
      await page.locator(action.selector).press(action.key);
    } else if (action.type === 'set-input-files') {
      await page
        .locator(action.selector)
        .setInputFiles(inputFilePayload(action));
    } else if (action.type === 'reload') {
      status =
        (await page.reload({ waitUntil: 'domcontentloaded' }))?.status() ??
        null;
    } else if (action.type === 'set-offline') {
      await context.setOffline(action.offline);
    } else if (action.type === 'navigate') {
      status =
        (
          await page.goto(new URL(action.path, page.url()).toString(), {
            waitUntil: 'domcontentloaded',
          })
        )?.status() ?? null;
    } else if (action.type === 'clear-cookies') {
      await context.clearCookies();
    } else {
      await page.locator(action.selector).waitFor({ state: 'visible' });
    }
  }
  return status;
}

// The manifest outcome union is deliberately evaluated in one exhaustive
// boundary so unsupported assertions cannot silently pass.
// eslint-disable-next-line max-params, complexity
async function checkOutcome(
  page: Page,
  outcome: ScenarioOutcome,
  status: number | null,
  side: 'source' | 'target'
): Promise<CaptureResult['outcomeChecks'][number]> {
  if (
    outcome.side !== undefined &&
    outcome.side !== 'both' &&
    outcome.side !== side
  ) {
    return { outcome, passed: true, actual: 'not-applicable' };
  }
  let actual: string | number | boolean;
  let passed: boolean;
  if (outcome.type === 'path') {
    actual = new URL(page.url()).pathname;
    passed = actual === outcome.value;
  } else if (outcome.type === 'query') {
    actual = new URL(page.url()).searchParams.get(outcome.key) ?? '';
    passed = actual === outcome.value;
  } else if (outcome.type === 'text') {
    actual = await page.locator('body').innerText();
    passed = actual.includes(outcome.value);
  } else if (outcome.type === 'text-absent') {
    actual = await page.locator('body').innerText();
    passed = !actual.includes(outcome.value);
  } else if (outcome.type === 'selector') {
    actual = await page.locator(outcome.value).count();
    passed = actual > 0;
  } else if (outcome.type === 'attribute') {
    actual =
      (await page.locator(outcome.selector).getAttribute(outcome.name)) ?? '';
    passed = actual === outcome.value;
  } else if (outcome.type === 'focused') {
    actual = await page
      .locator(outcome.selector)
      .evaluate(element => element === document.activeElement);
    passed = actual;
  } else if (outcome.type === 'no-horizontal-overflow') {
    actual = await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth
    );
    passed = actual;
  } else {
    actual = status ?? -1;
    passed = actual === outcome.value;
  }
  return { outcome, passed, actual };
}

// Capture deliberately keeps the context lifecycle and every artifact in one
// fail-closed boundary so a partial capture cannot be reported as complete.
// eslint-disable-next-line max-lines-per-function
export async function captureSide(
  options: CaptureOptions
): Promise<CaptureResult> {
  const {
    artifactDirectory,
    baseUrl,
    browser,
    colorScheme,
    fixtureToken,
    fixtureUrl,
    matrixId,
    repeat,
    scenario,
    side,
    viewport,
  } = options;
  await ensureDirectory(artifactDirectory);
  const screenshotPath = resolve(artifactDirectory, `${side}.png`);
  const domPath = resolve(artifactDirectory, `${side}.html`);
  const normalizedDomPath = resolve(
    artifactDirectory,
    `${side}.normalized.html`
  );
  const accessibilityPath = resolve(
    artifactDirectory,
    `${side}.accessibility.yml`
  );
  const networkPath = resolve(artifactDirectory, `${side}.network.json`);
  const browserLogPath = resolve(artifactDirectory, `${side}.browser-log.json`);
  const redirectsPath = resolve(artifactDirectory, `${side}.redirects.json`);
  const tracePath = resolve(artifactDirectory, `${side}.trace.zip`);
  const harPath = resolve(artifactDirectory, `${side}.har`);
  const browserResetPath = resolve(
    artifactDirectory,
    `${side}.browser-reset.json`
  );
  const videoDirectory = resolve(artifactDirectory, '.video');

  const context = await browser.newContext({
    viewport,
    colorScheme,
    bypassCSP: side === 'source',
    reducedMotion: 'reduce',
    locale: 'en-US',
    timezoneId: 'UTC',
    serviceWorkers: 'allow',
    recordHar: {
      path: harPath,
      content: 'embed',
      mode: 'full',
    },
    recordVideo: {
      dir: videoDirectory,
      size: viewport,
    },
  });
  const sourceAccessToken = await configureScenarioState({
    context,
    baseUrl,
    fixtureUrl,
    fixtureToken,
    scenario,
    side,
  });
  await context.route('https://api.web3modal.org/appkit/v1/config**', route =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ features: [] }),
    })
  );
  await context.route('https://pulse.walletconnect.org/**', route =>
    route.fulfill({ status: 204, body: '' })
  );
  if (side === 'source') {
    await context.route(
      /^http:\/\/(?:localhost|127\.0\.0\.1):8080\/.*/,
      route => proxySourceDependency(route, fixtureUrl, sourceAccessToken)
    );
    if (new URL(scenario.path, baseUrl).pathname === '/manual') {
      // Next 16's pinned Turbopack dev output lowers viem's native BigInt
      // exponentiation to Math.pow(BigInt, BigInt), which throws even though
      // the original production expression is valid. Restore the original
      // operator semantics only for the affected immutable source route;
      // numeric Math.pow behavior is delegated unchanged.
      await context.addInitScript(() => {
        const nativePow = Math.pow;
        Object.defineProperty(Math, 'pow', {
          configurable: true,
          writable: true,
          value: (
            base: number | bigint,
            exponent: number | bigint
          ): number | bigint => {
            if (typeof base === 'bigint' && typeof exponent === 'bigint') {
              return base ** exponent;
            }
            return nativePow(base as number, exponent as number);
          },
        });
      });
    }
  }
  await context.addInitScript((theme: string) => {
    localStorage.setItem('theme', theme);
    const applyTheme = (): void => {
      const root = document.querySelector<HTMLElement>('html');
      if (root !== null) {
        root.dataset.theme = theme;
        root.classList.toggle('dark', theme === 'dark');
      }
    };
    applyTheme();
    document.addEventListener('DOMContentLoaded', applyTheme, { once: true });
  }, colorScheme);
  await context.tracing.start({
    screenshots: true,
    snapshots: true,
    sources: true,
  });

  const page = await context.newPage();
  const consoleEntries: BrowserLogEntry[] = [];
  const pageErrors: string[] = [];
  const networkEntries: NetworkEntry[] = [];
  const redirects: string[] = [];
  page.on('console', message => {
    consoleEntries.push({
      type: message.type(),
      text: message.text(),
      location: errorLocation(message),
    });
  });
  page.on('pageerror', error => pageErrors.push(error.message));
  page.on('request', request => {
    networkEntries.push({
      kind: 'request',
      method: request.method(),
      resourceType: request.resourceType(),
      url: request.url(),
    });
  });
  page.on('response', response => {
    networkEntries.push({
      kind: 'response',
      method: response.request().method(),
      status: response.status(),
      resourceType: response.request().resourceType(),
      url: response.url(),
    });
  });
  page.on('requestfailed', request => {
    networkEntries.push({
      kind: 'failed',
      method: request.method(),
      resourceType: request.resourceType(),
      url: request.url(),
      failure: request.failure()?.errorText ?? 'unknown',
    });
  });
  page.on('framenavigated', frame => {
    if (frame === page.mainFrame()) {
      redirects.push(frame.url());
    }
  });

  const requestedUrl = new URL(scenario.path, baseUrl).toString();
  let videoPath: string | undefined;
  try {
    const response = await page.goto(requestedUrl, {
      waitUntil: 'domcontentloaded',
      timeout: 60_000,
    });
    await page
      .waitForLoadState('networkidle', { timeout: 7_500 })
      .catch(() => undefined);
    await page.addStyleTag({
      content:
        '*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}nextjs-portal{display:none!important}',
    });
    await page.evaluate(async () => {
      if ('fonts' in document) {
        await document.fonts.ready;
      }
      window.scrollTo(0, 0);
    });
    // Next.js Fast Refresh can transiently clear innerText after
    // domcontentloaded/networkidle while the accessibility tree and pixels are
    // already being rebuilt. Require a quiet, meaningful interval before
    // sampling so slower CI runners capture the same stable state as local runs.
    await waitForStableMeaningfulBody(page);
    await waitForVisibleImages(page);
    if (requiresDeterministicWallClock(scenario)) {
      // Install the deterministic wall clock only after hydration. The pinned
      // Next.js admin root renders a live client clock; installing before
      // hydration would intentionally disagree with its server render, while
      // leaving real time enabled makes otherwise clean repeats differ in DOM,
      // accessibility text, and pixels. Restrict control to that surface so
      // unrelated source routes preserve their original async timing.
      await page.clock.setFixedTime(MIGRATION_CAPTURE_TIME);
      // Keep this delay on the Node side: browser clock emulation deliberately
      // owns page-side timer primitives as well as Date.
      await new Promise(resolveDelay => setTimeout(resolveDelay, 1_100));
      await page.evaluate(() => undefined);
    }
    const actionStatus = await applyActions({
      context,
      page,
      actions: scenario.actions,
      side,
      matrixId,
    });
    await page
      .waitForLoadState('domcontentloaded', { timeout: 5_000 })
      .catch(() => undefined);
    await waitForStableMeaningfulBody(page);
    await waitForVisibleImages(page);

    const finalUrl = page.url();
    const finalStatus = actionStatus ?? response?.status() ?? null;
    const title = await page.title();
    const bodyTextLength = (await page.locator('body').innerText()).trim()
      .length;
    const html = await page.content();
    const collapseDuplicateSourceToasts = canonicalizeSourceTransientToasts(
      side,
      scenario.id
    );
    const hideSourceFrameworkTransients =
      canonicalizeSourceTransientFrameworkNodes(side, scenario.id);
    const normalizeSourceEditorProjection =
      canonicalizeSourceTransientEditorProjection(side, scenario.id);
    const semanticHtml = await page.evaluate(
      ({
        collapseDuplicateSourceToasts: collapseToasts,
        hideFrameworkTransients,
        normalizeEditorProjection,
      }) => {
        const clone = document.body.cloneNode(true) as HTMLElement;
        // Next.js can leave streamed document metadata in `<body>` for a
        // hydration turn, then move it into `<head>` without changing pixels or
        // the accessibility tree. Document metadata is not part of the body
        // semantic contract (and the final title is captured separately), so
        // exclude it alongside non-semantic runtime script/style nodes.
        for (const node of clone.querySelectorAll(
          'script, style, title, meta'
        )) {
          node.remove();
        }
        if (collapseToasts) {
          clone
            .querySelectorAll('[data-sonner-toast]')
            .forEach((element, index) => {
              if (index > 0) {
                element.remove();
              }
            });
        }
        if (hideFrameworkTransients) {
          clone
            .querySelectorAll(
              'next-route-announcer, [id^="DndDescribedBy-"], [id^="DndLiveRegion-"]'
            )
            .forEach(element => element.remove());
        }
        if (normalizeEditorProjection) {
          const editor = clone.querySelector<HTMLDivElement>(
            'div[contenteditable="true"]'
          );
          const textarea = clone.querySelector<HTMLTextAreaElement>(
            'textarea[placeholder="Write markdown content…"]'
          );
          if (editor && textarea) {
            editor.textContent = textarea.value;
          }
        }
        for (const element of [clone, ...clone.querySelectorAll('*')]) {
          if (
            element.tagName === 'INPUT' &&
            element.getAttribute('name') === 'idempotency_key'
          ) {
            // Idempotency tokens intentionally contain fresh entropy on every
            // render. Preserve the input and its contract while canonicalizing
            // only the volatile value for exact semantic-DOM repeat proofs.
            element.setAttribute('value', '__EPSX_IDEMPOTENCY_KEY__');
          }
          const runtimeAttributes = Array.from(element.attributes)
            .map(attribute => attribute.name)
            .filter(name => name.startsWith('data-nextjs') || name === 'nonce');
          for (const name of runtimeAttributes) {
            element.removeAttribute(name);
          }
          const attributes = Array.from(element.attributes)
            .map(attribute => [attribute.name, attribute.value] as const)
            .sort(([left], [right]) => left.localeCompare(right));
          while (element.attributes.length > 0) {
            element.removeAttribute(element.attributes[0].name);
          }
          for (const [name, value] of attributes) {
            element.setAttribute(name, value);
          }
        }
        return clone.outerHTML;
      },
      {
        collapseDuplicateSourceToasts,
        hideFrameworkTransients: hideSourceFrameworkTransients,
        normalizeEditorProjection: normalizeSourceEditorProjection,
      }
    );
    const canonicalDom = normalizedDom(semanticHtml);
    const accessibility = await accessibilitySnapshot(
      page,
      collapseDuplicateSourceToasts,
      hideSourceFrameworkTransients
    );
    const outcomeChecks = await Promise.all(
      scenario.outcomes.map(outcome =>
        checkOutcome(page, outcome, finalStatus, side)
      )
    );
    const layoutSelectors = (process.env.E2E_LAYOUT_SELECTORS ?? '')
      .split('|')
      .map(selector => selector.trim())
      .filter(Boolean);
    if (layoutSelectors.length > 0) {
      const layout = await page.evaluate(selectors => {
        return Object.fromEntries(
          selectors.map(selector => {
            const element = selector.startsWith('text=')
              ? Array.from(
                  document.querySelectorAll<HTMLElement>('body *')
                ).find(
                  candidate =>
                    candidate.childElementCount === 0 &&
                    candidate.textContent
                      .trim()
                      .startsWith(selector.slice('text='.length))
                )
              : document.querySelector(selector);
            if (!(element instanceof HTMLElement)) {
              return [selector, null];
            }
            const rect = element.getBoundingClientRect();
            const contentRange = document.createRange();
            contentRange.selectNodeContents(element);
            const contentRect = contentRange.getBoundingClientRect();
            const style = getComputedStyle(element);
            return [
              selector,
              {
                rect: {
                  x: rect.x,
                  y: rect.y,
                  width: rect.width,
                  height: rect.height,
                },
                contentRect: {
                  x: contentRect.x,
                  y: contentRect.y,
                  width: contentRect.width,
                  height: contentRect.height,
                },
                fontFamily: style.fontFamily,
                fontSize: style.fontSize,
                fontWeight: style.fontWeight,
                lineHeight: style.lineHeight,
                letterSpacing: style.letterSpacing,
                margin: style.margin,
                padding: style.padding,
                gap: style.gap,
                color: style.color,
                background: style.background,
                ancestors: Array.from(
                  (function* ancestors(): Generator<HTMLElement> {
                    let parent = element.parentElement;
                    let depth = 0;
                    while (parent !== null && depth < 6) {
                      yield parent;
                      parent = parent.parentElement;
                      depth += 1;
                    }
                  })()
                ).map(parent => {
                  const parentRect = parent.getBoundingClientRect();
                  const parentStyle = getComputedStyle(parent);
                  return {
                    tag: parent.tagName.toLowerCase(),
                    className: parent.className,
                    rect: {
                      x: parentRect.x,
                      y: parentRect.y,
                      width: parentRect.width,
                      height: parentRect.height,
                    },
                    margin: parentStyle.margin,
                    padding: parentStyle.padding,
                    gap: parentStyle.gap,
                  };
                }),
              },
            ];
          })
        );
      }, layoutSelectors);
      await writeJson(
        resolve(artifactDirectory, `${side}.layout.json`),
        layout
      );
    }
    await page.screenshot({
      path: screenshotPath,
      fullPage: false,
      animations: 'disabled',
    });
    const capturedConsoleEntries = [...consoleEntries];
    const capturedPageErrors = [...pageErrors];
    const capturedNetworkEntries = [...networkEntries];
    const capturedRedirects = [...redirects];
    await writeFile(domPath, html, 'utf8');
    await writeFile(normalizedDomPath, `${canonicalDom}\n`, 'utf8');
    await writeFile(accessibilityPath, `${accessibility}\n`, 'utf8');
    await writeJson(networkPath, capturedNetworkEntries);
    await writeJson(browserLogPath, {
      console: capturedConsoleEntries,
      pageErrors: capturedPageErrors,
    });
    await writeJson(redirectsPath, capturedRedirects);
    await clearBrowserStorage(context, page, browserResetPath);
    await context.tracing.stop({ path: tracePath });
    const pendingVideo = page.video();
    await context.close();
    if (pendingVideo) {
      const generatedPath = await pendingVideo.path();
      videoPath = resolve(artifactDirectory, `${side}.webm`);
      await rename(generatedPath, videoPath);
    }

    const capturedConsoleErrors = capturedConsoleEntries.filter(
      ({ type }) => type === 'error' || type === 'assert'
    );
    const explainedConsoleErrors = capturedConsoleErrors.filter(entry =>
      expectedDocumentConsoleError({
        entry,
        finalUrl,
        finalStatus,
        scenario,
        side,
        networkEntries: capturedNetworkEntries,
      })
    );
    const consoleErrors = capturedConsoleErrors.filter(
      entry => !explainedConsoleErrors.includes(entry)
    );
    const failedRequests = blockingFailedRequests(capturedNetworkEntries);
    const result: CaptureResult = {
      side,
      scenarioId: scenario.id,
      matrixId,
      repeat,
      requestedUrl,
      finalUrl,
      status: finalStatus,
      title,
      bodyTextLength,
      consoleErrors,
      explainedConsoleErrors,
      pageErrors: capturedPageErrors,
      failedRequests,
      artifactDirectory,
      screenshotPath,
      domPath,
      normalizedDomPath,
      accessibilityPath,
      networkPath,
      browserLogPath,
      redirectsPath,
      tracePath,
      videoPath,
      harPath,
      browserResetPath,
      screenshotSha256: await sha256File(screenshotPath),
      domSha256: sha256(canonicalDom),
      accessibilitySha256: sha256(accessibility),
      outcomeChecks,
    };
    await writeJson(resolve(artifactDirectory, `${side}.capture.json`), result);
    return result;
  } catch (error) {
    await context.tracing.stop({ path: tracePath }).catch(() => undefined);
    await context.close().catch(() => undefined);
    throw new Error(
      `${side} capture failed for ${scenario.id}/${matrixId}/repeat-${repeat}: ${
        error instanceof Error ? error.message : String(error)
      } (artifacts: ${basename(artifactDirectory)})`,
      { cause: error }
    );
  }
}
