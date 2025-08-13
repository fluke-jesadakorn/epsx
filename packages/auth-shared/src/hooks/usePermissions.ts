import { useMemo } from 'react'
import { useModernAuth } from './useModernAuth'
import { createUserAbility, type Actions, type Subjects, can, cannot, PermissionChecks, PackagePermissions } from '../permissions/casl-abilities'

/**
 * Modern permission hook using CASL
 * Replaces complex permission checking with simple, flexible abilities
 */
export function usePermissions() {
  const { user, isAuthenticated } = useModernAuth()

  // Create CASL ability based on user's JWT claims
  const ability = useMemo(() => {
    if (!user || !isAuthenticated) {
      // Return empty ability for unauthenticated users
      return createUserAbility({
        permissions: [],
        admin_modules: [],
        package_tier: 'FREE',
        role: 'guest'
      })
    }

    return createUserAbility({
      permissions: user.permissions,
      admin_modules: user.admin_modules,
      package_tier: user.package_tier,
      role: user.role
    })
  }, [user, isAuthenticated])

  /**
   * Check if user can perform action on subject
   */
  const checkPermission = (action: Actions, subject: Subjects): boolean => {
    return can(ability, action, subject)
  }

  /**
   * Check if user cannot perform action on subject
   */
  const checkForbidden = (action: Actions, subject: Subjects): boolean => {
    return cannot(ability, action, subject)
  }

  /**
   * Check multiple permissions (user must have ALL)
   */
  const checkAllPermissions = (checks: Array<[Actions, Subjects]>): boolean => {
    return checks.every(([action, subject]) => can(ability, action, subject))
  }

  /**
   * Check multiple permissions (user must have ANY)
   */
  const checkAnyPermissions = (checks: Array<[Actions, Subjects]>): boolean => {
    return checks.some(([action, subject]) => can(ability, action, subject))
  }

  /**
   * Get all permission rules (for debugging)
   */
  const getAllRules = () => {
    return ability.rules
  }

  return {
    // Core ability object
    ability,
    
    // Permission checking functions
    can: checkPermission,
    cannot: checkForbidden,
    checkAllPermissions,
    checkAnyPermissions,
    
    // Common permission checks
    canReadAnalytics: PermissionChecks.canReadAnalytics(ability),
    canExportData: PermissionChecks.canExportData(ability),
    canManageUsers: PermissionChecks.canManageUsers(ability),
    canAccessAdmin: PermissionChecks.canAccessAdmin(ability),
    canManagePayments: PermissionChecks.canManagePayments(ability),
    canReadStock: PermissionChecks.canReadStock(ability),
    isSystemAdmin: PermissionChecks.isSystemAdmin(ability),
    
    // Package tier checks
    hasPremiumAccess: PackagePermissions.requiresPremium(ability),
    hasAdvancedAccess: PackagePermissions.requiresAdvanced(ability),
    hasEnterpriseAccess: PackagePermissions.requiresEnterprise(ability),
    
    // User info
    user,
    isAuthenticated,
    
    // Debugging
    getAllRules,
  }
}

/**
 * Simple permission hook for basic use cases
 */
export function usePermission(action: Actions, subject: Subjects) {
  const { can } = usePermissions()
  return can(action, subject)
}

/**
 * Admin permission hook
 */
export function useAdminPermissions() {
  const { ability, user, isAuthenticated } = usePermissions()
  
  return {
    ability,
    user,
    isAuthenticated,
    
    // Admin-specific checks
    canManageUsers: can(ability, 'manage', 'User'),
    canManageSystem: can(ability, 'manage', 'System'),
    canManagePayments: can(ability, 'manage', 'Payment'),
    canManageAnalytics: can(ability, 'manage', 'Analytics'),
    canManageModules: can(ability, 'manage', 'Module'),
    
    // Check specific admin modules
    hasUserOperations: user?.admin_modules?.includes('user_operations') || false,
    hasSystemAdmin: user?.admin_modules?.includes('system_admin') || false,
    hasBillingAdmin: user?.admin_modules?.includes('billing_admin') || false,
    hasAnalyticsSpecialist: user?.admin_modules?.includes('analytics_specialist') || false,
    hasModuleCoordinator: user?.admin_modules?.includes('module_coordinator') || false,
  }
}

/**
 * Feature-based permission hook for package tiers
 */
export function useFeaturePermissions() {
  const { ability, user } = usePermissions()
  
  return {
    ability,
    user,
    packageTier: user?.package_tier || 'FREE',
    
    // Feature access based on package tier
    hasBasicFeatures: true, // Everyone has basic features
    hasPremiumFeatures: can(ability, 'read', 'Analytics'),
    hasAdvancedFeatures: can(ability, 'read', 'Stock'),
    hasEnterpriseFeatures: can(ability, 'manage', 'all'),
    
    // Specific feature checks
    canUseAdvancedAnalytics: can(ability, 'export', 'Analytics'),
    canAccessPremiumData: can(ability, 'read', 'Analytics'),
    canExportData: can(ability, 'export', 'Analytics'),
    canManageApiKeys: can(ability, 'manage', 'System'),
    
    // Check if upgrade is needed for feature
    needsUpgradeFor: (feature: 'premium' | 'advanced' | 'enterprise') => {
      switch (feature) {
        case 'premium':
          return !can(ability, 'read', 'Analytics')
        case 'advanced':
          return !can(ability, 'read', 'Stock')
        case 'enterprise':
          return !can(ability, 'manage', 'all')
        default:
          return false
      }
    }
  }
}