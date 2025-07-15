/**
 * Extracts EPS data from TradingView socket data, mapping each EPS entry from "earnings_fq_h"
 * to its corresponding date from the price/earnings value array (the "v" array in the socket data).
 *
 * @param earningsFqH The array from "earnings_fq_h" (parsed JSON)
 * @param vArray The array of objects like {i, v: [timestamp, ...]} from the socket data
 * @returns Array of EPS entries with mapped dates
 */
export function extractEpsWithDatesFromSocketData(
  earningsFqH: Array<any>,
  vArray: Array<{ i: number; v: any[] }>,
): Array<{
  actual: number | null;
  estimate: number | null;
  fiscalPeriod: string;
  isReported: boolean;
  type: number;
  date: string | null;
}> {
  // Map index to date from vArray (using the first value in v as the timestamp)
  const indexToDate: Record<number, string> = {};
  for (const entry of vArray) {
    // entry.i is the index, entry.v[0] is the timestamp (in seconds)
    if (typeof entry.i === 'number' && Array.isArray(entry.v) && typeof entry.v[0] === 'number') {
      indexToDate[entry.i] = new Date(entry.v[0] * 1000).toISOString().slice(0, 10);
    }
  }

  // Map EPS entries to their dates
  return earningsFqH.map((epsEntry, idx) => {
    // Try to find the date by index (idx or by Type)
    // If "Type" matches entry.v[8], you could use that, but usually order matches
    const date = indexToDate[idx] || null;
    return {
      actual: epsEntry.Actual ?? null,
      estimate: epsEntry.Estimate ?? null,
      fiscalPeriod: epsEntry.FiscalPeriod ?? '',
      isReported: epsEntry.IsReported ?? false,
      type: epsEntry.Type ?? null,
      date,
    };
  });
}
import WebSocket from 'ws';

export type FinancialsFromChart = {
  price: number | null;
  date: string;
  eps: number;
  quarter: number;
};

export interface CurrentPriceData {
  symbol: string;
  price: number | null;
  timestamp: string;
  date: string;
}

export type FinancialsWithCurrentPrice = {
  quarters: FinancialsFromChart[];
  currentPrice: number | null;
  currentPriceDate: string | null;
};

type PriceData = { date: string; price: number };
type EpsData = { date: string; eps: number };

const formatDate = (timestamp: number): string =>
  new Date(timestamp * 1000).toISOString().slice(0, 10);

const isValidEpsData = (v: any): v is [number, number] =>
  Array.isArray(v) && typeof v[0] === 'number' && typeof v[1] === 'number';

const extractPrices = (data: any): PriceData[] => {
  // TradingView puts price bars under p[1].series_1.s
  const bars = data.p?.[1]?.series_1?.s;
  if (!bars) return [];

  return bars.map((bar: any) => ({
    date: formatDate(bar.v[0]),
    price: bar.v[4],
  }));
};

const extractEps = (data: any): EpsData[] => {
  const result: EpsData[] = [];

  // Check for different message types that might contain earnings data
  if (data.m === 'timescale_update' || data.m === 'qsd' || data.m === 'study_update') {
    // Look for earnings data in the message parameters
    if (data.p && Array.isArray(data.p)) {
      data.p.forEach((param: any) => {
        // Check for study data with earnings information
        if (param && param.st && Array.isArray(param.st)) {
          param.st.forEach((st: any) => {
            if (st.v && Array.isArray(st.v) && st.v.length >= 3) {
              const timestamp = st.v[0];
              const actualEps = st.v[1];  // Reported/Actual EPS
              // st.v[2] contains estimate EPS if needed later
              
              // Use the actual/reported EPS value (st.v[1]) for our main EPS data
              // Filter out invalid EPS values (like 1e+100 which indicates no data)
              if (typeof timestamp === 'number' && 
                  typeof actualEps === 'number' && 
                  actualEps !== 1e+100 && 
                  Number.isFinite(actualEps)) {
                result.push({
                  date: formatDate(timestamp),
                  eps: actualEps,
                });
              }
            }
          });
        }
        
        // Also check for earnings_fq_h structure if present
        if (param && param.earnings_fq_h && Array.isArray(param.earnings_fq_h)) {
          param.earnings_fq_h.forEach((earning: any) => {
            if (earning.Actual !== null && earning.Actual !== undefined && 
                Number.isFinite(earning.Actual) && earning.IsReported) {
              // Try to extract date from fiscal period or use index-based mapping
              const fiscalPeriod = earning.FiscalPeriod;
              let date: string;
              
              if (fiscalPeriod && typeof fiscalPeriod === 'string') {
                // Parse fiscal period like "2024-Q3" to approximate date
                const match = fiscalPeriod.match(/(\d{4})-Q(\d)/);
                if (match) {
                  const year = parseInt(match[1]);
                  const quarter = parseInt(match[2]);
                  const month = (quarter - 1) * 3 + 2; // Approximate middle of quarter
                  date = new Date(year, month, 15).toISOString().slice(0, 10);
                } else {
                  date = new Date().toISOString().slice(0, 10); // Fallback to current date
                }
              } else {
                date = new Date().toISOString().slice(0, 10); // Fallback to current date
              }
              
              result.push({
                date: date,
                eps: earning.Actual,
              });
            }
          });
        }
      });
    }
  }

  // Fallback: Also check the old structure for backwards compatibility
  const study = data.p?.[1]?.study_1?.st;
  if (study) {
    study.forEach((st: any) => {
      if (isValidEpsData(st.v)) {
        result.push({
          date: formatDate(st.v[0]),
          eps: st.v[1],
        });
      }
    });
  }

  return result;
};

