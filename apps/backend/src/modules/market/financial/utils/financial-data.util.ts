import { ProcessedFinancialData } from '../types/financial.types';

interface RawFinancialData {
  nodes?: Array<{
    data?: Array<{
      fiscal_quarter?: string | number;
      fiscal_year?: number;
      revenue_growth?: number;
      operating_income?: number;
      interest_expense?: number;
      net_income?: number;
      eps_basic?: number;
      eps_diluted?: number;
      free_cash_flow?: number;
      profit_margin?: number;
      total_operating_expenses?: number;
    }>;
  }>;
}

export function processDynamicFinancialData(
  rawData: RawFinancialData,
): ProcessedFinancialData[] {
  if (!rawData.nodes?.length || !rawData.nodes[0]?.data?.length) {
    return [];
  }

  return rawData.nodes[0].data.reduce<ProcessedFinancialData[]>((acc, curr) => {
    if (!curr.fiscal_quarter || !curr.fiscal_year) {
      return acc;
    }

    // Convert fiscal quarter to number if it's a string (e.g., 'Q1' -> 1)
    const quarterNumber = typeof curr.fiscal_quarter === 'string'
      ? parseInt(curr.fiscal_quarter.replace('Q', ''))
      : curr.fiscal_quarter;

    if (isNaN(quarterNumber) || quarterNumber < 1 || quarterNumber > 4) {
      return acc;
    }

    const processedData: ProcessedFinancialData = {
      fiscalQuarter: quarterNumber,
      fiscalYear: curr.fiscal_year,
      revenueGrowth: curr.revenue_growth,
      operatingIncome: curr.operating_income,
      interestExpense: curr.interest_expense,
      netIncome: curr.net_income,
      epsBasic: curr.eps_basic,
      epsDiluted: curr.eps_diluted,
      freeCashFlow: curr.free_cash_flow,
      profitMargin: curr.profit_margin,
      totalOperatingExpenses: curr.total_operating_expenses,
    };

    acc.push(processedData);
    return acc;
  }, []);
}
