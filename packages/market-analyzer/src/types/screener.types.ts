export interface ScreenerRequestHeaders {
  accept: string;
  'accept-language': string;
  'content-type': string;
  priority: string;
  'sec-ch-ua': string;
  'sec-ch-ua-mobile': string;
  'sec-ch-ua-platform': string;
  'sec-fetch-dest': string;
  'sec-fetch-mode': string;
  'sec-fetch-site': string;
  cookie?: string;
  Referer: string;
  'Referrer-Policy': string;
}

export interface ScreenerFilter {
  left: string;
  operation: string;
  right: any;
}

export interface ScreenerOperand {
  expression?: {
    left: string;
    operation: string;
    right: any;
  };
  operation?: {
    operator: 'and' | 'or';
    operands: ScreenerOperand[];
  };
}

export interface ScreenerRequestBody {
  columns: string[];
  filter: ScreenerFilter[];
  ignore_unknown_fields: boolean;
  options: {
    lang: string;
  };
  price_conversion?: {
    to_currency: string;
  };
  range: [number, number];
  sort?: {
    sortBy: string;
    sortOrder: 'asc' | 'desc';
  };
  symbols: Record<string, any>;
  markets: string[];
  filter2?: {
    operator: 'and' | 'or';
    operands: ScreenerOperand[];
  };
}

export interface ScreenerResponse {
  totalCount: number;
  data: Array<{
    name: string;
    description: string;
    logoid?: string;
    close?: number;
    volume?: number;
    market_cap_basic?: number;
    price_earnings_ttm?: number;
    sector?: string;
    market?: string;
    exchange?: string;
    [key: string]: any;
  }>;
}

export interface ScreenerOptions {
  filters?: ScreenerFilter[];
  columns?: string[];
  range?: [number, number];
  sort?: {
    by: string;
    order: 'asc' | 'desc';
  };
  markets?: string[];
}
