import { 
  getServerAuth, 
  requireAuth as sharedRequireAuth,
  hasServerPermission,
  requirePermission,
  requireRole,
  type ServerAuthResult 
} from '@epsx/auth-shared/server';

/**
 * Re-export requireAuth from shared package
 */
export const requireAuth = sharedRequireAuth;

/**
 * Re-export other auth functions from shared package
 */
export { hasServerPermission, requirePermission, requireRole };

/**
 * Get initial auth state for server-side rendering
 * Used to hydrate client auth context with server-side auth data
 */
export async function getInitialAuthState() {
  try {
    const authResult = await getServerAuth();
    
    if (!authResult.isAuthenticated || !authResult.user) {
      return {
        isAuthenticated: false,
        user: null,
        permissions: [],
        packageTier: 'free',
      };
    }

    // Extract additional data from user object
    const userData = authResult.user as any;
    
    return {
      isAuthenticated: true,
      user: {
        id: authResult.user.id,
        email: authResult.user.email,
        role: authResult.user.role,
        displayName: authResult.user.displayName,
        avatar: authResult.user.avatar,
        isActive: authResult.user.isActive,
        createdAt: authResult.user.createdAt,
        updatedAt: authResult.user.updatedAt,
      },
      permissions: userData.permissions || [],
      packageTier: userData.package_tier || userData.subscription_tier || 'free',
    };
  } catch (error) {
    console.error('Failed to get initial auth state:', error);
    return {
      isAuthenticated: false,
      user: null,
      permissions: [],
      packageTier: 'free',
    };
  }
}

/**
 * Re-export types
 */
export type { ServerAuthResult };