'use client';

import { useSession } from 'next-auth/react';

/**
 * Auth utility functions for client-side permission checks
 */
class AuthUtils {
  private getSession() {
    // This needs to be called from within a component that has access to useSession
    // We'll implement this properly by having the component pass session data
    return null;
  }

  /**
   * Check if user has a specific permission
   */
  async hasPermission(permission: string, sessionData?: any): Promise<boolean> {
    try {
      if (!sessionData) {
        return false;
      }

      const permissions = sessionData.user?.permissions || [];
      const role = sessionData.user?.role;

      // Check direct permissions
      if (permissions.includes(permission)) {
        return true;
      }

      // Check role-based permissions
      if (role === 'admin') {
        return true; // Admin has all permissions
      }

      if (role === 'moderator') {
        const moderatorPermissions = ['user.view', 'user.edit', 'analytics.view'];
        return moderatorPermissions.includes(permission);
      }

      return false;
    } catch (error) {
      console.error('Permission check failed:', error);
      return false;
    }
  }

  /**
   * Check if user can access a specific route
   */
  async canAccessRoute(route: string, sessionData?: any): Promise<boolean> {
    try {
      if (!sessionData) {
        return false;
      }

      const role = sessionData.user?.role;

      // Admin routes
      if (route.startsWith('/admin') || route.startsWith('/iam') || route.startsWith('/users')) {
        return ['admin', 'moderator'].includes(role);
      }

      // Analytics routes
      if (route.startsWith('/analytics')) {
        return ['admin', 'moderator'].includes(role);
      }

      // Settings routes
      if (route.startsWith('/settings')) {
        return role === 'admin';
      }

      // Default: allow access to public routes
      return true;
    } catch (error) {
      console.error('Route access check failed:', error);
      return false;
    }
  }

  /**
   * Check if user has required tier level
   */
  async hasRequiredTier(requiredTier: string, sessionData?: any): Promise<boolean> {
    try {
      if (!sessionData) {
        return false;
      }

      const userTier = sessionData.user?.subscription_tier?.toLowerCase() || 'free';
      const required = requiredTier.toLowerCase();

      const tierHierarchy = {
        'free': 0,
        'bronze': 1,
        'silver': 2,
        'gold': 3,
        'platinum': 4,
        'enterprise': 5,
      };

      const userLevel = tierHierarchy[userTier as keyof typeof tierHierarchy] || 0;
      const requiredLevel = tierHierarchy[required as keyof typeof tierHierarchy] || 0;

      return userLevel >= requiredLevel;
    } catch (error) {
      console.error('Tier check failed:', error);
      return false;
    }
  }
}

export const authUtils = new AuthUtils();

/**
 * Hook-based auth utilities that can access session data directly
 */
export function useAuthUtils() {
  const { data: session, status } = useSession();

  return {
    hasPermission: (permission: string) => 
      authUtils.hasPermission(permission, session),
    canAccessRoute: (route: string) => 
      authUtils.canAccessRoute(route, session),
    hasRequiredTier: (tier: string) => 
      authUtils.hasRequiredTier(tier, session),
    isAuthenticated: status === 'authenticated',
    isLoading: status === 'loading',
    session,
  };
}