import { AnalyticsDashboard } from '@/components/admin/AnalyticsDashboard';

export const dynamic = 'force-dynamic';

export default async function AnalyticsPage() {
  // Fetch analytics data via API route - prevents hydration errors
  const fetchAnalyticsData = async () => {
    try {
      const response = await fetch(`${process.env.NEXTAUTH_URL || 'http://localhost:3001'}/api/v1/admin/analytics/dashboard?dateRange=7d&selectedModule=all`, {
        next: { revalidate: 300 }
      });

      if (!response.ok) {
        return {
          analytics: null,
          systemMetrics: null,
          revenue: null,
          realtime: null
        };
      }

      const result = await response.json();
      
      if (result.success && result.data) {
        return {
          analytics: { data: result.data },
          systemMetrics: { 
            usage: { totalUsers: result.data.metrics.totalUsers },
            performance: { 
              activeSessions: result.data.metrics.totalRequests,
              responseTime: result.data.metrics.averageResponseTime,
              uptime: '99.9%'
            },
            errors: { rate: result.data.metrics.errorRate }
          },
          revenue: { total: result.data.metrics.totalRevenue },
          realtime: { 
            activeUsers: result.data.metrics.totalUsers,
            requests: result.data.metrics.totalRequests
          }
        };
      }

      return {
        analytics: null,
        systemMetrics: null,
        revenue: null,
        realtime: null
      };
    } catch (error) {
      console.error('Failed to fetch analytics data:', error);
      return {
        analytics: null,
        systemMetrics: null,
        revenue: null,
        realtime: null
      };
    }
  };

  const { analytics, systemMetrics, revenue, realtime } = await fetchAnalyticsData();

  return (
    <AnalyticsDashboard 
      initialAnalytics={analytics}
      initialSystemMetrics={systemMetrics}
      initialRevenue={revenue}
      initialRealtime={realtime}
    />
  );
}