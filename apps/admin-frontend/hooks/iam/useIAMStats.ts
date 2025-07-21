import { useState, useEffect } from 'react';
import { iamService } from '../../services/iamService';

interface IAMStats {
  totalUsers: number;
  activeSubscriptions: number;
  permissionTemplates: number;
  userGrowth: { value: number; isPositive: boolean };
  subscriptionGrowth: { value: number; isPositive: boolean };
  templateGrowth: { value: number; isPositive: boolean };
}

export const useIAMStats = () => {
  const [stats, setStats] = useState<IAMStats>({
    totalUsers: 0,
    activeSubscriptions: 0,
    permissionTemplates: 0,
    userGrowth: { value: 0, isPositive: true },
    subscriptionGrowth: { value: 0, isPositive: true },
    templateGrowth: { value: 0, isPositive: true }
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        setLoading(true);
        
        // Get basic stats from existing IAM service
        const users = await iamService.getUsers();
        
        // Calculate stats
        const totalUsers = users.length;
        const activeSubscriptions = users.filter(u => u.subscriptionStatus === 'active').length;
        
        // Mock growth data - in real implementation, this would come from analytics
        const statsData: IAMStats = {
          totalUsers,
          activeSubscriptions,
          permissionTemplates: 5, // Mock value
          userGrowth: { value: 12, isPositive: true },
          subscriptionGrowth: { value: 8, isPositive: true },
          templateGrowth: { value: 3, isPositive: true }
        };
        
        setStats(statsData);
      } catch (error) {
        console.error('Failed to fetch IAM stats:', error);
        // Set fallback values
        setStats({
          totalUsers: 0,
          activeSubscriptions: 0,
          permissionTemplates: 0,
          userGrowth: { value: 0, isPositive: true },
          subscriptionGrowth: { value: 0, isPositive: true },
          templateGrowth: { value: 0, isPositive: true }
        });
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, []);

  return { stats, loading };
};
