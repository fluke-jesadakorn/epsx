import { test, expect } from '@playwright/test';

const THRESHOLDS = {
  lcp: 4500,   // ms — mobile throttled baseline; target <2500 after deploy
  fcp: 2000,   // ms — First Contentful Paint
  cls: 0.1,    // Cumulative Layout Shift
  tbt: 400,    // ms — Total Blocking Time proxy
  jsKb: 1300,  // KB — total JS transfer size on initial load
};

test.describe('Core Web Vitals — epsx.io', () => {
  test('LCP, FCP, CLS within thresholds', async ({ page }) => {
    // Collect perf entries before navigation
    await page.addInitScript(() => {
      (window as any).__perfResults = { lcp: 0, fcp: 0, cls: 0, tbt: 0 };

      // LCP
      new PerformanceObserver((list) => {
        const entries = list.getEntries();
        const last = entries[entries.length - 1];
        if (last) (window as any).__perfResults.lcp = last.startTime;
      }).observe({ type: 'largest-contentful-paint', buffered: true });

      // FCP
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.name === 'first-contentful-paint') {
            (window as any).__perfResults.fcp = entry.startTime;
          }
        }
      }).observe({ type: 'paint', buffered: true });

      // CLS
      let clsValue = 0;
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          const shift = entry as any;
          if (!shift.hadRecentInput) clsValue += shift.value ?? 0;
        }
        (window as any).__perfResults.cls = clsValue;
      }).observe({ type: 'layout-shift', buffered: true });

      // Long tasks → TBT proxy
      let tbt = 0;
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          tbt += Math.max(0, entry.duration - 50);
        }
        (window as any).__perfResults.tbt = tbt;
      }).observe({ type: 'longtask', buffered: true });
    });

    // Track JS bytes transferred
    const jsBytes: number[] = [];
    page.on('response', (res) => {
      const url = res.url();
      const ct = res.headers()['content-type'] ?? '';
      if (url.includes('.js') && ct.includes('javascript')) {
        const cl = res.headers()['content-length'];
        if (cl) jsBytes.push(parseInt(cl, 10));
      }
    });

    await page.goto('/', { waitUntil: 'networkidle' });

    // Give observers time to flush
    await page.waitForTimeout(1500);

    const perf = await page.evaluate(() => (window as any).__perfResults as {
      lcp: number; fcp: number; cls: number; tbt: number;
    });

    const totalJsKb = jsBytes.reduce((a, b) => a + b, 0) / 1024;

    // --- Assertions ---
    expect(perf.lcp, `LCP ${perf.lcp.toFixed(0)}ms exceeds ${THRESHOLDS.lcp}ms`).toBeLessThan(THRESHOLDS.lcp);
    expect(perf.fcp, `FCP ${perf.fcp.toFixed(0)}ms exceeds ${THRESHOLDS.fcp}ms`).toBeLessThan(THRESHOLDS.fcp);
    expect(perf.cls, `CLS ${perf.cls.toFixed(3)} exceeds ${THRESHOLDS.cls}`).toBeLessThan(THRESHOLDS.cls);
    expect(totalJsKb, `JS transfer ${totalJsKb.toFixed(0)}KB exceeds ${THRESHOLDS.jsKb}KB`).toBeLessThan(THRESHOLDS.jsKb);
  });

  test('page loads without JS errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text());
    });
    page.on('pageerror', (err) => errors.push(err.message));

    await page.goto('/', { waitUntil: 'networkidle' });

    // Filter known benign noise: wallet libs, WalletConnect, Cloudflare analytics CSP
    const critical = errors.filter(
      (e) =>
        !e.includes('WalletConnect') &&
        !e.includes('walletconnect') &&
        !e.includes('IndexedDB') &&
        !e.includes('WebSocket') &&
        !e.includes('localStorage') &&
        !e.includes('Cannot mix BigInt') &&
        !e.includes('Failed to fetch') &&
        !e.includes('onclose') &&
        !e.includes('cloudflareinsights') &&
        !e.includes('beacon.min.js'),
    );

    expect(critical, `Critical JS errors: ${critical.join('\n')}`).toHaveLength(0);
  });

  test('critical resources load lazily after interaction', async ({ page }) => {
    const loadedChunks: string[] = [];
    page.on('response', (res) => {
      const url = res.url();
      if (url.includes('_next/static/chunks/') && url.endsWith('.js')) {
        loadedChunks.push(url);
      }
    });

    await page.goto('/', { waitUntil: 'networkidle' });
    const initialChunks = [...loadedChunks];

    // RainbowKit/wallet chunks should NOT be loaded on initial paint
    // (they are deferred until user interaction)
    const rainbowKitOnInit = initialChunks.filter((url) =>
      url.toLowerCase().includes('rainbow'),
    );
    expect(
      rainbowKitOnInit,
      'RainbowKit chunk should be deferred, not loaded on initial paint',
    ).toHaveLength(0);

    // Page should still render main content without wallet chunks
    await expect(page.locator('nav')).toBeVisible();
    await expect(page).toHaveTitle(/EPSX/i);
  });
});
