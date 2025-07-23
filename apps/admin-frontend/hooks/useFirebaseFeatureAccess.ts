import { useState, useEffect } from 'react';
// import { firebaseIAMService } from '../services/firebaseIAMService'; // Service removed

// Placeholder for removed service
const firebaseIAMService = {
  hasFeatureAccess: async (...args: any[]) => false,
};

/**
 * Hook for checking if the current user has access to a specific feature
 */
export const useFeatureAccess = (userId: string | null, featureId: string) => {
  const [hasAccess, setHasAccess] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkAccess = async () => {
      if (!userId || !featureId) {
        setHasAccess(false);
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);
        
        const access = await firebaseIAMService.hasFeatureAccess(userId, featureId);
        setHasAccess(access);
      } catch (err) {
        console.error('Error checking feature access:', err);
        setError(err instanceof Error ? err.message : 'Unknown error');
        setHasAccess(false);
      } finally {
        setLoading(false);
      }
    };

    checkAccess();
  }, [userId, featureId]);

  return { hasAccess, loading, error };
};
