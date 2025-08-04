import { useAuth } from './useAuth';

/**
 * Hook for managing ranking feature access
 */
export function useRankingAccess() {
  const { user, loading } = useAuth();

  const hasRankingAccess = user?.permissions?.includes('ranking:read') || false;
  
  return {
    hasAccess: hasRankingAccess,
    loading,
    user
  };
}