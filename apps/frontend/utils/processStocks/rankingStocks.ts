import fetchScreenerStock from './fetchScreenerStock';
import { MarketCountry } from '../../../../types/marketCountries';
import { extractTableData } from './extractTable';
import { chromium } from 'playwright-core';
import * as chromiumBinary from '@sparticuz/chromium';

/**
 * Fetches stock data and returns EPS comparison data for a list of stocks.
 * @param {number} skip - Number of stocks to skip (default: 0).
 * @param {number} limit - Maximum number of stocks to fetch (default: 20).
 * @param {string|string[]} country - Market country or countries to filter stocks (default: all markets).
 * @returns {Promise<Array<{stockSymbol: string, eps: Array<{quarter: string, value: string}>}>>} - Array of stock symbols with their EPS data.
 */
export async function rankStocksByEps(
  skip = 0,
  limit = 20,
  country: typeof MarketCountry = MarketCountry,
) {
  try {
    // Fetch stock data from TradingView screener
    const stockData = await fetchScreenerStock(skip, limit, country);

    if (!stockData || !stockData.data || stockData.data.length === 0) {
      console.log('No stock data retrieved.');
      return [];
    }
    const result = [];

    // Create browser and context once for all stocks
    // Create browser and context once for all stocks
    let browser: any;
    let context: any;

    const options: any = { headless: true };
    if (process.env.CHROME_PATH) {
      options.executablePath = process.env.CHROME_PATH;
    } else if (process.platform === 'linux') {
      options.executablePath = await chromiumBinary.default.executablePath();
      options.args = chromiumBinary.default.args;
    }
    try {
      browser = await chromium.launch(options);
      context = await browser.newContext({
        userAgent:
          'Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)',
      });

      // Process each stock to extract or fetch EPS data
      for (const stock of stockData.data) {
        const stockSymbol = stock.s || 'Unknown';
        const epsData = [];

        // Optionally fetch detailed quarterly EPS data if needed
        try {
          const detailedData = await extractTableData(
            stockSymbol,
            browser,
            context,
          );
          if (detailedData && detailedData.length > 0) {
            // Sort and filter out invalid data, then limit to the last two quarters with valid EPS values for QoQ comparison
            const relevantData = detailedData
              .sort(
                (
                  a: { quarter: string; value: string },
                  b: { quarter: string; value: string },
                ) => {
                  const [aQ, aYear] = a.quarter.split(" '");
                  const [bQ, bYear] = b.quarter.split(" '");
                  if (!aQ || !aYear || !bQ || !bYear) return 0;
                  const aYNum = parseInt(aYear, 10);
                  const bYNum = parseInt(bYear, 10);
                  const aQNum = parseInt(aQ.replace('Q', ''), 10);
                  const bQNum = parseInt(bQ.replace('Q', ''), 10);
                  return aYNum !== bYNum ? bYNum - aYNum : bQNum - aQNum;
                },
              )
              .filter(
                (item: { quarter: string; value: string }) =>
                  item.value !== '—' && item.value !== 'N/A',
              ); // Filter out entries with no valid EPS data

            // Limit to the last two quarters with valid data
            const limitedData = relevantData.slice(0, 2);

            if (limitedData.length > 0) {
              limitedData.forEach(
                (item: { quarter: string; value: string }) => {
                  // Parse quarter_num and year_num from quarter string, e.g., "Q3 '21"
                  let quarter_num: number | null = null;
                  let year_num: number | null = null;
                  const match = item.quarter.match(/^Q(\d) '(\d{2})$/);
                  if (match) {
                    quarter_num = parseInt(match[1], 10);
                    year_num = 2000 + parseInt(match[2], 10);
                  }
                  epsData.push({
                    quarter_num,
                    year_num,
                    quarter: item.quarter,
                    value: item.value,
                  });
                },
              );
            } else {
              console.log(
                `No relevant EPS data for the last two quarters for ${stockSymbol}. Filtered data:`,
                relevantData,
              );
              // Fallback to basic EPS data if available in screener response
              if (stock.d && stock.d.length > 0) {
                if (stockData.columns) {
                  const epsTTMIndex = stockData.columns.indexOf(
                    'earnings_per_share_diluted_ttm',
                  );
                  if (epsTTMIndex !== -1 && stock.d[epsTTMIndex]) {
                    epsData.push({
                      quarter_num: null,
                      year_num: null,
                      quarter: 'QoQ',
                      value: stock.d[epsTTMIndex].toString(),
                    });
                  }
                } else {
                  const assumedEpsTTMIndex = 16; // Adjust based on known data structure if possible
                  if (stock.d[assumedEpsTTMIndex]) {
                    epsData.push({
                      quarter_num: null,
                      year_num: null,
                      quarter: 'QoQ (assumed)',
                      value: stock.d[assumedEpsTTMIndex].toString(),
                    });
                  }
                }
              }
            }
          } else {
            console.log(
              `No detailed EPS data returned for ${stockSymbol}. Data received:`,
              detailedData,
            );
            // Fallback to basic EPS data if available in screener response
            if (stock.d && stock.d.length > 0) {
              if (stockData.columns) {
                const epsTTMIndex = stockData.columns.indexOf(
                  'earnings_per_share_diluted_ttm',
                );
                if (epsTTMIndex !== -1 && stock.d[epsTTMIndex]) {
                  epsData.push({
                    quarter: 'QoQ',
                    value: stock.d[epsTTMIndex].toString(),
                  });
                }
              } else {
                const assumedEpsTTMIndex = 16; // Adjust based on known data structure if possible
                if (stock.d[assumedEpsTTMIndex]) {
                  epsData.push({
                    quarter: 'QoQ (assumed)',
                    value: stock.d[assumedEpsTTMIndex].toString(),
                  });
                }
              }
            }
          }
        } catch (error: unknown) {
          console.error(
            `Error fetching EPS data for ${stockSymbol}:`,
            error instanceof Error ? error.message : String(error),
          );
          // Fallback to basic EPS data if available
          if (stock.d && stock.d.length > 0) {
            // Manually search for EPS data in the array if columns are not defined
            if (stockData.columns) {
              const epsTTMIndex = stockData.columns.indexOf(
                'earnings_per_share_diluted_ttm',
              );
              if (epsTTMIndex !== -1 && stock.d[epsTTMIndex]) {
                epsData.push({
                  quarter: 'QoQ',
                  value: stock.d[epsTTMIndex].toString(),
                });
              }
            } else {
              // Fallback to assuming a position if columns are not available
              const assumedEpsTTMIndex = 16; // Adjust based on known data structure if possible
              if (stock.d[assumedEpsTTMIndex]) {
                epsData.push({
                  quarter: 'QoQ (assumed)',
                  value: stock.d[assumedEpsTTMIndex].toString(),
                });
              }
            }
          }
        }
        result.push({
          stockSymbol,
          eps: epsData,
        });
        console.log('EPS Comparison Data:', result);
      }
    } catch (error) {
      console.error('Error ranking stocks by EPS:', error);
      return [];
    } finally {
      if (context) {
        await context.close().catch((err: unknown) => {
          console.error(
            `Error closing context in rankingStocks:`,
            err instanceof Error ? err.message : String(err),
          );
        });
      }
      if (browser) {
        await browser.close().catch((err: unknown) => {
          console.error(
            `Error closing browser in rankingStocks:`,
            err instanceof Error ? err.message : String(err),
          );
        });
      }
    }
    return result;
  } catch (error) {
    console.error('Error in rankStocksByEps:', error);
    return [];
  }
}

console.log(await rankStocksByEps(0, 5, MarketCountry));
