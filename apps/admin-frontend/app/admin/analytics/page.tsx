import { AnalyticsDashboard } from '@/components/admin/AnalyticsDashboard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { 
  getAnalyticsData, 
  getUserAnalytics as _getUserAnalytics, 
  getSystemMetrics, 
  getRevenueAnalytics,
  getRealtimeMetrics 
} from '@epsx/server-actions';

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
    <AdminLayout>
      <AnalyticsDashboard 
        initialAnalytics={analytics}
        initialSystemMetrics={systemMetrics}
        initialRevenue={revenue}
        initialRealtime={realtime}
      />
    </AdminLayout>
  );
}
