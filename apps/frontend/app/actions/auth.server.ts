'use server';

import {
  createServerSession,
  destroyServerSession,
  getServerAuth,
} from '@/lib/auth-server';
import { ServerCookies } from '@/lib/cookies';
import { logger } from '@/lib/logger';
import {
  checkLoginAttempts,
  EventSeverity,
  getClientIP,
  logSecurityEvent,
  performSecurityChecks,
  resetLoginAttempts,
  sanitizeInput,
  SecurityEventCategory,
  validatePasswordStrength,
} from '@/lib/security';
import type { User } from '@/types/auth/user';
import { ServerApiClient, isApiError } from '@epsx/api-client';
import { redirect } from 'next/navigation';
import { z } from 'zod';

// Validation schemas
const emailSchema = z.string().email('Please enter a valid email address');
const passwordSchema = z
  .string()
  .min(8, 'Password must be at least 8 characters long')
  .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
  .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
  .regex(/[0-9]/, 'Password must contain at least one number');

const loginSchema = z.object({
  email: emailSchema,
  password: z.string().min(1, 'Password is required'),
});

const registerSchema = z
  .object({
    email: emailSchema,
    password: passwordSchema,
    confirmPassword: z.string(),
  })
  .refine(data => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
  });

const enhancedRegisterSchema = z.object({
  email: emailSchema,
  password: passwordSchema,
  package_tier: z.string().default('Bronze'),
  referral_code: z.string().optional(),
  source: z.string().default('web_registration'),
  region: z.string().optional(),
  utm_source: z.string().optional(),
  utm_campaign: z.string().optional(),
});

const passwordResetSchema = z.object({
  email: emailSchema,
});

/**
 * Server Action for user login with email/password
 */
