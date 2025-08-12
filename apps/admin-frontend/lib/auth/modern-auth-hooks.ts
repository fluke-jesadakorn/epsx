/**
 * Modern Authentication Hooks for Client Components
 * Pure admin module system - no legacy role support
 */

'use client'

import { useEffect, useState } from 'react'
import { ModernAuthUser, AdminModule, ADMIN_MODULES } from './modern-auth-service'

// Client-side auth state
interface AuthState {
  user: ModernAuthUser | null
  loading: boolean
  error: string | null
}

/**
 * Main authentication hook - provides current user and auth state
 */
export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>({
    user: null,
    loading: true,
    error: null
  })

  const fetchUser = async () => {
    try {
      setAuthState(prev => ({ ...prev, loading: true, error: null }))
      
      const response = await fetch('/api/auth/me')
      
      if (response.ok) {
        const userData = await response.json()
        setAuthState({
          user: userData,
          loading: false,
          error: null
        })
      } else if (response.status === 401) {
        setAuthState({
          user: null,
          loading: false,
          error: null
        })
      } else {
        throw new Error('Failed to fetch user')
      }
    } catch (error) {
      setAuthState({
        user: null,
        loading: false,
        error: error instanceof Error ? error.message : 'Authentication error'
      })
    }
  }

  useEffect(() => {
    fetchUser()
  }, [])

  const login = async (email: string, password: string) => {
    try {
      const response = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
      })

      if (response.ok) {
        await fetchUser() // Refresh user data after login
        return { success: true }
      } else {
        const error = await response.text()
        return { success: false, error }
      }
    } catch (error) {
      return { 
        success: false, 
        error: error instanceof Error ? error.message : 'Login failed' 
      }
    }
  }

  const logout = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' })
    } finally {
      setAuthState({
        user: null,
        loading: false,
        error: null
      })
      window.location.href = '/login'
    }
  }

  return {
    ...authState,
    login,
    logout,
    refresh: fetchUser,
    isAuthenticated: !!authState.user,
    isAdmin: authState.user?.admin || false,
    adminModules: authState.user?.admin_modules || [],
    permissions: authState.user?.permissions || [],
    accessLevel: authState.user?.access_level || 'none'
  }
}

/**
 * Hook for admin module checks
 */
export function useAdminModule(module: AdminModule) {
  const { user, loading } = useAuth()
  
  return {
    hasModule: user?.admin_modules.includes(module) || false,
    loading,
    isAdmin: user?.admin || false
  }
}

/**
 * Hook for permission checks
 */
export function usePermission(permission: string) {
  const { user, loading } = useAuth()
  
  return {
    hasPermission: user?.permissions.includes(permission) || false,
    loading,
    permissions: user?.permissions || []
  }
}

/**
 * Hook for capability checks
 */
export function useCapabilities() {
  const { user, loading } = useAuth()
  
  if (!user) {
    return {
      loading,
      canManageUsers: false,
      canViewAnalytics: false,
      canManageBilling: false,
      canManagePermissions: false,
      canManageSystem: false
    }
  }

  const hasModule = (module: AdminModule) => user.admin_modules.includes(module)
  const hasPermission = (permission: string) => user.permissions.includes(permission)

  return {
    loading,
    canManageUsers: hasModule(ADMIN_MODULES.USER_OPERATIONS) || 
                    hasModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
                    hasPermission('user:write'),
    
    canViewAnalytics: hasModule(ADMIN_MODULES.ANALYTICS_SPECIALIST) || 
                      hasModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
                      hasPermission('analytics:read'),
    
    canManageBilling: hasModule(ADMIN_MODULES.BILLING_ADMIN) || 
                      hasModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
                      hasPermission('billing:write'),
    
    canManagePermissions: hasModule(ADMIN_MODULES.PERMISSION_ADMIN) || 
                          hasModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
                          hasPermission('permission:write'),
    
    canManageSystem: hasModule(ADMIN_MODULES.SYSTEM_ADMIN) ||
                     hasPermission('system:admin')
  }
}

/**
 * Hook to require authentication (redirects if not authenticated)
 */
export function useRequireAuth() {
  const auth = useAuth()
  
  useEffect(() => {
    if (!auth.loading && !auth.isAuthenticated) {
      window.location.href = '/login'
    }
  }, [auth.loading, auth.isAuthenticated])
  
  return auth
}

/**
 * Hook to require admin access (redirects if not admin)
 */
export function useRequireAdmin() {
  const auth = useAuth()
  
  useEffect(() => {
    if (!auth.loading && (!auth.isAuthenticated || !auth.isAdmin)) {
      window.location.href = auth.isAuthenticated ? '/unauthorized' : '/login'
    }
  }, [auth.loading, auth.isAuthenticated, auth.isAdmin])
  
  return auth
}

/**
 * Hook to require specific admin module
 */
export function useRequireAdminModule(module: AdminModule) {
  const auth = useAuth()
  const hasModule = auth.adminModules.includes(module)
  
  useEffect(() => {
    if (!auth.loading) {
      if (!auth.isAuthenticated) {
        window.location.href = '/login'
      } else if (!hasModule) {
        window.location.href = `/access-denied?required_module=${module}`
      }
    }
  }, [auth.loading, auth.isAuthenticated, hasModule, module])
  
  return { ...auth, hasModule }
}

/**
 * Modern authentication context provider (for client components)
 */
export function AuthProvider({ children }: { children: React.ReactNode }) {
  // This is a simple provider - you could enhance this with React Context
  // if you need to share auth state across many components
  return <>{children}</>
}

/**
 * Higher-order component for authentication
 */
export function withAuth<P extends object>(Component: React.ComponentType<P>) {
  return function AuthenticatedComponent(props: P) {
    const auth = useRequireAuth()
    
    if (auth.loading) {
      return <div>Loading...</div>
    }
    
    if (!auth.isAuthenticated) {
      return null // Will redirect in useRequireAuth
    }
    
    return <Component {...props} />
  }
}

/**
 * Higher-order component for admin authentication
 */
export function withAdminAuth<P extends object>(Component: React.ComponentType<P>) {
  return function AdminAuthenticatedComponent(props: P) {
    const auth = useRequireAdmin()
    
    if (auth.loading) {
      return <div>Loading...</div>
    }
    
    if (!auth.isAuthenticated || !auth.isAdmin) {
      return null // Will redirect in useRequireAdmin
    }
    
    return <Component {...props} />
  }
}

/**
 * Higher-order component for specific admin module
 */
export function withAdminModule<P extends object>(module: AdminModule) {
  return function(Component: React.ComponentType<P>) {
    return function ModuleAuthenticatedComponent(props: P) {
      const auth = useRequireAdminModule(module)
      
      if (auth.loading) {
        return <div>Loading...</div>
      }
      
      if (!auth.hasModule) {
        return null // Will redirect in useRequireAdminModule
      }
      
      return <Component {...props} />
    }
  }
}