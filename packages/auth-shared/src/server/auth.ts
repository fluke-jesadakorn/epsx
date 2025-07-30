import { redirect } from 'next/navigation';
import type { AuthResult, UserProfile } from '../types';

export interface ServerAuthResult extends AuthResult {
  user?: UserProfile;
}

export interface AuthServerConfig {
  backendUrl?: string;
  sessionCookieName?: string;
  adminSessionCookieName?: string;
  fallbackToLocalParsing?: boolean;
}

const DEFAULT_CONFIG: Required<AuthServerConfig> = {
  backendUrl: process.env.BACKEND_URL || process.env.NEXTAUTH_URL || 'http://localhost:8080',
  sessionCookieName: 'sess_id',
  adminSessionCookieName: 'admin_sess_id',
  fallbackToLocalParsing: true,
};

/**
 * Get current authenticated user on the server side using backend API
 * Works for both regular users and admin users
 */
export async function getServerAuth(config: AuthServerConfig = {}): Promise<ServerAuthResult> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };
  
  try {
    let cookieStore;
    try {
      const { cookies } = await import('next/headers');
      cookieStore = await cookies();
    } catch (cookieError) {
      if (cookieError instanceof Error && cookieError.message.includes('cookies() was called outside a request scope')) {
        return { isAuthenticated: false, error: 'Request scope unavailable' };
      }
      throw cookieError;
    }
    
    // Try admin session first, then regular session
    const adminSession = cookieStore.get(finalConfig.adminSessionCookieName);
    const userSession = cookieStore.get(finalConfig.sessionCookieName);
    
    const sessionId = adminSession?.value || userSession?.value;
    const isAdminSession = !!adminSession?.value;
    
    if (!sessionId) {
      return { isAuthenticated: false };
    }

    // Determine API endpoint based on session type
    const endpoint = isAdminSession ? '/api/v1/admin/auth/profile' : '/api/v1/auth/profile';
    const cookieHeader = `${isAdminSession ? finalConfig.adminSessionCookieName : finalConfig.sessionCookieName}=${sessionId}`;

    try {
      const response = await fetch(`${finalConfig.backendUrl}${endpoint}`, {
        method: 'GET',
        headers: {
          'Cookie': cookieHeader,
          'Content-Type': 'application/json',
          'User-Agent': 'Auth-Shared Server Component',
        },
      });

      if (!response.ok) {
        return { 
          isAuthenticated: false, 
          error: response.status === 401 ? 'Session expired' : 'Authentication failed'
        };
      }

      const userData = await response.json();
      
      return {
        isAuthenticated: true,
        user: {
          id: userData.user_id || userData.id,
          email: userData.email,
          role: userData.role,
          isActive: true,
          createdAt: new Date(userData.created_at || Date.now()),
          updatedAt: new Date(userData.updated_at || Date.now()),
          displayName: userData.displayName || userData.display_name || userData.email?.split('@')[0],
          avatar: userData.photoURL || userData.photo_url,
        } as UserProfile & { 
          permissions?: string[];
          session_type?: string;
          package_tier?: string;
          subscription_tier?: string;
          expires_at?: string;
          permission_profiles?: string[];
        },
      };
    } catch (networkError) {
      console.warn('Backend not available for auth check, attempting local session parsing', { 
        error: networkError instanceof Error ? networkError.message : networkError 
      });
      
      if (!finalConfig.fallbackToLocalParsing) {
        return { 
          isAuthenticated: false, 
          error: 'Backend unavailable and fallback disabled'
        };
      }
      
      // Fallback: try to parse session data locally
      try {
        const parsedSession = JSON.parse(sessionId);
        
        if (parsedSession && parsedSession.user_id) {
          return {
            isAuthenticated: true,
            user: {
              id: parsedSession.user_id || parsedSession.id,
              email: parsedSession.email,
              role: parsedSession.role || 'user',
              isActive: true,
              createdAt: new Date(parsedSession.created_at || Date.now()),
              updatedAt: new Date(parsedSession.updated_at || Date.now()),
              displayName: parsedSession.displayName || parsedSession.display_name || parsedSession.email?.split('@')[0],
              avatar: parsedSession.photoURL || parsedSession.photo_url,
            } as UserProfile & { 
              permissions?: string[];
              session_type?: string;
              package_tier?: string;
              subscription_tier?: string;
              expires_at?: string;
              permission_profiles?: string[];
            },
          };
        }
      } catch (parseError) {
        console.error('Failed to parse session data', { 
          error: parseError instanceof Error ? parseError.message : parseError 
        });
      }
      
      return { 
        isAuthenticated: false, 
        error: 'Backend unavailable and session data invalid'
      };
    }
  } catch (error) {
    console.error('Server auth check failed', { 
      error: error instanceof Error ? error.message : error 
    });
    return { 
      isAuthenticated: false, 
      error: 'Authentication check failed' 
    };
  }
}