export async function loginAction(
  email: string,
  password: string
): Promise<{
  success: boolean;
  error?: string;
  fieldErrors?: Record<string, string[]>;
}> {
  // Sanitize inputs first so they're available in catch block
  const sanitizedEmail = sanitizeInput(email).toLowerCase();

  try {
    // Security checks
    const securityCheck = await performSecurityChecks({
      checkOrigin: true,
      checkUserAgent: true,
      rateLimitKey: 'login',
    });

    if (!securityCheck.success) {
      await logSecurityEvent({
        action: 'LOGIN_SECURITY_CHECK_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.HIGH,
        details: { error: securityCheck.error },
      });
      return { success: false, error: 'Security validation failed' };
    }

    // Sanitize remaining inputs
    const sanitizedPassword = sanitizeInput(password);

    // Check login attempts
    const clientIP = await getClientIP();
    if (!checkLoginAttempts(`login_${clientIP}`)) {
      await logSecurityEvent({
        action: 'LOGIN_ATTEMPTS_EXCEEDED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.HIGH,
        details: { email: sanitizedEmail },
      });
      return {
        success: false,
        error: 'Too many login attempts. Please try again later.',
      };
    }

    // Validate input
    const validation = loginSchema.safeParse({
      email: sanitizedEmail,
      password: sanitizedPassword,
    });

    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      await logSecurityEvent({
        action: 'LOGIN_VALIDATION_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.MEDIUM,
        details: { email: sanitizedEmail, errors: fieldErrors },
      });
      return {
        success: false,
        error: 'Please fix the validation errors',
        fieldErrors,
      };
    }

    const result = await createServerSession(sanitizedEmail, sanitizedPassword);

    if (!result.success) {
      await logSecurityEvent({
        action: 'LOGIN_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.HIGH,
        details: { email: sanitizedEmail, error: result.error },
      });
      return { success: false, error: result.error || 'Login failed' };
    }

    // Reset login attempts on successful login
    resetLoginAttempts(`login_${clientIP}`);

    await logSecurityEvent({
      action: 'LOGIN_SUCCESS',
      resource: 'auth',
      success: true,
      category: SecurityEventCategory.AUTHENTICATION,
      severity: EventSeverity.MEDIUM,
      details: { email: sanitizedEmail },
    });

    return { success: true };
  } catch (error) {
    logger.error('Login action failed', {
      error: error.message,
      email: sanitizedEmail,
    });
    await logSecurityEvent({
      action: 'LOGIN_ERROR',
      resource: 'auth',
      success: false,
      category: SecurityEventCategory.AUTHENTICATION,
      severity: EventSeverity.HIGH,
      details: {
        error: error instanceof Error ? error.message : 'Unknown error',
      },
    });
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Login failed',
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
    logger.error('Logout action failed', { error: error.message });
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
): Promise<{
  success: boolean;
  error?: string;
  fieldErrors?: Record<string, string[]>;
}> {
  // Sanitize inputs first so they're available in catch block
  const sanitizedEmail = sanitizeInput(email).toLowerCase();

  try {
    // Security checks
    const securityCheck = await performSecurityChecks({
      checkOrigin: true,
      checkUserAgent: true,
      rateLimitKey: 'register',
    });

    if (!securityCheck.success) {
      await logSecurityEvent({
        action: 'REGISTER_SECURITY_CHECK_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.HIGH,
        details: { error: securityCheck.error },
      });
      return { success: false, error: 'Security validation failed' };
    }

    // Sanitize remaining inputs
    const sanitizedPassword = sanitizeInput(password);
    const sanitizedConfirmPassword = sanitizeInput(confirmPassword);

    // Enhanced password strength validation
    const passwordStrength = validatePasswordStrength(sanitizedPassword);
    if (!passwordStrength.isValid) {
      await logSecurityEvent({
        action: 'REGISTER_WEAK_PASSWORD',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.MEDIUM,
        details: { email: sanitizedEmail, score: passwordStrength.score },
      });
      return {
        success: false,
        error: 'Password does not meet security requirements',
        fieldErrors: { password: passwordStrength.feedback },
      };
    }

    // Validate input
    const validation = registerSchema.safeParse({
      email: sanitizedEmail,
      password: sanitizedPassword,
      confirmPassword: sanitizedConfirmPassword,
    });

    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      await logSecurityEvent({
        action: 'REGISTER_VALIDATION_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.MEDIUM,
        details: { email: sanitizedEmail, errors: fieldErrors },
      });
      return {
        success: false,
        error: 'Please fix the validation errors',
        fieldErrors,
      };
    }

    const backendUrl = process.env.BACKEND_URL;
    if (!backendUrl) {
      throw new Error('BACKEND_URL environment variable is required');
    }
    const apiClient = new ServerApiClient();
    const response = await apiClient.register({
      email: sanitizedEmail,
      password: sanitizedPassword,
      ...additionalData,
    });

    if (isApiError(response)) {
      let errorMessage = response.error || 'Registration failed';
      let isDuplicateUser =
        response.details?.includes('409') ||
        errorMessage.includes('already exists');

      if (isDuplicateUser) {
        errorMessage = 'An account with this email already exists';

        await logSecurityEvent({
          action: 'REGISTER_DUPLICATE_USER',
          resource: 'auth',
          success: false,
          category: SecurityEventCategory.AUTHENTICATION,
          severity: EventSeverity.MEDIUM,
          details: { email: sanitizedEmail, error: errorMessage },
        });
      } else {
        await logSecurityEvent({
          action: 'REGISTER_BACKEND_FAILED',
          resource: 'auth',
          success: false,
          category: SecurityEventCategory.AUTHENTICATION,
          severity: EventSeverity.HIGH,
          details: {
            email: sanitizedEmail,
            error: errorMessage,
            details: response.details,
          },
        });
      }

      return {
        success: false,
        error: errorMessage,
      };
    }

    await logSecurityEvent({
      action: 'REGISTER_SUCCESS',
      resource: 'auth',
      success: true,
      category: SecurityEventCategory.AUTHENTICATION,
      severity: EventSeverity.MEDIUM,
      details: { email: sanitizedEmail },
    });

    // Registration successful, now login
    const loginResult = await loginAction(sanitizedEmail, sanitizedPassword);
    return {
      success: loginResult.success,
      error: loginResult.error,
    };
  } catch (error) {
    logger.error('Registration action failed', {
      error: error.message,
      email: sanitizedEmail,
    });
    await logSecurityEvent({
      action: 'REGISTER_ERROR',
      resource: 'auth',
      success: false,
      category: SecurityEventCategory.AUTHENTICATION,
      severity: EventSeverity.HIGH,
      details: {
        error: error instanceof Error ? error.message : 'Unknown error',
      },
    });
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Registration failed',
    };
  }
}

