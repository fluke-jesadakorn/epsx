export interface FinancialData {
  symbol: string;
  data: {
    description?: string;
    exchange?: string;
    type?: string;
    status?: string;
    session?: string;
    listed_exchange?: string;
    currency?: string;
    fundamental?: {
      [key: string]: any;
    };
  };
  status: 'success' | 'error';
  error?: string;
}

export interface FinancialRequestData {
  symbol: string;
  adjustment: string;
  session: string;
  type: string;
}
