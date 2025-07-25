import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';
import { ServerCookies, COOKIE_NAMES } from './cookies';
import { logger } from './logger';

export interface ServerAuthResult {
  isAuthenticated: boolean;
  user?: {
    user_id: string;
    email: string;
    role: string;
    permissions: string[];
    subscription_tier: string;
    package_tier: string;
    expires_at: string;
    session_type: string;
    // Legacy Firebase compatibility fields
    uid?: string; // Alias for user_id
    emailVerified?: boolean;
    displayName?: string;
    photoURL?: string;
  };
  error?: string;
}

/**
 * Get current authenticated user on the server side using backend API
 * This function can be used in Server Components and API routes
 */
export async function getServerAuth(): Promise<ServerAuthResult> {
  try {
    const sessionId = await ServerCookies.get('SESSION');
    
    if (!sessionId) {
      return { isAuthenticated: false };
    }

    // Call backend API to validate session
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    
    try {
      const response = await fetch(`${backendUrl}/api/v1/auth/profile`, {
        method: 'GET',
        headers: {
          'Cookie': `sess_id=${sessionId}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        // Session expired or invalid - don't try to delete cookie in Server Component context
        return { 
          isAuthenticated: false, 
          error: response.status === 401 ? 'Session expired' : 'Authentication failed'
        };
      }

      const userData = await response.json();
      
      return {
        isAuthenticated: true,
        user: {
          user_id: userData.user_id,
          email: userData.email,
          role: userData.role,
          permissions: userData.permissions || [],
          subscription_tier: userData.subscription_tier,
          package_tier: userData.package_tier,
          expires_at: userData.expires_at,
          session_type: userData.session_type,
          // Firebase compatibility
          uid: userData.user_id,
          emailVerified: true, // Backend users are considered verified
          displayName: userData.display_name || userData.email?.split('@')[0],
          photoURL: userData.photo_url,
        },
      };
    } catch (networkError) {
      logger.info('Backend not available for auth check, trying to parse session data locally', { error: networkError instanceof Error ? networkError.message : networkError });
      
      // If backend is not available, try to parse the session data directly
      try {
        const parsedSession = JSON.parse(sessionId);
        
        if (parsedSession && parsedSession.user_id) {
          return {
            isAuthenticated: true,
            user: {
              user_id: parsedSession.user_id,
              email: parsedSession.email,
              role: parsedSession.role || 'user',
              permissions: parsedSession.permissions || [],
              subscription_tier: parsedSession.subscription_tier || 'free',
              package_tier: parsedSession.package_tier || 'free',
              expires_at: parsedSession.expires_at,
              session_type: parsedSession.session_type || 'user',
              // Firebase compatibility
              uid: parsedSession.user_id,
              emailVerified: true,
              displayName: parsedSession.display_name || parsedSession.email?.split('@')[0],
              photoURL: parsedSession.photo_url,
            },
          };
        }
      } catch (parseError) {
        logger.error('Failed to parse session data', { error: parseError instanceof Error ? parseError.message : parseError });
      }
      
      // Don't try to clear invalid session in Server Component context
      return { 
        isAuthenticated: false, 
        error: 'Backend unavailable and session data invalid'
      };
    }
  } catch (error) {
    logger.error('Server auth check failed', { error: error instanceof Error ? error.message : error });
    return { 
      isAuthenticated: false, 
      error: 'Authentication check failed' 
    };
  }
}

/**
 * Require authentication on the server side
 * Redirects to login if not authenticated
 */
export async function requireAuth(redirectPath?: string): Promise<ServerAuthResult> {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated) {
    const loginUrl = '/login';
    const searchParams = redirectPath ? `?redirect=${encodeURIComponent(redirectPath)}` : '';
    redirect(`${loginUrl}${searchParams}`);
  }
  
  return authResult;
}

/**
 * Check if user has specific permission on the server side
 */
export async function hasServerPermission(permission: string): Promise<boolean> {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated || !authResult.user?.permissions) {
    return false;
  }
  
  const permissions = authResult.user.permissions;
  
  // Check exact match
  if (permissions.includes(permission)) {
    return true;
  }
  
  // Check wildcard permissions
  return permissions.some(userPermission => {
    if (userPermission.endsWith('.*')) {
      const prefix = userPermission.slice(0, -2);
      return permission.startsWith(prefix + '.');
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
  redirectPath?: string
): Promise<ServerAuthResult> {
  const authResult = await requireAuth(redirectPath);
  
  const hasPermission = await hasServerPermission(permission);
  
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
 * Create a new session on the server side using backend API
 */
export async function createServerSession(
  email: string,
  password: string,
  additionalData?: Record<string, any>
): Promise<{ success: boolean; error?: string; user?: any }> {
  try {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/v1/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        type: 'credentials',
        email,
        password,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return { 
        success: false, 
        error: errorData.error || 'Login failed' 
      };
    }

    const userData = await response.json();
    
    // Extract session cookie from backend response and set it
    const setCookieHeader = response.headers.get('set-cookie');
    if (setCookieHeader) {
      // Parse the session cookie from the Set-Cookie header (backend uses 'sess_id')
      const sessionCookieMatch = setCookieHeader.match(/(sess_id=[^;]+)/);
      if (sessionCookieMatch) {
        const sessionValue = sessionCookieMatch[1].split('=')[1];
        // Set the session cookie using our cookie utilities
        await ServerCookies.set('SESSION', sessionValue);
        logger.debug('Session cookie set', { sessionPreview: sessionValue.substring(0, 10) + '...' });
      } else {
        logger.error('No sess_id cookie found in Set-Cookie header', { setCookieHeader });
      }
    } else {
      // Fallback: If backend doesn't set cookie, create a session from userData
      if (userData && userData.user_id) {
        const sessionData = {
          user_id: userData.user_id,
          email: userData.email,
          role: userData.role,
          permissions: userData.permissions || [],
          subscription_tier: userData.subscription_tier,
          package_tier: userData.package_tier,
          expires_at: userData.expires_at || new Date(Date.now() + 86400000).toISOString(), // 24 hours default
          session_type: userData.session_type || 'user'
        };
        await ServerCookies.set('SESSION', JSON.stringify(sessionData));
        logger.debug('Created session from userData', { userId: userData.user_id, email: userData.email });
      } else {
        logger.error('No session cookie in response and no valid user data', { userData });
      }
    }
    
    return { 
      success: true, 
      user: userData 
    };
  } catch (error) {
    logger.error('Server session creation failed', { error: error instanceof Error ? error.message : error });
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Session creation failed' 
    };
  }
}

/**
 * Destroy session on the server side using backend API
 */
export async function destroyServerSession(): Promise<void> {
  try {
    const sessionId = await ServerCookies.get('SESSION');
    
    if (sessionId) {
      const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
      await fetch(`${backendUrl}/api/v1/auth/logout`, {
        method: 'POST',
        headers: {
          'Cookie': `sess_id=${sessionId}`,
          'Content-Type': 'application/json',
        },
      });
    }
    
    // Clear client-side cookies
    await ServerCookies.clearAuthCookies();
  } catch (error) {
    logger.error('Server session destruction failed', { error: error instanceof Error ? error.message : error });
  }
}

/**
 * Generate CSRF token
 */
function generateCSRFToken(): string {
  const array = new Uint8Array(32);
  const crypto = require('crypto');
  return crypto.randomBytes(32).toString('hex');
}

/**
 * Validate CSRF token on the server side
 */
export async function validateCSRFToken(providedToken: string): Promise<boolean> {
  try {
    const storedToken = await ServerCookies.get('CSRF');
    return storedToken === providedToken;
  } catch (error) {
    logger.error('CSRF token validation failed', { error: error instanceof Error ? error.message : error });
    return false;
  }
}

/**
 * Get session info for SSR (safe to use in Server Components)
 */
export async function getSessionInfo(): Promise<{
  isAuthenticated: boolean;
  email?: string;
  displayName?: string;
  permissions?: string[];
  packageTier?: string;
}> {
  const authResult = await getServerAuth();
  
  if (!authResult.isAuthenticated || !authResult.user) {
    return { isAuthenticated: false };
  }
  
  const { user } = authResult;
  
  return {
    isAuthenticated: true,
    email: user.email,
    displayName: user.displayName,
    permissions: user.permissions,
    packageTier: user.package_tier,
  };
}

/**
 * Check if the current session needs refresh by calling backend
 */
export async function needsSessionRefresh(): Promise<boolean> {
  try {
    const sessionId = await ServerCookies.get('SESSION');
    
    if (!sessionId) {
      return false;
    }
    
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/v1/auth/refresh`, {
      method: 'POST',
      headers: {
        'Cookie': `sess_id=${sessionId}`,
        'Content-Type': 'application/json',
      },
    });
    
    // If refresh endpoint returns 200, session is still valid
    // If it returns 401, session needs refresh/login
    return !response.ok;
  } catch (error) {
    logger.error('Session refresh check failed', { error: error instanceof Error ? error.message : error });
    return true; // Assume needs refresh on error
  }
}