const findClosestPrice = (
  prices: PriceData[],
  targetDate: string,
): PriceData | null => {
  if (!prices.length) return null;

  const targetTime = new Date(targetDate).getTime();

  // Find the closest price, but only if it's within a reasonable time range (60 days)
  const maxDiffMs = 60 * 24 * 60 * 60 * 1000; // 60 days in milliseconds

  let closest = prices[0];
  let closestDiff = Math.abs(new Date(closest.date).getTime() - targetTime);

  for (const current of prices) {
    const currentDiff = Math.abs(new Date(current.date).getTime() - targetTime);
    if (currentDiff < closestDiff) {
      closest = current;
      closestDiff = currentDiff;
    }
  }

  // Return null if the closest price is more than 60 days away from the target date
  if (closestDiff > maxDiffMs) {
    return null;
  }

  return closest;
};

const getQuarter = (date: string): number =>
  Math.ceil((new Date(date).getMonth() + 1) / 3);

// Remove unused 'eps' parameter from getZone
const getZone = (price: number | null): 'upper' | 'lower' | null => {
  if (price == null) return null;
  return price >= 30 ? 'upper' : 'lower';
};

const getPartition = (eps: number): string => {
  // Example logic: partition by EPS value (customize as needed)
  if (eps >= 0.5) return 'A';
  if (eps >= 0.45) return 'B';
  return 'C';
};

const mapEpsToPrice = (
  prices: PriceData[],
  eps: EpsData[],
): (FinancialsFromChart & {
  partition: string;
  zone: 'upper' | 'lower' | null;
})[] => {
  return eps.map((epsItem) => {
    const closestPrice = findClosestPrice(prices, epsItem.date);
    return {
      price: closestPrice?.price ?? null,
      date: epsItem.date,
      eps: epsItem.eps,
      quarter: getQuarter(epsItem.date),
      partition: getPartition(epsItem.eps),
      zone: getZone(closestPrice?.price ?? null),
    };
  });
};

/**
 * Adds eps_growth to each entry, calculated as percent change from previous quarter's EPS
 */
function addEpsGrowth(
  arr: FinancialsFromChart[],
): (FinancialsFromChart & { eps_growth?: number | null })[] {
  return arr.map((item, idx, array) => {
    if (idx === 0 || array[idx - 1].eps === 0) {
      return { ...item, eps_growth: null };
    }
    const prevEps = array[idx - 1].eps;
    const growth = Math.round(((item.eps - prevEps) / Math.abs(prevEps)) * 100);
    return { ...item, eps_growth: growth };
  });
}

/**
 * Adds price_growth to each entry, calculated as percent change from previous price
 */
function addPriceGrowth<T extends { price: number | null }>(
  arr: (T & { price_growth?: number | null })[],
): (T & { price_growth?: number | null })[] {
  let prevPrice: number | null = null;
  return arr.map((item, idx) => {
    if (
      idx === 0 ||
      prevPrice === null ||
      item.price == null ||
      prevPrice === 0
    ) {
      prevPrice = item.price;
      return { ...item, price_growth: null };
    }
    const growth = Math.round(
      ((item.price - prevPrice) / Math.abs(prevPrice)) * 100,
    );
    prevPrice = item.price;
    return { ...item, price_growth: growth };
  });
}

const parseMessages = (str: string): any[] => {
  const messages: any[] = [];
  let i = 0;

  while (i < str.length && str.startsWith('~m~', i)) {
    i += 3;
    const j = str.indexOf('~m~', i);
    if (j === -1) break;

    const len = parseInt(str.slice(i, j), 10);
    const payload = str.slice(j + 3, j + 3 + len);
    i = j + 3 + len;

    try {
      messages.push(JSON.parse(payload));
    } catch {
      // Ignore non-JSON payloads (e.g., ~h~1 heartbeat messages)
    }
  }

  return messages;
};

