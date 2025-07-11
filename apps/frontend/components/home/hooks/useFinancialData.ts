import type { StockFinancialData, QuarterData } from '@/types/financialChartData';
import { 
  getLatestQuarterData, 
  calculateAverageEpsGrowth as calculateAvgEpsGrowth 
} from '@/utils/transformers/stockDataTransformer';

/**
 * Business logic hooks for financial data processing
 */

export interface UseFinancialDataResult {
  latestQuarter: QuarterData | null;
  avgGrowth: number | null;
  displayPrice: number | null;
  hasGrowthData: boolean;
  hasValidData: boolean;
}

/**
 * Hook to process financial data for a single stock
 */
export function useFinancialData(data: StockFinancialData): UseFinancialDataResult {
  const latestQuarter = getLatestQuarterData(data);
  
  const avgGrowth = calculateAvgEpsGrowth(data);
  
  const displayPrice = data.currentPrice !== undefined && data.currentPrice !== null
    ? data.currentPrice
    : latestQuarter?.price ?? null;
  
  const hasGrowthData = data.quarters.length >= 2 &&
    data.quarters[0].eps_growth !== undefined &&
    data.quarters[1].eps_growth !== undefined;
  
  const hasValidData = data.quarters.length > 0 && latestQuarter !== null;
  
  return {
    latestQuarter,
    avgGrowth,
    displayPrice,
    hasGrowthData,
    hasValidData,
  };
}

/**
 * Filter quarters to exclude invalid data
 */
export function getValidQuarters(quarters: QuarterData[]): QuarterData[] {
  return quarters.filter((quarter, idx) => {
    // Skip first quarter if it has no growth data
    if (
      idx === 0 &&
      (quarter.eps_growth === undefined || quarter.eps_growth === null) &&
      (quarter.price_growth === undefined || quarter.price_growth === null)
    ) {
      return false;
    }
    return true;
  });
}

/**
 * Determine if a growth value is positive
 */
export function isPositiveGrowth(value: number | undefined | null): boolean {
  return (value ?? 0) >= 0;
}

/**
 * Get growth indicator based on comparison
 */
export function getGrowthIndicator(current: number, previous: number): 'up' | 'down' {
  return current > previous ? 'up' : 'down';
}
