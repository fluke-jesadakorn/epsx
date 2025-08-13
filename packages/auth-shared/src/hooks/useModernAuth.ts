import { useSession } from "next-auth/react"

/**
 * Extended session user type with EPSX-specific properties
 */
export interface EPSXUser {
  id?: string
  email?: string | null
  name?: string | null
  image?: string | null
  admin_modules: string[]
  permissions: string[]
  package_tier: string
  role: string
  firebase_uid?: string
  isAdmin: boolean
  hasPermission: (permission: string) => boolean
  hasAdminModule: (module: string) => boolean
}

/**
 * Modern Auth hook that replaces all complex client-side auth logic
 * Uses Auth.js v5 session with JWT-based permissions
 */
export function useModernAuth(): {
  user: EPSXUser | null
  session: any
  status: string
  isLoading: boolean
  isAuthenticated: boolean
  hasPermission: (permission: string) => boolean
  hasAdminModule: (module: string) => boolean
  hasPackageTier: (tier: string) => boolean
  hasRole: (role: string) => boolean
  isAdmin: () => boolean
  isSystemAdmin: () => boolean
  canAccessFeature: (feature: any) => boolean
  getPermissions: () => string[]
  getAdminModules: () => string[]
  updateSession: any
} {
  const { data: session, status, update } = useSession()
  
  // Type-safe user data with permissions
  const user: EPSXUser | null = session?.user ? {
    id: session.user.id,
    email: session.user.email,
    name: session.user.name,
    image: session.user.image,
    admin_modules: (session.user as any).admin_modules || [],
    permissions: (session.user as any).permissions || [],
    package_tier: (session.user as any).package_tier || 'FREE',
    role: (session.user as any).role || 'user',
    firebase_uid: (session.user as any).firebase_uid,
    isAdmin: ((session.user as any).admin_modules?.length || 0) > 0,
    hasPermission: (permission: string) => 
      ((session.user as any).permissions || []).includes(permission),
    hasAdminModule: (module: string) =>
      ((session.user as any).admin_modules || []).includes(module),
  } : null

  /**
   * Check if user has specific permission
   */
  const hasPermission = (permission: string): boolean => {
    if (!user?.permissions) return false
    
    // Check exact match
    if (user.permissions.includes(permission)) {
      return true
    }
    
    // Check wildcard permissions
    return user.permissions.some(userPermission => {
      if (userPermission.endsWith('.*') || userPermission.endsWith(':*')) {
        const prefix = userPermission.slice(0, -2)
        return permission.startsWith(prefix + '.') || permission.startsWith(prefix + ':')
      }
      if (userPermission === '*') {
        return true
      }
      return false
    })
  }

  /**
   * Check if user has specific admin module
   */
  const hasAdminModule = (module: string): boolean => {
    return user?.admin_modules?.includes(module) || false
  }

  /**
   * Check if user has package tier or higher
   */
  const hasPackageTier = (requiredTier: string): boolean => {
    if (!user?.package_tier) return false
    
    const tierHierarchy: Record<string, number> = {
      'FREE': 1,
      'BRONZE': 2,
      'SILVER': 3,
      'GOLD': 4,
      'PLATINUM': 5,
      'ENTERPRISE': 6
    }
    
    const userLevel = tierHierarchy[user.package_tier] || 0
    const requiredLevel = tierHierarchy[requiredTier] || 1
    
    return userLevel >= requiredLevel
  }

  /**
   * Check role hierarchy
   */
  const hasRole = (requiredRole: string): boolean => {
    if (!user?.role) return false
    
    const roleHierarchy: Record<string, number> = {
      'user': 1,
      'premium': 2,
      'moderator': 3,
      'admin': 4,
      'super_admin': 5
    }
    
    const userLevel = roleHierarchy[user.role.toLowerCase()] || 0
    const requiredLevel = roleHierarchy[requiredRole.toLowerCase()] || 1
    
    return userLevel >= requiredLevel
  }

  /**
   * Check if user has any admin access
   */
  const isAdmin = (): boolean => {
    return user?.isAdmin || false
  }

  /**
   * Check if user has system admin access
   */
  const isSystemAdmin = (): boolean => {
    return hasAdminModule('system_admin') || user?.role === 'super_admin'
  }

  /**
   * Get all user permissions
   */
  const getPermissions = (): string[] => {
    return user?.permissions || []
  }

  /**
   * Get all admin modules
   */
  const getAdminModules = (): string[] => {
    return user?.admin_modules || []
  }

  /**
   * Check if user can access a feature based on package tier
   */
  const canAccessFeature = (feature: {
    requiredPermission?: string
    requiredAdminModule?: string
    requiredPackageTier?: string
    requiredRole?: string
  }): boolean => {
    // Check permission if required
    if (feature.requiredPermission && !hasPermission(feature.requiredPermission)) {
      return false
    }
    
    // Check admin module if required
    if (feature.requiredAdminModule && !hasAdminModule(feature.requiredAdminModule)) {
      return false
    }
    
    // Check package tier if required
    if (feature.requiredPackageTier && !hasPackageTier(feature.requiredPackageTier)) {
      return false
    }
    
    // Check role if required
    if (feature.requiredRole && !hasRole(feature.requiredRole)) {
      return false
    }
    
    return true
  }

  return {
    // Session data
    user,
    session,
    status,
    isLoading: status === "loading",
    isAuthenticated: !!session,
    
    // Permission checks
    hasPermission,
    hasAdminModule,
    hasPackageTier,
    hasRole,
    isAdmin,
    isSystemAdmin,
    canAccessFeature,
    
    // Data getters
    getPermissions,
    getAdminModules,
    
    // Session management
    updateSession: update,
  }
}

/**
 * Simplified auth hook for basic use cases
 */
export function useAuth() {
  const { user, isAuthenticated, isLoading, hasPermission, hasRole, isAdmin } = useModernAuth()
  
  return {
    user,
    isAuthenticated,
    isLoading,
    isAdmin: isAdmin(),
    hasPermission,
    hasRole,
  }
}

/**
 * Admin-specific auth hook
 */
export function useAdminAuth() {
  const { 
    user, 
    isAuthenticated, 
    isLoading, 
    hasAdminModule, 
    isAdmin, 
    isSystemAdmin,
    getAdminModules 
  } = useModernAuth()
  
  return {
    user,
    isAuthenticated,
    isLoading,
    isAdmin: isAdmin(),
    isSystemAdmin: isSystemAdmin(),
    hasAdminModule,
    adminModules: getAdminModules(),
  }
}

/**
 * Feature access hook for package-tier based features
 */
export function useFeatureAccess() {
  const { 
    user, 
    isAuthenticated, 
    hasPackageTier, 
    canAccessFeature 
  } = useModernAuth()
  
  return {
    user,
    isAuthenticated,
    packageTier: user?.package_tier || 'FREE',
    hasPackageTier,
    canAccessFeature,
    
    // Convenience methods for common features
    canAccessPremiumAnalytics: () => hasPackageTier('BRONZE'),
    canAccessAdvancedFeatures: () => hasPackageTier('SILVER'),
    canAccessEnterpriseFeatures: () => hasPackageTier('ENTERPRISE'),
  }
}