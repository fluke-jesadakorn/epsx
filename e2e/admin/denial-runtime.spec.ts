import { expect, test, type Page } from '@playwright/test';

const accessToken = process.env.A8_ADMIN_DENIAL_ACCESS_TOKEN;
const viewports = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1440, height: 900 },
] as const;

const metadata = {
  title: 'EPSX Admin',
  description:
    'Administrative interface for EPSX data analytics platform - User management and system monitoring',
  keywords: 'EPSX, admin, analytics, user management, dashboard',
} as const;

async function expectResponsiveDenial(page: Page) {
  const horizontalOverflow = await page.evaluate(
    () =>
      document.documentElement.scrollWidth -
      document.documentElement.clientWidth
  );
  expect(horizontalOverflow).toBeLessThanOrEqual(1);
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('[data-admin-denial-runtime="true"]')).toHaveCSS(
    'overflow-x',
    'hidden'
  );
}

async function expectMetadata(page: Page) {
  await expect(page).toHaveTitle(metadata.title);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    'content',
    metadata.description
  );
  await expect(page.locator('meta[name="keywords"]')).toHaveAttribute(
    'content',
    metadata.keywords
  );
}

test.describe('A8 admin denial runtime proof', () => {
  test.skip(
    !accessToken,
    'Run through scripts/migration/run-admin-denial-runtime-proof.sh'
  );

  test.beforeEach(async ({ context }) => {
    await context.addCookies([
      {
        name: 'epsx.admin.access_token',
        value: accessToken!,
        url: 'http://localhost:3001',
        httpOnly: true,
        sameSite: 'Lax',
      },
    ]);
  });

  test('access-denied preserves source fields with bounded escaped output and safe links', async ({
    page,
  }) => {
    const pageErrors: string[] = [];
    const consoleErrors: string[] = [];
    page.on('pageerror', error => pageErrors.push(error.message));
    page.on('console', message => {
      if (message.type() === 'error') consoleErrors.push(message.text());
    });

    const reason =
      'Denied <script data-a8-probe>window.__a8Injected = true</script>';
    const route = '/payments?tab=history';
    const context = 'admin';
    const permission = 'admin:payments:read<img data-a8-probe>';
    const detail =
      'Backend rejected <svg data-a8-probe onload="window.__a8Injected = true">';
    const query = new URLSearchParams({
      reason,
      route,
      context,
      permission,
      detail,
    });

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      let response = await page.goto(`/access-denied?${query.toString()}`, {
        waitUntil: 'domcontentloaded',
      });
      await page.evaluate(() => localStorage.setItem('epsx-theme', 'light'));
      response = await page.reload({ waitUntil: 'domcontentloaded' });

      expect(response?.status(), `${viewport.name} denial status`).toBe(200);
      await expectMetadata(page);
      await expect(
        page.getByRole('heading', { level: 1, name: 'Access Denied' })
      ).toBeVisible();
      await expect(page.getByRole('heading', { level: 1 })).toHaveCount(1);
      await expect(page.getByRole('alert')).toBeVisible();
      await expect(
        page.getByRole('navigation', { name: 'Access denied actions' })
      ).toBeVisible();
      await expect(page.getByRole('alert')).toContainText(reason);
      await expect(page.getByText(route, { exact: true })).toBeVisible();
      await expect(page.getByText(context, { exact: true })).toBeVisible();
      await expect(page.getByText(permission, { exact: true })).toBeVisible();
      await expect(page.getByText(detail, { exact: true })).toBeVisible();
      await expect(
        page.getByText('Admin Access Required:', { exact: true })
      ).toBeVisible();
      await expect(
        page.locator(
          'script[data-a8-probe], img[data-a8-probe], svg[data-a8-probe]'
        )
      ).toHaveCount(0);
      expect(
        await page.evaluate(
          () =>
            (window as typeof window & { __a8Injected?: boolean }).__a8Injected
        )
      ).toBeUndefined();

      const auth = page.getByRole('link', { name: 'Go to Auth' });
      const back = page.getByRole('link', { name: 'Go Back' });
      await expect(auth).toHaveAttribute(
        'href',
        '/auth?return_url=%2Fpayments%3Ftab%3Dhistory'
      );
      await expect(back).toHaveAttribute('href', '/');
      for (const link of [auth, back]) {
        expect(
          await link.evaluate(element => {
            const target = new URL((element as HTMLAnchorElement).href);
            return target.origin === window.location.origin;
          })
        ).toBe(true);
      }
      await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);
      await expect(
        page.locator('.admin-header, .admin-sidebar, footer')
      ).toHaveCount(0);
      await expectResponsiveDenial(page);

      await auth.focus();
      await expect(auth).toBeFocused();
      await page.keyboard.press('Tab');
      await expect(back).toBeFocused();
      await expect(page.locator('html')).not.toHaveClass(/dark/);

      await page.evaluate(() => localStorage.setItem('epsx-theme', 'dark'));
      await page.reload({ waitUntil: 'domcontentloaded' });
      await expect(page.locator('html')).toHaveClass(/dark/);
      await expectResponsiveDenial(page);
      const orbBackground = await page
        .locator('[data-admin-denial-orb="primary"]')
        .evaluate(element => getComputedStyle(element).backgroundColor);
      expect(orbBackground).not.toBe('rgba(0, 0, 0, 0)');
      expect(orbBackground).not.toBe('transparent');
    }
    expect(pageErrors).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });

  test('unsafe and reserved return targets fail closed', async ({ page }) => {
    for (const route of [
      'https://evil.example/steal',
      '//evil.example/steal',
      '/auth?return_url=/payments',
      '/section/../auth',
      '/api/v1/auth/logout',
    ]) {
      const response = await page.goto(
        `/access-denied?${new URLSearchParams({ route })}`,
        { waitUntil: 'domcontentloaded' }
      );
      expect(response?.status()).toBe(200);
      await expect(
        page.getByRole('link', { name: 'Go to Auth' })
      ).toHaveAttribute('href', '/auth?return_url=%2F');
      await expect(page.getByRole('link', { name: 'Go Back' })).toHaveAttribute(
        'href',
        '/'
      );
      await expectResponsiveDenial(page);
    }
  });

  test('unauthorized uses exact static source copy at both accepted viewports', async ({
    page,
  }) => {
    const pageErrors: string[] = [];
    const consoleErrors: string[] = [];
    page.on('pageerror', error => pageErrors.push(error.message));
    page.on('console', message => {
      if (message.type() === 'error') consoleErrors.push(message.text());
    });
    const sourceReason =
      "You don't have permission to access the admin panel. Please contact your administrator if you believe this is an error.";

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto(
        '/unauthorized?reason=ignored&route=https%3A%2F%2Fevil.example',
        { waitUntil: 'domcontentloaded' }
      );
      expect(response?.status(), `${viewport.name} unauthorized status`).toBe(
        200
      );
      await expectMetadata(page);
      await expect(page.getByRole('heading', { level: 1 })).toHaveCount(1);
      await expect(page.getByRole('alert')).toContainText(sourceReason);
      await expect(page.getByRole('alert')).not.toContainText('ignored');
      await expect(page.getByText('Requested Route:')).toHaveCount(0);
      await expect(
        page.getByRole('link', { name: 'Go to Auth' })
      ).toHaveAttribute('href', '/auth?return_url=%2F');
      await expect(page.getByRole('link', { name: 'Go Back' })).toHaveAttribute(
        'href',
        '/'
      );
      await expectResponsiveDenial(page);
    }
    expect(pageErrors).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });

  test('keyboard reauthentication uses canonical logout and a sanitized target', async ({
    context,
    page,
  }) => {
    const logoutRequests: string[] = [];
    page.on('request', request => {
      if (new URL(request.url()).pathname === '/api/v1/auth/logout') {
        logoutRequests.push(request.method());
      }
    });
    await page.goto(
      `/access-denied?${new URLSearchParams({ route: '/payments?tab=history' })}`,
      { waitUntil: 'domcontentloaded' }
    );
    const auth = page.getByRole('link', { name: 'Go to Auth' });
    await auth.focus();
    await expect(auth).toBeFocused();
    const [authRequest] = await Promise.all([
      page.waitForRequest(request => new URL(request.url()).pathname === '/auth'),
      page.waitForURL(url => url.pathname === '/' && url.search === ''),
      page.keyboard.press('Enter'),
    ]);
    const authRequestUrl = new URL(authRequest.url());
    expect(authRequestUrl.searchParams.get('return_url')).toBe(
      '/payments?tab=history'
    );
    await expect(page).toHaveURL('http://localhost:3001/');
    expect(logoutRequests).toEqual(['POST']);
    expect(
      (await context.cookies()).filter(
        cookie => cookie.name === 'epsx.admin.access_token'
      )
    ).toEqual([]);
  });

  test('Go Back uses same-origin browser history and ignores the route display value', async ({
    page,
  }) => {
    await page.goto('/unauthorized', { waitUntil: 'domcontentloaded' });
    const denialUrl = `/access-denied?${new URLSearchParams({ route: '/payments?tab=history' })}`;
    await Promise.all([
      page.waitForURL(url => url.pathname === '/access-denied'),
      page.evaluate(url => window.location.assign(url), denialUrl),
    ]);
    const back = page.getByRole('link', { name: 'Go Back' });
    await expect(back).toHaveAttribute('href', '/');
    await back.focus();
    await expect(back).toBeFocused();
    await Promise.all([
      page.waitForURL(url => url.pathname === '/unauthorized'),
      page.keyboard.press('Enter'),
    ]);
    await expect(page).toHaveURL('http://localhost:3001/unauthorized');
  });
});
