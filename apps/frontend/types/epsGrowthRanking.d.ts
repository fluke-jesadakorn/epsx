export interface EpsGrowthRankingResponse {
  data: EpsGrowthData[];
  metadata: Metadata;
}

export interface EpsGrowthData {
  symbol: string;
  company_name: string;
  market_code: string;
  eps_diluted: number;
  eps_growth: number;
  previous_eps_diluted: number;
  report_date: string;
  quarter: number;
  year: number;
}

export interface Metadata {
  skip: number;
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}
