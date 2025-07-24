'use server';

import { redirect } from 'next/navigation';
import { z } from 'zod';
import { createServerSession, destroyServerSession, getServerAuth } from '@/lib/auth-server';
import { ServerCookies, COOKIE_NAMES } from '@/lib/cookies';
import { 
  performSecurityChecks,
  sanitizeInput,
  checkLoginAttempts,
  resetLoginAttempts,
  getClientIP,
  logSecurityEvent,
  validatePasswordStrength
} from '@/lib/security';
import type { User } from '@/types/auth/user';

// Validation schemas
const emailSchema = z.string().email('Please enter a valid email address');
const passwordSchema = z.string()
  .min(8, 'Password must be at least 8 characters long')
  .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
  .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
  .regex(/[0-9]/, 'Password must contain at least one number');

const loginSchema = z.object({
  email: emailSchema,
  password: z.string().min(1, 'Password is required')
});

const registerSchema = z.object({
  email: emailSchema,
  password: passwordSchema,
  confirmPassword: z.string()
}).refine(data => data.password === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"]
});

const passwordResetSchema = z.object({
  email: emailSchema
});

/**
 * Server Action for user login with email/password
 */
export async function loginAction(
  email: string,
  password: string
): Promise<{ success: boolean; error?: string; fieldErrors?: Record<string, string[]> }> {
  try {
    // Security checks
    const securityCheck = await performSecurityChecks({
      checkOrigin: true,
      checkUserAgent: true,
      rateLimitKey: 'login'
    });
    
    if (!securityCheck.success) {
      await logSecurityEvent({
        action: 'LOGIN_SECURITY_CHECK_FAILED',
        resource: 'auth',
        success: false,
        details: { error: securityCheck.error }
      });
      return { success: false, error: 'Security validation failed' };
    }

    // Sanitize inputs
    const sanitizedEmail = sanitizeInput(email).toLowerCase();
    const sanitizedPassword = sanitizeInput(password);

    // Check login attempts
    const clientIP = await getClientIP();
    if (!checkLoginAttempts(`login_${clientIP}`)) {
      await logSecurityEvent({
        action: 'LOGIN_ATTEMPTS_EXCEEDED',
        resource: 'auth',
        success: false,
        details: { email: sanitizedEmail }
      });
      return { success: false, error: 'Too many login attempts. Please try again later.' };
    }

    // Validate input
    const validation = loginSchema.safeParse({ 
      email: sanitizedEmail, 
      password: sanitizedPassword 
    });
    
    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      await logSecurityEvent({
        action: 'LOGIN_VALIDATION_FAILED',
        resource: 'auth',
        success: false,
        details: { email: sanitizedEmail, errors: fieldErrors }
      });
      return { 
        success: false, 
        error: 'Please fix the validation errors',
        fieldErrors 
      };
    }

    const result = await createServerSession(sanitizedEmail, sanitizedPassword);
    
    if (!result.success) {
      await logSecurityEvent({
        action: 'LOGIN_FAILED',
        resource: 'auth',
        success: false,
        details: { email: sanitizedEmail, error: result.error }
      });
      return { success: false, error: result.error || 'Login failed' };
    }
    
    // Reset login attempts on successful login
    resetLoginAttempts(`login_${clientIP}`);
    
    await logSecurityEvent({
      action: 'LOGIN_SUCCESS',
      resource: 'auth',
      success: true,
      details: { email: sanitizedEmail }
    });
    
    return { success: true };
  } catch (error) {
    console.error('Login action error:', error);
    await logSecurityEvent({
      action: 'LOGIN_ERROR',
      resource: 'auth',
      success: false,
      details: { error: error instanceof Error ? error.message : 'Unknown error' }
    });
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Login failed' 
    };
  }
}

/**
 * Server Action for user logout
 */
export async function logoutAction(): Promise<void> {
  try {
    await destroyServerSession();
  } catch (error) {
    console.error('Logout action error:', error);
  }
  
  redirect('/login');
}

/**
 * Server Action for user registration
 */
