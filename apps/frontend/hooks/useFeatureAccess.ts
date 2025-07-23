import { useAuth } from '@/context/auth-context';

export function useFeatureAccess() {
  const { user } = useAuth();
  
  return {
    canAccessTrading: !!user,
    canAccessRankings: !!user,
    canAccessAnalytics: !!user,
    userTier: 'BRONZE', // Default tier for now - TODO: fetch from backend
    loading: false
  };
}
