'use client'

// Export server functions for compatibility
export { getServerSession, getAuthUser } from './server/auth';

import { create } from 'zustand'
import { 
  User, 
  AuthState, 
  AdminAuthState
} from '../../../shared/types/auth'
import { 
  derivePackageTierFromPermissions, 
  deriveAccessiblePlatformsFromPermissions, 
  derivePrimaryPlatformFromPermissions 
} from '../../../shared/permissions/utils/platform'
import { config } from '@/config/env'

// Re-export types for compatibility
export type { User, AuthState, AdminAuthState } from '../../../shared/types/auth'

// Helper function to check structured permissions with wildcard support
function checkPermissionAccess(userPermissions: string[], requiredPermission: string): boolean {
  const required = parsePermission(requiredPermission);
  if (!required) return false;
  
  for (const permStr of userPermissions) {
    const userPerm = parsePermission(permStr);
    if (!userPerm) continue;
    
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true;
    }
    
    if (userPerm.platform === required.platform) {
      if (userPerm.resource === '*' && userPerm.action === '*') {
        return true;
      }
      
      if (userPerm.resource === required.resource && userPerm.action === '*') {
        return true;
      }
    }
    
    if (userPerm.platform === 'admin' && userPerm.resource === '*' && userPerm.action === '*') {
      return true;
    }
  }
  
  return false;
}

function parsePermission(permissionString: string): { platform: string; resource: string; action: string } | null {
  const parts = permissionString.split(':');
  if (parts.length !== 3) return null;
  
  return {
    platform: parts[0],
    resource: parts[1],
    action: parts[2]
  };
}

