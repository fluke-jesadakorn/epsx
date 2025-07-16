'use server';

// Refactored to use shared stock service
import type { StockFinancialData } from '@/types/financialChartData';
import { getStockFinancialData } from '@/lib/services/stock.service';

export async function fetchStockFinancialData(
  skip = 0,
  limit = 10,
  country?: any,
  quarters = 2,
): Promise<StockFinancialData[]> {
  return getStockFinancialData(skip, limit, country, quarters);
}

// Keep legacy function for backward compatibility during transition
export async function fetchStockScreenerData(): Promise<StockFinancialData[]> {
  return fetchStockFinancialData();
}
