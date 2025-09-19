'use client'

import { create } from 'zustand'
import { 
  User, 
  AuthState
} from '../../../../shared/types/auth'
import { authLogger, devLog, safeError } from '@/lib/utils/logging'
import { getBackendUrl, getFrontendUrl, oidcUrls, callbackUrls } from '../../../../shared/utils/url-resolver'

// Re-export types for compatibility
export type { User, AuthState } from '../../../../shared/types/auth'

// Create auth store with enhanced features
export const useAuth = create<AuthState>((set, get) => ({
  user: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  expiresAt: null,
  
  // Auto-refresh state
  autoRefreshEnabled: true,
  refreshInProgress: false,
  lastRefreshTime: null,

  // Permission check methods
  can: (permission: string): boolean => {
    const state = get()
    return state.user?.permissions.includes(permission) || false
  },

  hasAnyPermission: (permissions: string[]): boolean => {
    const state = get()
    if (!state.user?.permissions) return false
    return permissions.some(perm => state.user!.permissions.includes(perm))
  },

  hasAllPermissions: (permissions: string[]): boolean => {
    const state = get()
    if (!state.user?.permissions) return false
    return permissions.every(perm => state.user!.permissions.includes(perm))
  },

  hasTier: (tier: string): boolean => {
    const state = get()
    return state.user?.tier === tier || false
  },

  // Platform management
  switchPlatform: async (platform: string): Promise<void> => {
    try {
      const response = await fetch('/api/auth/switch-platform', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ platform })
      })

      if (!response.ok) {
        throw new Error('Failed to switch platform')
      }

      const userData = await response.json()
      set({ user: userData })
    } catch (error) {
      authLogger.error('Platform switch failed', { error: safeError(error).message })
      throw error
    }
  },

  getCurrentPlatform: (): string => {
    const state = get()
    return state.user?.platform_context || 'epsx'
  },

  getAvailablePlatforms: (): string[] => {
    const state = get()
    if (!state.user?.permissions) return ['epsx']
    
    const platforms = new Set(['epsx'])
    state.user.permissions.forEach(perm => {
      const [platform] = perm.split(':')
      if (platform && platform !== 'epsx') {
        platforms.add(platform)
      }
    })
    return Array.from(platforms)
  },

  canAccessPlatform: (platform: string): boolean => {
    const state = get()
    if (!state.user?.permissions) return false
    return state.user.permissions.some(perm => perm.startsWith(`${platform}:`))
  },

  login: async () => {
    try {
      const currentUrl = window.location.href
      
      devLog('Initiating OAuth login with PKCE...')
      
      // Get CSRF token first
      const csrfResponse = await fetch('/api/auth/csrf', {
        credentials: 'include'
      })
      const { csrfToken } = await csrfResponse.json()
      
      const response = await fetch('/api/auth/initiate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': csrfToken,
        },
        body: JSON.stringify({
          redirectTo: currentUrl
        }),
        credentials: 'include'
      })

      const data = await response.json()

      if (!response.ok) {
        throw new Error(data.error || 'Failed to initiate authentication')
      }

      authLogger.info('OAuth PKCE flow initiated', { 
        authUrl: data.authUrl,
        state: data.state?.substring(0, 8) + '...'
      })

      // Redirect to authorization server
      window.location.href = data.authUrl

    } catch (error) {
      const errorMsg = safeError(error).message
      authLogger.error('Login failed', { error: errorMsg })
      
      set({ 
        error: errorMsg, 
        isLoading: false 
      })
    }
  },

  logout: async () => {
    try {
      set({ isLoading: true, error: null })

      devLog('Initiating logout...')
      
      // Get CSRF token first
      const csrfResponse = await fetch('/api/auth/csrf', {
        credentials: 'include'
      })
      const { csrfToken } = await csrfResponse.json()
      
      // Call backend logout first to invalidate tokens
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json',
          'X-CSRF-Token': csrfToken,
        }
      })

      // Clear local state
      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: null,
        expiresAt: null,
        refreshInProgress: false,
        lastRefreshTime: null
      })

      authLogger.info('User logged out successfully')

      // Redirect to home
      window.location.href = '/'

    } catch (error) {
      const errorMsg = safeError(error).message
      authLogger.error('Logout failed', { error: errorMsg })
      
      set({ 
        error: errorMsg,
        isLoading: false 
      })
    }
  },

  getUser: async (): Promise<User | null> => {
    try {
      const response = await fetch('/api/auth/user', {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        return null
      }

      const userData = await response.json()
      return userData
    } catch (error) {
      authLogger.error('Failed to get user', { error: safeError(error).message })
      return null
    }
  },

  loadUser: async () => {
    try {
      set({ isLoading: true, error: null })

      const response = await fetch('/api/auth/user', {
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        if (response.status === 401) {
          // User not authenticated
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            error: null
          })
          return
        }
        throw new Error('Failed to load user data')
      }

      const userData = await response.json()

      set({
        user: userData,
        isAuthenticated: true,
        isLoading: false,
        error: null,
        expiresAt: userData.expiresAt,
        lastRefreshTime: Date.now()
      })

      authLogger.info('User data loaded successfully', { 
        userId: userData.id,
        email: userData.email 
      })

    } catch (error) {
      const errorMsg = safeError(error).message
      authLogger.error('Failed to load user', { error: errorMsg })
      
      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: errorMsg
      })
    }
  },

  refreshSession: async (): Promise<void> => {
    await get().refreshToken()
  },

  refreshToken: async () => {
    const state = get()
    
    if (state.refreshInProgress) {
      devLog('Token refresh already in progress, skipping...')
      return false
    }

    try {
      set({ refreshInProgress: true, error: null })

      const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        if (response.status === 401) {
          // Refresh token expired, logout user
          devLog('Refresh token expired, logging out...')
          get().logout()
          return false
        }
        throw new Error('Token refresh failed')
      }

      const userData = await response.json()

      set({
        user: userData,
        isAuthenticated: true,
        refreshInProgress: false,
        error: null,
        expiresAt: userData.expiresAt,
        lastRefreshTime: Date.now()
      })

      authLogger.info('Token refreshed successfully')
      return true

    } catch (error) {
      const errorMsg = safeError(error).message
      authLogger.error('Token refresh failed', { error: errorMsg })
      
      set({
        refreshInProgress: false,
        error: errorMsg
      })
      
      return false
    }
  },

  clearError: () => set({ error: null }),

  setAutoRefresh: (enabled: boolean) => set({ autoRefreshEnabled: enabled }),

  // Auto-refresh management methods
  enableAutoRefresh: () => set({ autoRefreshEnabled: true }),
  
  disableAutoRefresh: () => set({ autoRefreshEnabled: false }),

  checkTokenHealth: (): boolean => {
    const state = get()
    if (!state.isAuthenticated || !state.expiresAt) return false
    
    const timeUntilExpiry = state.expiresAt - Date.now()
    return timeUntilExpiry > 5 * 60 * 1000 // Token is healthy if > 5 minutes left
  }
}))

// Auto-refresh functionality
let refreshInterval: NodeJS.Timeout | null = null

const startAutoRefresh = () => {
  if (refreshInterval) return

  refreshInterval = setInterval(() => {
    const state = useAuth.getState()
    
    if (!state.isAuthenticated || !state.autoRefreshEnabled || !state.expiresAt) {
      return
    }

    // Refresh 5 minutes before expiry
    const timeUntilExpiry = state.expiresAt - Date.now()
    const shouldRefresh = timeUntilExpiry < 5 * 60 * 1000 // 5 minutes

    if (shouldRefresh && !state.refreshInProgress) {
      devLog('Auto-refreshing token...')
      state.refreshToken()
    }
  }, 60000) // Check every minute
}

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

// Subscribe to auth state changes to manage auto-refresh
useAuth.subscribe((state) => {
  if (state.isAuthenticated && state.autoRefreshEnabled) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
})

// Export helper hooks
export const useAuthUser = () => useAuth(state => state.user)
export const useAuthLoading = () => useAuth(state => state.isLoading)
export const useAuthError = () => useAuth(state => state.error)
export const useIsAuthenticated = () => useAuth(state => state.isAuthenticated)