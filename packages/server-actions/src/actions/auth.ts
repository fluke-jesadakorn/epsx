'use server';

import { 
  withServerAction, 
  createServerAction, 
  createAuthenticatedAction,
  type ActionResult 
} from '../core/action-wrapper';
import { serverGet, serverPost } from '../core/enhanced-request';
import { z } from 'zod';
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
export const enhancedLogin = withServerAction(
  'auth.login',
  async (credentials: z.infer<typeof LoginRequestSchema>, context) => {
    const result = await serverPost('/api/v1/auth/login', credentials, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
    return result;
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
    return await serverPost('/api/v1/auth/logout', undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

export const enhancedGetCurrentUser = createServerAction(
  'auth.getCurrentUser',
  async (_, context) => {
    return await serverGet('/api/v1/auth/profile', undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

export const enhancedRegister = withServerAction(
  'auth.register',
  async (userData: z.infer<typeof RegisterRequestSchema>, context) => {
    return await serverPost('/api/v1/auth/register', userData, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: RegisterRequestSchema,
    logLevel: 'info'
  }
);

export const enhancedUpdateProfile = createAuthenticatedAction(
  'auth.updateProfile',
  async (data: z.infer<typeof ProfileUpdateRequestSchema>, context) => {
    return await serverPost('/api/v1/auth/profile/update', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: ProfileUpdateRequestSchema
  }
);

export const enhancedChangePassword = createAuthenticatedAction(
  'auth.changePassword',
  async (data: z.infer<typeof PasswordChangeRequestSchema>, context) => {
    return await serverPost('/api/v1/auth/change-password', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: PasswordChangeRequestSchema,
    logLevel: 'warn' // Higher log level for security operations
  }
);

export const enhancedResetPassword = withServerAction(
  'auth.resetPassword',
  async (data: z.infer<typeof PasswordResetRequestSchema>, context) => {
    return await serverPost('/api/v1/auth/password-reset', data, {
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
    return await serverPost('/api/v1/auth/refresh', undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

// Feature access functions with enhanced error handling
export const enhancedCheckFeatureAccess = createAuthenticatedAction(
  'auth.checkFeatureAccess',
  async (feature: string, context) => {
    return await serverGet('/api/v1/auth/features/check', { feature }, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.string().min(1, 'Feature name is required')
  }
);

export const enhancedGetUserFeatures = createAuthenticatedAction(
  'auth.getUserFeatures',
  async (_, context) => {
    return await serverGet('/api/v1/auth/features', undefined, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  }
);

// Admin authentication functions  
export const enhancedAdminLogin = withServerAction(
  'auth.adminLogin',
  async (credentials: z.infer<typeof LoginRequestSchema>, context) => {
    return await serverPost('/api/v1/admin/auth/login', credentials, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: LoginRequestSchema,
    logLevel: 'warn' // Higher log level for admin operations
  }
);

export const enhancedCheckAdminPermission = createAuthenticatedAction(
  'auth.checkAdminPermission',
  async (permission: string, context) => {
    return await serverGet('/api/v1/admin/auth/permissions/check', { permission }, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
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