import { expect, test, type BrowserContext, type Page } from '@playwright/test';

const viewports = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1440, height: 900 },
] as const;

const sensitiveProbes = [
  '/api/health?user=cache-probe',
  '/auth?return_url=/account&user=cache-probe',
  '/account?user=cache-probe',
  '/profile?user=cache-probe',
  '/notifications?wallet=0xcache-probe',
  '/analytics?user=cache-probe',
  '/admin?user=cache-probe',
  '/payment?user=cache-probe',
  '/offline?user=cache-probe',
] as const;

async function waitForController(page: Page) {
  return page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    if (!navigator.serviceWorker.controller) {
      await new Promise<void>((resolve, reject) => {
        const timeout = window.setTimeout(
          () =>
            reject(
              new Error('service worker did not claim the bootstrap page')
            ),
          10_000
        );
        navigator.serviceWorker.addEventListener(
          'controllerchange',
          () => {
            window.clearTimeout(timeout);
            resolve();
          },
          { once: true }
        );
      });
    }
    return {
      controlled: navigator.serviceWorker.controller !== null,
      scope: registration.scope,
      scriptURL: registration.active?.scriptURL ?? '',
    };
  });
}

async function cacheSnapshot(page: Page) {
  return page.evaluate(async () => {
    const names = await caches.keys();
    const entries = [] as Array<{
      cache: string;
      url: string;
      method: string;
      status: number;
      marker: string | null;
    }>;
    for (const name of names) {
      const cache = await caches.open(name);
      for (const request of await cache.keys()) {
        const response = await cache.match(request);
        entries.push({
          cache: name,
          url: request.url,
          method: request.method,
          status: response?.status ?? 0,
          marker: response?.headers.get('x-epsx-public-cache') ?? null,
        });
      }
    }
    const registrations = await navigator.serviceWorker.getRegistrations();
    return {
      names: names.sort(),
      entries: entries.sort((left, right) => left.url.localeCompare(right.url)),
      registrationScopes: registrations.map(item => item.scope).sort(),
    };
  });
}

async function removeOfflineState(page: Page) {
  return page.evaluate(async () => {
    const registrations = await navigator.serviceWorker.getRegistrations();
    const unregisterResults = await Promise.all(
      registrations.map(item => item.unregister())
    );
    const names = await caches.keys();
    const deleteResults = await Promise.all(
      names.map(name => caches.delete(name))
    );
    return {
      unregistered: unregisterResults.every(Boolean),
      deleted: deleteResults.every(Boolean),
      registrations: (await navigator.serviceWorker.getRegistrations()).length,
      caches: (await caches.keys()).length,
    };
  });
}

