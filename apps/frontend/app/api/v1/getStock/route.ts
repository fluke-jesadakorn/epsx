import { NextResponse } from 'next/server';
import { chromium } from 'playwright-core';
import chromiumBinary from '@sparticuz/chromium';

export const dynamic = 'force-dynamic';

const TARGET_URL =
  'https://www.tradingview.com/symbols/NASDAQ-NVDA/financials-earnings/?earnings-period=FQ&revenues-period=FQ';
const SELECTOR =
  '#js-category-content > div.js-financials-block-init-ssr > div > div > div:nth-child(7) > div.wrapper-Tv7LSjUz > div > div.container-vKM0WfUu.table-GQWAi9kx.legacy-mode-GQWAi9kx > div.container-C9MdAMrq.selected-C9MdAMrq.beforeSelected-C9MdAMrq.lastRowBorder-C9MdAMrq.legacy-mode-C9MdAMrq > div.values-C9MdAMrq.values-AtxjAQkN > div:nth-child(19) > div > div';

let browser: any = null;

async function getBrowser() {
  if (browser) return browser;

  let launchOptions: any = { headless: true };

  if (process.env.CHROME_PATH) {
    launchOptions.executablePath = process.env.CHROME_PATH;
  } else if (process.platform === 'linux') {
    // Use @sparticuz/chromium only on Linux (e.g., AWS Lambda)
    launchOptions.executablePath = await chromiumBinary.executablePath();
    launchOptions.args = chromiumBinary.args;
  }

  browser = await chromium.launch(launchOptions);
  return browser;
}

export async function GET() {
  try {
    const browser = await getBrowser();
    const context = await browser.newContext({
      userAgent:
        'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)',
    });
    const page = await context.newPage();
    await page.goto(TARGET_URL, { waitUntil: 'networkidle' });

    // Wait for the specific element to be visible
    await page.waitForSelector(SELECTOR, {
      timeout: 5000, // Adjust timeout as needed
    });

    const result = await page.evaluate((selector: string) => {
      const el = document.querySelector(selector);
      return el
        ? {
            html: el.innerHTML,
            text: el.textContent,
          }
        : { html: '', text: '' };
    }, SELECTOR);

    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in getStock API:', error);
    return NextResponse.json(
      { error: 'Failed to fetch or parse data.' },
      { status: 500 },
    );
  }
}
