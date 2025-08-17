import { AnalyticsDashboard } from '@/components/admin/AnalyticsDashboard';

// TODO: Replace with direct API calls
// import { 
//   getAnalyticsData, 
//   getUserAnalytics as _getUserAnalytics, 
//   getSystemMetrics, 
//   getRevenueAnalytics,
//   getRealtimeMetrics 
// } from '@epsx/server-actions';

// Temporary placeholder functions for migration
const getAnalyticsData = async () => ({ data: [] });
const _getUserAnalytics = async () => ({ users: [] });
const getSystemMetrics = async () => ({ metrics: {} });
const getRevenueAnalytics = async () => ({ revenue: 0 });
const getRealtimeMetrics = async () => ({ active: 0 });

export const dynamic = 'force-dynamic';

export default async function AnalyticsPage() {
  // Fetch analytics data server-side
  const [
    analyticsResult, 
    systemMetricsResult, 
    revenueResult, 
    realtimeResult
  ] = await Promise.allSettled([
    getAnalyticsData(),
    getSystemMetrics(),
    getRevenueAnalytics(),
    getRealtimeMetrics()
  ]);

  const analytics = analyticsResult.status === 'fulfilled' ? analyticsResult.value : null;
  const systemMetrics = systemMetricsResult.status === 'fulfilled' ? systemMetricsResult.value : null;
  const revenue = revenueResult.status === 'fulfilled' ? revenueResult.value : null;
  const realtime = realtimeResult.status === 'fulfilled' ? realtimeResult.value : null;

  return (
    <AnalyticsDashboard 
      initialAnalytics={analytics}
      initialSystemMetrics={systemMetrics}
      initialRevenue={revenue}
      initialRealtime={realtime}
    />
  );
}
