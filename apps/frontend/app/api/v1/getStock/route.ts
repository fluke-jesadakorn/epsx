import { NextRequest, NextResponse } from 'next/server';
import * as cheerio from 'cheerio';

const TARGET_URL =
  'https://www.tradingview.com/symbols/NASDAQ-NVDA/financials-earnings/?earnings-period=FQ&revenues-period=FQ';
const SELECTOR =
  '#js-category-content > div.js-financials-block-init-ssr > div > div > div:nth-child(11) > div.wrapper-Tv7LSjUz > div > div.container-vKM0WfUu.table-GQWAi9kx.legacy-mode-GQWAi9kx';

export async function GET(req: NextRequest) {
  try {
    const response = await fetch(TARGET_URL, {
      headers: {
        'User-Agent':
          'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)',
      },
    });
    const html = await response.text();
    const $ = cheerio.load(html);
    const element = $(SELECTOR);

    return NextResponse.json({
      html: element.html(),
      text: element.text(),
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to fetch or parse data.' },
      { status: 500 },
    );
  }
}
