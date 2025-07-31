import { useCallback, useMemo } from 'react';
import type { 
  AuthenticatedUser, 
  FrontendUser, 
  AdminUser
} from '@epsx/types';

export interface AuthContextValue<TUser extends AuthenticatedUser = AuthenticatedUser> {
  user: TUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  permissions: string[];
  roles: string[];
  profiles: string[];
}

export interface AuthActions {
  login: (credentials: { email: string; password: string }) => Promise<void>;
  logout: () => Promise<void>;
  signIn: (credentials: { email: string; password: string }) => Promise<void>;
  signOut: () => Promise<void>;
  refreshPermissions: () => Promise<void>;
}

export interface UseAuthOptions {
  /** Context type for type safety */
  contextType?: 'admin' | 'user';
  /** Auto-refresh permissions on mount */
  autoRefresh?: boolean;
}

/**
 * Unified authentication hook that works with both admin and user contexts
 * Provides type-safe access to auth state and actions
 */
export function useAuth<TUser extends AuthenticatedUser = AuthenticatedUser>(
  context: AuthContextValue<TUser> & AuthActions,
  options: UseAuthOptions = {}
) {
  const { contextType = 'user', autoRefresh = false } = options;

  // Memoized computed values
  const computed = useMemo(() => {
    const { user, permissions = [], roles = [], profiles = [] } = context;
    
    return {
      // User type guards
      isAdmin: roles.includes('admin') || roles.includes('super_admin'),
      isSuperAdmin: roles.includes('super_admin'),
      isPremium: roles.includes('premium') || roles.includes('admin'),
      
      // Permission helpers
      hasAnyPermission: (requiredPermissions: string[]) => 
        requiredPermissions.some(permission => permissions.includes(permission)),
      
      hasAllPermissions: (requiredPermissions: string[]) =>
        requiredPermissions.every(permission => permissions.includes(permission)),
      
      hasPermission: (permission: string) => {
        // Handle wildcard permissions
        if (permission.endsWith('.*')) {
          const basePermission = permission.slice(0, -2);
          return permissions.some(p => p.startsWith(basePermission));
        }
        
        if (permission.endsWith(':*')) {
          const basePermission = permission.slice(0, -2);
          return permissions.some(p => p.startsWith(basePermission + ':'));
        }
        
        return permissions.includes(permission);
      },
      
      // Role helpers
      hasRole: (role: string) => roles.includes(role),
      hasAnyRole: (requiredRoles: string[]) => 
        requiredRoles.some(role => roles.includes(role)),
      
      // Profile helpers
      hasProfile: (profile: string) => profiles.includes(profile),
      hasAnyProfile: (requiredProfiles: string[]) =>
        requiredProfiles.some(profile => profiles.includes(profile)),
      
      // Route access helper
      canAccessRoute: (routePermissions?: string[], routeRoles?: string[]) => {
        if (!user) return false;
        
        if (routePermissions?.length) {
          return computed.hasAnyPermission(routePermissions);
        }
        
        if (routeRoles?.length) {
          return computed.hasAnyRole(routeRoles);
        }
        
        return true; // No restrictions
      },
      
      // Admin-specific helpers (when contextType is 'admin')
      ...(contextType === 'admin' && {
        canManageUsers: () => 
          computed.hasPermission('admin.users.*') || computed.hasRole('super_admin'),
        canViewPayments: () =>
          computed.hasPermission('admin.payments.*') || computed.hasRole('admin'),
        canManageSystem: () =>
          computed.hasPermission('admin.system.*') || computed.hasRole('super_admin'),
        canViewAnalytics: () =>
          computed.hasPermission('admin.analytics.*') || computed.hasRole('admin'),
      }),
      
      // User-specific helpers (when contextType is 'user')
      ...(contextType === 'user' && {
        canAccessRankings: () => !!user, // Simple authenticated check
        getSubscriptionTier: () => (user as FrontendUser)?.subscription_tier || 'free',
        getTokenBalance: () => (user as FrontendUser)?.token_balance || 0,
      }),
    };
  }, [context.user, context.permissions, context.roles, context.profiles, contextType]);

  // Memoized action wrappers with error handling
  const actions = useMemo(() => ({
    login: useCallback(async (credentials: { email: string; password: string }) => {
      try {
        await context.login(credentials);
        if (autoRefresh) {
          await context.refreshPermissions?.();
        }
      } catch (error) {
        console.error(`${contextType} login failed:`, error);
        throw error;
      }
    }, [context.login, context.refreshPermissions, contextType, autoRefresh]),

    logout: useCallback(async () => {
      try {
        await context.logout();
      } catch (error) {
        console.error(`${contextType} logout failed:`, error);
        throw error;
      }
    }, [context.logout, contextType]),

    signIn: useCallback(async (credentials: { email: string; password: string }) => {
      try {
        await (context.signIn || context.login)(credentials);
        if (autoRefresh) {
          await context.refreshPermissions?.();
        }
      } catch (error) {
        console.error(`${contextType} signIn failed:`, error);
        throw error;
      }
    }, [context.signIn, context.login, context.refreshPermissions, contextType, autoRefresh]),

    signOut: useCallback(async () => {
      try {
        await (context.signOut || context.logout)();
      } catch (error) {
        console.error(`${contextType} signOut failed:`, error);
        throw error;
      }
    }, [context.signOut, context.logout, contextType]),

    refreshPermissions: useCallback(async () => {
      try {
        await context.refreshPermissions?.();
      } catch (error) {
        console.error(`${contextType} refresh permissions failed:`, error);
        throw error;
      }
    }, [context.refreshPermissions, contextType]),
  }), [context, contextType, autoRefresh]);

  return {
    // Core state
    user: context.user,
    isAuthenticated: context.isAuthenticated,
    isLoading: context.isLoading,
    permissions: context.permissions,
    roles: context.roles,
    profiles: context.profiles,
    
    // Computed helpers
    ...computed,
    
    // Actions
    ...actions,
  };
}

// Type-specific hooks for better developer experience
export function useAdminAuth(context: AuthContextValue<AdminUser> & AuthActions) {
  return useAuth(context, { contextType: 'admin', autoRefresh: true });
}

export function useUserAuth(context: AuthContextValue<FrontendUser> & AuthActions) {
  return useAuth(context, { contextType: 'user', autoRefresh: false });
}