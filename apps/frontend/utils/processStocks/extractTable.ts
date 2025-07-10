import { chromium } from 'playwright-core';
import * as chromiumBinary from '@sparticuz/chromium';

/**
 * Creates a new browser instance for each data extraction.
 * @returns {Promise<any>} A new browser instance.
 */
async function createBrowser(): Promise<any> {
  const options: any = { headless: true };
  if (process.env.CHROME_PATH) {
    options.executablePath = process.env.CHROME_PATH;
  } else if (process.platform === 'linux') {
    options.executablePath = await chromiumBinary.default.executablePath();
    options.args = chromiumBinary.default.args;
  }
  return await chromium.launch(options);
}

/**
 * Extracts table data from TradingView for a given stock symbol.
 * @param {string} stockSymbol - The stock symbol to fetch data for (default: 'NASDAQ-NVDA').
 * @returns {Promise<any>} The extracted table data.
 */
export async function extractTableData(
  stockSymbol: string = 'NASDAQ-NVDA',
  browserArg?: any,
  contextArg?: any,
): Promise<any> {
  const url = `https://www.tradingview.com/symbols/${stockSymbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`;
  let browser = browserArg;
  let context = contextArg;
  let createdBrowser = false;
  let createdContext = false;
  try {
    if (!browser) {
      browser = await createBrowser();
      createdBrowser = true;
    }
    if (!context) {
      context = await browser.newContext({
        userAgent:
          'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)',
      });
      createdContext = true;
    }
    const page = await context.newPage();
    await page.goto(url, { waitUntil: 'networkidle', timeout: 60000 });

    const data = await page.evaluate(() => {
      const quarters = Array.from(
        document.querySelectorAll(
          '.container-OWKkVLyj > .values-OWKkVLyj > .container-OxVAcLqi > .wrap-OxVAcLqi > .value-OxVAcLqi',
        ),
      ).map((el) => el.textContent?.trim() || '');
      const reported = Array.from(
        document.querySelectorAll(
          'div:nth-child(7) > div.wrapper-Tv7LSjUz > .shadowWrap-vKM0WfUu > .table-GQWAi9kx > [data-name="Reported"] > .values-C9MdAMrq > .container-OxVAcLqi > .wrap-OxVAcLqi > .value-OxVAcLqi',
        ),
      ).map((el) => el.textContent?.trim() || '');
      return reported
        .slice()
        .reverse()
        .map((value, i) => ({
          quarter: quarters[quarters.length - 1 - i] || 'N/A',
          value: value,
        }))
        .reverse();
    });

    const thisYear = new Date().getFullYear().toString().slice(-2);
    const thisQuarter = Math.ceil((new Date().getMonth() + 1) / 3);

    const filteredData = data.filter(
      (item: { quarter: string; value: string }) => {
        const [itemQuarter, itemYear] = item.quarter.split(" '");
        if (!itemQuarter || !itemYear) return false;
        const qNum = parseInt(itemQuarter.replace('Q', ''), 10);
        const yNum = parseInt(itemYear, 10);
        const currentYearNum = parseInt(thisYear, 10);
        return (
          yNum < currentYearNum ||
          (yNum === currentYearNum && qNum <= thisQuarter)
        );
      },
    );

    return filteredData;
  } catch (error: unknown) {
    console.error(
      `Failed to extract data for ${stockSymbol}:`,
      error instanceof Error ? error.message : String(error),
    );
    return [];
  } finally {
    if (createdContext && context) {
      await context.close().catch((err: unknown) => {
        console.error(
          `Error closing context for ${stockSymbol}:`,
          err instanceof Error ? err.message : String(err),
        );
      });
    }
    if (createdBrowser && browser) {
      await browser.close().catch((err: unknown) => {
        console.error(
          `Error closing browser for ${stockSymbol}:`,
          err instanceof Error ? err.message : String(err),
        );
      });
    }
  }
}

extractTableData();
