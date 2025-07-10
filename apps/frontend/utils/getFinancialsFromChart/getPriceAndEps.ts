import WebSocket from 'ws';

export type FinancialsFromChart = {
  price: number | null;
  date: string;
  eps: number;
  quarter: number;
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

  // TradingView puts EPS under p[1].study_1.st
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

const mapEpsToPrice = (
  prices: PriceData[],
  eps: EpsData[],
): FinancialsFromChart[] => {
  return eps.map((epsItem) => {
    const closestPrice = findClosestPrice(prices, epsItem.date);

    // Log when we can't find a suitable price match
    if (!closestPrice) {
      console.warn(
        `No suitable price found for EPS date ${epsItem.date}. Available price range: ${prices.length > 0 ? `${prices[0].date} to ${prices[prices.length - 1].date}` : 'No prices available'}`,
      );
    }

    return {
      price: closestPrice?.price ?? null,
      date: epsItem.date,
      eps: epsItem.eps,
      quarter: getQuarter(epsItem.date),
    };
  });
};

// Adds eps_growth to each entry, calculated as percent change from sum of all previous eps values
function addEpsGrowth(
  arr: FinancialsFromChart[],
): (FinancialsFromChart & { eps_growth?: number | null })[] {
  let sumPrev = 0;
  return arr.map((item, idx) => {
    if (idx === 0) {
      sumPrev += item.eps;
      return { ...item };
    }
    const growth =
      sumPrev !== 0
        ? Math.round(((item.eps - sumPrev) / Math.abs(sumPrev)) * 100)
        : null;
    sumPrev += item.eps;
    return { ...item, eps_growth: growth };
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
  quarters = 2,
): Promise<Record<string, FinancialsFromChart[]>> {
  const results: Record<string, FinancialsFromChart[]> = {};
  for (const symbol of symbols) {
    results[symbol] = await new Promise<FinancialsFromChart[]>(
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

        const initSession = () => {
          sendMsg('chart_create_session', [sessionId, '']);
          sendMsg('resolve_symbol', [
            sessionId,
            symbolKey,
            `={"adjustment":"splits","symbol":"${symbol}"}`,
          ]);
        };

        const handleMessage = (data: any) => {
          const str = Buffer.isBuffer(data)
            ? data.toString('utf8')
            : String(data);
          const messages = parseMessages(str);

          messages.forEach((parsed) => {
            if (!resolved && parsed.m === 'symbol_resolved') {
              resolved = true;
              // Request more historical data to cover multiple quarters
              // Using 500 bars of daily data (~2 years) to ensure we have prices for EPS dates
              sendMsg('create_series', [
                sessionId,
                seriesKey,
                's1',
                symbolKey,
                '1D',
                500,
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

            prices.push(...extractPrices(parsed));
            eps.push(...extractEps(parsed));

            if (parsed.m === 'study_completed' && eps.length > 0) {
              ws.close();
              // Filter out unofficial EPS values (e.g., 1e+100) and take last N quarters
              const mapped = mapEpsToPrice(prices, eps)
                .filter((item) => item.eps !== 1e100)
                .slice(0, quarters);
              const withGrowth = addEpsGrowth(mapped);
              resolve(withGrowth);
            }
          });
        };

        ws.on('open', initSession);
        ws.on('message', (data) => {
          handleMessage(data);
        });
        ws.on('error', (err) => {
          ws.close();
          reject(err);
        });
        ws.on('close', () => {
          if (eps.length === 0) {
            reject(new Error('WebSocket closed before any data was received.'));
          } else {
            // Filter out unofficial EPS values (e.g., 1e+100) and take last N quarters
            const mapped = mapEpsToPrice(prices, eps)
              .filter((item) => item.eps !== 1e100)
              .slice(0, quarters);
            const withGrowth = addEpsGrowth(mapped);
            resolve(withGrowth);
          }
        });
      },
    ).catch(() => []);
  }
  return results;
}
