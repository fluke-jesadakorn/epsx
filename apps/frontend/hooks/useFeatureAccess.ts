import { useAuth } from '@/lib/auth';

export function useFeatureAccess() {
  const { user } = useAuth();
  
  return {
    canAccessTrading: !!user,
    canAccessRankings: !!user,
    canAccessAnalytics: !!user,
    loading: false
  };
}
