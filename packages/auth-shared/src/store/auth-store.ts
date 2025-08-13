import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { createUserAbility, type AppAbility } from '../permissions/casl-abilities'
import type { EPSXUser } from '../hooks/useModernAuth'

/**
 * Modern Zustand auth store
 * Replaces complex React context with simple, performant state management
 */

interface AuthState {
  // Core state
  user: EPSXUser | null
  isAuthenticated: boolean
  isLoading: boolean
  
  // Permission ability (computed from user)
  ability: AppAbility | null
  
  // Session data
  sessionToken: string | null
  sessionExpiry: Date | null
  
  // UI state
  showLoginModal: boolean
  showUpgradeModal: boolean
  redirectAfterLogin: string | null
  
  // Error state
  error: string | null
  lastError: Error | null
}

interface AuthActions {
  // User actions
  setUser: (user: EPSXUser | null) => void
  updateUser: (updates: Partial<EPSXUser>) => void
  clearUser: () => void
  
  // Session actions
  setSession: (token: string, expiry?: Date) => void
  clearSession: () => void
  refreshSession: () => Promise<void>
  
  // Permission actions
  updateAbility: () => void
  checkPermission: (action: string, subject: string) => boolean
  
  // UI actions
  setShowLoginModal: (show: boolean) => void
  setShowUpgradeModal: (show: boolean) => void
  setRedirectAfterLogin: (path: string | null) => void
  
  // Error actions
  setError: (error: string | null) => void
  setLastError: (error: Error | null) => void
  clearError: () => void
  
  // Loading actions
  setLoading: (loading: boolean) => void
  
  // Computed getters
  getPackageTier: () => string
  getAdminModules: () => string[]
  getPermissions: () => string[]
  isAdmin: () => boolean
  isSystemAdmin: () => boolean
  
  // Auth flow helpers
  login: (user: EPSXUser, token: string) => void
  logout: () => void
  requireAuth: () => boolean
  requirePermission: (permission: string) => boolean
  requireUpgrade: (tier: string) => boolean
}

type AuthStore = AuthState & AuthActions

/**
 * Create the auth store with Zustand
 */
export const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      // Initial state
      user: null,
      isAuthenticated: false,
      isLoading: false,
      ability: null,
      sessionToken: null,
      sessionExpiry: null,
      showLoginModal: false,
      showUpgradeModal: false,
      redirectAfterLogin: null,
      error: null,
      lastError: null,

      // User actions
      setUser: (user) => {
        set({ user, isAuthenticated: !!user })
        get().updateAbility()
      },

      updateUser: (updates) => {
        const currentUser = get().user
        if (currentUser) {
          const updatedUser = { ...currentUser, ...updates }
          set({ user: updatedUser })
          get().updateAbility()
        }
      },

      clearUser: () => {
        set({ 
          user: null, 
          isAuthenticated: false, 
          ability: null 
        })
      },

      // Session actions
      setSession: (token, expiry) => {
        set({ 
          sessionToken: token, 
          sessionExpiry: expiry || new Date(Date.now() + 2 * 60 * 60 * 1000) // 2 hours default
        })
      },

      clearSession: () => {
        set({ 
          sessionToken: null, 
          sessionExpiry: null 
        })
      },

      refreshSession: async () => {
        // This would integrate with Auth.js session refresh
        // For now, just clear if expired
        const { sessionExpiry } = get()
        if (sessionExpiry && new Date() > sessionExpiry) {
          get().logout()
        }
      },

      // Permission actions
      updateAbility: () => {
        const { user } = get()
        if (user) {
          const ability = createUserAbility({
            permissions: user.permissions,
            admin_modules: user.admin_modules,
            package_tier: user.package_tier,
            role: user.role
          })
          set({ ability })
        } else {
          set({ ability: null })
        }
      },

      checkPermission: (action, subject) => {
        const { ability } = get()
        if (!ability) return false
        
        // Simple permission check using CASL ability
        try {
          return ability.can(action as any, subject as any)
        } catch {
          return false
        }
      },

      // UI actions
      setShowLoginModal: (show) => set({ showLoginModal: show }),
      setShowUpgradeModal: (show) => set({ showUpgradeModal: show }),
      setRedirectAfterLogin: (path) => set({ redirectAfterLogin: path }),

      // Error actions
      setError: (error) => set({ error }),
      setLastError: (error) => set({ lastError: error }),
      clearError: () => set({ error: null, lastError: null }),

      // Loading actions
      setLoading: (loading) => set({ isLoading: loading }),

      // Computed getters
      getPackageTier: () => get().user?.package_tier || 'FREE',
      getAdminModules: () => get().user?.admin_modules || [],
      getPermissions: () => get().user?.permissions || [],
      isAdmin: () => (get().user?.admin_modules?.length || 0) > 0,
      isSystemAdmin: () => get().user?.admin_modules?.includes('system_admin') || false,

      // Auth flow helpers
      login: (user, token) => {
        set({ 
          user, 
          isAuthenticated: true, 
          sessionToken: token,
          sessionExpiry: new Date(Date.now() + 2 * 60 * 60 * 1000),
          error: null 
        })
        get().updateAbility()
      },

      logout: () => {
        set({ 
          user: null, 
          isAuthenticated: false, 
          ability: null,
          sessionToken: null,
          sessionExpiry: null,
          error: null,
          redirectAfterLogin: null
        })
      },

      requireAuth: () => {
        const { isAuthenticated, setShowLoginModal, setRedirectAfterLogin } = get()
        if (!isAuthenticated) {
          setRedirectAfterLogin(window.location.pathname)
          setShowLoginModal(true)
          return false
        }
        return true
      },

      requirePermission: (permission) => {
        const { checkPermission, setError } = get()
        const hasPermission = checkPermission(permission.split(':')[0], permission.split(':')[1])
        if (!hasPermission) {
          setError(`Permission '${permission}' required`)
        }
        return hasPermission
      },

      requireUpgrade: (tier) => {
        const { user, setShowUpgradeModal } = get()
        const currentTier = user?.package_tier || 'FREE'
        
        const tierHierarchy: Record<string, number> = {
          'FREE': 1,
          'BRONZE': 2,
          'SILVER': 3,
          'GOLD': 4,
          'PLATINUM': 5,
          'ENTERPRISE': 6
        }
        
        const currentLevel = tierHierarchy[currentTier] || 0
        const requiredLevel = tierHierarchy[tier] || 1
        
        if (currentLevel < requiredLevel) {
          setShowUpgradeModal(true)
          return false
        }
        return true
      }
    }),
    {
      name: 'epsx-auth-store',
      // Only persist essential data, not computed values
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
        sessionToken: state.sessionToken,
        sessionExpiry: state.sessionExpiry,
      }),
    }
  )
)