/**
 * Server Action for password reset request
 */
export async function requestPasswordResetAction(email: string): Promise<{
  success: boolean;
  error?: string;
  fieldErrors?: Record<string, string[]>;
}> {
  // Sanitize input first so it's available in catch block
  const sanitizedEmail = sanitizeInput(email).toLowerCase();

  try {
    // Validate input
    const validation = passwordResetSchema.safeParse({ email });
    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      return {
        success: false,
        error: 'Please enter a valid email address',
        fieldErrors,
      };
    }

    const backendUrl = process.env.BACKEND_URL;
    if (!backendUrl) {
      throw new Error('BACKEND_URL environment variable is required');
    }
    const apiClient = new ServerApiClient();
    const response = await apiClient.resetPassword({ email });

    if (isApiError(response)) {
      return {
        success: false,
        error: response.error || 'Password reset request failed',
      };
    }

    return { success: true };
  } catch (error) {
    logger.error('Password reset request failed', {
      error: error.message,
      email: sanitizedEmail,
    });
    return {
      success: false,
      error:
        error instanceof Error
          ? error.message
          : 'Password reset request failed',
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
    const backendUrl = process.env.BACKEND_URL;
    if (!backendUrl) {
      throw new Error('BACKEND_URL environment variable is required');
    }
    const apiClient = new ServerApiClient();
    const response = await apiClient.resetPassword({
      token,
      new_password: newPassword,
    });

    if (isApiError(response)) {
      return {
        success: false,
        error: response.error || 'Password reset failed',
      };
    }

    return { success: true };
  } catch (error) {
    logger.error('Password reset failed', { error: error.message });
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Password reset failed',
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
      role: (user.role?.toUpperCase() as any) || 'USER',
      displayName: user.displayName || user.email?.split('@')[0],
      photoURL: user.photoURL,
    };
  } catch (error) {
    logger.error('Get current user failed', { error: error.message });
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

    const backendUrl = process.env.BACKEND_URL;
    if (!backendUrl) {
      throw new Error('BACKEND_URL environment variable is required');
    }
    const apiClient = new ServerApiClient();
    const response = await apiClient.refreshSession();

    return isApiError(response);
  } catch (error) {
    logger.error('Session refresh check failed', { error: error.message });
    return true;
  }
}

/**
 * Server Action for login form handling with redirect
 */
export async function loginFormAction(formData: FormData) {
  const email = formData.get('email') as string;
  const password = formData.get('password') as string;
  const redirectTo = (formData.get('redirectTo') as string) || '/dashboard';

  const result = await loginAction(email, password);

  if (result.success) {
    redirect(redirectTo);
  }

  // In case of error, redirect to login with error parameter
  // This is a simplified approach - in production you might want better error handling
  redirect('/login?error=Login failed');
}

/**
 * Enhanced registration action with permission profile auto-assignment
 */
