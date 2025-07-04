import { NextResponse } from 'next/server';
import chromium from '@sparticuz/chromium-min';
import puppeteerCore from 'puppeteer-core';
import puppeteer from 'puppeteer';

export const dynamic = 'force-dynamic';

const TARGET_URL =
  'https://www.tradingview.com/symbols/NASDAQ-NVDA/financials-earnings/?earnings-period=FQ&revenues-period=FQ';
const SELECTOR =
  '#js-category-content > div.js-financials-block-init-ssr > div > div > div:nth-child(7) > div.wrapper-Tv7LSjUz > div > div.container-vKM0WfUu.table-GQWAi9kx.legacy-mode-GQWAi9kx > div.container-C9MdAMrq.selected-C9MdAMrq.beforeSelected-C9MdAMrq.lastRowBorder-C9MdAMrq.legacy-mode-C9MdAMrq > div.values-C9MdAMrq.values-AtxjAQkN > div:nth-child(19) > div > div';

const remoteExecutablePath =
  'https://github.com/Sparticuz/chromium/releases/download/v121.0.0/chromium-v121.0.0-pack.tar';

let browser: any = null;

async function getBrowser() {
  if (browser) return browser;

  if (process.env.NEXT_PUBLIC_VERCEL_ENVIRONMENT === 'production') {
    browser = await puppeteerCore.launch({
      args: chromium.args,
      executablePath: await chromium.executablePath(remoteExecutablePath),
      headless: true,
    });
  } else {
    browser = await puppeteer.launch({
      args: ['--no-sandbox', '--disable-setuid-sandbox'],
      headless: true,
    });
  }
  return browser;
}

export async function GET() {
  try {
    const browser = await getBrowser();
    const page = await browser.newPage();
    await page.setUserAgent(
      'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)',
    );
    await page.goto(TARGET_URL, { waitUntil: 'networkidle2' });

    const result = await page.$eval(SELECTOR, (el: Element) => {
      return {
        html: el.innerHTML,
        text: el.textContent,
      };
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('Error in getStock API:', error);
    return NextResponse.json(
      { error: 'Failed to fetch or parse data.' },
      { status: 500 },
    );
  }
}
