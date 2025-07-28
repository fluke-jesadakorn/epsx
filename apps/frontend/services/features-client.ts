/**
 * Client-side features service
 * This wraps server actions for use in client components
 */

export const featuresClient = {
  async checkFeatureAccess(featureId: string): Promise<{
    allowed: boolean;
    reason?: string;
    requiredTier?: string;
  }> {
    try {
      const response = await fetch('/api/features/check', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ featureId }),
      });

      if (!response.ok) {
        console.error(`Feature check failed: ${response.statusText}`);
        return { allowed: false, reason: 'Feature check failed' };
      }

      return await response.json();
    } catch (error) {
      console.error('Error checking feature access:', error);
      return { allowed: false, reason: 'Feature check failed' };
    }
  },

  async checkRankingAccess(): Promise<{
    allowed: boolean;
    tier?: string;
    reason?: string;
  }> {
    try {
      const response = await fetch('/api/features/ranking-access', {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        console.error(`Ranking access check failed: ${response.statusText}`);
        return { allowed: false, reason: 'Ranking access check failed' };
      }

      return await response.json();
    } catch (error) {
      console.error('Error checking ranking access:', error);
      return { allowed: false, reason: 'Ranking access check failed' };
    }
  },
};