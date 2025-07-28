import { useState, useEffect } from 'react';
import { useAuth } from '@/context/auth-context';

export function useFeatureAccess() {
  const { user } = useAuth();
  const [loading, setLoading] = useState(true);
  const [featureAccess, setFeatureAccess] = useState({
    canAccessTrading: false,
    canAccessRankings: false,
    canAccessAnalytics: false,
    userTier: 'BRONZE'
  });
  
  useEffect(() => {
    const loadFeatureAccess = async () => {
      if (!user) {
        setFeatureAccess({
          canAccessTrading: false,
          canAccessRankings: false,
          canAccessAnalytics: false,
          userTier: 'BRONZE'
        });
        setLoading(false);
        return;
      }

      try {
        // Use server actions for feature access
        const { checkFeatureAccess, checkRankingAccess } = await import('@epsx/server-actions');
        const [tradingAccess, rankingAccess, analyticsAccess] = await Promise.all([
          checkFeatureAccess('trading'),
          checkRankingAccess(),
          checkFeatureAccess('analytics')
        ]);

        setFeatureAccess({
          canAccessTrading: tradingAccess?.allowed || false,
          canAccessRankings: rankingAccess?.allowed || false,
          canAccessAnalytics: analyticsAccess?.allowed || false,
          userTier: rankingAccess?.tier || 'BRONZE'
        });
      } catch (error) {
        console.error('Error loading feature access:', error);
        setFeatureAccess({
          canAccessTrading: !!user,
          canAccessRankings: !!user,
          canAccessAnalytics: !!user,
          userTier: 'BRONZE'
        });
      } finally {
        setLoading(false);
      }
    };

    loadFeatureAccess();
  }, [user]);
  
  return {
    ...featureAccess,
    loading
  };
}
