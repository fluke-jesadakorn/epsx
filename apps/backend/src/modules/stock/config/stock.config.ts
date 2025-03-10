export const STOCK_CONFIG = {
  stockBatchSize: 100,
  maxParallelRequests: 3,
  batchDelay: 1000,
} as const;

export type StockConfig = typeof STOCK_CONFIG;
