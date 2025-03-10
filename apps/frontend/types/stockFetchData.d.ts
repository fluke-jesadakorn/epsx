export interface StockScreenerResponse {
  status: number;
  data: {
    data: Record<string, StockData>;
  };
}

export interface StockData {
  eps: number;
  earningsEpsEstimate: number;
  epsGrowthQ: number;
  epsNextQuarter: number;
  epsGrowthQuarters: number;
  earningsDate: string;
  epsGrowth: number;
  country: string;
  epsGrowthYears?: number;
  epsThisYear: number;
  epsThisQuarter: number;
  epsNextYear: number;
  fiscalYearEnd: string;
  sector: string;
  exchange: string;
  lastReportDate: string;
  tags: string[];
  earningsEpsEstimateGrowth: number;
  ma20: number;
}

export interface TableStockData {
  symbol: string;
  eps: string;
  epsGrowthQ: string;
  epsNextQuarter: string;
  earningsDate: string;
  country: string;
  sector: string;
  exchange: string;
  lastReportDate: string;
  tags: string[];
  earningsEpsEstimateGrowth: string;
  ma20: string;
}
