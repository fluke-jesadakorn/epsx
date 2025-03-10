"use server";

import { EpsGrowthRankingResponse } from "@/types/epsGrowthRanking";
import { StockScreenerResponse, TableStockData } from "@/types/stockFetchData";

export async function fetchEpsGrowthRanking({
  limit = 3,
  skip = 0,
}: {
  limit?: number;
  skip?: number;
}): Promise<EpsGrowthRankingResponse> {
  const baseUrl = process.env.NEXT_PUBLIC_API_URL;

  if (!baseUrl) {
    throw new Error(
      "NEXT_PUBLIC_API_URL is not defined in environment variables"
    );
  }

  const response = await fetch(
    `${baseUrl}/market/financials/eps-growth?limit=${limit}&skip=${skip}`,
    {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      next: {
        revalidate: 300, // Cache for 5 minutes
      },
    }
  );

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return response.json();
}

const formatNumber = (num: number | undefined | null): string => {
  if (num === undefined || num === null) return 'N/A';
  return num.toFixed(2);
};

const formatDate = (dateStr: string | undefined | null): string => {
  if (!dateStr) return 'N/A';
  try {
    return new Date(dateStr).toLocaleDateString();
  } catch {
    return 'N/A';
  }
};

export async function fetchStockScreenerData(): Promise<TableStockData[]> {
  const response = await fetch(
    "https://api.stockanalysis.com/api/screener/s/bd/eps+earningsEpsEstimate+epsGrowthQ+epsNextQuarter+epsGrowthQuarters+earningsDate+epsGrowth+country+epsGrowthYears+epsThisYear+epsThisQuarter+epsNextYear+fiscalYearEnd+sector+exchange+lastReportDate+tags+earningsEpsEstimateGrowth+ma20.json",
    {
      next: {
        revalidate: 300, // Cache for 5 minutes
        tags: ['stockData'], // Add a cache tag
      },
      cache: 'no-store' // Disable response caching due to size
    }
  );

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const data: StockScreenerResponse = await response.json();
  
  return Object.entries(data.data.data).map(([symbol, stock]) => ({
    symbol,
    eps: formatNumber(stock.eps),
    epsGrowthQ: formatNumber(stock.epsGrowthQ),
    epsNextQuarter: formatNumber(stock.epsNextQuarter),
    earningsDate: formatDate(stock.earningsDate),
    country: stock.country || 'N/A',
    sector: stock.sector || 'N/A',
    exchange: stock.exchange || 'N/A',
    lastReportDate: formatDate(stock.lastReportDate),
    tags: Array.isArray(stock.tags) ? stock.tags : [],
    earningsEpsEstimateGrowth: formatNumber(stock.earningsEpsEstimateGrowth),
    ma20: formatNumber(stock.ma20)
  }));
}
