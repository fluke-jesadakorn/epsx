/**
 * Feature Flags Configuration
 * Controls the gradual rollout of new unified admin interface features
 */

export const FEATURE_FLAGS = {
  // Unified User Management Hub
  UNIFIED_USER_MANAGEMENT: process.env.NEXT_PUBLIC_ENABLE_UNIFIED_USERS === 'true',
  
  // Server Components Migration
  SERVER_COMPONENTS: process.env.NEXT_PUBLIC_ENABLE_SERVER_COMPONENTS === 'true',
  
  // New Navigation and URL Structure  
  NEW_NAVIGATION: process.env.NEXT_PUBLIC_ENABLE_NEW_NAV === 'true',
  
  // Performance Optimizations
  BUNDLE_OPTIMIZATION: process.env.NEXT_PUBLIC_ENABLE_BUNDLE_OPT === 'true',
  
  // Development and Testing
  DEV_MODE: process.env.NODE_ENV === 'development',
} as const

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
  const rollouts: Record<string, number> = {
    'unified_user_management': parseInt(process.env.NEXT_PUBLIC_ROLLOUT_UNIFIED_USERS || '0'),
    'server_components': parseInt(process.env.NEXT_PUBLIC_ROLLOUT_SERVER_COMPONENTS || '0'),
    'new_navigation': parseInt(process.env.NEXT_PUBLIC_ROLLOUT_NEW_NAV || '0'),
  }
  
  return rollouts[featureName] || 0
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