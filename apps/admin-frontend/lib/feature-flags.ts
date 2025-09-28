/**
 * ADMIN FRONTEND - FEATURE FLAGS COMPATIBILITY LAYER
 * Migrated to use consolidated shared/config/feature-flags.ts
 * Provides backward compatibility for existing admin components
 */

import { 
  isFeatureEnabled as sharedIsFeatureEnabled,
  createFeatureContext,
  featureFlags,
  FEATURE_FLAGS as SHARED_FEATURE_FLAGS
} from '../../../../shared/config/feature-flags';
import { featureFlags as legacyFeatureFlags } from '@/config/env';

// Admin-specific feature flag context helper
function createAdminContext(userId?: string, userPermissions?: string[]) {
  return createFeatureContext(userId, userPermissions, true); // isAdmin = true
}

/**
 * Check if a feature flag is enabled (consolidated version)
 * @param flag - The feature flag to check from consolidated system
 * @param userId - Optional user ID for rollout percentage
 * @param userPermissions - Optional user permissions for permission-based flags
 */
export function isFeatureEnabled(
  flag: keyof typeof SHARED_FEATURE_FLAGS, 
  userId?: string,
  userPermissions?: string[]
): boolean {
  const context = createAdminContext(userId, userPermissions);
  return sharedIsFeatureEnabled(flag, context);
}

/**
 * Legacy feature flags for backward compatibility
 * Gradually migrate components to use consolidated feature flags
 */
export const LEGACY_FEATURE_FLAGS = legacyFeatureFlags;

/**
 * Consolidated feature flags (new system)
 */
export const FEATURE_FLAGS = SHARED_FEATURE_FLAGS;

/**
 * Get user-specific feature rollout (for gradual rollout)
 * @param userId - The user's ID
 * @param featureName - The feature to check
 * @returns boolean indicating if the feature should be shown to this user
 */
export function shouldShowFeatureToUser(userId: string, featureName: string): boolean {
  // Use consolidated feature flag system
  if (featureName in SHARED_FEATURE_FLAGS) {
    return isFeatureEnabled(featureName as keyof typeof SHARED_FEATURE_FLAGS, userId);
  }
  
  // Fallback to legacy system for unmigrated features
  if (LEGACY_FEATURE_FLAGS.DEV_MODE) {
    return true;
  }
  
  // Simple hash-based rollout (deterministic based on user ID)
  const hash = hashString(userId + featureName);
  const rolloutPercentage = getRolloutPercentage(featureName);
  
  return (hash % 100) < rolloutPercentage;
}

/**
 * Get the rollout percentage for a specific feature
 * Can be configured per feature for gradual rollout
 */
function getRolloutPercentage(featureName: string): number {
  return (LEGACY_FEATURE_FLAGS as any).rolloutPercentages?.[featureName] || 0;
}

/**
 * Simple string hash function for deterministic user-based rollouts
 */
function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash);
}

/**
 * Feature flag hook for React components (consolidated version)
 * Usage: const isEnabled = useFeatureFlag('UNIFIED_USER_MANAGEMENT', userId, userPermissions)
 */
export function useFeatureFlag(
  flag: keyof typeof SHARED_FEATURE_FLAGS,
  userId?: string,
  userPermissions?: string[]
) {
  return isFeatureEnabled(flag, userId, userPermissions);
}


// Export all consolidated feature flag utilities for admin use
export {
  featureFlags,
  createFeatureContext,
  getAllFeatureFlags,
  getFeatureConfig,
  canToggleFeature
} from '../../../../shared/config/feature-flags';