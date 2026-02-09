import type {
  StockFinancialData,
  QuarterData,
} from '@/types/financialChartData';

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
 * Get the latest quarter data from stock financial data
 */
function getLatestQuarterData(data: StockFinancialData): QuarterData | null {
  return data.quarters && data.quarters.length > 0 ? data.quarters[0] : null;
}

/**
 * Calculate average EPS growth from all available quarters
 * Uses all quarters (including the 3rd calculation quarter) for accurate averaging
 */
function calculateAvgEpsGrowth(data: StockFinancialData): number | null {
  if (!data.quarters || data.quarters.length === 0) {return null;}
  
  // Use all available quarters for growth calculation (including the 3rd quarter)
  const growth = data.quarters
    .map((q) => q.eps_growth)
    .filter((g) => g !== undefined && g !== null && g !== 0) as number[];

  return growth.length
    ? Math.round(growth.reduce((a, b) => a + b, 0) / growth.length)
    : null;
}

/**
 * Hook to process financial data for a single stock
 */
export function useFinancialData(
  data: StockFinancialData,
): UseFinancialDataResult {
  const latestQuarter = getLatestQuarterData(data);

  // Calculate average growth using ALL available quarters for accuracy
  // This is intentionally NOT limited to display quarters
  const avgGrowth = calculateAvgEpsGrowth(data);

  const displayPrice =
    data.currentPrice ?? (latestQuarter?.price ?? null);

  const hasGrowthData =
    data.quarters && data.quarters.length >= 2 &&
    data.quarters[0].eps_growth !== undefined &&
    data.quarters[1].eps_growth !== undefined;

  const hasValidData = data.quarters && data.quarters.length > 0 && latestQuarter !== null;

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
 * We keep 3 quarters in the data for QoQ growth calculation but only display first 2.
 */
export function getValidQuarters(quarters: QuarterData[]): QuarterData[] {
  if (!quarters || quarters.length === 0) {
    return [];
  }

  // FORCE exactly 2 quarters for display (current + previous)
  // The 3rd quarter exists in the data for QoQ calculations but isn't displayed
  const limitedQuarters = quarters.slice(0, 2);
  
  // Additional safety check to ensure we never return more than 2 quarters
  if (limitedQuarters.length > 2) {
    console.warn('getValidQuarters: More than 2 quarters detected, trimming to 2');
    return limitedQuarters.slice(0, 2);
  }
  
  return limitedQuarters;
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
export function getGrowthIndicator(
  current: number,
  previous: number,
): 'up' | 'down' {
  return current > previous ? 'up' : 'down';
}