export async function getFinancialsFromChart(
  symbols: string[],
  quarters = 4,
): Promise<Record<string, FinancialsFromChart[]>> {
  const results: Record<string, FinancialsFromChart[]> = {};
  const delay = (ms: number) =>
    new Promise((resolve) => setTimeout(resolve, ms));

  console.log(`📊 Processing ${symbols.length} symbols for financial data`);

  // Process symbols with retry logic and rate limiting
  for (let i = 0; i < symbols.length; i++) {
    const symbol = symbols[i];

    // Add delay between requests to avoid rate limiting (except for first symbol)
    if (i > 0) {
      await delay(2000); // 2 second delay between requests
    }

    const financialData = await getFinancialsWithCurrentPriceForSymbol(
      symbol,
      quarters,
    );
    results[symbol] = financialData.quarters;
  }

  const totalSuccessful = Object.values(results).filter(
    (data) => data.length > 0,
  ).length;
  console.log(
    `✅ Completed: ${totalSuccessful}/${symbols.length} symbols returned data`,
  );

  return results;
}

/**
 * New function that returns both quarterly data and current price
 */
export async function getFinancialsWithCurrentPriceFromChart(
  symbols: string[],
  quarters = 4,
): Promise<Record<string, FinancialsWithCurrentPrice>> {
  const results: Record<string, FinancialsWithCurrentPrice> = {};
  const delay = (ms: number) =>
    new Promise((resolve) => setTimeout(resolve, ms));

  console.log(
    `💰 Processing ${symbols.length} symbols for financial data with current prices`,
  );

  // Process symbols with retry logic and rate limiting
  for (let i = 0; i < symbols.length; i++) {
    const symbol = symbols[i];

    // Add delay between requests to avoid rate limiting (except for first symbol)
    if (i > 0) {
      await delay(2000); // 2 second delay between requests
    }

    results[symbol] = await getFinancialsWithCurrentPriceForSymbol(
      symbol,
      quarters,
    );
  }

  const totalSuccessful = Object.values(results).filter(
    (data) => data.quarters.length > 0,
  ).length;
  console.log(
    `✅ Completed: ${totalSuccessful}/${symbols.length} symbols returned data with current prices`,
  );

  return results;
}

/**
 * Helper function to find the most recent price from price data
 */
