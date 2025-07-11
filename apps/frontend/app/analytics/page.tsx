import * as React from 'react';

import { fetchStockFinancialData } from '@/app/actions/stock';
import { SkeletonLoader } from '@/components/common/Skeleton';

import RankingClient from './RankingClient';

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
