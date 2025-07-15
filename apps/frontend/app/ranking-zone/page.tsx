import * as React from 'react';

import { fetchStockRankingData } from '@/app/actions/stockRanking';
import { SkeletonLoader } from '@/components/common/Skeleton';
import StockRankingClient from '@/components/shared/StockRankingClient';

/**
 * Example page showing how to use the copied table from /analytics
 * Uses the same API for caching support
 * Only difference is rank shift (as mentioned in requirements)
 */
async function StockRankingZonePage() {
  // Use the same data fetching as /analytics page
  const initialData = await fetchStockRankingData();

  return (
    <React.Suspense fallback={<SkeletonLoader />}>
      <StockRankingClient 
        initialData={initialData}
        title="📊 Alternative Stock Rankings 🚀"
        subtitle="The same comprehensive analytics with a different perspective"
        rankShift={0} // Future: can be adjusted for different ranking views
        showRank={true}
      />
    </React.Suspense>
  );
}

export default StockRankingZonePage;

// Same revalidation as analytics page for consistency
export const revalidate = 300;
