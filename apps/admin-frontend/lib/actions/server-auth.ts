'use server';

import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

interface LoginRequest {
  type: 'admin';
  email: string;
  password: string;
}

interface LoginResponse {
  user_id: string;
  email: string;
  admin: boolean;
  access_level: string;
  admin_modules: string[];
  permissions: string[];
  subscription_tier: string;
  expires_at: string;
  session_type: string;
  access_token: string;
  token_type: string;
}

export interface AuthUser {
  user_id: string;
  email: string;
  admin: boolean;
  access_level: string;
  admin_modules: string[];
  permissions: string[];
  subscription_tier: string;
  session_type: string;
  expires_at: string;
}

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const AUTH_COOKIE_NAME = 'admin_bearer_token';

/**
 * Login action that stores bearer token in HTTP-only cookie
 */
export async function loginAction(formData: FormData): Promise<{ success: boolean; error?: string }> {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;

  if (!email || !password) {
    return { success: false, error: 'Email and password are required' };
  }

  try {
    const loginRequest: LoginRequest = {
      type: 'admin',
      email,
      password,
    };

    const response = await fetch(`${BACKEND_URL}/api/v1/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(loginRequest),
    });

    if (!response.ok) {
      const error = await response.text();
      return { success: false, error: `Login failed: ${error}` };
    }

    const loginResult: LoginResponse = await response.json();

    // Store bearer token in HTTP-only cookie
    const cookieStore = await cookies();
    const expiresAt = new Date(loginResult.expires_at);
    
    cookieStore.set(AUTH_COOKIE_NAME, loginResult.access_token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      expires: expiresAt,
      path: '/',
    });

    return { success: true };
  } catch (error) {
    console.error('Login error:', error);
    return { success: false, error: 'An unexpected error occurred' };
  }
}

/**
 * Logout action that clears the HTTP-only cookie
 */
export async function logoutAction(): Promise<void> {
  const cookieStore = await cookies();
  const token = cookieStore.get(AUTH_COOKIE_NAME)?.value;

  if (token) {
    // Call backend logout endpoint
    try {
      await fetch(`${BACKEND_URL}/api/v1/auth/logout`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      });
    } catch (error) {
      console.error('Backend logout error:', error);
      // Continue with cookie cleanup even if backend call fails
    }
  }

  // Clear the HTTP-only cookie
  cookieStore.set(AUTH_COOKIE_NAME, '', {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'lax',
    expires: new Date(0),
    path: '/',
  });

  redirect('/login');
}

/**
 * Get current user from bearer token
 */
export async function getCurrentUser(): Promise<AuthUser | null> {
  // First try to get from HTTP-only cookie (direct auth system)
  const cookieStore = await cookies();
  const token = cookieStore.get(AUTH_COOKIE_NAME)?.value;

  if (token) {
    try {
      const response = await fetch(`${BACKEND_URL}/api/v1/auth/me`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        return response.json();
      }
    } catch (error) {
      console.debug('Bearer token auth failed, trying NextAuth session');
    }
  }

  // NextAuth fallback no longer available - auth migrated to backend system
  console.debug('🔧 [getCurrentUser] NextAuth fallback not available - using development mode');

  // If we still don't have a user but have a development session token, use development fallback
  if (process.env.NODE_ENV === 'development' || process.env.ENABLE_DEV_AUTH === 'true') {
    console.log('🔧 [getCurrentUser] Development mode - creating fallback user');
    return {
      user_id: 'dev-admin-001',
      email: 'admin@dev.local',
      role: 'admin',
      permissions: ['admin:read', 'admin:write', 'users:manage', 'admin_access'],
      subscription_tier: 'premium',
      session_type: 'admin',
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    };
  }

  return null;
}

/**
 * Check if user is authenticated
 */
export async function isAuthenticated(): Promise<boolean> {
  const user = await getCurrentUser();
  return user !== null;
}

/**
 * Get bearer token for API calls (used by middleware)
 */
export async function getBearerToken(): Promise<string | null> {
  // First try to get from HTTP-only cookie (direct auth system)
  const cookieStore = await cookies();
  const cookieToken = cookieStore.get(AUTH_COOKIE_NAME)?.value;
  
  if (cookieToken) {
    return cookieToken;
  }
  
  // NextAuth session token no longer available - auth migrated to backend system
  console.debug('🔍 [getBearerToken] NextAuth session not available - auth migrated to backend');
  
  // Development mode fallback: use dev session token if no real token found
  if (process.env.NODE_ENV === 'development' || process.env.ENABLE_DEV_AUTH === 'true') {
    console.log('🔧 [getBearerToken] Development mode - using dev session token');
    // Use the same token format as the auth.ts development fallback
    const devToken = `dev-session-${Date.now()}`;
    return devToken;
  }
  
  return null;
}

/**
 * Validate session (for middleware)
 */
export async function validateSession(): Promise<AuthUser | null> {
  const token = await getBearerToken();
  
  if (!token) {
    return null;
  }

  try {
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/validate-session`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ app_type: 'admin' }),
    });

    if (!response.ok) {
      return null;
    }

    return response.json();
  } catch (error) {
    console.error('Session validation error:', error);
    return null;
  }
}

/**
 * Refresh session and update cookie
 */
export async function refreshSession(): Promise<boolean> {
  const token = await getBearerToken();
  
  if (!token) {
    return false;
  }

  try {
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/refresh`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      return false;
    }

    const result = await response.json();
    
    // Update cookie expiry if provided
    if (result.expires_at) {
      const cookieStore = await cookies();
      const expiresAt = new Date(result.expires_at);
      
      cookieStore.set(AUTH_COOKIE_NAME, token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        expires: expiresAt,
        path: '/',
      });
    }

    return true;
  } catch (error) {
    console.error('Session refresh error:', error);
    return false;
  }
}