// Create admin auth store
export const useAuth = create<AdminAuthState>((set, get) => ({
  user: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  expiresAt: null,

  login: async () => {
    try {
      const currentUrl = window.location.href
      
      console.log('🔄 Admin: Initiating OAuth login with PKCE...')
      
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          redirectTo: currentUrl
        }),
        credentials: 'include'
      })
      
      if (!response.ok) {
        throw new Error(`OAuth initiation failed: ${response.status}`)
      }
      
      const data = await response.json()
      
      if (!data.success) {
        throw new Error(data.message || 'OAuth initiation failed')
      }
      
      console.log('✅ Admin: PKCE parameters set, redirecting to authorization...')
      window.location.href = data.authorizationUrl
      
    } catch (error) {
      console.error('❌ Admin: Login initiation failed:', error)
      const backendUrl = config.backendUrl
      const adminUrl = config.adminUrl
      
      const params = new URLSearchParams({
        client_id: 'epsx-admin',
        response_type: 'code',
        scope: 'openid profile email',
        redirect_uri: `${adminUrl}/api/auth/callback/epsx-backend`,
        state: Buffer.from(JSON.stringify({ redirectTo: window.location.href })).toString('base64url'),
      })
      
      window.location.href = `${backendUrl}/oauth/authorize?${params.toString()}`
    }
  },

  logout: async () => {
    set({ isLoading: true, error: null })
    
    try {
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include'
      })
      
      set({ 
        user: null, 
        isAuthenticated: false,
        expiresAt: null,
        isLoading: false
      })
      
      window.location.href = '/login'
    } catch (error) {
      console.error('❌ Logout failed:', error)
      set({ 
        error: 'Logout failed. Please try again.',
        isLoading: false
      })
    }
  },

  getUser: async () => {
    const { user, expiresAt } = get()
    if (user && expiresAt && Date.now() < expiresAt) {
      return user
    }

    set({ isLoading: true, error: null })

    try {
      const response = await fetch('/api/auth/session', {
        credentials: 'include'
      })

      if (!response.ok) {
        if (response.status === 401) {
          set({ 
            user: null,
            isAuthenticated: false,
            expiresAt: null,
            isLoading: false
          })
          return null
        }
        throw new Error(`Session fetch failed: ${response.status}`)
      }

      const data = await response.json()
      
      if (!data.isAuthenticated || !data.user) {
        set({ 
          user: null,
          isAuthenticated: false,
          expiresAt: null,
          isLoading: false
        })
        return null
      }

      const userData: User = {
        id: data.user.id,
        email: data.user.email,
        name: data.user.name,
        permissions: data.user.permissions || [],
      }

      set({ 
        user: userData,
        isAuthenticated: true,
        expiresAt: data.expiresAt,
        isLoading: false
      })

      setupTokenAutoRefresh(data.expiresAt)
      return userData

    } catch (error) {
      console.error('❌ Get user failed:', error)
      set({ 
        user: null,
        isAuthenticated: false,
        expiresAt: null,
        error: 'Failed to load session. Please try logging in again.',
        isLoading: false
      })
      return null
    }
  },

  refreshSession: async () => {
    try {
      await get().getUser()
    } catch (error) {
      console.error('❌ Session refresh failed:', error)
      get().logout()
    }
  },

  clearError: () => {
    set({ error: null })
  },

  can: (permission: string) => {
    const { user } = get()
    if (!user) return false
    
    let checkPermission = permission
    if (!permission.includes(':')) {
      const currentPlatform = derivePrimaryPlatformFromPermissions(user.permissions)
      checkPermission = `${currentPlatform}:${permission}`
    }
    
    return checkPermissionAccess(user.permissions, checkPermission)
  },

  hasAnyPermission: (permissions: string[]) => {
    const { user } = get()
    if (!user) return false
    
    return permissions.some(permission => checkPermissionAccess(user.permissions, permission))
  },

  hasAllPermissions: (permissions: string[]) => {
    const { user } = get()
    if (!user) return false
    
    return permissions.every(permission => checkPermissionAccess(user.permissions, permission))
  },

  hasTier: (tier: string) => {
    const { user } = get()
    if (!user) return false
    
    const tierHierarchy = {
      FREE: 1,
      BRONZE: 2,
      SILVER: 3,
      GOLD: 4,
      PLATINUM: 5,
      ENTERPRISE: 6,
    }
    
    const userPackageTier = derivePackageTierFromPermissions(user.permissions)
    const userLevel = tierHierarchy[userPackageTier as keyof typeof tierHierarchy] || 0
    const requiredLevel = tierHierarchy[tier as keyof typeof tierHierarchy] || 1
    
    return userLevel >= requiredLevel
  },
  
  switchPlatform: async (platform: string) => {
    const { user } = get()
    if (!user) return
    
    const availablePlatforms = deriveAccessiblePlatformsFromPermissions(user.permissions)
    if (!availablePlatforms.includes(platform)) {
      set({ error: `Access denied to platform: ${platform}` })
      return
    }
    
    set({ isLoading: true, error: null })
    
    try {
      const response = await fetch('/api/auth/switch-platform', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ platform }),
        credentials: 'include',
      })
      
      if (!response.ok) {
        throw new Error(`Platform switch failed: ${response.status}`)
      }
      
      set({ 
        user: { 
          ...user, 
        },
        isLoading: false
      })
      
    } catch (error) {
      console.error('❌ Admin platform switch failed:', error)
      set({ 
        error: 'Failed to switch platform. Please try again.',
        isLoading: false
      })
    }
  },
  
  getCurrentPlatform: () => {
    const { user } = get()
    return derivePrimaryPlatformFromPermissions(user?.permissions || []) || 'epsx'
  },
  
  getAvailablePlatforms: () => {
    const { user } = get()
    return deriveAccessiblePlatformsFromPermissions(user?.permissions || [])
  },
  
  canAccessPlatform: (platform: string) => {
    const { user } = get()
    if (!user) return false
    
    const availablePlatforms = deriveAccessiblePlatformsFromPermissions(user.permissions)
    return availablePlatforms.includes(platform)
  },
  
  // Admin-specific permission checks
  isAdmin: () => {
    const { can } = get()
    return can('admin:*:*')
  },
  
  canManageUsers: () => {
    const { hasAnyPermission } = get()
    return hasAnyPermission(['admin:users:manage', 'epsx:users:manage'])
  },
  
  canManageSystem: () => {
    const { hasAnyPermission } = get()
    return hasAnyPermission(['admin:system:manage', 'admin:*:*'])
  },
  
  canViewAnalytics: () => {
    const { hasAnyPermission } = get()
    return hasAnyPermission(['epsx:analytics:view', 'epsx:analytics:*', 'admin:*:*'])
  },
  
  canManagePlatforms: () => {
    const { hasAnyPermission } = get()
    return hasAnyPermission(['admin:platforms:manage', 'admin:*:*'])
  },

  canViewAudit: () => {
    const { hasAnyPermission } = get()
    return hasAnyPermission(['admin:audit:read', 'epsx:audit:read', 'admin:*:*'])
  },
}))

