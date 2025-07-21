// DEPRECATED: Use stk.ts instead
console.warn('processStocks/stockDataTransformer.ts is deprecated. Use stk.ts instead')

// Re-export everything from the new stk.ts file for backward compatibility
export {
  // Main exports
  getLatestQuarterData,
  calculateAverageEpsGrowth,
  getLastEpsVsCurrentPriceComparison,
  getPriceEpsAlignment,
  transformFinancialData,
  transformFinancialDataWithCurrentPrice,
  // Additional exports for completeness
  xform,
  xformPrice,
  latest,
  avgEps,
  cmpLast,
  align,
} from '@/utils/stk';

// Additional utility functions that might be used elsewhere
export const formatPrice = (price: number | null): string => {
  if (price === null || price === undefined) return 'N/A';
  return `$${price.toFixed(2)}`;
};

export const formatDate = (date: string): string => {
  if (!date) return 'N/A';
  return new Date(date).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
};
