export interface EpsGrowthResponse {
  symbol: string;
  companyName: string;
  marketCode: string;
  epsDiluted: number;
  previousEpsDiluted: number;
  epsGrowth: number;
  reportDate: string;
  year: number;
  quarter: number;
}
