'use client';

import { useEffect, useState, useCallback } from 'react';
import { useAuth } from '@/context/auth-context';
import { UserRole } from '@/types/auth/roles';
import { getUserCustomClaims, type CustomClaims } from '@/lib/custom-claims';

export function useCustomClaims() {
  const { user, loading: authLoading } = useAuth();
  const [claims, setClaims] = useState<CustomClaims | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchClaims = useCallback(async () => {
    if (!user?.user_id) {
      setClaims(null);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);

      // For BackendUser, we get claims from the user object itself
      const customClaims: CustomClaims = {
        role: user.role as any,
        permissions: user.permissions,
        emailVerified: true, // Backend users are considered verified
        createdAt: Date.now(), // Fallback timestamp
        lastUpdated: Date.now(),
      };

      setClaims(customClaims);
    } catch (err) {
      console.error('Failed to fetch custom claims:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch user claims');
      setClaims(null);
    } finally {
      setLoading(false);
    }
  }, [user]);

  useEffect(() => {
    if (!authLoading) {
      fetchClaims();
    }
  }, [authLoading, fetchClaims]);

  const refreshClaims = useCallback(() => {
    fetchClaims();
  }, [fetchClaims]);

  return {
    claims,
    loading,
    error,
    refreshClaims,
    // Convenience getters
    role: claims?.role || UserRole.USER,
    permissions: claims?.permissions || [],
    isEmailVerified: claims?.emailVerified || false,
    isAdmin: claims?.role === UserRole.ADMIN,
    isUser: claims?.role === UserRole.USER,
  };
}

export function usePermissions() {
  const { permissions, role, loading } = useCustomClaims();

  const hasPermission = useCallback((permission: string): boolean => {
    if (loading) return false;
    
    // Admin role has all permissions
    if (role === UserRole.ADMIN) return true;
    
    // Check specific permission
    return permissions.includes(permission) || 
           permissions.includes('*') ||
           permissions.some(p => p.endsWith(':all') && permission.startsWith(p.split(':')[0] + ':'));
  }, [permissions, role, loading]);

  const hasAnyPermission = useCallback((requiredPermissions: string[]): boolean => {
    return requiredPermissions.some(permission => hasPermission(permission));
  }, [hasPermission]);

  const hasAllPermissions = useCallback((requiredPermissions: string[]): boolean => {
    return requiredPermissions.every(permission => hasPermission(permission));
  }, [hasPermission]);

  return {
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    permissions,
    loading,
  };
}

export function useRoleAccess() {
  const { role, isAdmin, isUser, loading } = useCustomClaims();

  const hasRole = useCallback((requiredRole: UserRole): boolean => {
    if (loading) return false;
    return role === requiredRole;
  }, [role, loading]);

  const hasMinRole = useCallback((minRole: UserRole): boolean => {
    if (loading) return false;
    
    const roleHierarchy = {
      [UserRole.USER]: 0,
      [UserRole.ADMIN]: 1,
    };

    return roleHierarchy[role] >= roleHierarchy[minRole];
  }, [role, loading]);

  return {
    role,
    isAdmin,
    isUser,
    hasRole,
    hasMinRole,
    loading,
  };
}
