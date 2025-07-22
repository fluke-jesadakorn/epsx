import { useAuth } from '@/context/shared-auth-provider';

export function useFeatureAccess() {
  const { user } = useAuth();
  
  return {
    canAccessTrading: !!user,
    canAccessRankings: !!user,
    canAccessAnalytics: !!user,
    loading: false
  };
}
