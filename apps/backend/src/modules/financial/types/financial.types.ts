import { Document } from 'mongoose';

export interface WorkerConfig {
  maxConcurrentRequests: number;
  batchDelay: number;
}

export interface StockWithMarketCode {
  _id: string;
  symbol: string;
  company_name: string | null;
  exchanges: Array<{
    market_code: string;
    primary: boolean;
  }>;
  market_code: string;
}

export interface ProcessedFinancialData {
  fiscalQuarter?: number;
  fiscalYear?: number;
  revenue?: number;
  revenueGrowth?: number;
  operatingIncome?: number;
  interestExpense?: number;
  netIncome?: number;
  epsBasic?: number;
  epsDiluted?: number;
  freeCashFlow?: number;
  profitMargin?: number;
  totalOperatingExpenses?: number;
}

export interface StockFinancialResponse {
  nodes: Array<{ data: ProcessedFinancialData[] }>;
  report_date: string;
  fiscal_quarter?: number | string;
  fiscal_year?: number;
  revenue?: number;
  revenue_growth?: number;
  operating_income?: number;
  interest_expense?: number;
  net_income?: number;
  eps_basic?: number;
  eps_diluted?: number;
  free_cash_flow?: number;
  profit_margin?: number;
  total_operating_expenses?: number;
}

export interface EPSGrowthData {
  symbol: string;
  company_name: string;
  market_code: string;
  exchange_name: string;
  eps: number;
  eps_growth: number;
  rank: number;
  last_report_date: Date;
}

export interface EPSGrowthResult {
  symbol: string;
  company_name: string;
  market_code: string;
  exchange_name: string;
  eps: number;
  eps_growth: number;
  rank: number;
  last_report_date: Date;
}

export interface FetchState extends Document {
  currentPage: number;
  totalProcessed: number;
  lastProcessedStock: string | null;
  lastUpdated: string;
}

export interface InitialFetchState {
  currentPage: number;
  totalProcessed: number;
  lastProcessedStock: string | null;
  lastUpdated: string;
}

// Financial model interfaces
export interface Financial extends Document {
  stock: string | any; // Reference to Stock model
  report_date: Date;
  fiscal_quarter: number;
  fiscal_year: number;
  revenue?: number;
  revenue_growth?: number;
  operating_income?: number;
  interest_expense?: number;
  net_income?: number;
  eps_basic?: number;
  eps_diluted?: number;
  free_cash_flow?: number;
  profit_margin?: number;
  total_operating_expenses?: number;
}

export interface Stock extends Document {
  symbol: string;
  company_name?: string;
  exchange?: {
    market_code: string;
  };
  financials?: string[] | Financial[]; // Array of references to Financial model
}

export interface PaginationParams {
  skip?: number;
  limit?: number;
}

export interface PaginationMetadata {
  total: number;
  skip: number;
  limit: number;
  hasMore: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  metadata: PaginationMetadata;
}