async function exerciseViewport(
  context: BrowserContext,
  viewport: (typeof viewports)[number]
) {
  expect(
    context.serviceWorkers(),
    `${viewport.name} starts isolated`
  ).toHaveLength(0);
  const pageErrors: string[] = [];
  const bootstrap = await context.newPage();
  bootstrap.on('pageerror', error => pageErrors.push(error.message));

  const bootstrapResponse = await bootstrap.goto('/', {
    waitUntil: 'domcontentloaded',
  });
  expect(bootstrapResponse?.status(), `${viewport.name} bootstrap status`).toBe(
    200
  );
  await expect(
    bootstrap.locator('script[data-epsx-offline-worker-registration]')
  ).toHaveCount(1);

  const worker = await waitForController(bootstrap);
  expect(worker.controlled, `${viewport.name} controlled bootstrap`).toBe(true);
  expect(new URL(worker.scope).pathname).toBe('/');
  expect(new URL(worker.scriptURL).pathname).toBe('/service-worker.js');

  const installed = await cacheSnapshot(bootstrap);
  expect(installed.names).toEqual(['epsx-public-offline-v1']);
  expect(
    installed.registrationScopes.map(scope => new URL(scope).pathname)
  ).toEqual(['/']);
  expect(installed.entries).toEqual([
    {
      cache: 'epsx-public-offline-v1',
      url: new URL('/offline', bootstrap.url()).href,
      method: 'GET',
      status: 200,
      marker: 'offline-shell-v1',
    },
  ]);

  // Exercise sensitive and query-bearing local routes through a controlled
  // client. None may add a CacheStorage request or response.
  await bootstrap.evaluate(
    async paths => {
      await Promise.all(
        paths.map(path =>
          fetch(path, { credentials: 'include', redirect: 'manual' }).catch(
            () => undefined
          )
        )
      );
    },
    [...sensitiveProbes]
  );
  expect(await cacheSnapshot(bootstrap)).toEqual(installed);

  for (const entry of installed.entries) {
    const url = new URL(entry.url);
    expect(url.pathname).toBe('/offline');
    expect(url.search).toBe('');
    expect(url.hash).toBe('');
    expect(url.pathname).not.toMatch(
      /^\/(?:api|auth|account|profile|notifications|analytics|admin|pay|payment|user)(?:\/|$)/
    );
  }

  // The browser has never navigated to `/offline`: only the worker's
  // credential-free install fetch populated it. A new page must now complete
  // its first navigation while the browser is disconnected.
  await context.setOffline(true);
  const offlinePage = await context.newPage();
  offlinePage.on('pageerror', error => pageErrors.push(error.message));
  const offlineResponse = await offlinePage.goto('/offline', {
    waitUntil: 'domcontentloaded',
  });
  expect(offlineResponse?.status(), `${viewport.name} cached status`).toBe(200);
  expect(
    offlineResponse?.fromServiceWorker(),
    `${viewport.name} cache delivery`
  ).toBe(true);
  await expect(
    offlinePage.getByRole('heading', { level: 1, name: "You're offline" })
  ).toBeVisible();
  await expect(
    offlinePage.getByText('Open this offline help page', { exact: true })
  ).toBeVisible();
  await expect(
    offlinePage.getByText('Connection required: account and live features', {
      exact: true,
    })
  ).toBeVisible();
  await expect(offlinePage.getByText('View cached notifications')).toHaveCount(
    0
  );
  await expect(
    offlinePage.getByText('Browse previously loaded analytics')
  ).toHaveCount(0);
  await expect(offlinePage.getByText('Access user settings')).toHaveCount(0);
  await expect(offlinePage.getByText('Your data will sync')).toHaveCount(0);
  expect(
    await offlinePage.evaluate(
      () =>
        document.documentElement.scrollWidth -
        document.documentElement.clientWidth
    )
  ).toBeLessThanOrEqual(1);

  const retry = offlinePage.getByRole('button', { name: 'Try again' });
  await retry.focus();
  await expect(retry).toBeFocused();
  await context.setOffline(false);
  const recovered = await Promise.all([
    offlinePage.waitForNavigation({ waitUntil: 'domcontentloaded' }),
    offlinePage.keyboard.press('Enter'),
  ]);
  expect(recovered[0]?.status(), `${viewport.name} retry status`).toBe(200);
  await expect(offlinePage).toHaveURL(/\/offline$/);
  await expect(
    offlinePage.getByRole('button', { name: 'Try again' })
  ).toBeEnabled();

  // Query-bearing `/offline` is deliberately a network-only navigation. It
  // must not alias the exact cached key or add a second CacheStorage entry.
  const queryPage = await context.newPage();
  const queryResponse = await queryPage.goto('/offline?user=cache-probe', {
    waitUntil: 'domcontentloaded',
  });
  expect(
    queryResponse?.status(),
    `${viewport.name} query navigation status`
  ).toBe(200);
  expect(
    queryResponse?.fromServiceWorker(),
    `${viewport.name} query bypass`
  ).toBe(false);
  await queryPage.close();
  expect(await cacheSnapshot(offlinePage)).toEqual(installed);
  expect(pageErrors).toEqual([]);

  const cleanup = await removeOfflineState(offlinePage);
  expect(cleanup).toEqual({
    unregistered: true,
    deleted: true,
    registrations: 0,
    caches: 0,
  });
}

test.describe('B7 fresh offline recovery', () => {
  test('serves only the public offline shell on fresh controlled mobile and desktop navigation', async ({
    browser,
  }) => {
    for (const viewport of viewports) {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        serviceWorkers: 'allow',
      });
      try {
        await exerciseViewport(context, viewport);
      } finally {
        await context.setOffline(false).catch(() => undefined);
        const page = context.pages().find(candidate => !candidate.isClosed());
        if (page) await removeOfflineState(page).catch(() => undefined);
        await context.close();
      }
    }
  });
});
