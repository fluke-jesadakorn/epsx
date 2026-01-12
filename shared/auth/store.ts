/**
 * Shared Authentication Store
 * Core auth logic shared between frontend and admin-frontend
 */

'use client'

import { create } from 'zustand'
import {
  AdminAuthState,
  AuthSessionData,
  AuthState,
  SmartRefreshResponse,
  User
} from '../types/auth'
import { callbackUrls, oidcUrls } from '../utils/url-resolver'

import { createJSONStorage, persist } from 'zustand/middleware'

// Core auth store factory
export function createAuthStore<T extends AuthState>(
  config: {
    clientId: string
    redirectPath: string
    logoutRedirectPath: string
    enableAutoRefresh?: boolean
    enableSmartRefresh?: boolean
    refreshBuffer?: number
    storageKey?: string
  }
) {
  // @ts-ignore - TypeScript has issues with Zustand generic store types
  return create<T>()(
    persist(
      (set: any, get: any) => ({
        user: null,
        isLoading: false,
        isAuthenticated: false,
        isAuthenticating: false,
        hasInitialized: false,
        error: null,
        expiresAt: null,

        // Identity and connection
        walletAddress: undefined,
        isConnected: false,

        // Enterprise Data
        permissions: [],
        enterpriseTier: 'Starter',
        hasApiAccess: false,
        verifiedTokensUsd: 0,
        nftCollections: [],
        daoMemberships: [],

        // Setters
        setConnected: (connected: boolean) => set({ isConnected: connected }),
        setAuthenticated: (authenticated: boolean) => set({ isAuthenticated: authenticated }),
        setAuthenticating: (authenticating: boolean) => set({ isAuthenticating: authenticating }),
        setLoading: (loading: boolean) => set({ isLoading: loading }),
        setInitialized: (initialized: boolean) => set({ hasInitialized: initialized }),
        setWalletAddress: (address: string | undefined) => set({ walletAddress: address }),
        setPermissions: (permissions: string[]) => set({ permissions }),
        setEnterpriseTier: (tier: string) => set({ enterpriseTier: tier }),
        setApiAccess: (hasAccess: boolean) => set({ hasApiAccess: hasAccess }),
        setAccessToken: (token: string | undefined) => set({ accessToken: token }),
        setExpiresAt: (expiresAt: number | undefined) => set({ expiresAt }),
        setError: (error: string | undefined) => set({ error }),

        // Auto-refresh state (optional for admin)
        ...(config.enableAutoRefresh && {
          autoRefreshEnabled: true,
          refreshInProgress: false,
          lastRefreshTime: null,
        }),

        login: async (router?: any) => {
          try {
            const currentUrl = typeof window !== 'undefined' ? window.location.href : ''

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
            if (router) {
              router.push(data.authorizationUrl)
            } else {
              window.location.href = data.authorizationUrl
            }

          } catch (error) {
            console.error('❌ Login initiation failed:', error)
            // Fallback to direct redirect if PKCE initiation fails
            const params = new URLSearchParams({
              client_id: config.clientId,
              response_type: 'code',
              scope: 'openid profile email',
              redirect_uri: config.redirectPath,
              state: Buffer.from(JSON.stringify({ redirectTo: typeof window !== 'undefined' ? window.location.href : '' })).toString('base64url'),
            })

            if (router) {
              router.push(`${oidcUrls.authorize('client')}?${params.toString()}`)
            } else {
              window.location.href = `${oidcUrls.authorize('client')}?${params.toString()}`
            }
          }
        },

        logout: async (router?: any) => {
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
              isLoading: false,
              permissions: [],
              enterpriseTier: 'Starter',
              hasApiAccess: false
            })

            if (router) {
              router.push(config.logoutRedirectPath)
            } else {
              window.location.href = config.logoutRedirectPath
            }
          } catch (error) {
            console.error('❌ Logout failed:', error)
            set({ error: 'Logout failed', isLoading: false })
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
                set({ user: null, isAuthenticated: false, expiresAt: null, isLoading: false })
                return null
              }
              throw new Error(`Session fetch failed: ${response.status}`)
            }

            const data: AuthSessionData = await response.json()

            if (!data.isAuthenticated || !data.user) {
              set({ user: null, isAuthenticated: false, expiresAt: null, isLoading: false })
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
              enterpriseTier: (data.user as any).enterpriseTier || data.user.tier || 'Starter',
              hasApiAccess: (data.user as any).hasApiAccess || false,
            }

            set({
              user: userData,
              isAuthenticated: true,
              expiresAt: data.expiresAt,
              isLoading: false,
              permissions: data.user.permissions || [],
              enterpriseTier: userData.enterpriseTier,
              hasApiAccess: userData.hasApiAccess,
            })

            if (config.enableAutoRefresh && data.expiresAt) {
              setupTokenAutoRefresh(data.expiresAt, config.refreshBuffer || 5000)
            }

            return userData
          } catch (error) {
            console.error('❌ Get user failed:', error)
            set({ user: null, isAuthenticated: false, expiresAt: null, error: 'Failed to load session', isLoading: false })
            return null
          }
        },

        refreshEnterpriseData: async () => {
          const { user } = get()
          if (!user) return

          try {
            const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/users/profile`, {
              credentials: 'include'
            })

            if (response.ok) {
              const profile = await response.json()
              set({
                enterpriseTier: profile.tier || profile.enterprise_tier || 'Starter',
                hasApiAccess: profile.has_api_access || false,
                verifiedTokensUsd: profile.verified_tokens_usd || 0,
                permissions: profile.permissions || get().permissions
              })
            }
          } catch (error) {
            console.error('Failed to refresh enterprise data:', error)
          }
        },

        generateApiKey: async (name: string): Promise<string> => {
          const { user, hasApiAccess, enterpriseTier } = get()
          if (!user || !hasApiAccess) {
            throw new Error('API access not available')
          }

          const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/admin/notifications/send`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              wallet_address: user.id || (user as any).wallet_address,
              name,
              enterprise_tier: enterpriseTier,
            }),
            credentials: 'include',
          })

          if (!response.ok) {
            const error = await response.json()
            throw new Error(error.message || 'Failed to generate API key')
          }

          const { api_key } = await response.json()
          return api_key
        },

        refreshSession: async () => {
          if (config.enableSmartRefresh) {
            return smartRefreshSession(set, get, config.refreshBuffer || 5000)
          } else {
            await get().getUser()
          }
        },

        clearError: () => set({ error: null }),

        ...(config.enableAutoRefresh && {
          autoRefreshEnabled: true,
          refreshInProgress: false,
          lastRefreshTime: null,
          enableAutoRefresh: () => set({ autoRefreshEnabled: true }),
          disableAutoRefresh: () => {
            set({ autoRefreshEnabled: false })
            if (refreshTimeout) {
              clearTimeout(refreshTimeout)
              refreshTimeout = null
            }
          },
          checkTokenHealth: () => {
            const { expiresAt, user } = get()
            if (!expiresAt || !user) return false
            return expiresAt - Date.now() > 10000
          },
        }),
      } as any),
      {
        name: config.storageKey || 'epsx-auth-storage',
        storage: createJSONStorage(() => localStorage),
        partialize: (state: any) => ({
          user: state.user,
          isAuthenticated: state.isAuthenticated,
          expiresAt: state.expiresAt,
          permissions: state.permissions,
          enterpriseTier: state.enterpriseTier,
          hasApiAccess: state.hasApiAccess,
        }),
      }
    )
  )
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

  return baseStore
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