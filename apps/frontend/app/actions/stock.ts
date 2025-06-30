'use server';

import type { TableDataMetrics } from '@/types/stockFetchData';

const getBackendUrl = () => {
  return 'http://localhost:3000';
};

export async function fetchStockScreenerData(): Promise<TableDataMetrics[]> {
  const response = await fetch(`${getBackendUrl()}/api/v1/stock/screener`, {
    headers: {
      'Content-Type': 'application/json',
    },
    next: {
      revalidate: 300, // Cache for 5 minutes
      tags: ['stockData'],
    },
  });

  if (!response.ok) {
    const text = await response.text();
    console.error('Stock screener fetch failed:', response.status, text);
    throw new Error(`Stock screener fetch failed: ${response.status}`);
  }

  const data = await response.json();
  // console.log('Fetched stock screener data:', data);
  return data;
}

export async function fetchEpsGrowthRanking(params: {
  limit?: number;
  skip?: number;
  sortBy?: 'growthIndicator' | 'activityScore';
}): Promise<{ data: TableDataMetrics[] }> {
  const searchParams = new URLSearchParams();
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.skip) searchParams.set('skip', params.skip.toString());
  if (params.sortBy) searchParams.set('sort_by', params.sortBy);

  const response = await fetch(
    `${getBackendUrl()}/v1/stock/eps-growth-ranking?${searchParams}`,
    {
      headers: {
        'Content-Type': 'application/json',
      },
      next: {
        revalidate: 300, // Cache for 5 minutes
        tags: ['stockData'],
      },
    },
  );

  if (!response.ok) {
    throw new Error(`EPS growth ranking fetch failed: ${response.status}`);
  }

  const data = await response.json();
  return { data };
}
