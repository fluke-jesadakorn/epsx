import * as React from 'react';

import { fetchStockRankingData } from '@/app/actions/stockRanking';
import { SkeletonLoader } from '@/components/common/Skeleton';
import StockRankingClient from '@/components/shared/StockRankingClient';

/**
 * Role-based ranking page showing stocks based on user subscription level
 * Uses the same API for caching support with role-based access control
 */
async function StockRankingZonePage() {
  // Fetch more data for premium users (filtering happens on client based on user role)
  const initialData = await fetchStockRankingData(0, 100);

  return (
    <React.Suspense fallback={<SkeletonLoader />}>
      <StockRankingClient 
        initialData={initialData}
        title="📊 Role-Based Data Rankings 🚀"
        subtitle="Your personalized view based on your subscription level"
        rankShift={0}
        showRank={true}
      />
    </React.Suspense>
  );
}

export default StockRankingZonePage;

// Same revalidation as analytics page for consistency
export const revalidate = 300;
