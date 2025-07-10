import * as React from 'react';

import { fetchStockFinancialData } from '@/app/actions/stock';
import { SkeletonLoader } from '@/components/common/Skeleton';

import RankingClient from './RankingClient';

// Define columns for financial data ranking page
export const financialColumns = [
  { key: 'number' as const, header: 'No.' },
  { key: 'symbol' as const, header: 'Symbol' },
  { key: 'latestPrice' as const, header: 'Latest Price' },
  { key: 'latestEps' as const, header: 'Latest EPS' },
  {
    key: 'latestGrowth' as const,
    header: 'EPS Growth %',
    tooltip: 'Latest quarter EPS growth percentage',
  },
  { key: 'latestDate' as const, header: 'Latest Date' },
  {
    key: 'avgGrowth' as const,
    header: 'Avg Growth %',
    tooltip: 'Average EPS growth across all quarters',
  },
  {
    key: 'quarters' as const,
    header: 'Historical Data',
    tooltip: 'View quarterly financial data',
  },
];

async function RankingPage() {
  const initialData = await fetchStockFinancialData();

  return (
    <React.Suspense fallback={<SkeletonLoader />}>
      <RankingClient initialData={initialData} />
    </React.Suspense>
  );
}

export default RankingPage;

// Revalidate page every 5 minutes
export const revalidate = 300;