export async function registerUserWithPermissionProfiles(
  email: string,
  password: string,
  packageTier: string = 'Bronze',
  referralCode?: string,
  utmSource?: string,
  utmCampaign?: string
): Promise<{
  success: boolean;
  error?: string;
  fieldErrors?: Record<string, string[]>;
  featuresUnlocked?: string[];
  totalFeaturesAssigned?: number;
}> {
  try {
    // Security checks
    const securityCheck = await performSecurityChecks({
      checkOrigin: true,
      checkUserAgent: true,
      rateLimitKey: 'register',
    });

    if (!securityCheck.success) {
      await logSecurityEvent({
        action: 'ENHANCED_REGISTER_SECURITY_CHECK_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.HIGH,
        details: { error: securityCheck.error },
      });
      return { success: false, error: 'Security validation failed' };
    }

    // Sanitize inputs
    const sanitizedEmail = sanitizeInput(email).toLowerCase();
    const sanitizedPassword = sanitizeInput(password);
    const sanitizedPackageTier = sanitizeInput(packageTier);

    // Enhanced password strength validation
    const passwordStrength = validatePasswordStrength(sanitizedPassword);
    if (!passwordStrength.isValid) {
      await logSecurityEvent({
        action: 'ENHANCED_REGISTER_WEAK_PASSWORD',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.MEDIUM,
        details: { email: sanitizedEmail, score: passwordStrength.score },
      });
      return {
        success: false,
        error: 'Password does not meet security requirements',
        fieldErrors: { password: passwordStrength.feedback },
      };
    }

    // Validate input with enhanced schema
    const validation = enhancedRegisterSchema.safeParse({
      email: sanitizedEmail,
      password: sanitizedPassword,
      package_tier: sanitizedPackageTier,
      referral_code: referralCode,
      source: 'web_registration',
      region: 'US', // Could be determined from geo-location
      utm_source: utmSource,
      utm_campaign: utmCampaign,
    });

    if (!validation.success) {
      const fieldErrors = validation.error.flatten().fieldErrors;
      await logSecurityEvent({
        action: 'ENHANCED_REGISTER_VALIDATION_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.MEDIUM,
        details: { email: sanitizedEmail, errors: fieldErrors },
      });
      return {
        success: false,
        error: 'Please fix the validation errors',
        fieldErrors,
      };
    }

    // Call enhanced registration endpoint
    const backendUrl = process.env.BACKEND_URL || 'http://localhost:8080';
    console.log('BACKEND_URL from env:', process.env.BACKEND_URL, '-> Using:', backendUrl); // Debug log
    const apiClient = new ServerApiClient();
    console.log('Sending registration data:', JSON.stringify(validation.data, null, 2)); // Debug log
    const response = await apiClient.enhancedRegister(validation.data);
    console.log('Registration response:', JSON.stringify(response, null, 2)); // Debug log

    if (isApiError(response)) {
      console.log('API Error Response:', JSON.stringify(response, null, 2)); // Debug log
      await logSecurityEvent({
        action: 'ENHANCED_REGISTER_BACKEND_FAILED',
        resource: 'auth',
        success: false,
        category: SecurityEventCategory.AUTHENTICATION,
        severity: EventSeverity.HIGH,
        details: {
          email: sanitizedEmail,
          error: response.error,
          details: response.details,
        },
      });

      let errorMessage = response.error || 'Registration failed';
      if (
        response.details?.includes('409') ||
        errorMessage.includes('already exists')
      ) {
        errorMessage = 'User already exists';
      }

      return {
        success: false,
        error: errorMessage,
      };
    }

    const registrationResult = response.data!;

    await logSecurityEvent({
      action: 'ENHANCED_REGISTER_SUCCESS',
      resource: 'auth',
      success: true,
      category: SecurityEventCategory.AUTHENTICATION,
      severity: EventSeverity.MEDIUM,
      details: {
        email: sanitizedEmail,
        featuresAssigned: registrationResult.total_features_assigned,
        packageTier: sanitizedPackageTier,
      },
    });

    // Set auth cookie from registration response
    const cookies = await import('next/headers');
    const cookieStore = await cookies.cookies();

    cookieStore.set({
      name: 'auth-token',
      value: registrationResult.access_token,
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'lax',
      maxAge: registrationResult.expires_in,
      path: '/',
    });

    return {
      success: true,
      featuresUnlocked: registrationResult.features_unlocked || [],
      totalFeaturesAssigned: registrationResult.total_features_assigned || 0,
    };
  } catch (error) {
    logger.error('Enhanced registration action failed', {
      error: error instanceof Error ? error.message : 'Unknown error',
      email,
    });
    await logSecurityEvent({
      action: 'ENHANCED_REGISTER_ERROR',
      resource: 'auth',
      success: false,
      category: SecurityEventCategory.AUTHENTICATION,
      severity: EventSeverity.HIGH,
      details: {
        error: error instanceof Error ? error.message : 'Unknown error',
      },
    });
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Registration failed',
    };
  }
}
