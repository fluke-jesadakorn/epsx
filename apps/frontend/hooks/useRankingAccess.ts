import { useAuth } from '@/context/auth-context';

export function useRankingAccess() {
  const { user } = useAuth();
  
  return {
    canAccessRankings: !!user,
    loading: false
  };
}
