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
    string, // Fundamental Currency
  ];
}

export interface TableDataMetrics {
  symbol: string;
  name: string;
  valueIndex: string;
  growthRate: string;
  activityScore: string;
  marketSize: string;
  growthFactor: string;
  sector: string;
  country: string;
  exchange: string;
  currency: string;
  entryPhase: {
    date: string;
    active: boolean;
  };
  phaseStatus: {
    date: string;
    type: 'monitor' | 'exit';
    active: boolean;
  };
  // Analytics data kept but renamed
  metricScore: string;
  growthIndicator: string;
  currentMetric: string;
  predictedMetric: string;
  lastAnalysisDate: string;
  nextAnalysisDate: string;
  // Additional fields used in ClientEpsCardSection
  startBuy?: {
    active: boolean;
  };
  startAction?: {
    type: 'sell' | 'hold';
    active: boolean;
  };
  epsGrowth?: string;
  lastEarningsDate?: string;
  // Additional fields used in StockGrowthTable
  currentQuarterEps?: string;
  nextEps?: string;
  dataValue?: string;
  changePercent?: string;
  volume?: string;
  nextEarningsDate?: string;
}
