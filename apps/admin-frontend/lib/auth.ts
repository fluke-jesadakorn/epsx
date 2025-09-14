'use client'

// Export server functions for compatibility
export { getServerSession, getAuthUser } from './server/auth';

import { create } from 'zustand'
import { derivePackageTierFromPermissions, deriveAccessiblePlatformsFromPermissions, derivePrimaryPlatformFromPermissions } from './auth-utils'
import { config } from '@/config/env'

export interface User {
  id: string
  email: string
  name?: string
  permissions: string[]  // Structured permissions only: "platform:resource:action"
}

export interface AuthState {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  error: string | null
  expiresAt: number | null
  
  // Actions
  login: () => void
  logout: () => Promise<void>
  getUser: () => Promise<User | null>
  refreshSession: () => Promise<void>
  clearError: () => void
  
  // Permission checks - pure permission system
  can: (permission: string) => boolean
  hasAnyPermission: (permissions: string[]) => boolean
  hasAllPermissions: (permissions: string[]) => boolean
  hasTier: (tier: string) => boolean
  
  // Cross-platform functionality
  switchPlatform: (platform: string) => Promise<void>
  getCurrentPlatform: () => string
  getAvailablePlatforms: () => string[]
  canAccessPlatform: (platform: string) => boolean
  
  // Admin-specific permission checks
  isAdmin: () => boolean
  canManageUsers: () => boolean
  canManageSystem: () => boolean
  canViewAnalytics: () => boolean
  canManagePlatforms: () => boolean
  canViewAudit: () => boolean
}

// Helper function to check structured permissions with wildcard support
function checkPermissionAccess(userPermissions: string[], requiredPermission: string): boolean {
  const required = parsePermission(requiredPermission);
  if (!required) return false;
  
  for (const permStr of userPermissions) {
    const userPerm = parsePermission(permStr);
    if (!userPerm) continue;
    
    // Check for exact match
    if (userPerm.platform === required.platform && 
        userPerm.resource === required.resource && 
        userPerm.action === required.action) {
      return true;
    }
    
    // Check for wildcard matches
    if (userPerm.platform === required.platform) {
      // Platform-level wildcard: "epsx:*:*"
      if (userPerm.resource === '*' && userPerm.action === '*') {
        return true;
      }
      
      // Resource-level wildcard: "epsx:analytics:*"
      if (userPerm.resource === required.resource && userPerm.action === '*') {
        return true;
      }
    }
    
    // Global admin permission: "admin:*:*"
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

// Create auth store without persistence (relies on server-side cookies)
export const useAuth = create<AuthState>((set, get) => ({
  user: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  expiresAt: null,

  login: async () => {
    try {
      // Use PKCE initiation route for secure OAuth flow
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
      
      // Redirect to authorization URL
      window.location.href = data.authorizationUrl
      
    } catch (error) {
      console.error('❌ Admin: Login initiation failed:', error)
      // Fallback to direct redirect if PKCE initiation fails
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
      // Call logout API to clear cookies and revoke tokens
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include'
      })
      
      // Clear local state
      set({ 
        user: null, 
        isAuthenticated: false,
        expiresAt: null,
        isLoading: false
      })
      
      // Redirect to login page
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
    // If we already have a user and it's still valid, return it
    const { user, expiresAt } = get()
    if (user && expiresAt && Date.now() < expiresAt) {
      return user
    }

    set({ isLoading: true, error: null })

    try {
      // Fetch session from server
      const response = await fetch('/api/auth/session', {
        credentials: 'include'
      })

      if (!response.ok) {
        if (response.status === 401) {
          // Unauthorized - clear state
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

      // Set up auto-refresh based on JWT expiration
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
      // For now, just re-fetch the user data
      // In the future, this could implement refresh token logic
      await get().getUser()
    } catch (error) {
      console.error('❌ Session refresh failed:', error)
      // On refresh failure, force logout
      get().logout()
    }
  },

  clearError: () => {
    set({ error: null })
  },

  can: (permission: string) => {
    const { user } = get()
    if (!user) return false
    
    // If permission doesn't contain platform prefix, add current platform
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
  
  // Cross-platform functionality
  switchPlatform: async (platform: string) => {
    const { user } = get()
    if (!user) return
    
    // Check if user can access the platform
    const availablePlatforms = deriveAccessiblePlatformsFromPermissions(user.permissions)
    if (!availablePlatforms.includes(platform)) {
      set({ error: `Access denied to platform: ${platform}` })
      return
    }
    
    set({ isLoading: true, error: null })
    
    try {
      // Update platform context on server
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
      
      // Update local user state
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
  
  // Admin-specific permission checks - using structured permissions
  isAdmin: () => {
    const { can } = get()
    return can('admin:*:*')
  },
  
  canManageUsers: () => {
    const { can, hasAnyPermission } = get()
    return hasAnyPermission(['admin:users:manage', 'epsx:users:manage'])
  },
  
  canManageSystem: () => {
    const { can, hasAnyPermission } = get()
    return hasAnyPermission(['admin:system:manage', 'admin:*:*'])
  },
  
  canViewAnalytics: () => {
    const { can, hasAnyPermission } = get()
    return hasAnyPermission(['epsx:analytics:view', 'epsx:analytics:*', 'admin:*:*'])
  },
  
  canManagePlatforms: () => {
    const { can, hasAnyPermission } = get()
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
  // Clear any existing timeout
  if (refreshTimeout) {
    clearTimeout(refreshTimeout)
  }
  
  // Calculate time until token expires (refresh 5 minutes before expiration)
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

// OIDC Authorization URL generation with PKCE
export async function getAuthorizationUrl() {
  // Use consolidated auth config
  const { authConfig } = await import('../config/env')
  
  // Generate PKCE parameters
  const codeVerifier = generateCodeVerifier()
  const codeChallenge = await generateCodeChallenge(codeVerifier)
  const state = generateRandomString(32)
  
  // Build authorization URL using consolidated config
  const authorizationEndpoint = authConfig.authorizationEndpoint
  const clientId = authConfig.clientId
  const redirectUri = authConfig.callbackUrl
  
  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: 'openid profile email',
    state: state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  })
  
  const url = `${authorizationEndpoint}?${params.toString()}`
  
  return {
    url,
    codeVerifier,
    state,
  }
}

// PKCE helper functions
function generateCodeVerifier(): string {
  const array = new Uint8Array(32)
  crypto.getRandomValues(array)
  return base64URLEncode(array)
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(verifier)
  const digest = await crypto.subtle.digest('SHA-256', data)
  return base64URLEncode(new Uint8Array(digest))
}

function generateRandomString(length: number): string {
  const array = new Uint8Array(length)
  crypto.getRandomValues(array)
  return base64URLEncode(array)
}

function base64URLEncode(array: Uint8Array): string {
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
}

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