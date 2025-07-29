import { getCurrentUser, getUserPermissions } from '@epsx/server-actions';
import { PackageTier } from '@epsx/types';
import { redirect } from 'next/navigation';
import { getServerAuth, requireAuth as baseRequireAuth, hasServerPermission } from './auth-server';
import { logger } from './logger';

// Re-export the user type from existing auth-server
export type { ServerAuthResult } from './auth-server';

/**
 * Get current user using server actions (compatible with server components)
 * This is the preferred method for the new architecture
 */
export async function getServerUser() {
  try {
    return await getCurrentUser();
  } catch (error) {
    logger.error('Failed to get server user via server actions', { 
      error: error instanceof Error ? error.message : String(error) 
    });
    
    // Fallback to existing auth-server method
    const authResult = await getServerAuth();
    return authResult.isAuthenticated ? authResult.user : null;
  }
}

/**
 * Get user permissions using server actions
 */
export async function getServerPermissions() {
  try {
    return await getUserPermissions();
  } catch (error) {
    logger.error('Failed to get server permissions via server actions', {
      error: error instanceof Error ? error.message : String(error)
    });
    
    // Fallback to existing auth-server method
    const authResult = await getServerAuth();
    return authResult.isAuthenticated ? authResult.user?.permissions || [] : [];
  }
}

/**
 * Server-side auth guard that redirects to login if not authenticated
 * Uses server actions for user data
 */
export async function requireAuth(redirectPath?: string) {
  try {
    const user = await getServerUser();
    
    if (!user) {
      const loginUrl = '/login';
      const searchParams = redirectPath ? `?redirect=${encodeURIComponent(redirectPath)}` : '';
      redirect(`${loginUrl}${searchParams}`);
    }
    
    return user;
  } catch (error) {
    logger.error('Server auth guard failed, falling back to cookie-based auth', {
      error: error instanceof Error ? error.message : String(error)
    });
    
    // Fallback to existing cookie-based auth
    return baseRequireAuth(redirectPath);
  }
}

/**
 * Server-side guest guard that redirects authenticated users away
 */
export async function requireGuest(redirectPath: string = '/dashboard') {
  try {
    const user = await getServerUser();
    
    if (user) {
      redirect(redirectPath);
    }
  } catch (error) {
    logger.error('Server guest guard failed', {
      error: error instanceof Error ? error.message : String(error)
    });
    
    // Fallback to existing method
    const authResult = await getServerAuth();
    if (authResult.isAuthenticated) {
      redirect(redirectPath);
    }
  }
}

/**
 * Check if user has specific permission (server-side)
 */
export async function checkServerPermission(permission: string): Promise<boolean> {
  try {
    const permissions = await getServerPermissions();
    
    // Check exact match
    if (permissions.some(p => (typeof p === 'string' ? p : p.name) === permission)) {
      return true;
    }
    
    // Check wildcard permissions
    return permissions.some(p => {
      const permName = typeof p === 'string' ? p : p.name;
      if (permName.endsWith('.*')) {
        const prefix = permName.slice(0, -2);
        return permission.startsWith(prefix + '.');
      }
      if (permName.endsWith(':*')) {
        const prefix = permName.slice(0, -2);
        return permission.startsWith(prefix + ':');
      }
      return false;
    });
  } catch (error) {
    logger.error('Server permission check failed, falling back to cookie-based check', {
      error: error instanceof Error ? error.message : String(error),
      permission
    });
    
    // Fallback to existing method
    return hasServerPermission(permission);
  }
}

/**
 * Require specific permission on the server side
 */
export async function requirePermission(permission: string, redirectPath?: string) {
  const user = await requireAuth(redirectPath);
  
  const hasPermission = await checkServerPermission(permission);
  
  if (!hasPermission) {
    const accessDeniedUrl = '/access-denied';
    const searchParams = new URLSearchParams({
      reason: `Missing required permission: ${permission}`,
    });
    if (redirectPath) {
      searchParams.set('route', redirectPath);
    }
    redirect(`${accessDeniedUrl}?${searchParams.toString()}`);
  }
  
  return user;
}

/**
 * Get initial auth state for SSR hydration
 * This is safe to use in Server Components and provides data for client hydration
 */
export async function getInitialAuthState() {
  try {
    const user = await getServerUser();
    const permissions = user ? await getServerPermissions() : [];
    
    if (!user) {
      return {
        user: null,
        permissions: [],
        packageTier: PackageTier.FREE,
        isAuthenticated: false
      };
    }
    
    // Normalize user data for client consumption
    const normalizedUser = {
      user_id: user.user_id,
      email: user.email,
      role: user.role,
      permissions: user.permissions || [],
      subscription_tier: user.subscription_tier,
      package_tier: user.package_tier,
      expires_at: user.expires_at,
      session_type: user.session_type,
      emailVerified: true,
      displayName: user.displayName || user.email?.split('@')[0] || null,
      photoURL: user.photoURL || null,
      phoneNumber: user.phoneNumber || null,
    };
    
    return {
      user: normalizedUser,
      permissions: permissions.map(p => typeof p === 'string' ? p : p.name),
      packageTier: (user.package_tier as PackageTier) || PackageTier.FREE,
      isAuthenticated: true
    };
  } catch (error) {
    logger.error('Failed to get initial auth state, falling back to cookie-based auth', {
      error: error instanceof Error ? error.message : String(error)
    });
    
    // Fallback to existing cookie-based method
    const authResult = await getServerAuth();
    
    if (!authResult.isAuthenticated || !authResult.user) {
      return {
        user: null,
        permissions: [],
        packageTier: PackageTier.FREE,
        isAuthenticated: false
      };
    }
    
    return {
      user: authResult.user,
      permissions: authResult.user.permissions || [],
      packageTier: (authResult.user.package_tier as PackageTier) || PackageTier.FREE,
      isAuthenticated: true
    };
  }
}