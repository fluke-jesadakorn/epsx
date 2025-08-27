import AnalyticsClientWrapper from '@/components/analytics/AnalyticsClientWrapper';
import { getFilterOptions, getInitialAnalyticsData } from '@/components/analytics/AnalyticsServerData';
import type { Metadata } from 'next';

// Server-side rendering for better performance
export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'EPS Rankings - EPSX Analytics',
  description:
    'Server-side EPS growth analysis and stock performance rankings with advanced filtering',
};

export default async function AnalyticsPage() {
  // Fetch initial data on the server
  const [filterOptions, initialData] = await Promise.all([
    getFilterOptions(),
    getInitialAnalyticsData({
      page: 1,
      limit: 20,
      sort_by: 'growth_factor',
    }),
  ]);

  return (
    <AnalyticsClientWrapper 
      initialData={initialData}
      filterOptions={filterOptions}
    />
  );
}
