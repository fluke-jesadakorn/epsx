
import {
  getApiKeysAction,
  getDeveloperPortalStatsAction,
  getPermissionAnalyticsAction,
  getSystemMetricsAction,
  getUserStatsAction
} from './actions';
import AnalyticsDashboard from './analytics-dashboard';

/**
 * Analytics Page (Server Component)
 * Fetches initial data server-side and hydrates the client dashboard
 */
export default async function AnalyticsPage() {
  // Fetch all initial data in parallel
  const [
    userStats,
    permissionAnalytics,
    systemMetrics,
    developerPortalStats,
    apiKeys
  ] = await Promise.all([
    getUserStatsAction(),
    getPermissionAnalyticsAction(),
    getSystemMetricsAction(),
    getDeveloperPortalStatsAction(),
    getApiKeysAction()
  ]);

  // Construct initial data object for SWR fallback
  const initialData = {
    'user-stats': userStats,
    'permission-analytics': permissionAnalytics,
    'system-metrics': systemMetrics,
    'developer-portal-stats': developerPortalStats,
    'api-keys': apiKeys
  };

  return <AnalyticsDashboard initialData={initialData} />;
}

// Force dynamic rendering since analytics data changes frequently
export const dynamic = 'force-dynamic';