// Auto-refresh token management
let refreshTimeout: NodeJS.Timeout | null = null

function setupTokenAutoRefresh(expiresAt: number) {
  if (refreshTimeout) {
    clearTimeout(refreshTimeout)
  }
  
  const refreshTime = Math.max(0, expiresAt - Date.now() - (5 * 60 * 1000))
  
  refreshTimeout = setTimeout(() => {
    const { isAuthenticated, refreshSession } = useAuth.getState()
    if (isAuthenticated) {
      console.log('🔄 Auto-refreshing admin session')
      refreshSession()
    }
  }, refreshTime)
}


// Initialize auth on client mount
if (typeof window !== 'undefined') {
  // Auto-fetch user on app start
  useAuth.getState().getUser()
}

// Helper functions for components - permission-only
export function checkPermission(permission: string): boolean {
  return useAuth.getState().can(permission)
}

export function checkAnyPermission(permissions: string[]): boolean {
  return useAuth.getState().hasAnyPermission(permissions)
}

export function checkTier(tier: string): boolean {
  return useAuth.getState().hasTier(tier)
}

// Admin permission helpers - pure permission system
export function useAdminPermissions() {
  const { 
    isAdmin,
    canManageUsers, 
    canManageSystem, 
    canViewAnalytics, 
    canManagePlatforms,
    canViewAudit,
    can,
    getCurrentPlatform 
  } = useAuth.getState()
  const currentPlatform = getCurrentPlatform()
  
  return {
    // Core admin functions
    isAdmin: isAdmin(),
    canManageUsers: canManageUsers(),
    canManageSystem: canManageSystem(),
    canViewAnalytics: canViewAnalytics(),
    canManagePlatforms: canManagePlatforms(),
    canViewAudit: canViewAudit(),
    
    // Specific admin permissions
    hasUserManagement: can('admin:users:manage') || can('epsx:users:manage'),
    hasPermissionManagement: can('admin:permissions:manage'),
    hasAnalytics: can('epsx:analytics:view') || can('epsx:analytics:*'),
    hasBilling: can('epsx:billing:manage'),
    hasSystemConfig: can('admin:system:manage'),
    hasAuditLogs: can('admin:audit:read') || can('epsx:audit:read'),
    hasNotifications: can('admin:notifications:manage'),
    
    // Platform-specific admin permissions
    canAdminCurrentPlatform: can(`${currentPlatform}:*:*`),
    canSwitchPlatforms: can('admin:platforms:switch'),
    
    // Helper function for checking any permission
    can,
    currentPlatform,
  }
}

// Utility Functions for Admin UI
export function getUserDisplayName(user: User | null): string {
  if (!user) return 'Unknown User'
  return user.name || user.email.split('@')[0] || 'Admin User'
}