export async function registerAction(
  email: string,
  password: string,
  confirmPassword: string,
  additionalData?: Record<string, any>
): Promise<{ success: boolean; error?: string; fieldErrors?: Record<string, string[]> }> {
  try {
    // Security checks
    const securityCheck = await performSecurityChecks({
      checkOrigin: true,
      checkUserAgent: true,
      rateLimitKey: 'register'
    });
    
    if (!securityCheck.success) {
      await logSecurityEvent({
        action: 'REGISTER_SECURITY_CHECK_FAILED',
        resource: 'auth',
        success: false,
        details: { error: securityCheck.error }
      });
      return { success: false, error: 'Security validation failed' };
    }

    // Sanitize inputs
    const sanitizedEmail = sanitizeInput(email).toLowerCase();
    const sanitizedPassword = sanitizeInput(password);
    const sanitizedConfirmPassword = sanitizeInput(confirmPassword);

    // Enhanced password strength validation
    const passwordStrength = validatePasswordStrength(sanitizedPassword);
    if (!passwordStrength.isValid) {
      await logSecurityEvent({
        action: 'REGISTER_WEAK_PASSWORD',
        resource: 'auth',
        success: false,
        details: { email: sanitizedEmail, score: passwordStrength.score }
      });
      return { 
        success: false, 
        error: 'Password does not meet security requirements',
        fieldErrors: { password: passwordStrength.feedback }
      };
    }

    // Validate input
    const validation = registerSchema.safeParse({ 
      email: sanitizedEmail, 
      password: sanitizedPassword, 
      confirmPassword: sanitizedConfirmPassword 
    });
    
    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      await logSecurityEvent({
        action: 'REGISTER_VALIDATION_FAILED',
        resource: 'auth',
        success: false,
        details: { email: sanitizedEmail, errors: fieldErrors }
      });
      return { 
        success: false, 
        error: 'Please fix the validation errors',
        fieldErrors 
      };
    }

    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: sanitizedEmail,
        password: sanitizedPassword,
        ...additionalData,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      await logSecurityEvent({
        action: 'REGISTER_BACKEND_FAILED',
        resource: 'auth',
        success: false,
        details: { email: sanitizedEmail, error: errorData.error }
      });
      return { 
        success: false, 
        error: errorData.error || 'Registration failed' 
      };
    }

    await logSecurityEvent({
      action: 'REGISTER_SUCCESS',
      resource: 'auth',
      success: true,
      details: { email: sanitizedEmail }
    });

    // Registration successful, now login
    const loginResult = await loginAction(sanitizedEmail, sanitizedPassword);
    return {
      success: loginResult.success,
      error: loginResult.error
    };
  } catch (error) {
    console.error('Registration action error:', error);
    await logSecurityEvent({
      action: 'REGISTER_ERROR',
      resource: 'auth',
      success: false,
      details: { error: error instanceof Error ? error.message : 'Unknown error' }
    });
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Registration failed' 
    };
  }
}

/**
 * Server Action for password reset request
 */
export async function requestPasswordResetAction(
  email: string
): Promise<{ success: boolean; error?: string; fieldErrors?: Record<string, string[]> }> {
  try {
    // Validate input
    const validation = passwordResetSchema.safeParse({ email });
    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      return { 
        success: false, 
        error: 'Please enter a valid email address',
        fieldErrors 
      };
    }

    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/auth/forgot-password`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return { 
        success: false, 
        error: errorData.error || 'Password reset request failed' 
      };
    }

    return { success: true };
  } catch (error) {
    console.error('Password reset request error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Password reset request failed' 
    };
  }
}

/**
 * Server Action for password reset
 */
export async function resetPasswordAction(
  token: string,
  newPassword: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/auth/reset-password`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ token, new_password: newPassword }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return { 
        success: false, 
        error: errorData.error || 'Password reset failed' 
      };
    }

    return { success: true };
  } catch (error) {
    console.error('Password reset error:', error);
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Password reset failed' 
    };
  }
}

/**
 * Get current authenticated user (Server Action)
 */
export async function getCurrentUser(): Promise<User | null> {
  try {
    const authResult = await getServerAuth();
    
    if (!authResult.isAuthenticated || !authResult.user) {
      return null;
    }

    const { user } = authResult;
    
    return {
      id: user.user_id,
      email: user.email,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      emailVerified: user.emailVerified || true,
      role: user.role?.toUpperCase() as any || 'USER',
      displayName: user.displayName || user.email?.split('@')[0],
      photoURL: user.photoURL,
    };
  } catch (error) {
    console.error('Get current user error:', error);
    return null;
  }
}

/**
 * Check if user needs to refresh session
 */
export async function needsSessionRefresh(): Promise<boolean> {
  try {
    const sessionId = await ServerCookies.get('SESSION');
    
    if (!sessionId) {
      return true;
    }
    
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/auth/refresh`, {
      method: 'POST',
      headers: {
        'Cookie': `sess_id=${sessionId}`,
        'Content-Type': 'application/json',
      },
    });
    
    return !response.ok;
  } catch (error) {
    console.error('Session refresh check failed:', error);
    return true;
  }
}

/**
 * Legacy function for Firebase compatibility - use loginAction instead
 * @deprecated Use loginAction instead
 */
export async function handleSignIn(idToken: string): Promise<{ success: boolean; error?: string }> {
  console.warn('handleSignIn is deprecated, use loginAction instead');
  return { success: false, error: 'This method is deprecated' };
}

/**
 * Legacy function for Firebase compatibility - use logoutAction instead
 * @deprecated Use logoutAction instead
 */
export async function handleSignOut(): Promise<void> {
  console.warn('handleSignOut is deprecated, use logoutAction instead');
  await logoutAction();
}

/**
 * Legacy function for Firebase compatibility - use needsSessionRefresh instead
 * @deprecated Use needsSessionRefresh instead
 */
export async function refreshSession(): Promise<{ success: boolean }> {
  console.warn('refreshSession is deprecated, use needsSessionRefresh instead');
  const needs = await needsSessionRefresh();
  return { success: !needs };
}
