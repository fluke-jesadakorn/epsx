import { rename, writeFile } from 'node:fs/promises';
import { basename, resolve } from 'node:path';

import type { Browser, BrowserContext, Page } from '@playwright/test';

import { ensureDirectory, sha256, sha256File, writeJson } from './files';
import type {
  BrowserLogEntry,
  CaptureResult,
  ColorScheme,
  NetworkEntry,
  Scenario,
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
}

interface BrowserStorageState {
  cookies: number;
  localStorage: number;
  sessionStorage: number;
  indexedDatabases: number;
  cacheStorage: number;
  serviceWorkers: number;
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
  return semanticHtml
    .replaceAll(/\bnonce="[^"]*"/g, 'nonce="<normalized>"')
    .replaceAll(/\sdata-nextjs-router-state-tree="[^"]*"/g, '')
    .replaceAll(/\s+/g, ' ')
    .trim();
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

function blockingFailedRequests(entries: NetworkEntry[]): NetworkEntry[] {
  return entries
    .filter(entry => entry.kind === 'failed')
    .filter(failure => {
      const successfulHeadResponse =
        failure.method === 'HEAD' &&
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
      // Chromium can report a body abort after a successful HEAD response.
      // HEAD has no response body, so the completed 2xx/3xx headers are the
      // authoritative outcome; the raw response and abort remain in network.json.
      return !successfulHeadResponse;
    });
}

async function waitForStableMeaningfulBody(page: Page): Promise<void> {
  await page
    .waitForFunction(
      ({ minimumLength, stableForMs }) => {
        const bodyTextLength = document.body.innerText.trim().length;
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
            observedAt: Date.now(),
          };
          return false;
        }
        return (
          bodyTextLength > minimumLength &&
          Date.now() - previous.observedAt >= stableForMs
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
  await context.route(
    'https://api.web3modal.org/appkit/v1/config**',
    route =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ features: [] }),
      })
  );
  await context.route('https://pulse.walletconnect.org/**', route =>
    route.fulfill({ status: 204, body: '' })
  );
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

    const finalUrl = page.url();
    const title = await page.title();
    const bodyTextLength = (await page.locator('body').innerText()).trim()
      .length;
    const html = await page.content();
    const semanticHtml = await page.evaluate(() => {
      const clone = document.body.cloneNode(true) as HTMLElement;
      for (const node of clone.querySelectorAll('script, style')) {
        node.remove();
      }
      for (const element of [clone, ...clone.querySelectorAll('*')]) {
        const runtimeAttributes = Array.from(element.attributes)
          .map(attribute => attribute.name)
          .filter(
            name => name.startsWith('data-nextjs') || name === 'nonce'
          );
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
    });
    const canonicalDom = normalizedDom(semanticHtml);
    const accessibility = await page.locator('body').ariaSnapshot();
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

    const consoleErrors = capturedConsoleEntries.filter(
      ({ type }) => type === 'error' || type === 'assert'
    );
    const failedRequests = blockingFailedRequests(capturedNetworkEntries);
    const result: CaptureResult = {
      side,
      scenarioId: scenario.id,
      matrixId,
      repeat,
      requestedUrl,
      finalUrl,
      status: response?.status() ?? null,
      title,
      bodyTextLength,
      consoleErrors,
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
