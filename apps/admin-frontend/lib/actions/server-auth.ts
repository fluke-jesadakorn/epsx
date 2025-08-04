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
  role: string;
  permissions: string[];
  subscription_tier: string;
  expires_at: string;
  session_type: string;
  access_token: string;
  token_type: string;
}

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
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
  const cookieStore = await cookies();
  const token = cookieStore.get(AUTH_COOKIE_NAME)?.value;

  if (!token) {
    return null;
  }

  try {
    const response = await fetch(`${BACKEND_URL}/api/v1/auth/me`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      // Token might be expired, clear the cookie
      const cookieStoreForClear = await cookies();
      cookieStoreForClear.set(AUTH_COOKIE_NAME, '', {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        expires: new Date(0),
        path: '/',
      });
      return null;
    }

    return response.json();
  } catch (error) {
    console.error('Get current user error:', error);
    return null;
  }
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
  const cookieStore = await cookies();
  return cookieStore.get(AUTH_COOKIE_NAME)?.value || null;
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