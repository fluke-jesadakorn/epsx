'use client'

import { create } from 'zustand'
import { isJWTExpired, getJWTTimeToExpiry } from '@epsx/auth-shared'

export interface User {
  id: string
  email: string
  name?: string
  role: string
  permissions: string[]
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
        role: data.user.role,
        permissions: data.user.permissions || [],
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

export function checkRole(role: string): boolean {
  return useAuth.getState().hasRole(role)
}

export function checkTier(tier: string): boolean {
  return useAuth.getState().hasTier(tier)
}

// Package tier helpers for frontend trading platform
export function usePackageTier(requiredTier?: string) {
  const { user } = useAuth.getState()
  const packageTier = user?.package_tier || 'FREE'
  
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

// OIDC Authorization URL generation with PKCE for main frontend
export async function getAuthorizationUrl() {
  // Generate PKCE parameters
  const codeVerifier = generateCodeVerifier()
  const codeChallenge = await generateCodeChallenge(codeVerifier)
  const state = generateRandomString(32)
  
  // Build authorization URL
  const authorizationEndpoint = `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'}/oauth/authorize`
  const clientId = process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || 'epsx-frontend'
  const redirectUri = `${process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'}/api/auth/callback/epsx-backend`
  
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

// Simplified OAuth token exchange for main frontend
export async function exchangeCodeForTokens(code: string, codeVerifier: string, state: string) {
  // In our simplified implementation, the authorization code IS the access token
  return {
    accessToken: code,
    idToken: null, // Not used in simplified flow
    refreshToken: null, // Not used in simplified flow
  }
}

// Simplified user info fetcher for main frontend  
export async function getUserInfo(accessToken: string) {
  const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'
  const response = await fetch(`${apiUrl}/oauth/userinfo`, {
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
  })

  if (!response.ok) {
    throw new Error(`UserInfo fetch failed: ${response.status} ${response.statusText}`)
  }

  return await response.json()
}

// PKCE helper functions for main frontend
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