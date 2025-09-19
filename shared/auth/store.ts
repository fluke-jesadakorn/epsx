/**
 * Shared Authentication Store
 * Core auth logic shared between frontend and admin-frontend
 */

'use client'

import { create } from 'zustand'
import { 
  User, 
  AuthState, 
  AdminAuthState, 
  AuthSessionData,
  SmartRefreshRequest,
  SmartRefreshResponse 
} from '../types/auth'
import { 
  derivePackageTierFromPermissions, 
  deriveAccessiblePlatformsFromPermissions, 
  derivePrimaryPlatformFromPermissions 
} from '../permissions/utils/platform'
import { getBackendUrl, getFrontendUrl, oidcUrls, callbackUrls } from '../utils/url-resolver'

// Core auth store factory
export function createAuthStore<T extends AuthState>(
  config: {
    clientId: string
    redirectPath: string
    logoutRedirectPath: string
    enableAutoRefresh?: boolean
    enableSmartRefresh?: boolean
    refreshBuffer?: number
  }
) {
  return create<T>((set: any, get: any) => ({
    user: null,
    isLoading: false,
    isAuthenticated: false,
    error: null,
    expiresAt: null,
    
    // Auto-refresh state (optional for admin)
    ...(config.enableAutoRefresh && {
      autoRefreshEnabled: true,
      refreshInProgress: false,
      lastRefreshTime: null,
    }),

    login: async () => {
      try {
        const currentUrl = window.location.href
        
        console.log('🔄 Initiating OAuth login with PKCE...')
        
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
        
        console.log('✅ PKCE parameters set, redirecting to authorization...')
        
        // Redirect to authorization URL
        window.location.href = data.authorizationUrl
        
      } catch (error) {
        console.error('❌ Login initiation failed:', error)
        // Fallback to direct redirect if PKCE initiation fails
        const backendUrl = getBackendUrl('client')
        
        const params = new URLSearchParams({
          client_id: config.clientId,
          response_type: 'code',
          scope: 'openid profile email',
          redirect_uri: config.redirectPath,
          state: Buffer.from(JSON.stringify({ redirectTo: window.location.href })).toString('base64url'),
        })
        
        window.location.href = `${oidcUrls.authorize('client')}?${params.toString()}`
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
        
        // Redirect based on app type
        window.location.href = config.logoutRedirectPath
        
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

        const data: AuthSessionData = await response.json()
        
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
          permission_version: data.user.permission_version,
          permission_last_updated: data.user.permission_last_updated,
          tier: data.user.tier,
          verified: data.user.verified,
        }

        set({ 
          user: userData,
          isAuthenticated: true,
          expiresAt: data.expiresAt,
          isLoading: false
        })

        // Set up auto-refresh based on JWT expiration
        if (config.enableAutoRefresh && data.expiresAt) {
          setupTokenAutoRefresh(data.expiresAt, config.refreshBuffer || 5000)
        }

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
      if (config.enableSmartRefresh) {
        return smartRefreshSession(set, get, config.refreshBuffer || 5000)
      } else {
        // Simple refresh - just re-fetch user data
        try {
          await get().getUser()
        } catch (error) {
          console.error('❌ Session refresh failed:', error)
          get().logout()
        }
      }
    },

    clearError: () => {
      set({ error: null })
    },
    
    // Auto-refresh management (only if enabled)
    ...(config.enableAutoRefresh && {
      enableAutoRefresh: () => {
        console.log('🔄 Auto-refresh enabled')
        set({ autoRefreshEnabled: true })
      },
      
      disableAutoRefresh: () => {
        console.log('🔄 Auto-refresh disabled')
        set({ autoRefreshEnabled: false })
        if (refreshTimeout) {
          clearTimeout(refreshTimeout)
          refreshTimeout = null
        }
      },
      
      checkTokenHealth: () => {
        const { expiresAt, lastRefreshTime, user } = get()
        const now = Date.now()
        
        if (!expiresAt || !user) return false
        
        // Check if token expires soon (within 10 seconds)
        const timeToExpiry = expiresAt - now
        const needsRefresh = timeToExpiry <= 10000
        
        // Check if permissions might be stale (older than 5 minutes)
        const permissionAge = user.permission_last_updated ? 
          now / 1000 - user.permission_last_updated : Infinity
        const permissionsStale = permissionAge > 300 // 5 minutes
        
        const isHealthy = !needsRefresh && !permissionsStale
        
        if (!isHealthy) {
          console.log(`Token health check: expires in ${timeToExpiry}ms, permissions age: ${permissionAge}s`)
        }
        
        return isHealthy
      },
    }),

    can: (permission: string) => {
      const { user } = get()
      if (!user) return false
      
      // Enhanced health check if available
      if (config.enableAutoRefresh && (get() as any).checkTokenHealth) {
        const isHealthy = (get() as any).checkTokenHealth()
        if (!isHealthy && (get() as any).autoRefreshEnabled) {
          // Trigger background refresh for stale tokens/permissions
          setTimeout(() => get().refreshSession(), 0)
        }
      }
      
      // If permission doesn't contain platform prefix, add current platform
      let checkPermission = permission
      if (!permission.includes(':')) {
        const currentPlatform = user.platform_context || derivePrimaryPlatformFromPermissions(user.permissions)
        checkPermission = `${currentPlatform}:${permission}`
      }
      
      // Validate embedded permissions with timestamp filtering
      const validPermissions = user.permissions.filter((p: string) => {
        // Check if permission has embedded timestamp (format: platform:resource:action:timestamp)
        const parts = p.split(':')
        if (parts.length === 4) {
          const timestamp = parseInt(parts[3], 10)
          if (!isNaN(timestamp)) {
            // Check if timestamp permission is still valid
            return Date.now() / 1000 <= timestamp
          }
        }
        return true // Permanent permission (no timestamp)
      })
      
      return checkStructuredPermission(validPermissions, checkPermission)
    },

    hasAnyPermission: (permissions: string[]) => {
      const { user } = get()
      if (!user) return false
      
      return permissions.some(permission => get().can(permission))
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
  } as T))
}

// Admin-specific store factory
export function createAdminAuthStore() {
  const baseStore = createAuthStore<AdminAuthState>({
    clientId: 'epsx-admin',
    redirectPath: callbackUrls.admin('client'),
    logoutRedirectPath: '/login',
    enableAutoRefresh: false, // Admin doesn't need auto-refresh
    enableSmartRefresh: false,
  })

  return create<AdminAuthState>((set: any, get: any) => ({
    ...baseStore(set, get),
    
    // Admin-specific permission checks
    isAdmin: () => {
      return get().can('admin:*:*')
    },
    
    canManageUsers: () => {
      return get().hasAnyPermission(['admin:users:manage', 'epsx:users:manage'])
    },
    
    canManageSystem: () => {
      return get().hasAnyPermission(['admin:system:manage', 'admin:*:*'])
    },
    
    canViewAnalytics: () => {
      return get().hasAnyPermission(['epsx:analytics:view', 'epsx:analytics:*', 'admin:*:*'])
    },
    
    canManagePlatforms: () => {
      return get().hasAnyPermission(['admin:platforms:manage', 'admin:*:*'])
    },

    canViewAudit: () => {
      return get().hasAnyPermission(['admin:audit:read', 'epsx:audit:read', 'admin:*:*'])
    },
  }))
}

// Frontend store factory with enhanced features
export function createFrontendAuthStore() {
  return createAuthStore<AuthState>({
    clientId: 'epsx-frontend',
    redirectPath: callbackUrls.frontend('client'),
    logoutRedirectPath: '/',
    enableAutoRefresh: true,
    enableSmartRefresh: true,
    refreshBuffer: 5000,
  })
}

// Shared permission checking logic
function checkStructuredPermission(userPermissions: string[], requiredPermission: string): boolean {
  // Check exact match
  if (userPermissions.includes(requiredPermission)) {
    return true
  }
  
  // Check wildcard permissions
  return userPermissions.some(p => {
    if (p.endsWith('*')) {
      const prefix = p.slice(0, -1)
      return requiredPermission.startsWith(prefix)
    }
    
    // Check platform-level wildcards (e.g., "epsx:*")
    const parts = p.split(':')
    if (parts.length >= 2 && parts[1] === '*') {
      const checkParts = requiredPermission.split(':')
      return checkParts[0] === parts[0]
    }
    
    return false
  })
}

// Smart refresh implementation for frontend
async function smartRefreshSession(
  set: any, 
  get: any, 
  refreshBuffer: number
): Promise<void> {
  const { refreshInProgress, autoRefreshEnabled, user } = get()
  
  // Prevent concurrent refresh attempts
  if (refreshInProgress) {
    console.log('🔄 Refresh already in progress, skipping')
    return
  }
  
  if (!autoRefreshEnabled) {
    console.log('🔄 Auto-refresh disabled, skipping')
    return
  }
  
  set({ refreshInProgress: true, error: null })
  
  try {
    console.log('🔄 Starting smart session refresh with permission sync')
    
    const response = await fetch('/api/auth/smart-refresh', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        current_permission_version: user?.permission_version || 0,
        force_permission_reload: false
      }),
      credentials: 'include'
    })
    
    if (!response.ok) {
      throw new Error(`Smart refresh failed: ${response.status}`)
    }
    
    const refreshData: SmartRefreshResponse = await response.json()
    
    if (refreshData.success) {
      // Update user data with fresh permissions if they changed
      if (refreshData.permissions_changed) {
        console.log(`✅ Permissions updated: v${user?.permission_version || 0} -> v${refreshData.permission_version}`)
        
        const updatedUser: User = {
          ...user!,
          permissions: refreshData.new_permissions || [],
          permission_version: refreshData.permission_version,
          permission_last_updated: Date.now() / 1000,
        }
        
        set({ 
          user: updatedUser,
          lastRefreshTime: Date.now(),
          refreshInProgress: false
        })
      } else {
        console.log('✅ Permissions unchanged, token refreshed')
        set({ 
          lastRefreshTime: Date.now(),
          refreshInProgress: false
        })
      }
      
      // Set up next auto-refresh
      if (refreshData.expiresAt) {
        setupTokenAutoRefresh(refreshData.expiresAt, refreshBuffer)
      }
    } else {
      throw new Error(refreshData.message || 'Smart refresh failed')
    }
    
  } catch (error) {
    console.warn('⚠️ Smart refresh failed, falling back to user re-fetch', error)
    
    // Fallback: Try regular user fetch
    try {
      await get().getUser()
      set({ refreshInProgress: false })
    } catch (fallbackError) {
      console.error('❌ Both smart refresh and fallback failed', fallbackError)
      // Force logout on complete failure
      get().logout()
    }
  }
}

// Auto-refresh token management
let refreshTimeout: NodeJS.Timeout | null = null

function setupTokenAutoRefresh(expiresAt: number, refreshBuffer: number) {
  // Clear any existing timeout
  if (refreshTimeout) {
    clearTimeout(refreshTimeout)
  }
  
  const now = Date.now()
  const timeToExpiry = expiresAt - now
  
  // Proactive refresh for short-lived tokens
  const actualBuffer = Math.min(refreshBuffer, timeToExpiry * 0.1) // refreshBuffer or 10% of token life
  const refreshTime = Math.max(0, timeToExpiry - actualBuffer)
  
  console.log(`🔄 Setting up auto-refresh in ${refreshTime}ms (${actualBuffer}ms buffer for ${timeToExpiry}ms token)`)
  
  refreshTimeout = setTimeout(() => {
    // This will be called from the store instance
    console.log('🔄 Auto-refreshing session (proactive)')
  }, refreshTime)
}

// Initialize store based on environment
export function initializeAuthStore() {
  if (typeof window !== 'undefined') {
    // Client-side initialization will happen in the specific app files
  }
}