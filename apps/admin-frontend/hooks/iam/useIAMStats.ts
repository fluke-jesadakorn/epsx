import { useEffect, useState } from 'react';

interface IAMStats {
  totalUsers: number;
  activeSubscriptions: number;
  permissionProfiles: number;
  userGrowth: { value: number; isPositive: boolean };
  subscriptionGrowth: { value: number; isPositive: boolean };
  permissionProfileGrowth: { value: number; isPositive: boolean };
}

export const useIAMStats = () => {
  const [stats, setStats] = useState<IAMStats>({
    totalUsers: 0,
    activeSubscriptions: 0,
    permissionProfiles: 0,
    userGrowth: { value: 0, isPositive: true },
    subscriptionGrowth: { value: 0, isPositive: true },
    permissionProfileGrowth: { value: 0, isPositive: true },
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        setLoading(true);

        // TODO: Implement with shared API client
        // Stub data for now
        const analyticsData = {
          total_users: 0,
          active_users: 0,
          user_growth: 0,
          package_distribution: {}
        };

        // Transform backend analytics to IAM stats format
        const statsData: IAMStats = {
          totalUsers: analyticsData.total_users,
          activeSubscriptions: analyticsData.active_users,
          permissionProfiles: Object.keys(analyticsData.package_distribution).length,
          userGrowth: {
            value: Math.abs(analyticsData.user_growth),
            isPositive: analyticsData.user_growth >= 0
          },
          subscriptionGrowth: {
            value: Math.abs(analyticsData.active_users - analyticsData.total_users * 0.8),
            isPositive: analyticsData.active_users >= analyticsData.total_users * 0.8
          },
          permissionProfileGrowth: { value: 3, isPositive: true },
        };

        setStats(statsData);
      } catch (_error) {
        // eslint-disable-next-line no-console
        console.error('Failed to fetch IAM stats', {
          error: _error instanceof Error ? _error.message : String(_error),
        });
        // Set fallback values
        setStats({
          totalUsers: 0,
          activeSubscriptions: 0,
          permissionProfiles: 0,
          userGrowth: { value: 0, isPositive: true },
          subscriptionGrowth: { value: 0, isPositive: true },
          permissionProfileGrowth: { value: 0, isPositive: true },
        });
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, []);

  return { stats, loading };
};
