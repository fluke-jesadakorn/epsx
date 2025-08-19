/**
 * Feature Flags Configuration
 * Controls the gradual rollout of new unified admin interface features
 */

import { featureFlags } from '@/config/env';

export const FEATURE_FLAGS = featureFlags;

/**
 * Check if a feature flag is enabled
 * @param flag - The feature flag to check
 * @returns boolean indicating if the feature is enabled
 */
export function isFeatureEnabled(flag: keyof typeof FEATURE_FLAGS): boolean {
  return FEATURE_FLAGS[flag] || false
}

/**
 * Get user-specific feature rollout (for gradual rollout)
 * @param userId - The user's ID
 * @param featureName - The feature to check
 * @returns boolean indicating if the feature should be shown to this user
 */
export function shouldShowFeatureToUser(userId: string, featureName: string): boolean {
  // Development mode: show all features
  if (FEATURE_FLAGS.DEV_MODE) {
    return true
  }
  
  // Simple hash-based rollout (deterministic based on user ID)
  const hash = hashString(userId + featureName)
  const rolloutPercentage = getRolloutPercentage(featureName)
  
  return (hash % 100) < rolloutPercentage
}

/**
 * Get the rollout percentage for a specific feature
 * Can be configured per feature for gradual rollout
 */
function getRolloutPercentage(featureName: string): number {
  return FEATURE_FLAGS.rolloutPercentages[featureName as keyof typeof FEATURE_FLAGS.rolloutPercentages] || 0;
}

/**
 * Simple string hash function for deterministic user-based rollouts
 */
function hashString(str: string): number {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // Convert to 32-bit integer
  }
  return Math.abs(hash)
}

/**
 * Feature flag hook for React components
 * Usage: const isEnabled = useFeatureFlag('UNIFIED_USER_MANAGEMENT')
 */
export function useFeatureFlag(flag: keyof typeof FEATURE_FLAGS) {
  return FEATURE_FLAGS[flag]
}