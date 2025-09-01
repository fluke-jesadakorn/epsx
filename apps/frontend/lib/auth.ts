'use client'

import { create } from 'zustand'
import { isJWTExpired, getJWTTimeToExpiry, derivePackageTierFromPermissions, deriveAccessiblePlatformsFromPermissions, derivePrimaryPlatformFromPermissions } from '@/lib/auth-utils';

export interface User {
  id: string
  email: string
  name?: string
  permissions: string[]  // Structured permissions: "platform:resource:action"
  platform_context?: string    // Current platform context
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
      
      console.log('🔄 Frontend: Initiating OAuth login with PKCE...')
      
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
      
      console.log('✅ Frontend: PKCE parameters set, redirecting to authorization...')
      
      // Redirect to authorization URL
      window.location.href = data.authorizationUrl
      
    } catch (error) {
      console.error('❌ Frontend: Login initiation failed:', error)
      // Fallback to direct redirect if PKCE initiation fails
      const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
      const frontendUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'
      
      const params = new URLSearchParams({
        client_id: 'epsx-frontend',
        response_type: 'code',
        scope: 'openid profile email',
        redirect_uri: `${frontendUrl}/api/auth/callback/epsx-backend`,
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
      
      // Redirect to home
      window.location.href = '/'
      
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
        platform_context: data.user.platform_context,
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
      const currentPlatform = user.platform_context || derivePrimaryPlatformFromPermissions(user.permissions)
      checkPermission = `${currentPlatform}:${permission}`
    }
    
    // Check exact match
    if (user.permissions.includes(checkPermission)) {
      return true
    }
    
    // Check wildcard permissions
    return user.permissions.some(p => {
      if (p.endsWith('*')) {
        const prefix = p.slice(0, -1)
        return checkPermission.startsWith(prefix)
      }
      
      // Check platform-level wildcards (e.g., "epsx:*")
      const parts = p.split(':')
      if (parts.length >= 2 && parts[1] === '*') {
        const checkParts = checkPermission.split(':')
        return checkParts[0] === parts[0]
      }
      
      return false
    })
  },

  hasAnyPermission: (permissions: string[]) => {
    const { user } = get()
    if (!user) return false
    
    return permissions.some(permission => {
      return user.permissions.some(userPerm => {
        // Support wildcard matching for admin permissions
        if (userPerm === 'admin:*:*') return true
        
        const [userPlat, userRes, userAct] = userPerm.split(':')
        const [reqPlat, reqRes, reqAct] = permission.split(':')
        
        return (userPlat === '*' || reqPlat === '*' || userPlat === reqPlat) &&
               (userRes === '*' || reqRes === '*' || userRes === reqRes) &&
               (userAct === '*' || reqAct === '*' || userAct === reqAct)
      })
    })
  },
  hasAllPermissions: (permissions: string[]) => {
    const { user } = get()
    if (!user) return false
    
    return permissions.every(permission => get().can(permission))
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
          platform_context: platform 
        },
        isLoading: false
      })
      
    } catch (error) {
      console.error('❌ Platform switch failed:', error)
      set({ 
        error: 'Failed to switch platform. Please try again.',
        isLoading: false
      })
    }
  },
  
  getCurrentPlatform: () => {
    const { user } = get()
    return user?.platform_context || derivePrimaryPlatformFromPermissions(user?.permissions || []) || 'epsx'
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
      console.log('🔄 Auto-refreshing session')
      refreshSession()
    }
  }, refreshTime)
}

// Initialize auth on client mount
if (typeof window !== 'undefined') {
  // Auto-fetch user on app start
  useAuth.getState().getUser()
}

// Helper functions for components
export function checkPermission(permission: string): boolean {
  return useAuth.getState().can(permission)
}

export function checkAnyPermission(permissions: string[]): boolean {
  return useAuth.getState().hasAnyPermission(permissions)
}

export function checkAllPermissions(permissions: string[]): boolean {
  return useAuth.getState().hasAllPermissions(permissions)
}

export function checkTier(tier: string): boolean {
  return useAuth.getState().hasTier(tier)
}

// Package tier helpers for frontend trading platform
export function usePackageTier(requiredTier?: string) {
  const { user } = useAuth.getState()
  const packageTier = user ? derivePackageTierFromPermissions(user.permissions) : 'FREE'
  
  const hasRequiredTier = requiredTier ? 
    useAuth.getState().hasTier(requiredTier) : 
    true
  
  return {
    currentTier: packageTier,
    hasRequiredTier,
    isPremium: useAuth.getState().hasTier('BRONZE'),
    isEnterprise: useAuth.getState().hasTier('ENTERPRISE'),
  }
}

// Utility Functions
export function getUserDisplayName(user: User | null): string {
  if (!user) return 'Unknown User'
  return user.name || user.email.split('@')[0] || 'User'
}

export function formatPackageTier(tier: string): string {
  const tierLabels: Record<string, string> = {
    'FREE': 'Free',
    'BRONZE': 'Bronze',
    'SILVER': 'Silver',
    'GOLD': 'Gold',
    'PLATINUM': 'Platinum',
    'ENTERPRISE': 'Enterprise'
  }
  
  return tierLabels[tier] || tier
}