/**
 * Auth store selectors for performance optimization
 */
export const authSelectors = {
  // User selectors
  user: (state: AuthStore) => state.user,
  isAuthenticated: (state: AuthStore) => state.isAuthenticated,
  isLoading: (state: AuthStore) => state.isLoading,
  
  // Permission selectors
  ability: (state: AuthStore) => state.ability,
  permissions: (state: AuthStore) => state.user?.permissions || [],
  adminModules: (state: AuthStore) => state.user?.admin_modules || [],
  packageTier: (state: AuthStore) => state.user?.package_tier || 'FREE',
  
  // UI selectors
  showLoginModal: (state: AuthStore) => state.showLoginModal,
  showUpgradeModal: (state: AuthStore) => state.showUpgradeModal,
  error: (state: AuthStore) => state.error,
  
  // Computed selectors
  isAdmin: (state: AuthStore) => state.isAdmin(),
  isSystemAdmin: (state: AuthStore) => state.isSystemAdmin(),
  
  // Permission check selectors
  canReadAnalytics: (state: AuthStore) => state.checkPermission('read', 'Analytics'),
  canManageUsers: (state: AuthStore) => state.checkPermission('manage', 'User'),
  canAccessAdmin: (state: AuthStore) => state.checkPermission('read', 'Admin'),
}

/**
 * Simplified auth hooks using Zustand store
 */
export const useSimpleAuth = () => useAuthStore(authSelectors.user)
export const useAuthStatus = () => useAuthStore(authSelectors.isAuthenticated)
export const useAuthLoading = () => useAuthStore(authSelectors.isLoading)
export const useAuthError = () => useAuthStore(authSelectors.error)
export const useUserPermissions = () => useAuthStore(authSelectors.permissions)
export const useUserAdminModules = () => useAuthStore(authSelectors.adminModules)
export const usePackageTier = () => useAuthStore(authSelectors.packageTier)

/**
 * Auth action hooks
 */
export const useAuthActions = () => {
  const login = useAuthStore(state => state.login)
  const logout = useAuthStore(state => state.logout)
  const setError = useAuthStore(state => state.setError)
  const clearError = useAuthStore(state => state.clearError)
  const requireAuth = useAuthStore(state => state.requireAuth)
  const requirePermission = useAuthStore(state => state.requirePermission)
  const requireUpgrade = useAuthStore(state => state.requireUpgrade)
  
  return {
    login,
    logout,
    setError,
    clearError,
    requireAuth,
    requirePermission,
    requireUpgrade,
  }
}