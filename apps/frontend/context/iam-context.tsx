'use client';

import React, { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { type User } from 'firebase/auth';
import { 
  authService, 
  iamService, 
  type UserPermissions, 
  type UserRole,
  type PermissionCheck 
} from '@/lib/firebase-iam';

interface IAMContextType {
  user: User | null;
  permissions: UserPermissions | null;
  loading: boolean;
  error: string | null;
  signIn: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string, role?: string) => Promise<void>;
  signOut: () => Promise<void>;
  checkPermission: (permission: string) => Promise<boolean>;
  checkPermissions: (permissions: string[]) => Promise<Record<string, boolean>>;
  hasRole: (role: string) => boolean;
  hasAnyRole: (roles: string[]) => boolean;
  refreshPermissions: () => Promise<void>;
  availableRoles: UserRole[];
}

const IAMContext = createContext<IAMContextType | undefined>(undefined);

interface IAMProviderProps {
  children: ReactNode;
}

export function IAMProvider({ children }: IAMProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [permissions, setPermissions] = useState<UserPermissions | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unsubscribe = authService.onAuthStateChanged(async (firebaseUser) => {
      setLoading(true);
      setError(null);

      if (firebaseUser) {
        setUser(firebaseUser);
        try {
          const userPermissions = await authService.getCurrentUserPermissions();
          setPermissions(userPermissions);
        } catch (err) {
          console.error('Error fetching user permissions:', err);
          setError('Failed to load user permissions');
        }
      } else {
        setUser(null);
        setPermissions(null);
      }

      setLoading(false);
    });

    return unsubscribe;
  }, []);

  const signIn = async (email: string, password: string) => {
    try {
      setLoading(true);
      setError(null);
      const { user: firebaseUser, permissions: userPermissions } = await authService.signIn(email, password);
      setUser(firebaseUser);
      setPermissions(userPermissions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign in failed');
      throw err;
    } finally {
      setLoading(false);
    }
  };

  const signUp = async (email: string, password: string, role: string = 'user') => {
    try {
      setLoading(true);
      setError(null);
      const { user: firebaseUser, permissions: userPermissions } = await authService.signUp(email, password, role);
      setUser(firebaseUser);
      setPermissions(userPermissions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign up failed');
      throw err;
    } finally {
      setLoading(false);
    }
  };

  const signOut = async () => {
    try {
      setLoading(true);
      setError(null);
      await authService.signOut();
      setUser(null);
      setPermissions(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Sign out failed');
      throw err;
    } finally {
      setLoading(false);
    }
  };

  const checkPermission = async (permission: string): Promise<boolean> => {
    if (!user || !permissions) return false;
    
    try {
      const check = await iamService.checkPermission(user.uid, permission);
      return check.hasPermission;
    } catch (err) {
      console.error('Error checking permission:', err);
      return false;
    }
  };

  const checkPermissions = async (permissionsToCheck: string[]): Promise<Record<string, boolean>> => {
    if (!user || !permissions) {
      return permissionsToCheck.reduce((acc, perm) => ({ ...acc, [perm]: false }), {});
    }
    
    try {
      return await iamService.checkPermissions(user.uid, permissionsToCheck);
    } catch (err) {
      console.error('Error checking permissions:', err);
      return permissionsToCheck.reduce((acc, perm) => ({ ...acc, [perm]: false }), {});
    }
  };

  const hasRole = (role: string): boolean => {
    return permissions?.role === role;
  };

  const hasAnyRole = (roles: string[]): boolean => {
    return permissions ? roles.includes(permissions.role) : false;
  };

  const refreshPermissions = async () => {
    if (!user) return;
    
    try {
      const userPermissions = await authService.getCurrentUserPermissions();
      setPermissions(userPermissions);
    } catch (err) {
      console.error('Error refreshing permissions:', err);
      setError('Failed to refresh permissions');
    }
  };

  const availableRoles = iamService.getAvailableRoles();

  const value: IAMContextType = {
    user,
    permissions,
    loading,
    error,
    signIn,
    signUp,
    signOut,
    checkPermission,
    checkPermissions,
    hasRole,
    hasAnyRole,
    refreshPermissions,
    availableRoles
  };

  return <IAMContext.Provider value={value}>{children}</IAMContext.Provider>;
}

export function useIAM() {
  const context = useContext(IAMContext);
  if (context === undefined) {
    throw new Error('useIAM must be used within an IAMProvider');
  }
  return context;
}

// Hook for permission-based rendering
export function usePermission(permission: string) {
  const { checkPermission, loading } = useIAM();
  const [hasPermission, setHasPermission] = useState<boolean>(false);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    const check = async () => {
      setChecking(true);
      const result = await checkPermission(permission);
      setHasPermission(result);
      setChecking(false);
    };

    check();
  }, [permission, checkPermission]);

  return { hasPermission, loading: loading || checking };
}

// Hook for role-based rendering
export function useRole(role: string) {
  const { hasRole, loading } = useIAM();
  return { hasRole: hasRole(role), loading };
}

// Hook for multiple roles
export function useRoles(roles: string[]) {
  const { hasAnyRole, loading } = useIAM();
  return { hasAnyRole: hasAnyRole(roles), loading };
}

// Component for permission-based rendering
interface PermissionGateProps {
  permission: string;
  fallback?: ReactNode;
  children: ReactNode;
}

export function PermissionGate({ permission, fallback = null, children }: PermissionGateProps) {
  const { hasPermission, loading } = usePermission(permission);

  if (loading) {
    return <div className="animate-pulse bg-gray-200 rounded h-4 w-full" />;
  }

  return hasPermission ? <>{children}</> : <>{fallback}</>;
}

// Component for role-based rendering
interface RoleGateProps {
  role: string;
  fallback?: ReactNode;
  children: ReactNode;
}

export function RoleGate({ role, fallback = null, children }: RoleGateProps) {
  const { hasRole, loading } = useRole(role);

  if (loading) {
    return <div className="animate-pulse bg-gray-200 rounded h-4 w-full" />;
  }

  return hasRole ? <>{children}</> : <>{fallback}</>;
}

// Component for multiple roles
interface RolesGateProps {
  roles: string[];
  fallback?: ReactNode;
  children: ReactNode;
}

export function RolesGate({ roles, fallback = null, children }: RolesGateProps) {
  const { hasAnyRole, loading } = useRoles(roles);

  if (loading) {
    return <div className="animate-pulse bg-gray-200 rounded h-4 w-full" />;
  }

  return hasAnyRole ? <>{children}</> : <>{fallback}</>;
}
