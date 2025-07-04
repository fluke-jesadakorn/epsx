import { NextResponse } from 'next/server';
import puppeteer from 'puppeteer';

const TARGET_URL =
  'https://www.tradingview.com/symbols/NASDAQ-NVDA/financials-earnings/?earnings-period=FQ&revenues-period=FQ';
const SELECTOR =
  '#js-category-content > div.js-financials-block-init-ssr > div > div > div:nth-child(7) > div.wrapper-Tv7LSjUz > div > div.container-vKM0WfUu.table-GQWAi9kx.legacy-mode-GQWAi9kx > div.container-C9MdAMrq.selected-C9MdAMrq.beforeSelected-C9MdAMrq.lastRowBorder-C9MdAMrq.legacy-mode-C9MdAMrq > div.values-C9MdAMrq.values-AtxjAQkN > div:nth-child(19) > div > div';

export async function GET() {
  let browser;
  try {
    browser = await puppeteer.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox'],
    });
    const page = await browser.newPage();
    await page.setUserAgent(
      'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)',
    );
    await page.goto(TARGET_URL, { waitUntil: 'networkidle2' });

    const result = await page.$eval(SELECTOR, (el) => ({
      html: el.innerHTML,
      text: el.textContent,
    }));

    return NextResponse.json(result);
  } catch (_error) {
    return NextResponse.json(
      { error: 'Failed to fetch or parse data.' },
      { status: 500 },
    );
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}
