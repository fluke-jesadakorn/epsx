import type { StockFinancialData } from '@/types/financialChartData';
import type { FinancialsFromChart } from '@/utils/getFinancialsFromChart/getPriceAndEps';

/**
 * Transforms financial chart data from the rankingStocks utility format
 * to the new StockFinancialData format for the analytics page
 */
export function transformFinancialData(
  chartData: Record<string, FinancialsFromChart[]>,
): StockFinancialData[] {
  return Object.entries(chartData).map(([symbol, quarters]) => {
    const mappedQuarters = quarters.map((q) => ({
      price: q.price,
      date: q.date,
      eps: q.eps,
      quarter: q.quarter,
      eps_growth: (q as any).eps_growth, // Cast to any since eps_growth is added dynamically
    }));

    // Fix: If latest quarter price is null, use average of previous 4 quarters' prices
    if (
      mappedQuarters.length > 1 &&
      (mappedQuarters[0].price === null || mappedQuarters[0].price === undefined)
    ) {
      const previousPrices = mappedQuarters
        .slice(1, 5)
        .map((q) => q.price)
        .filter((p) => p !== null && p !== undefined) as number[];
      if (previousPrices.length > 0) {
        const avgPrice =
          previousPrices.reduce((sum, p) => sum + p, 0) / previousPrices.length;
        mappedQuarters[0].price = avgPrice;
      }
    }

    return {
      symbol,
      quarters: mappedQuarters,
    };
  });
}

/**
 * Extracts the latest quarter data for a stock
 */
export function getLatestQuarterData(stock: StockFinancialData) {
  return stock.quarters[0]; // First quarter is the latest due to sorting
}

/**
 * Gets the latest quarter that has eps_growth data
 */
export function getLatestQuarterWithGrowth(stock: StockFinancialData) {
  return (
    stock.quarters.find(
      (q) => q.eps_growth !== undefined && q.eps_growth !== null,
    ) || stock.quarters[0]
  );
}

/**
 * Calculates average EPS growth for a stock
 */
export function calculateAverageEpsGrowth(
  stock: StockFinancialData,
): number | null {
  const growthValues = stock.quarters
    .map((q) => q.eps_growth)
    .filter((growth) => growth !== undefined && growth !== null) as number[];

  if (growthValues.length === 0) return null;

  const sum = growthValues.reduce((acc, val) => acc + val, 0);
  return Math.round(sum / growthValues.length);
}

/**
 * Formats price for display
 */
export function formatPrice(price: number | null): string {
  if (price === null) return 'N/A';

  // Format with appropriate decimal places based on price magnitude
  if (price >= 1000) {
    return price.toLocaleString('en-US', { maximumFractionDigits: 2 });
  } else if (price >= 1) {
    return price.toFixed(2);
  } else {
    return price.toFixed(4);
  }
}

/**
 * Formats EPS growth percentage
 */
export function formatEpsGrowth(growth: number | null | undefined): string {
  if (growth === null || growth === undefined) return 'N/A';
  return `${growth > 0 ? '+' : ''}${growth}%`;
}

/**
 * Formats date for display
 */
export function formatDate(date: string): string {
  return new Date(date).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}