/**
 * Require authentication on the server side
 * Redirects to appropriate login if not authenticated
 */
export async function requireAuth(
  redirectPath?: string, 
  config: AuthServerConfig = {}
): Promise<ServerAuthResult> {
  const authResult = await getServerAuth(config);
  
  if (!authResult.isAuthenticated) {
    const loginUrl = (authResult.user as any)?.session_type === 'admin' ? '/login' : '/login';
    const searchParams = redirectPath ? `?redirect=${encodeURIComponent(redirectPath)}` : '';
    redirect(`${loginUrl}${searchParams}`);
  }
  
  return authResult;
}

/**
 * Check if user has specific permission on the server side
 */
export async function hasServerPermission(
  permission: string, 
  config: AuthServerConfig = {}
): Promise<boolean> {
  const authResult = await getServerAuth(config);
  
  if (!authResult.isAuthenticated || !(authResult.user as any)?.permissions) {
    return false;
  }
  
  const permissions = (authResult.user as any).permissions;
  
  // Check exact match
  if (permissions.includes(permission)) {
    return true;
  }
  
  // Check wildcard permissions
  return permissions.some((userPermission: string) => {
    if (userPermission.endsWith('.*') || userPermission.endsWith(':*')) {
      const prefix = userPermission.slice(0, -2);
      return permission.startsWith(prefix + '.') || permission.startsWith(prefix + ':');
    }
    if (userPermission === '*') {
      return true;
    }
    return false;
  });
}

/**
 * Require specific permission on the server side
 * Redirects to access denied if permission not found
 */
export async function requirePermission(
  permission: string, 
  redirectPath?: string,
  config: AuthServerConfig = {}
): Promise<ServerAuthResult> {
  const authResult = await requireAuth(redirectPath, config);
  
  const hasPermission = await hasServerPermission(permission, config);
  
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
  
  return authResult;
}

/**
 * Check role hierarchy (admin > user, etc.)
 */
export function checkRoleHierarchy(userRole: string, requiredRole: string): boolean {
  const ROLE_HIERARCHY: Record<string, number> = {
    'user': 1,
    'premium': 2,
    'moderator': 3,
    'admin': 4,
    'super_admin': 5
  };

  const userLevel = ROLE_HIERARCHY[userRole.toLowerCase()] || 0;
  const requiredLevel = ROLE_HIERARCHY[requiredRole.toLowerCase()] || 1;

  return userLevel >= requiredLevel;
}

/**
 * Require specific role or higher on the server side
 */
export async function requireRole(
  requiredRole: string,
  redirectPath?: string,
  config: AuthServerConfig = {}
): Promise<ServerAuthResult> {
  const authResult = await requireAuth(redirectPath, config);
  
  if (!authResult.user?.role || !checkRoleHierarchy(authResult.user.role, requiredRole)) {
    const accessDeniedUrl = '/access-denied';
    const searchParams = new URLSearchParams({
      reason: `Access denied: Required role '${requiredRole}' or higher, current role: '${authResult.user?.role || 'none'}'`,
    });
    if (redirectPath) {
      searchParams.set('route', redirectPath);
    }
    redirect(`${accessDeniedUrl}?${searchParams.toString()}`);
  }
  
  return authResult;
}