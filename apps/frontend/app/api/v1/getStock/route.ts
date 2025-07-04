import { NextResponse } from 'next/server';
import puppeteer from 'puppeteer-core';
import * as chromium from '@sparticuz/chromium';

const TARGET_URL =
  'https://www.tradingview.com/symbols/NASDAQ-NVDA/financials-earnings/?earnings-period=FQ&revenues-period=FQ';
const SELECTOR =
  '#js-category-content > div.js-financials-block-init-ssr > div > div > div:nth-child(7) > div.wrapper-Tv7LSjUz > div > div.container-vKM0WfUu.table-GQWAi9kx.legacy-mode-GQWAi9kx > div.container-C9MdAMrq.selected-C9MdAMrq.beforeSelected-C9MdAMrq.lastRowBorder-C9MdAMrq.legacy-mode-C9MdAMrq > div.values-C9MdAMrq.values-AtxjAQkN > div:nth-child(19) > div > div';

export async function GET() {
  let browser;
  try {
    // Check if running in production (Vercel or AWS Lambda)
    const isProduction = process.env.VERCEL || process.env.AWS_LAMBDA_FUNCTION_VERSION;

    let launchOptions: any = {
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-web-security'],
      defaultViewport: { width: 1280, height: 720 },
    };

    if (isProduction) {
      // On Vercel or AWS Lambda, use @sparticuz/chromium and puppeteer-core
      // Configure launch options for serverless environment
      launchOptions.args = [
        '--no-sandbox',
        '--disable-setuid-sandbox',
        '--disable-dev-shm-usage',
        '--disable-gpu',
        '--single-process',
        '--no-zygote',
        '--disable-features=VizDisplayCompositor'
      ];
      launchOptions.executablePath = '/tmp/chromium';
      browser = await puppeteer.launch(launchOptions);
    } else {
      // Local development - use Puppeteer's default Chromium
      const puppeteerLocal = require('puppeteer');
      browser = await puppeteerLocal.launch(launchOptions);
    }

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
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}
