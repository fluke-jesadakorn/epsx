'use server';

import { 
  withServerAction, 
  createServerAction, 
  createAuthenticatedAction,
  type ActionResult 
} from '../core/action-wrapper';
import { serverGet, serverPost } from '../core/request';
import { z } from 'zod';
import { revalidatePath } from 'next/cache';
import { 
  LoginRequestSchema, 
  RegisterRequestSchema, 
  UserProfileSchema, 
  PasswordChangeRequestSchema, 
  PasswordResetRequestSchema,
  ProfileUpdateRequestSchema 
} from '@epsx/types';

// Use imported schemas from @epsx/types

// Enhanced Authentication Actions
// Note: Direct login via API is deprecated in favor of OIDC authorization code flow
export const enhancedLogin = withServerAction(
  'auth.login',
  async (credentials: z.infer<typeof LoginRequestSchema>, context) => {
    // For OIDC, login should redirect to /oauth/authorize instead of API calls
    // This function is kept for compatibility but should use frontend redirect
    throw new Error('Direct API login is deprecated. Use OIDC authorization code flow via /oauth/authorize');
  },
  {
    validateInput: LoginRequestSchema,
    validateOutput: UserProfileSchema,
    logLevel: 'info'
  }
);

export const enhancedLogout = createServerAction(
  'auth.logout',
  async (_, context) => {
    // Use OIDC logout endpoint
    return await serverPost('/oauth/logout', undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

export const logoutWithRevalidation = createServerAction(
  'auth.logoutWithRevalidation',
  async (_, context) => {
    try {
      // Call the OIDC logout endpoint to clear server-side session
      await serverPost('/oauth/logout', undefined, {
        action: context.action,
        userId: context.userId,
        requestId: context.requestId
      });
      
      // Revalidate the entire app to refresh server components with new auth state
      // This will cause NavigationWithAuth to re-fetch user data
      revalidatePath('/', 'layout');
      
      return { success: true, message: 'Logged out successfully' };
    } catch (error) {
      console.error('OIDC logout error:', error);
      // Still revalidate even if backend logout fails
      revalidatePath('/', 'layout');
      throw error;
    }
  }
);

export const enhancedGetCurrentUser = createServerAction(
  'auth.getCurrentUser',
  async (_, context) => {
    try {
      console.log('🔍 [getCurrentUser] Attempting to fetch user profile using OIDC userinfo');
      
      // Use OIDC userinfo endpoint instead of REST API
      const result = await serverGet('/oauth/userinfo', undefined, {
        action: context.action,
        userId: context.userId,
        requestId: context.requestId
      });
      
      // OIDC userinfo returns standard claims
      if (result) {
        console.log('✅ [getCurrentUser] Profile fetched successfully for:', result?.email || result?.sub || 'unknown');
        
        // Transform OIDC claims to expected frontend format
        return {
          id: result.sub,
          email: result.email,
          name: result.name,
          role: result.role,
          permissions: result.permissions || [],
          package_tier: result.package_tier,
          admin_modules: result.admin_modules || [],
          email_verified: result.email_verified,
          exp: result.exp,
          iat: result.iat
        };
      }
      
      return result;
      
    } catch (error) {
      console.error('❌ [getCurrentUser] Failed to fetch user profile via OIDC userinfo:', error);
      throw error;
    }
  }
);

export const enhancedRegister = withServerAction(
  'auth.register',
  async (userData: z.infer<typeof RegisterRequestSchema>, context) => {
    // Registration in OIDC is typically handled via external identity providers
    // or through special registration flows. For now, disable direct registration.
    throw new Error('Direct API registration is not supported in OIDC flow. Use identity provider registration.');
  },
  {
    validateInput: RegisterRequestSchema,
    logLevel: 'info'
  }
);

export const enhancedUpdateProfile = createAuthenticatedAction(
  'auth.updateProfile',
  async (data: z.infer<typeof ProfileUpdateRequestSchema>, context) => {
    // Profile updates in OIDC should go through the identity provider
    // or via OIDC-compliant user management endpoints
    throw new Error('Profile updates should be handled through identity provider or OIDC user management endpoints');
  },
  {
    validateInput: ProfileUpdateRequestSchema
  }
);

export const enhancedChangePassword = createAuthenticatedAction(
  'auth.changePassword',
  async (data: z.infer<typeof PasswordChangeRequestSchema>, context) => {
    // Password changes in OIDC should go through the identity provider
    // For Firebase Auth, this should be handled via Firebase SDK
    throw new Error('Password changes should be handled through identity provider (Firebase Auth)');
  },
  {
    validateInput: PasswordChangeRequestSchema,
    logLevel: 'warn' // Higher log level for security operations
  }
);

export const enhancedResetPassword = withServerAction(
  'auth.resetPassword',
  async (data: z.infer<typeof PasswordResetRequestSchema>, context) => {
    // Use OIDC password reset endpoint
    return await serverPost('/oauth/password-reset', data, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: PasswordResetRequestSchema,
    logLevel: 'warn' // Higher log level for security operations
  }
);

export const enhancedRefreshToken = createServerAction(
  'auth.refreshToken',
  async (_, context) => {
    // Use OIDC token endpoint for refresh token grant
    return await serverPost('/oauth/token', {
      grant_type: 'refresh_token'
    }, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

// Feature access functions with enhanced error handling
// Note: Feature access should be determined from JWT claims in OIDC
export const enhancedCheckFeatureAccess = createAuthenticatedAction(
  'auth.checkFeatureAccess',
  async (feature: string, context) => {
    // In OIDC, feature access should be determined from JWT token claims
    // rather than making additional API calls. This should be handled client-side.
    throw new Error('Feature access should be checked via JWT claims, not API calls in OIDC flow');
  },
  {
    validateInput: z.string().min(1, 'Feature name is required')
  }
);

export const enhancedGetUserFeatures = createAuthenticatedAction(
  'auth.getUserFeatures',
  async (_, context) => {
    // User features should be included in JWT token claims from userinfo endpoint
    throw new Error('User features should be retrieved from JWT claims via /oauth/userinfo');
  }
);

// Admin authentication functions  
export const enhancedAdminLogin = withServerAction(
  'auth.adminLogin',
  async (credentials: z.infer<typeof LoginRequestSchema>, context) => {
    // Admin login should use the same OIDC flow as regular users
    // Admin permissions are determined by JWT claims and roles
    throw new Error('Admin login should use OIDC authorization code flow via /oauth/authorize with admin scope');
  },
  {
    validateInput: LoginRequestSchema,
    logLevel: 'warn' // Higher log level for admin operations
  }
);

export const enhancedCheckAdminPermission = createAuthenticatedAction(
  'auth.checkAdminPermission',
  async (permission: string, context) => {
    // Admin permissions should be checked from JWT token claims
    // obtained via /oauth/userinfo endpoint
    throw new Error('Admin permissions should be checked via JWT claims from /oauth/userinfo');
  },
  {
    validateInput: z.string().min(1, 'Permission name is required'),
    logLevel: 'info'
  }
);

// Backward compatibility exports (legacy simple versions)
export const login = enhancedLogin;
export const logout = enhancedLogout;
export const getCurrentUser = enhancedGetCurrentUser;

// The logoutWithRevalidation function is already exported above
export const register = enhancedRegister;
export const updateProfile = enhancedUpdateProfile;
export const changePassword = enhancedChangePassword;
export const resetPassword = enhancedResetPassword;
export const refreshToken = enhancedRefreshToken;
export const checkFeatureAccess = enhancedCheckFeatureAccess;
export const getUserFeatures = enhancedGetUserFeatures;
export const adminLogin = enhancedAdminLogin;
export const checkAdminPermission = enhancedCheckAdminPermission;

// Additional exports needed by index.ts
export const getAdminSession = enhancedGetCurrentUser; // Admin session is same as getCurrentUser
export const requestPasswordReset = enhancedResetPassword; // Reset password functionality\n\n// Type exports for enhanced actions
export type EnhancedLoginResult = ActionResult<z.infer<typeof UserProfileSchema>>;
export type EnhancedRegisterResult = ActionResult<{ message: string; userId: string }>;
export type EnhancedProfileResult = ActionResult<z.infer<typeof UserProfileSchema>>;