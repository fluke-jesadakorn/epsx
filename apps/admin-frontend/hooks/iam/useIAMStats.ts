import { useEffect, useState } from 'react';
import { adminLogger } from '../../lib/logger';
import { getIAMUsers } from '@epsx/server-actions';

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

        // Get basic stats from server action
        const users = await getIAMUsers();

        // Calculate stats
        const totalUsers = users.length;
        const activeSubscriptions = users.filter(
          u => u.subscriptionStatus === 'active'
        ).length;

        // Mock growth data - in real implementation, this would come from analytics
        const statsData: IAMStats = {
          totalUsers,
          activeSubscriptions,
          permissionProfiles: 5, // Mock value
          userGrowth: { value: 12, isPositive: true },
          subscriptionGrowth: { value: 8, isPositive: true },
          permissionProfileGrowth: { value: 3, isPositive: true },
        };

        setStats(statsData);
      } catch (error) {
        adminLogger.error('Failed to fetch IAM stats', {
          error: error instanceof Error ? error.message : String(error),
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
