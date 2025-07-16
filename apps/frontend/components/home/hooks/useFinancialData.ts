import type { StockFinancialData, QuarterData } from '@/types/financialChartData';
import { 
  getLatestQuarterData, 
  calculateAverageEpsGrowth as calculateAvgEpsGrowth 
} from '@/utils/processStocks/stockDataTransformer';

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
  
  // Calculate average growth using ALL available quarters for accuracy
  // This is intentionally NOT limited to display quarters
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
 * Filter quarters to only show exactly 2 quarters FOR DISPLAY
 * Note: This is only for frontend display. Backend calculations use all available quarters.
 */
export function getValidQuarters(quarters: QuarterData[]): QuarterData[] {
  if (!quarters || quarters.length === 0) {
    return [];
  }

  // FORCE exactly 2 quarters for display, regardless of what data contains
  // Take the first 2 quarters (most recent)
  return quarters.slice(0, 2);
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
