import { useState, useEffect } from 'react';
import { iamService } from '../services/iamService';

export const useFeatureAccess = (featureId: string, userId?: string) => {
  const [hasAccess, setHasAccess] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkAccess = async () => {
      if (!userId) {
        setLoading(false);
        setHasAccess(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);
        const access = await iamService.hasFeatureAccess(userId, featureId);
        setHasAccess(access);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to check feature access');
        setHasAccess(false);
      } finally {
        setLoading(false);
      }
    };

    checkAccess();
  }, [featureId, userId]);

  return { hasAccess, loading, error };
};

// Hook for multiple feature access checks
export const useMultipleFeatureAccess = (featureIds: string[], userId?: string) => {
  const [accessMap, setAccessMap] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkMultipleAccess = async () => {
      if (!userId || featureIds.length === 0) {
        setLoading(false);
        setAccessMap({});
        return;
      }

      try {
        setLoading(true);
        setError(null);
        
        // Check access for each feature
        const results = await Promise.all(
          featureIds.map(async (featureId) => {
            const access = await iamService.hasFeatureAccess(userId, featureId);
            return { featureId, access };
          })
        );

        // Build access map
        const newAccessMap: Record<string, boolean> = {};
        results.forEach(({ featureId, access }) => {
          newAccessMap[featureId] = access;
        });

        setAccessMap(newAccessMap);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to check feature access');
        setAccessMap({});
      } finally {
        setLoading(false);
      }
    };

    checkMultipleAccess();
  }, [featureIds.join(','), userId]);

  return { accessMap, loading, error };
};
