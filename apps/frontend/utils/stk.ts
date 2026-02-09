// Stock processing utilities
export interface Quarter {
  price: number | null
  date: string
  eps: number
  quarter: string | number
  eps_growth?: number
  price_growth?: number
  last_eps_vs_current_price?: {
    lastEpsGrowth: number | null
    currentPriceGrowth: number | null
  }
}

export interface StockData {
  symbol: string
  quarters: Quarter[]
  currentPrice?: number | null
  currentPriceDate?: string | null
}

// Transform financial data
export const xform = (data: Record<string, unknown[]>): StockData[] =>
  Object.entries(data).map(([symbol, quarters]) => ({
    symbol,
    quarters: (quarters).map(q => {
      const item = q as Record<string, unknown>;
      return {
        price: item.price as number | null,
        date: item.date as string,
        eps: item.eps as number,
        quarter: item.quarter as string | number,
        eps_growth: item.eps_growth as number | undefined,
        price_growth: item.price_growth as number | undefined,
        last_eps_vs_current_price: item.last_eps_vs_current_price as { lastEpsGrowth: number | null; currentPriceGrowth: number | null } | undefined
      };
    })
  }))

export const xformPrice = (data: Record<string, unknown>): StockData[] =>
  Object.entries(data).map(([symbol, d]) => {
    const stock = d as Record<string, unknown>;
    return {
      symbol,
      quarters: (stock.quarters as unknown[]).map((q: unknown) => {
        const item = q as Record<string, unknown>;
        return {
          price: item.price as number | null,
          date: item.date as string,
          eps: item.eps as number,
          quarter: item.quarter as string | number,
          eps_growth: item.eps_growth as number | undefined,
          price_growth: item.price_growth as number | undefined,
          last_eps_vs_current_price: item.last_eps_vs_current_price as { lastEpsGrowth: number | null; currentPriceGrowth: number | null } | undefined
        };
      }),
      currentPrice: stock.currentPrice as number | null | undefined,
      currentPriceDate: stock.currentPriceDate as string | null | undefined
    };
  })

// Stock data helpers
export const latest = (stock: StockData): Quarter => {
  if (stock.quarters.length === 0) {
    throw new Error('No quarters available in stock data');
  }
  return stock.quarters[0];
}

export const avgEps = (stock: StockData): number | null => {
  const growth = stock.quarters.map(q => q.eps_growth).filter((g): g is number => g !== undefined);
  return growth.length ? Math.round(growth.reduce((a, b) => a + b, 0) / growth.length) : null
}

export const cmpLast = (stock: StockData) => stock.quarters[0]?.last_eps_vs_current_price ?? null

export const align = (comp: { lastEpsGrowth: number | null; currentPriceGrowth: number | null } | null): 'pos' | 'neg' | 'neutral' | null => {
   
  if (comp?.lastEpsGrowth == null || comp.currentPriceGrowth == null) { return null }
  const { lastEpsGrowth, currentPriceGrowth } = comp
  if ((lastEpsGrowth > 0 && currentPriceGrowth > 0) || (lastEpsGrowth < 0 && currentPriceGrowth < 0)) { return 'pos' }
  if ((lastEpsGrowth > 0 && currentPriceGrowth < 0) || (lastEpsGrowth < 0 && currentPriceGrowth > 0)) { return 'neg' }
  return 'neutral'
}

// Backward compatibility
export const transformFinancialData = xform
export const transformFinancialDataWithCurrentPrice = xformPrice
export const getLatestQuarterData = latest
export const calculateAverageEpsGrowth = avgEps
export const getLastEpsVsCurrentPriceComparison = cmpLast
export const getPriceEpsAlignment = align
