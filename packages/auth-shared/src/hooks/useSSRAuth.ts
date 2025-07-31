import { useEffect, useState, useMemo } from 'react';
import type { AuthenticatedUser } from '@epsx/types';

export interface SSRAuthState<TUser extends AuthenticatedUser = AuthenticatedUser> {
  user: TUser | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isHydrated: boolean;
  permissions: string[];
  roles: string[];
  profiles: string[];
}

export interface UseSSRAuthOptions {
  /** Skip hydration check (for server-only contexts) */
  skipHydration?: boolean;
  /** Initial server-side user data */
  initialUser?: AuthenticatedUser | null;
  /** Initial server-side permissions */
  initialPermissions?: string[];
  /** Initial server-side roles */
  initialRoles?: string[];
  /** Initial server-side profiles */
  initialProfiles?: string[];
}

/**
 * SSR-compatible authentication hook
 * Handles hydration, prevents hydration mismatches, and provides consistent auth state
 */
export function useSSRAuth<TUser extends AuthenticatedUser = AuthenticatedUser>(
  clientAuthState: SSRAuthState<TUser>,
  options: UseSSRAuthOptions = {}
) {
  const {
    skipHydration = false,
    initialUser,
    initialPermissions = [],
    initialRoles = [],
    initialProfiles = [],
  } = options;

  // Hydration state
  const [isHydrated, setIsHydrated] = useState(skipHydration);
  
  // Server-side state (used until hydration)
  const [serverState] = useState({
    user: initialUser,
    permissions: initialPermissions,
    roles: initialRoles,
    profiles: initialProfiles,
  });

  // Handle hydration
  useEffect(() => {
    if (!skipHydration && !isHydrated) {
      setIsHydrated(true);
    }
  }, [skipHydration, isHydrated]);

  // Determine which state to use based on hydration status
  const activeState = useMemo(() => {
    if (!isHydrated) {
      // Use server-side state before hydration
      return {
        user: serverState.user as TUser | null,
        isAuthenticated: !!serverState.user,
        isLoading: false,
        isHydrated: false,
        permissions: serverState.permissions,
        roles: serverState.roles,
        profiles: serverState.profiles,
      };
    }

    // Use client state after hydration
    return {
      ...clientAuthState,
      isHydrated: true,
    };
  }, [isHydrated, serverState, clientAuthState]);

  // SSR-safe permission checking
  const hasPermission = useMemo(() => 
    (permission: string) => {
      const { permissions } = activeState;
      
      if (!permissions.length) return false;
      
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
    }, [activeState]);

  // SSR-safe role checking
  const hasRole = useMemo(() => 
    (role: string) => {
      return activeState.roles.includes(role);
    }, [activeState.roles]);

  // SSR-safe profile checking
  const hasProfile = useMemo(() => 
    (profile: string) => {
      return activeState.profiles.includes(profile);
    }, [activeState.profiles]);

  // Multiple permission checking
  const hasAnyPermission = useMemo(() => 
    (requiredPermissions: string[]) => {
      return requiredPermissions.some(permission => hasPermission(permission));
    }, [hasPermission]);

  const hasAllPermissions = useMemo(() => 
    (requiredPermissions: string[]) => {
      return requiredPermissions.every(permission => hasPermission(permission));
    }, [hasPermission]);

  // Multiple role checking
  const hasAnyRole = useMemo(() => 
    (requiredRoles: string[]) => {
      return requiredRoles.some(role => hasRole(role));
    }, [hasRole]);

  // Route access checking (SSR-safe)
  const canAccessRoute = useMemo(() => 
    (routeConfig: {
      permissions?: string[];
      roles?: string[];
      profiles?: string[];
      requireAuth?: boolean;
    } = {}) => {
      const {
        permissions: routePermissions,
        roles: routeRoles,
        profiles: routeProfiles,
        requireAuth = true,
      } = routeConfig;

      // Check authentication requirement
      if (requireAuth && !activeState.isAuthenticated) {
        return false;
      }

      // Check permissions
      if (routePermissions?.length && !hasAnyPermission(routePermissions)) {
        return false;
      }

      // Check roles
      if (routeRoles?.length && !hasAnyRole(routeRoles)) {
        return false;
      }

      // Check profiles
      if (routeProfiles?.length) {
        const hasRequiredProfile = routeProfiles.some(profile => hasProfile(profile));
        if (!hasRequiredProfile) {
          return false;
        }
      }

      return true;
    }, [activeState.isAuthenticated, hasAnyPermission, hasAnyRole, hasProfile]);

  // Compute derived state
  const derivedState = useMemo(() => ({
    isAdmin: activeState.roles.includes('admin') || activeState.roles.includes('super_admin'),
    isSuperAdmin: activeState.roles.includes('super_admin'),
    isPremium: activeState.roles.includes('premium') || activeState.roles.includes('admin'),
  }), [activeState.roles]);

  return {
    // Core state
    user: activeState.user,
    isAuthenticated: activeState.isAuthenticated,
    isLoading: activeState.isLoading,
    isHydrated: activeState.isHydrated,
    permissions: activeState.permissions,
    roles: activeState.roles,
    profiles: activeState.profiles,
    
    // Derived state
    ...derivedState,
    
    // Permission methods (SSR-safe)
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    
    // Role methods (SSR-safe)
    hasRole,
    hasAnyRole,
    
    // Profile methods (SSR-safe)
    hasProfile,
    
    // Route access (SSR-safe)
    canAccessRoute,
  };
}

// Convenience hooks for specific SSR scenarios
export function useSSRPermission(
  permission: string,
  clientAuthState: SSRAuthState,
  options: UseSSRAuthOptions = {}
) {
  const auth = useSSRAuth(clientAuthState, options);
  return auth.hasPermission(permission);
}

export function useSSRRole(
  role: string,
  clientAuthState: SSRAuthState,
  options: UseSSRAuthOptions = {}
) {
  const auth = useSSRAuth(clientAuthState, options);
  return auth.hasRole(role);
}

export function useSSRAuthActions<TUser extends AuthenticatedUser = AuthenticatedUser>(
  clientAuthState: SSRAuthState<TUser>,
  actions: {
    login: (credentials: { email: string; password: string }) => Promise<void>;
    logout: () => Promise<void>;
    refreshPermissions?: () => Promise<void>;
  },
  options: UseSSRAuthOptions = {}
) {
  const auth = useSSRAuth(clientAuthState, options);
  
  return {
    ...auth,
    
    // Actions (only available after hydration)
    login: auth.isHydrated ? actions.login : undefined,
    logout: auth.isHydrated ? actions.logout : undefined,
    refreshPermissions: (auth.isHydrated && actions.refreshPermissions) ? actions.refreshPermissions : undefined,
  };
}