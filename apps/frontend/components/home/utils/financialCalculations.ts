import type { StockFinancialData, QuarterData } from '@/types/financialChartData';

/**
 * Utility functions for financial data calculations
 */

/**
 * Calculate percentage change between two values
 */
export function calculatePercentageChange(current: number, previous: number): number {
  if (previous === 0) return 0;
  return ((current - previous) / previous) * 100;
}

/**
 * Format large numbers with appropriate suffixes (K, M, B)
 */
export function formatLargeNumber(num: number): string {
  if (num >= 1e9) {
    return (num / 1e9).toFixed(1) + 'B';
  }
  if (num >= 1e6) {
    return (num / 1e6).toFixed(1) + 'M';
  }
  if (num >= 1e3) {
    return (num / 1e3).toFixed(1) + 'K';
  }
  return num.toString();
}

/**
 * Get the best available price from stock data
 */
export function getBestAvailablePrice(data: StockFinancialData): number | null {
  // Prefer current price if available
  if (data.currentPrice !== undefined && data.currentPrice !== null) {
    return data.currentPrice;
  }
  
  // Fall back to latest quarter price
  if (data.quarters.length > 0) {
    const latestQuarter = data.quarters[0];
    return latestQuarter.price;
  }
  
  return null;
}

/**
 * Calculate volatility (standard deviation of growth rates)
 */
export function calculateVolatility(quarters: QuarterData[]): number | null {
  const growthRates = quarters
    .map(q => q.eps_growth)
    .filter((growth): growth is number => growth !== undefined && growth !== null);
  
  if (growthRates.length < 2) return null;
  
  const mean = growthRates.reduce((sum, rate) => sum + rate, 0) / growthRates.length;
  const variance = growthRates.reduce((sum, rate) => sum + Math.pow(rate - mean, 2), 0) / growthRates.length;
  
  return Math.sqrt(variance);
}

/**
 * Determine risk level based on volatility
 */
export function getRiskLevel(volatility: number | null): 'low' | 'medium' | 'high' {
  if (volatility === null) return 'medium';
  
  if (volatility < 10) return 'low';
  if (volatility < 25) return 'medium';
  return 'high';
}

/**
 * Get trend direction over multiple quarters
 */
export function getTrendDirection(quarters: QuarterData[]): 'up' | 'down' | 'sideways' {
  const validGrowthRates = quarters
    .map(q => q.eps_growth)
    .filter((growth): growth is number => growth !== undefined && growth !== null);
  
  if (validGrowthRates.length < 2) return 'sideways';
  
  let upCount = 0;
  let downCount = 0;
  
  for (let i = 1; i < validGrowthRates.length; i++) {
    if (validGrowthRates[i] > validGrowthRates[i - 1]) {
      upCount++;
    } else if (validGrowthRates[i] < validGrowthRates[i - 1]) {
      downCount++;
    }
  }
  
  if (upCount > downCount) return 'up';
  if (downCount > upCount) return 'down';
  return 'sideways';
}

/**
 * Calculate compound annual growth rate (CAGR)
 */
export function calculateCAGR(startValue: number, endValue: number, periods: number): number | null {
  if (startValue <= 0 || endValue <= 0 || periods <= 0) return null;
  
  return (Math.pow(endValue / startValue, 1 / periods) - 1) * 100;
}

/**
 * Get performance metrics summary
 */
export interface PerformanceMetrics {
  avgGrowth: number | null;
  volatility: number | null;
  riskLevel: 'low' | 'medium' | 'high';
  trend: 'up' | 'down' | 'sideways';
  cagr: number | null;
  consistency: number; // 0-100 score
}

export function getPerformanceMetrics(data: StockFinancialData): PerformanceMetrics {
  const growthRates = data.quarters
    .map(q => q.eps_growth)
    .filter((growth): growth is number => growth !== undefined && growth !== null);
  
  const avgGrowth = growthRates.length > 0 
    ? growthRates.reduce((sum, rate) => sum + rate, 0) / growthRates.length 
    : null;
  
  const volatility = calculateVolatility(data.quarters);
  const riskLevel = getRiskLevel(volatility);
  const trend = getTrendDirection(data.quarters);
  
  // CAGR calculation (if we have enough data)
  let cagr: number | null = null;
  if (data.quarters.length >= 4) {
    const oldestEps = data.quarters[data.quarters.length - 1]?.eps;
    const latestEps = data.quarters[0]?.eps;
    if (oldestEps && latestEps && oldestEps > 0) {
      cagr = calculateCAGR(oldestEps, latestEps, data.quarters.length / 4); // Convert quarters to years
    }
  }
  
  // Consistency score (inverse of volatility, normalized)
  let consistency = 50; // Default medium consistency
  if (volatility !== null) {
    consistency = Math.max(0, Math.min(100, 100 - (volatility * 2)));
  }
  
  return {
    avgGrowth,
    volatility,
    riskLevel,
    trend,
    cagr,
    consistency,
  };
}
