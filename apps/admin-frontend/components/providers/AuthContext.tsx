'use client';

/**
 * NextAuth.js Client-side Authentication Hooks
 * Provides auth state to client components using NextAuth session
 */

import { useSession } from 'next-auth/react';

/**
 * Hook to access NextAuth.js session in client components
 */
export function useAuth() {
  const { data: session, status } = useSession();
  
  return {
    user: session?.user || null,
    session: session,
    isAuthenticated: !!session?.user,
    isLoading: status === 'loading',
    isAdmin: !!(session?.user as any)?.admin || !!(session?.user as any)?.admin_modules?.length,
    adminModules: (session?.user as any)?.admin_modules as string[] || [],
    permissions: (session?.user as any)?.permissions as string[] || [],
  };
}

/**
 * Hook to check if user has specific admin module
 */
export function useAdminModule(module: string) {
  const { adminModules, isAdmin } = useAuth();
  return isAdmin && adminModules.includes(module);
}

/**
 * Hook to check if user has specific permission
 */
export function usePermission(permission: string) {
  const { permissions } = useAuth();
  return permissions.includes(permission);
}

/**
 * Hook to check if user is system admin
 */
export function useIsSystemAdmin() {
  return useAdminModule('system_admin');
}