const findMostRecentPrice = (
  prices: PriceData[],
): { price: number | null; date: string | null } => {
  if (!prices.length) return { price: null, date: null };

  // Sort prices by date descending to get the most recent first
  const sortedPrices = [...prices].sort(
    (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
  );
  const mostRecent = sortedPrices[0];

  return {
    price: mostRecent.price,
    date: mostRecent.date,
  };
};

async function getFinancialsWithCurrentPriceForSymbol(
  symbol: string,
  quarters: number,
  retryCount = 0,
  maxRetries = 3,
): Promise<FinancialsWithCurrentPrice> {
  const delay = (ms: number) =>
    new Promise((resolve) => setTimeout(resolve, ms));

  try {
    const data = await new Promise<FinancialsWithCurrentPrice>(
      (resolve, reject) => {
        const ws = new WebSocket(
          'wss://data.tradingview.com/socket.io/websocket',
          {
            headers: {
              Origin: 'https://www.tradingview.com',
              'User-Agent': 'Mozilla/5.0',
            },
          },
        );

        const sendMsg = (method: string, params: any[]) => {
          const msg = JSON.stringify({ m: method, p: params });
          ws.send(`~m~${msg.length}~m~${msg}`);
        };

        const prices: PriceData[] = [];
        const eps: EpsData[] = [];

        const sessionId = `cs_${Math.abs(
          Array.from(symbol).reduce(
            (acc, char) => acc + char.charCodeAt(0),
            0,
          ) + Math.floor(Math.random() * 10000),
        )}`;
        const symbolKey = `symbol_1`;
        const seriesKey = `series_1`;
        const studyKey = `study_1`;
        let resolved = false;

        // Add timeout for WebSocket operations
        const timeout = setTimeout(() => {
          console.warn(`⏰ Timeout for ${symbol} after 30 seconds`);
          ws.close();
          reject(new Error(`Timeout after 30 seconds for symbol: ${symbol}`));
        }, 30000);

        const initSession = () => {
          sendMsg('chart_create_session', [sessionId, '']);
          sendMsg('resolve_symbol', [
            sessionId,
            symbolKey,
            `={"adjustment":"splits","symbol":"${symbol}"}`,
          ]);
        };

        const handleMessage = (data: any) => {
          try {
            const str = Buffer.isBuffer(data)
              ? data.toString('utf8')
              : String(data);
            const messages = parseMessages(str);

            messages.forEach((parsed) => {
              if (
                parsed.m === 'symbol_error' ||
                parsed.m === 'critical_error'
              ) {
                console.error(`❌ ${symbol}: Symbol error:`, parsed);
                clearTimeout(timeout);
                ws.close();
                reject(
                  new Error(
                    `Symbol error for ${symbol}: ${JSON.stringify(parsed)}`,
                  ),
                );
                return;
              }

              if (!resolved && parsed.m === 'symbol_resolved') {
                resolved = true;

                // Request more historical data to cover multiple quarters
                sendMsg('create_series', [
                  sessionId,
                  seriesKey,
                  's1',
                  symbolKey,
                  '1D',
                  1000,
                  '',
                ]);
                sendMsg('create_study', [
                  sessionId,
                  studyKey,
                  's1_study',
                  seriesKey,
                  'Earnings@tv-basicstudies-251',
                  {},
                ]);
              }

              const newPrices = extractPrices(parsed);
              const newEps = extractEps(parsed);

              if (newPrices.length > 0) {
                prices.push(...newPrices);
                console.log(`📈 ${symbol}: Found ${newPrices.length} price data points`);
              }

              if (newEps.length > 0) {
                eps.push(...newEps);
                console.log(`📊 ${symbol}: Found ${newEps.length} EPS data points from ${parsed.m} message`);
                // Debug: Log all EPS entries to see what values we're getting
                console.log(`📊 ${symbol}: EPS values:`, newEps.map(e => `${e.date}: ${e.eps}`));
              }

              if (parsed.m === 'study_completed') {
                clearTimeout(timeout);
                ws.close();

                if (eps.length === 0) {
                  console.warn(`⚠️ ${symbol}: No EPS data found`);
                  resolve({
                    quarters: [],
                    currentPrice: null,
                    currentPriceDate: null,
                  });
                  return;
                }

                // Get the most recent price from all price data
                const { price: currentPrice, date: currentPriceDate } =
                  findMostRecentPrice(prices);

                // Filter out unofficial or invalid EPS values and take last N quarters
                const mapped = mapEpsToPrice(prices, eps)
                  .filter(
                    (item) => item.eps !== 1e100 && Number.isFinite(item.eps),
                  )
                  .slice(-quarters); // Take the last N quarters instead of first N

                // Debug: Log the final mapped results
                console.log(`📊 ${symbol}: Final mapped EPS data:`, mapped.map(m => `${m.date}: ${m.eps} (Q${m.quarter})`));

                const withEpsGrowth = addEpsGrowth(mapped);
                const withBothGrowths = addPriceGrowth(withEpsGrowth);

                resolve({
                  quarters: withBothGrowths,
                  currentPrice,
                  currentPriceDate,
                });
              }
            });
          } catch (error) {
            console.error(`🚨 ${symbol}: Error handling message:`, error);
            clearTimeout(timeout);
            ws.close();
            reject(error);
          }
        };

        ws.on('open', initSession);

        ws.on('message', handleMessage);

        ws.on('error', (err) => {
          console.error(`🚨 ${symbol}: WebSocket error:`, err);
          clearTimeout(timeout);
          ws.close();
          reject(err);
        });

        // Remove unused 'reason' parameter in ws.on('close')
        ws.on('close', (code) => {
          clearTimeout(timeout);

          if (eps.length === 0) {
            console.warn(
              `⚠️ ${symbol}: No EPS data received before close (Code: ${code})`,
            );
            resolve({
              quarters: [],
              currentPrice: null,
              currentPriceDate: null,
            });
          } else {
            // Get the most recent price from all price data
            const { price: currentPrice, date: currentPriceDate } =
              findMostRecentPrice(prices);

            // Filter out unofficial or invalid EPS values and take last N quarters
            const mapped = mapEpsToPrice(prices, eps)
              .filter((item) => item.eps !== 1e100 && Number.isFinite(item.eps))
              .slice(-quarters); // Take the last N quarters instead of first N
            const withEpsGrowth = addEpsGrowth(mapped);
            const withBothGrowths = addPriceGrowth(withEpsGrowth);

            resolve({
              quarters: withBothGrowths,
              currentPrice,
              currentPriceDate,
            });
          }
        });
      },
    );

    return data;
  } catch (error) {
    console.error(`❌ ${symbol}: Attempt ${retryCount + 1} failed:`, error);

    // Retry with exponential backoff
    if (retryCount < maxRetries) {
      const backoffDelay = Math.min(1000 * Math.pow(2, retryCount), 10000); // Cap at 10 seconds
      console.log(`🔄 ${symbol}: Retrying in ${backoffDelay}ms...`);
      await delay(backoffDelay);
      return getFinancialsWithCurrentPriceForSymbol(
        symbol,
        quarters,
        retryCount + 1,
        maxRetries,
      );
    }

    console.error(
      `💥 ${symbol}: All retry attempts failed. Returning empty data.`,
    );
    return {
      quarters: [],
      currentPrice: null,
      currentPriceDate: null,
    };
  }
}
