import { expect, test, type Page } from '@playwright/test';

const accessToken = process.env.A7_ABOUT_ACCESS_TOKEN;
const viewports = [
  { name: 'mobile', width: 390, height: 844 },
  { name: 'desktop', width: 1440, height: 900 },
] as const;

const sourceCopy = [
  'Empowering businesses with advanced data analytics and comprehensive platform solutions',
  'A DataTech Platform is a comprehensive technology ecosystem designed to handle your complete data journey.',
  'From initial collection and storage to advanced analysis and visualization, these platforms integrate cutting-edge tools to maximize data value.',
  'Complete data lifecycle management',
  'Integrated tools & technologies',
  'Business decision support',
  'Multi-sector applications',
  'Extract data from sensors, websites, IoT devices, applications, and databases',
  'This initial data gathering is crucial as the raw data will be used for in-depth analysis later.',
  'Secure and scalable storage using Cloud Storage and Big Data Repositories',
  'Handle large volumes of data that can be quickly accessed when needed.',
  'Organize, verify, and maintain data consistency',
  'Including data quality management, data cleansing, and integration of data from multiple sources.',
  'Advanced processing with ML and AI for predictive analysis',
  'Analyze and understand data, predict behaviors or trends from historical data.',
  'In-depth analysis using Predictive, Descriptive, and Prescriptive techniques',
  'Provide insights valuable for business decisions through various analytical methods.',
  'Create interactive dashboards and visual representations',
  'Help users better understand data insights through visual representations.',
  'Enable accurate and efficient data-driven decisions',
  'Increase speed in accessing and processing big data',
  'Improve data management organization and security',
  'Reduce costs through cloud systems and scalable storage',
  'Support efficient team collaboration in data analysis',
  "At EPSX, we're dedicated to transforming how businesses interact with their data. Our mission is to democratize advanced analytics and make powerful data insights accessible to organizations of all sizes, enabling smarter decisions and driving sustainable growth through innovative technology solutions.",
  'We envision a future where every business decision is powered by intelligent, real-time data insights. By building cutting-edge analytics platforms and fostering a data-driven culture, we aim to be the catalyst that helps organizations unlock their full potential and achieve extraordinary outcomes.',
] as const;

async function expectResponsiveDocument(page: Page) {
  const overflow = await page.evaluate(
    () =>
      document.documentElement.scrollWidth -
      document.documentElement.clientWidth
  );
  expect(overflow).toBeLessThanOrEqual(1);
  await expect(page.locator('main')).toBeVisible();
}

test.describe('A7 /about pinned-source runtime proof', () => {
  test.skip(
    !accessToken,
    'Run through scripts/migration/run-about-runtime-proof.sh'
  );

  test.beforeEach(async ({ context }) => {
    await context.addCookies([
      {
        name: 'epsx.frontend.access_token',
        value: accessToken!,
        url: 'http://localhost:3000',
        httpOnly: true,
        sameSite: 'Lax',
      },
    ]);
  });

  test('matches accepted source content, metadata, semantics, and keyboard chrome', async ({
    page,
  }) => {
    const pageErrors: string[] = [];
    const consoleErrors: string[] = [];
    page.on('pageerror', error => pageErrors.push(error.message));
    page.on('console', message => {
      if (message.type() === 'error') consoleErrors.push(message.text());
    });

    for (const viewport of viewports) {
      await page.setViewportSize(viewport);
      const response = await page.goto('/about', {
        waitUntil: 'domcontentloaded',
      });

      expect(response?.status(), `${viewport.name} /about status`).toBe(200);
      await expect(page).toHaveTitle('About Us - EPSX Analytics Platform');
      await expect(page.locator('meta[name="description"]')).toHaveAttribute(
        'content',
        'Learn about EPSX DataTech Platform - comprehensive technology platform designed to manage the complete data lifecycle, from collection and storage to analysis and visualization.'
      );
      await expect(page.locator('meta[name="keywords"]')).toHaveAttribute(
        'content',
        'EPSX, DataTech Platform, data analytics, business intelligence, data management'
      );

      await expect(
        page.getByRole('heading', { level: 1, name: 'About EPSX' })
      ).toBeVisible();
      await expect(page.getByRole('heading', { level: 1 })).toHaveCount(1);
      await expect(
        page.locator('main').getByRole('heading', { level: 2 })
      ).toHaveCount(11);
      await expect(
        page.getByRole('region', { name: 'DataTech Platform' })
      ).toBeVisible();
      await expect(
        page.getByRole('region', { name: 'EPSX mission and vision' })
      ).toBeVisible();
      await expect(page.locator('article')).toHaveCount(11);

      for (const copy of sourceCopy) {
        await expect(page.locator('main')).toContainText(copy);
      }
      for (const invented of [
        'Meet the team',
        'Alex Tan',
        'EPSX by the numbers',
        '12K+',
        'Our journey',
        'The founding',
        'Our Values',
        'Join us',
        "We're hiring",
      ]) {
        await expect(page.locator('main')).not.toContainText(invented);
      }

      const sectionOrder = await page
        .locator('.about-hero-section, .datatech-section, .mission-section')
        .evaluateAll(nodes => nodes.map(node => node.className));
      expect(sectionOrder).toEqual([
        'about-hero-section',
        'datatech-section',
        'mission-section',
      ]);
      await expectResponsiveDocument(page);

      const theme = page.locator('button[data-epsx-theme-toggle]');
      await theme.focus();
      await expect(theme).toBeFocused();
      const wasDark = await page
        .locator('html')
        .evaluate(html => html.classList.contains('dark'));
      await page.keyboard.press('Enter');
      await expect
        .poll(() =>
          page.locator('html').evaluate(html => html.classList.contains('dark'))
        )
        .toBe(!wasDark);

      if (viewport.name === 'mobile') {
        const menu = page.getByRole('button', { name: 'Open menu' });
        await menu.focus();
        await expect(menu).toBeFocused();
        await page.keyboard.press('Enter');
        await expect(page.locator('#epsx-mobile-sheet')).toBeVisible();
        const aboutLink = page.locator('#epsx-mobile-sheet a[href="/about"]');
        await aboutLink.focus();
        await expect(aboutLink).toBeFocused();
        await page
          .getByRole('dialog', { name: 'Menu' })
          .getByRole('button', { name: 'Close menu' })
          .click();
      } else {
        const company = page.getByRole('button', { name: 'Company' });
        await company.focus();
        await expect(company).toBeFocused();
        await page.keyboard.press('Enter');
        await expect(company.locator('xpath=..')).toHaveClass(/open/);
        const aboutLink = company
          .locator('xpath=..')
          .locator('a[href="/about"]');
        await aboutLink.focus();
        await expect(aboutLink).toBeFocused();
        await page.keyboard.press('Escape');
        await expect(company.locator('xpath=..')).not.toHaveClass(/open/);
      }
    }

    expect(pageErrors).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });
});
