export interface TradingViewResponse {
  totalCount: number;
  data: TradingViewStock[];
}

export interface TradingViewStock {
  s: string; // Symbol with exchange prefix
  d: [
    string, // Company Name
    string, // Symbol
    number, // Close Price
    number, // Change %
    number, // Change Abs
    number, // Recommend
    number, // Volume
    number, // Value Traded
    number, // Market Cap
    number, // P/E Ratio
    number, // EPS TTM
    string, // Sector
    string, // Country
    string, // Exchange
    number, // EPS FQ
    number, // EPS QoQ Growth
    number, // EPS Next Quarter
    number, // Earnings Release Date
    number, // Next Earnings Date
    string, // Description
    string, // Type
    string, // Subtype
    string, // Update Mode
    number, // Price Scale
    number, // Min Move
    string, // Fractional
    number, // Min Move 2
    string, // Currency
    string  // Fundamental Currency
  ];
}

export interface TableStockData {
  symbol: string;
  name: string;
  price: string;
  changePercent: string;
  volume: string;
  marketCap: string;
  peRatio: string;
  sector: string;
  country: string;
  exchange: string;
  currency: string;
  startBuy: {
    date: string;
    active: boolean;
  };
  startAction: {
    date: string;
    type: 'hold' | 'sell';
    active: boolean;
  };
  // Keep EPS data in type but don't display in frontend
  eps: string;
  epsGrowth: string;
  currentQuarterEps: string;
  nextEps: string;
  lastEarningsDate: string;
  nextEarningsDate: string;
}
