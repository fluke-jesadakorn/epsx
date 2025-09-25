/**
 * FRONTEND - FEATURE FLAGS COMPATIBILITY LAYER
 * Migrated to use consolidated shared/config/feature-flags.ts
 * Provides backward compatibility for existing frontend components
 */

import { 
  isFeatureEnabled as sharedIsFeatureEnabled,
  createFeatureContext,
  featureFlags,
  FEATURE_FLAGS as SHARED_FEATURE_FLAGS
} from '../../../shared/config/feature-flags';

// User-specific feature flag context helper
function createUserContext(userId?: string, userPermissions?: string[]) {
  return createFeatureContext(userId, userPermissions, false); // isAdmin = false for users
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
  const context = createUserContext(userId, userPermissions);
  return sharedIsFeatureEnabled(flag, context);
}

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
  
  // If feature not found in consolidated system, default to false
  return false;
}

/**
 * Feature flag hook for React components (consolidated version)
 * Usage: const isEnabled = useFeatureFlag('WEB3_AUTHENTICATION', userId, userPermissions)
 */
export function useFeatureFlag(
  flag: keyof typeof SHARED_FEATURE_FLAGS,
  userId?: string,
  userPermissions?: string[]
) {
  return isFeatureEnabled(flag, userId, userPermissions);
}

/**
 * Frontend-specific feature flag helpers
 */

/**
 * Check if Web3 authentication is enabled
 */
export function isWeb3AuthEnabled(userId?: string, userPermissions?: string[]): boolean {
  return isFeatureEnabled('WEB3_AUTHENTICATION', userId, userPermissions);
}

/**
 * Check if real-time analytics is enabled
 */
export function isRealTimeAnalyticsEnabled(userId?: string, userPermissions?: string[]): boolean {
  return isFeatureEnabled('REAL_TIME_ANALYTICS', userId, userPermissions);
}

/**
 * Check if advanced charting is enabled
 */
export function isAdvancedChartingEnabled(userId?: string, userPermissions?: string[]): boolean {
  return isFeatureEnabled('ADVANCED_CHARTING', userId, userPermissions);
}

/**
 * Check if portfolio tracking is enabled
 */
export function isPortfolioTrackingEnabled(userId?: string, userPermissions?: string[]): boolean {
  return isFeatureEnabled('PORTFOLIO_TRACKING', userId, userPermissions);
}

/**
 * Check if crypto payments are enabled
 */
export function isCryptoPaymentsEnabled(userId?: string, userPermissions?: string[]): boolean {
  return isFeatureEnabled('CRYPTO_PAYMENTS', userId, userPermissions);
}

/**
 * Check if progressive auth UI is enabled
 */
export function isProgressiveAuthUIEnabled(userId?: string, userPermissions?: string[]): boolean {
  return isFeatureEnabled('PROGRESSIVE_AUTH_UI', userId, userPermissions);
}

// Export all consolidated feature flag utilities for frontend use
export {
  featureFlags,
  createFeatureContext,
  getAllFeatureFlags,
  getFeatureConfig,
  getEnabledFeatures
} from '../../../shared/config/feature-flags';

// Legacy compatibility object
export const legacyFeatureFlags = {
  // Map old feature names to new consolidated flags
  isWeb3AuthEnabled,
  isRealTimeAnalyticsEnabled,
  isAdvancedChartingEnabled,
  isPortfolioTrackingEnabled,
  isCryptoPaymentsEnabled,
  isProgressiveAuthUIEnabled,
} as const;