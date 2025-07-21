import { useAuth } from '@/context/shared-auth-provider';

export function useRankingAccess() {
  const { user } = useAuth();
  
  return {
    canAccessRankings: !!user,
    loading: false
  };
}
