import { useAuth } from '@/lib/auth';

export function useRankingAccess() {
  const { user } = useAuth();
  
  return {
    canAccessRankings: !!user,
    loading: false
  };
}