export function getPackageFeatures(tier: string): string[] {
  const features: Record<string, string[]> = {
    'FREE': ['Basic stock data', 'Limited API calls'],
    'BRONZE': ['Enhanced data', 'More API calls', 'Basic analytics'],
    'SILVER': ['Real-time data', 'Advanced analytics', 'Email alerts'],
    'GOLD': ['Premium data feeds', 'Custom indicators', 'Phone support'],
    'PLATINUM': ['All features', 'Priority support', 'Custom dashboards'],
    'ENTERPRISE': ['White-label solution', 'Dedicated support', 'Custom integrations']
  }
  
  return features[tier] || []
}


// Sign in helper for components
export async function signIn(callbackUrl?: string) {
  try {
    // Use PKCE initiation route for secure OAuth flow
    const redirectTo = callbackUrl || window.location.href
    
    console.log('🔄 Frontend: Initiating OAuth login with PKCE...')
    
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
    
    console.log('✅ Frontend: PKCE parameters set, redirecting to authorization...')
    
    // Redirect to authorization URL
    window.location.href = data.authorizationUrl
    
  } catch (error) {
    console.error('❌ Frontend: SignIn initiation failed:', error)
    // Fallback to direct redirect if PKCE initiation fails
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
    const frontendUrl = process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'
    const redirectTo = callbackUrl || window.location.href
    
    const params = new URLSearchParams({
      client_id: 'epsx-frontend',
      response_type: 'code',
      scope: 'openid profile email',
      redirect_uri: `${frontendUrl}/api/auth/callback/epsx-backend`,
      state: Buffer.from(JSON.stringify({ redirectTo })).toString('base64url'),
    })
    
    window.location.href = `${backendUrl}/oauth/authorize?${params.toString()}`
  }
}

// Cross-Platform Utility Functions
export function parseStructuredPermission(permission: string): {
  platform: string
  resource: string
  action: string
} | null {
  const parts = permission.split(':')
  if (parts.length !== 3) return null
  
  return {
    platform: parts[0],
    resource: parts[1],
    action: parts[2],
  }
}

export function hasStructuredPermission(
  userPermissions: string[],
  requiredPermission: string,
  currentPlatform?: string
): boolean {
  // If permission doesn't contain platform prefix, add current platform
  let checkPermission = requiredPermission
  if (!requiredPermission.includes(':') && currentPlatform) {
    checkPermission = `${currentPlatform}:${requiredPermission}`
  }
  
  // Check exact match
  if (userPermissions.includes(checkPermission)) {
    return true
  }
  
  // Check wildcard permissions
  return userPermissions.some(p => {
    if (p.endsWith('*')) {
      const prefix = p.slice(0, -1)
      return checkPermission.startsWith(prefix)
    }
    
    // Check platform-level wildcards (e.g., "epsx:*")
    const parts = p.split(':')
    if (parts.length >= 2 && parts[1] === '*') {
      const checkParts = checkPermission.split(':')
      return checkParts[0] === parts[0]
    }
    
    return false
  })
}

export function getPlatformDisplayName(platform: string): string {
  const platformNames: Record<string, string> = {
    'epsx': 'EPSX Trading',
    'epsx-pay': 'EPSX Pay',
    'epsx-token': 'EPSX Token',
  }
  
  return platformNames[platform] || platform.toUpperCase()
}

export function getPlatformIcon(platform: string): string {
  const platformIcons: Record<string, string> = {
    'epsx': '📈',
    'epsx-pay': '💳',
    'epsx-token': '🪙',
  }
  
  return platformIcons[platform] || '⚡'
}

export function createPlatformPermission(
  platform: string,
  resource: string,
  action: string
): string {
  return `${platform}:${resource}:${action}`
}

// Cross-platform hooks
export function usePlatformContext() {
  const { user, getCurrentPlatform, getAvailablePlatforms, canAccessPlatform, switchPlatform } = useAuth.getState()
  
  return {
    currentPlatform: getCurrentPlatform(),
    availablePlatforms: getAvailablePlatforms(),
    canAccessPlatform,
    switchPlatform,
    platformDisplayName: getPlatformDisplayName(getCurrentPlatform()),
    platformIcon: getPlatformIcon(getCurrentPlatform()),
  }
}

export function useStructuredPermissions() {
  const { user, can } = useAuth.getState()
  const currentPlatform = user?.platform_context || derivePrimaryPlatformFromPermissions(user?.permissions || []) || 'epsx'
  
  return {
    can,
    hasPermission: (resource: string, action: string, platform?: string) => {
      const targetPlatform = platform || currentPlatform
      return can(`${targetPlatform}:${resource}:${action}`)
    },
    canRead: (resource: string, platform?: string) => {
      const targetPlatform = platform || currentPlatform
      return can(`${targetPlatform}:${resource}:read`)
    },
    canWrite: (resource: string, platform?: string) => {
      const targetPlatform = platform || currentPlatform
      return can(`${targetPlatform}:${resource}:write`)
    },
    canManage: (resource: string, platform?: string) => {
      const targetPlatform = platform || currentPlatform
      return can(`${targetPlatform}:${resource}:manage`)
    },
    currentPlatform,
  }
}