export function getPermissionLabels(permissions: string[]): string[] {
  const permissionLabels: Record<string, string> = {
    // Admin permissions
    'admin:*:*': 'Global Administrator',
    'admin:users:manage': 'User Management',
    'admin:users:read': 'User Viewing',
    'admin:system:manage': 'System Configuration',
    'admin:audit:read': 'Audit Logs',
    'admin:security:read': 'Security Monitoring',
    'admin:permissions:manage': 'Permission Management',
    'admin:platforms:manage': 'Platform Management',
    
    // EPSX permissions
    'epsx:analytics:view': 'Analytics Viewing',
    'epsx:analytics:export': 'Analytics Export',
    'epsx:analytics:advanced': 'Advanced Analytics',
    'epsx:realtime:access': 'Real-time Data',
    'epsx:profile:manage': 'Profile Management',
    'epsx:notifications:receive': 'Notifications',
    'epsx:billing:manage': 'Billing Management',
    'epsx:users:manage': 'User Management',
    
    // EPSX Pay permissions
    'epsx-pay:transactions:create': 'Create Transactions',
    'epsx-pay:transactions:read': 'View Transactions',
    'epsx-pay:payments:process': 'Process Payments',
    'epsx-pay:payments:refund': 'Refund Payments',
    
    // EPSX Token permissions
    'epsx-token:governance:vote': 'Governance Voting',
    'epsx-token:governance:propose': 'Create Proposals',
    'epsx-token:tokens:stake': 'Token Staking',
    'epsx-token:treasury:view': 'Treasury Access',
  }
  
  return permissions.map(permission => {
    return permissionLabels[permission] || permission
  })
}

// Check if user has admin access at all - permission-based
export function hasAdminAccess(): boolean {
  const { user, can, hasAnyPermission } = useAuth.getState()
  if (!user) return false
  
  // Check if user has any admin permissions
  return hasAnyPermission([
    'admin:*:*',
    'admin:users:manage',
    'admin:system:manage',
    'admin:audit:read',
    'epsx:users:manage',
    'epsx:analytics:view'
  ])
}

// Cross-platform utility functions
export function getPlatformDisplayName(platform: string): string {
  const platformNames: Record<string, string> = {
    'epsx': 'EPSX Trading',
    'epsx-pay': 'EPSX Pay',
    'epsx-token': 'EPSX Token',
    'admin': 'Admin Panel'
  }
  
  return platformNames[platform] || platform.toUpperCase()
}

export function getPlatformIcon(platform: string): string {
  const platformIcons: Record<string, string> = {
    'epsx': '📈',
    'epsx-pay': '💳',
    'epsx-token': '🪙',
    'admin': '⚙️'
  }
  
  return platformIcons[platform] || '⚡'
}

// OAuth authorization URL generation now handled by shared utilities

// PKCE helper functions now available from shared utilities

// Sign in helper for components
export async function signIn(callbackUrl?: string) {
  try {
    // Use PKCE initiation route for secure OAuth flow
    const redirectTo = callbackUrl || window.location.href
    
    console.log('🔄 Admin: Initiating OAuth login with PKCE...')
    
    const response = await fetch('/api/auth/initiate', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        redirectTo
      }),
      credentials: 'include'
    })
    
    if (!response.ok) {
      throw new Error(`OAuth initiation failed: ${response.status}`)
    }
    
    const data = await response.json()
    
    if (!data.success) {
      throw new Error(data.message || 'OAuth initiation failed')
    }
    
    console.log('✅ Admin: PKCE parameters set, redirecting to authorization...')
    
    // Redirect to authorization URL
    window.location.href = data.authorizationUrl
    
  } catch (error) {
    console.error('❌ Admin: SignIn initiation failed:', error)
    // Fallback to direct redirect if PKCE initiation fails
    const backendUrl = config.backendUrl
    const adminUrl = config.adminUrl
    const redirectTo = callbackUrl || window.location.href
    
    const params = new URLSearchParams({
      client_id: 'epsx-admin',
      response_type: 'code',
      scope: 'openid profile email',
      redirect_uri: `${adminUrl}/api/auth/callback/epsx-backend`,
      state: Buffer.from(JSON.stringify({ redirectTo })).toString('base64url'),
    })
    
    window.location.href = `${backendUrl}/oauth/authorize?${params.toString()}`
  }
}