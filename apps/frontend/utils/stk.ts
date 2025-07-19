// Stock processing utilities
export interface Quarter {
  price: number | null
  date: string
  eps: number
  quarter: string
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
  currentPrice?: number
  currentPriceDate?: string
}

// Transform financial data
export const xform = (data: any): StockData[] => 
  Object.entries(data).map(([symbol, quarters]) => ({
    symbol,
    quarters: (quarters as any[]).map(q => ({
      price: q.price,
      date: q.date,
      eps: q.eps,
      quarter: q.quarter,
      eps_growth: q.eps_growth,
      price_growth: q.price_growth,
      last_eps_vs_current_price: q.last_eps_vs_current_price
    }))
  }))

export const xformPrice = (data: any): StockData[] => 
  Object.entries(data).map(([symbol, d]) => ({
    symbol,
    quarters: (d as any).quarters.map((q: any) => ({
      price: q.price,
      date: q.date,
      eps: q.eps,
      quarter: q.quarter,
      eps_growth: q.eps_growth,
      price_growth: q.price_growth,
      last_eps_vs_current_price: q.last_eps_vs_current_price
    })),
    currentPrice: (d as any).currentPrice,
    currentPriceDate: (d as any).currentPriceDate
  }))

// Stock data helpers
export const latest = (stock: StockData): Quarter => stock.quarters[0]

export const avgEps = (stock: StockData): number | null => {
  const growth = stock.quarters.map(q => q.eps_growth).filter(g => g !== undefined && g !== null) as number[]
  return growth.length ? Math.round(growth.reduce((a, b) => a + b, 0) / growth.length) : null
}

export const cmpLast = (stock: StockData) => stock.quarters[0]?.last_eps_vs_current_price || null

export const align = (comp: any): 'pos' | 'neg' | 'neutral' | null => {
  if (!comp || comp.lastEpsGrowth === null || comp.currentPriceGrowth === null) return null
  const { lastEpsGrowth, currentPriceGrowth } = comp
  if ((lastEpsGrowth > 0 && currentPriceGrowth > 0) || (lastEpsGrowth < 0 && currentPriceGrowth < 0)) return 'pos'
  if ((lastEpsGrowth > 0 && currentPriceGrowth < 0) || (lastEpsGrowth < 0 && currentPriceGrowth > 0)) return 'neg'
  return 'neutral'
}

// Backward compatibility
export const transformFinancialData = xform
export const transformFinancialDataWithCurrentPrice = xformPrice
export const getLatestQuarterData = latest
export const calculateAverageEpsGrowth = avgEps
export const getLastEpsVsCurrentPriceComparison = cmpLast
export const getPriceEpsAlignment = align
