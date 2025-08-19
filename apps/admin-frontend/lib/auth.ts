'use client'

// Export server functions for compatibility
export { getServerSession, getAuthUser } from './server/auth';

import { create } from 'zustand'

export interface User {
  id: string
  email: string
  name?: string
  role: string
  permissions: string[]
  admin_modules: string[]
  package_tier: string
  firebase_uid?: string
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
  
  // Permission checks
  can: (permission: string) => boolean
  hasRole: (role: string) => boolean
  hasModule: (module: string) => boolean
  hasTier: (tier: string) => boolean
}

// Create auth store without persistence (relies on server-side cookies)
export const useAuth = create<AuthState>((set, get) => ({
  user: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  expiresAt: null,

  login: () => {
    // Redirect to OAuth signin endpoint
    const currentUrl = window.location.href
    const callbackUrl = encodeURIComponent(currentUrl)
    window.location.href = `/api/auth/signin/epsx-backend?callbackUrl=${callbackUrl}`
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
        role: data.user.role,
        permissions: data.user.permissions || [],
        admin_modules: data.user.admin_modules || [],
        package_tier: data.user.package_tier || 'FREE',
        firebase_uid: data.user.firebase_uid,
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
    
    // Check exact match
    if (user.permissions.includes(permission)) {
      return true
    }
    
    // Check wildcard permissions
    return user.permissions.some(p => {
      if (p.endsWith('*')) {
        const prefix = p.slice(0, -1)
        return permission.startsWith(prefix)
      }
      return false
    })
  },

  hasRole: (role: string) => {
    const { user } = get()
    if (!user) return false
    
    const roleHierarchy = {
      user: 1,
      premium: 2,
      moderator: 3,
      admin: 4,
      super_admin: 5,
    }
    
    const userLevel = roleHierarchy[user.role as keyof typeof roleHierarchy] || 0
    const requiredLevel = roleHierarchy[role as keyof typeof roleHierarchy] || 1
    
    return userLevel >= requiredLevel
  },

  hasModule: (module: string) => {
    const { user } = get()
    if (!user) return false
    
    // Check if user has the specific admin module
    return user.admin_modules.includes(module)
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
    
    const userLevel = tierHierarchy[user.package_tier as keyof typeof tierHierarchy] || 0
    const requiredLevel = tierHierarchy[tier as keyof typeof tierHierarchy] || 1
    
    return userLevel >= requiredLevel
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

// Helper functions for components
export function checkPermission(permission: string): boolean {
  return useAuth.getState().can(permission)
}

export function checkRole(role: string): boolean {
  return useAuth.getState().hasRole(role)
}

export function checkModule(module: string): boolean {
  return useAuth.getState().hasModule(module)
}

export function checkTier(tier: string): boolean {
  return useAuth.getState().hasTier(tier)
}

// Admin module helpers
export function useAdminModules(requiredModule?: string) {
  const { user } = useAuth.getState()
  const userModules = user?.admin_modules || []
  
  const hasRequiredModule = requiredModule ? 
    useAuth.getState().hasModule(requiredModule) : 
    true
  
  return {
    userModules,
    hasRequiredModule,
    hasUserManagement: useAuth.getState().hasModule('user_management'),
    hasRoleManagement: useAuth.getState().hasModule('role_management'),
    hasPermissionManagement: useAuth.getState().hasModule('permission_management'),
    hasAnalytics: useAuth.getState().hasModule('analytics'),
    hasBilling: useAuth.getState().hasModule('billing'),
    hasSystemConfig: useAuth.getState().hasModule('system_config'),
  }
}

// Utility Functions for Admin UI
export function getUserDisplayName(user: User | null): string {
  if (!user) return 'Unknown User'
  return user.name || user.email.split('@')[0] || 'Admin User'
}

export function formatAdminRole(role: string): string {
  const roleLabels: Record<string, string> = {
    'moderator': 'Moderator',
    'admin': 'Administrator',
    'super_admin': 'Super Administrator'
  }
  
  return roleLabels[role] || role
}

export function getAdminModuleLabels(modules: string[]): string[] {
  const moduleLabels: Record<string, string> = {
    'user_management': 'User Management',
    'role_management': 'Role Management', 
    'permission_management': 'Permission Management',
    'analytics': 'Analytics & Reports',
    'billing': 'Billing & Subscriptions',
    'system_config': 'System Configuration',
    'audit_logs': 'Audit Logs',
    'notifications': 'Notifications'
  }
  
  return modules.map(module => moduleLabels[module] || module)
}

// Check if user has admin access at all
export function hasAdminAccess(): boolean {
  const { user } = useAuth.getState()
  if (!user) return false
  
  // Must have admin role and at least one admin module
  return useAuth.getState().hasRole('moderator') && user.admin_modules.length > 0
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
    scope: 'openid profile email admin_modules',
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
export function signIn(callbackUrl?: string) {
  const redirectPath = callbackUrl ? `?callbackUrl=${encodeURIComponent(callbackUrl)}` : ''
  window.location.href = `/api/auth/signin/epsx-backend${redirectPath}`
}