/**
 * Client-side features service
 * This service now uses server actions directly instead of API routes
 */

import { checkFeatureAccess, checkRankingAccess } from '@epsx/server-actions';

export const featuresClient = {
  async checkFeatureAccess(featureId: string): Promise<{
    allowed: boolean;
    reason?: string;
    requiredTier?: string;
  }> {
    try {
      return await checkFeatureAccess(featureId);
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
      return await checkRankingAccess();
    } catch (error) {
      console.error('Error checking ranking access:', error);
      return { allowed: false, reason: 'Ranking access check failed' };
    }
  },
};