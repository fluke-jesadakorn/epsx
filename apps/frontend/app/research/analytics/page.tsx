import { Suspense } from 'react';

import { fetchEpsGrowthRanking } from '@/app/actions/stock';
import { SkeletonLoader } from '@/components/common/Skeleton';
import StockGrowthTable from '@/components/home/StockGrowthTable';

async function AnalyticsPage() {
  const data = await fetchEpsGrowthRanking();

  return (
    <Suspense fallback={<SkeletonLoader />}>
      <StockGrowthTable data={data} />
    </Suspense>
  );
}

export default AnalyticsPage;

// Revalidate page every 5 minutes
export const revalidate = 300;
