// ============================================================================
// FRONTEND PERMISSION TYPES
// ============================================================================
// Frontend-specific permission types that extend shared permission system

// Re-export all shared types
export * from '@/shared/permissions/types'

// Frontend-specific extensions
import { EnhancedUserClaims, UserClaims } from '@/shared/permissions/types'

export interface FrontendUserClaims extends EnhancedUserClaims {
  // Frontend-specific claim extensions
  user_level?: string
  subscription_tier?: string
  last_analytics_access?: number
  preferred_theme?: 'light' | 'dark' | 'auto'
}

export interface FrontendPermissionContext {
  userClaims: FrontendUserClaims | null
  legacyUserClaims: UserClaims | null // For backward compatibility
  canViewAnalytics: boolean
  canExportData: boolean
  canAccessRealtime: boolean
  canManageProfile: boolean
  loading: boolean
  error: any
  refreshPermissions: () => Promise<void>
}

export interface FrontendGuardOptions {
  showUpgradePrompt?: boolean
  trackAnalytics?: boolean
  fallbackToTier?: boolean
  enableLegacySupport?: boolean
}