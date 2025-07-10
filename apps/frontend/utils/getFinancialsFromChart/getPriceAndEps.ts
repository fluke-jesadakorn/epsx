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

  return prices.reduce((closest, current) => {
    const currentDiff = Math.abs(
      new Date(current.date).getTime() - new Date(targetDate).getTime(),
    );
    const closestDiff = Math.abs(
      new Date(closest.date).getTime() - new Date(targetDate).getTime(),
    );
    return currentDiff < closestDiff ? current : closest;
  });
};

const getQuarter = (date: string): number =>
  Math.ceil((new Date(date).getMonth() + 1) / 3);

const mapEpsToPrice = (
  prices: PriceData[],
  eps: EpsData[],
): FinancialsFromChart[] => {
  return eps.map((epsItem) => {
    const closestPrice = findClosestPrice(prices, epsItem.date);

    return {
      price: closestPrice?.price ?? null,
      date: epsItem.date,
      eps: epsItem.eps,
      quarter: getQuarter(epsItem.date),
    };
  });
};

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
      throw new Error('Invalid JSON payload: ' + payload);
    }
  }

  return messages;
};

export async function getFinancialsFromChart(): Promise<FinancialsFromChart[]> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket('wss://data.tradingview.com/socket.io/websocket', {
      headers: {
        Origin: 'https://www.tradingview.com',
        'User-Agent': 'Mozilla/5.0',
      },
    });

    const sendMsg = (method: string, params: any[]) => {
      const msg = JSON.stringify({ m: method, p: params });
      ws.send(`~m~${msg.length}~m~${msg}`);
    };

    const prices: PriceData[] = [];
    const eps: EpsData[] = [];

    // Generate a unique sessionId based on 'cs_' + a hash of symbol and a random number
    const symbol = 'NASDAQ:AAPL';
    const hash = Math.abs(
      Array.from(symbol).reduce((acc, char) => acc + char.charCodeAt(0), 0) +
        Math.floor(Math.random() * 10000),
    );
    let sessionId = `cs_${hash}`;
    let symbolResolved = false;

    const initSession = () => {
      sessionId = 'cs_session';
      sendMsg('chart_create_session', [sessionId, '']);
      sendMsg('resolve_symbol', [
        sessionId,
        'symbol_1',
        `={"adjustment":"splits","symbol":"${symbol}"}`,
      ]);
    };

    const handleMessage = (data: any) => {
      const str = Buffer.isBuffer(data) ? data.toString('utf8') : String(data);
      const messages = parseMessages(str);

      messages.forEach((parsed) => {
        // Wait for symbol to be resolved before proceeding
        if (!symbolResolved && parsed.m === 'symbol_resolved') {
          symbolResolved = true;
          sendMsg('create_series', [
            sessionId,
            'series_1',
            's1',
            'symbol_1',
            '1D',
            20,
            '',
          ]);
          sendMsg('create_study', [
            sessionId,
            'study_1',
            's1_study',
            'series_1',
            'Earnings@tv-basicstudies-251',
            {},
          ]);
        }

        prices.push(...extractPrices(parsed));
        eps.push(...extractEps(parsed));

        if (parsed.m === 'study_completed') {
          const result = mapEpsToPrice(prices, eps);
          ws.close();
          resolve(result);
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
      throw new Error(`WebSocket error: ${err.message}`);
    });
    ws.on('close', () => {
      if (!prices.length && !eps.length) {
        reject(new Error('WebSocket closed before any data was received.'));
      } else {
        // If partial data was received, resolve with what we have
        const result = mapEpsToPrice(prices, eps);
        resolve(result);
      }
    });
  });
}

getFinancialsFromChart()
  .then((data) => {
    console.log('Financials from chart:', data);
  })
  .catch((error) => {
    console.error('Error fetching financials from chart:', error);
  });
