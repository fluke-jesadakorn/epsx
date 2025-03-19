import yahooFinance from 'yahoo-finance2';

type Quarter = {
  date: string;
  actual: number;
  estimate: number;
};

type YHResponse = {
  earnings?: {
    earningsChart?: {
      quarterly?: Quarter[];
    };
  };
  calendarEvents?: {
    earnings?: {
      earningsDate?: Date[];
    };
  };
};

type ChartData = {
  quotes: Array<{
    date: Date;
    close: number | null;
  }>;
};

type Metrics = {
  [key: string]: number | Date;
  nxtEps: number;
  nxtDate: Date;
  curEps: number;
  curGrowth: number;
  curDate: Date;
  curPrice: number;
  prvEps: number;
  prvGrowth: number;
  prvDate: Date;
  prvPrice: number;
  oldEps: number;
  oldDate: Date;
  oldPrice: number;
};

const DEF_METRICS: Metrics = {
  nxtEps: 0,
  nxtDate: new Date(0),
  curEps: 0, 
  curGrowth: 0,
  curDate: new Date(0),
  curPrice: 0,
  prvEps: 0,
  prvGrowth: 0,
  prvDate: new Date(0),
  prvPrice: 0,
  oldEps: 0,
  oldDate: new Date(0),
  oldPrice: 0,
};

const retry = async <T>(fn: () => Promise<T>): Promise<T> => {
  try {
    return await fn();
  } catch (e: unknown) {
    if (e instanceof Error && e instanceof yahooFinance.errors.HTTPError && e.message.includes('429')) {
      await new Promise(r => setTimeout(r, 2000));
      return fn();
    }
    throw e;
  }
};

const parseQDate = (qStr: string): Date => {
  const [q, y] = qStr.match(/(\d)Q(\d{4})/)?.slice(1) || [];
  if (!q || !y) return new Date(0);
  const d = new Date(+y, (+q - 1) * 3 + 3, 0); 
  d.setUTCHours(17, 0, 0, 0);
  return d;
};

const findQPrice = (qStr: string, data: ChartData): number => {
  if (!qStr || !data?.quotes?.length) return 0;
  const qEnd = parseQDate(qStr);
  const FIVE_D = 432e6; // 5 days in ms
  const nearest = data.quotes
    .filter(q => q.close != null && Math.abs(q.date.getTime() - qEnd.getTime()) <= FIVE_D)
    .sort((a, b) => Math.abs(a.date.getTime() - qEnd.getTime()) - Math.abs(b.date.getTime() - qEnd.getTime()))
    .slice(0, 5);

  return nearest.length ? nearest.reduce((s, q) => s + (q.close || 0), 0) / nearest.length : 0;
};

const nxtQ = (curDate: string): string => {
  const [q, y] = curDate.match(/(\d)Q(\d{4})/)?.slice(1) || [];
  return q && y ? `${q === '4' ? '1' : +q + 1}Q${q === '4' ? +y + 1 : y}` : '';
};

export const getYHPrice = async (sym: string): Promise<Metrics> => {
  if (!sym) throw new Error('Stock symbol required');

  try {
    const fin = await retry(() => yahooFinance.quoteSummary(sym, {
      modules: ['earnings', 'calendarEvents']
    })) as YHResponse;

    const qs = fin?.earnings?.earningsChart?.quarterly || [];
    if (!qs.length) {
      console.warn(`No earnings for ${sym}`);
      return DEF_METRICS;
    }

    const sortedQ = [...qs].sort((a, b) => {
      const [aQ, aY] = a.date.match(/(\d)Q(\d{4})/)?.slice(1) || [];
      const [bQ, bY] = b.date.match(/(\d)Q(\d{4})/)?.slice(1) || [];
      return +bY - +aY || +bQ - +aQ;
    });

    const [cur, prv, old] = sortedQ;
    const hist = await retry(() => yahooFinance.chart(sym, {
      period1: parseQDate(sortedQ[sortedQ.length - 1].date).toISOString().split('T')[0],
      period2: new Date().toISOString().split('T')[0],
      interval: '1d'
    })) as ChartData;

    const nxtDate = fin?.calendarEvents?.earnings?.earningsDate?.[0] || 
      (cur?.date ? parseQDate(nxtQ(cur.date)) : new Date(0));

    const curEps = Number(cur?.actual) || 0;
    const prvEps = Number(prv?.actual) || 0;
    const oldEps = Number(old?.actual) || 0;

    return {
      nxtEps: Number(cur?.estimate) || 0,
      nxtDate,
      curEps,
      curGrowth: prvEps ? (curEps - prvEps) / prvEps : 0,
      curDate: cur?.date ? parseQDate(cur.date) : new Date(0),
      curPrice: findQPrice(cur?.date, hist),
      prvEps,
      prvGrowth: oldEps ? (prvEps - oldEps) / oldEps : 0,
      prvDate: prv?.date ? parseQDate(prv.date) : new Date(0),
      prvPrice: findQPrice(prv?.date, hist),
      oldEps,
      oldDate: old?.date ? parseQDate(old.date) : new Date(0),
      oldPrice: findQPrice(old?.date, hist),
    };
  } catch (e: unknown) {
    if ((e instanceof yahooFinance.errors.FailedYahooValidationError || 
         e instanceof yahooFinance.errors.HTTPError) && 
         e instanceof Error) {
      console.warn(`${e.constructor.name} for ${sym}: ${e.message}`);
      return DEF_METRICS;
    }
    throw e;
  }
};

// Example usage
getYHPrice('AAPL').then((data) => console.log('From Yahoo:', data)).catch(console.error);
