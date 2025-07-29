'use server';

import { 
  withServerAction, 
  createServerAction, 
  createAuthenticatedAction,
  type ActionResult 
} from '../core/action-wrapper';
import { serverGet, serverPost } from '../core/enhanced-request';
import { z } from 'zod';
import type { 
  LoginRequest, 
  RegisterRequest, 
  UserProfile, 
  PasswordChangeRequest, 
  PasswordResetRequest,
  ProfileUpdateRequest 
} from '@epsx/types';

// Enhanced Input/Output Schemas
const LoginInputSchema = z.object({
  email: z.string().email('Please enter a valid email address'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
  type: z.literal('credentials').default('credentials')
});

const RegisterInputSchema = z.object({
  email: z.string().email('Please enter a valid email address'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
  name: z.string().min(2, 'Name must be at least 2 characters').optional(),
  package_tier: z.string().optional()
});

const UserProfileSchema = z.object({
  id: z.string(),
  email: z.string().email(),
  name: z.string().optional(),
  role: z.string(),
  isActive: z.boolean(),
  createdAt: z.date(),
  updatedAt: z.date()
});

// Enhanced Authentication Actions
export const enhancedLogin = withServerAction(
  'auth.login',
  async (credentials: LoginRequest, context) => {
    const result = await serverPost('/api/v1/auth/login', credentials, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
    return result;
  },
  {
    validateInput: LoginInputSchema,
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
  async (userData: RegisterRequest, context) => {
    return await serverPost('/api/v1/auth/register', userData, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: RegisterInputSchema,
    logLevel: 'info'
  }
);

export const enhancedUpdateProfile = createAuthenticatedAction(
  'auth.updateProfile',
  async (data: ProfileUpdateRequest, context) => {
    return await serverPost('/api/v1/auth/profile/update', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.object({
      name: z.string().min(2).optional(),
      displayName: z.string().min(2).optional(),
      avatar: z.string().url().optional(),
      preferences: z.record(z.any()).optional()
    })
  }
);

export const enhancedChangePassword = createAuthenticatedAction(
  'auth.changePassword',
  async (data: PasswordChangeRequest, context) => {
    return await serverPost('/api/v1/auth/change-password', data, {
      action: context.action,
      userId: context.userId,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.object({
      currentPassword: z.string().min(1, 'Current password is required'),
      newPassword: z.string().min(8, 'New password must be at least 8 characters')
    }),
    logLevel: 'warn' // Higher log level for security operations
  }
);

export const enhancedResetPassword = withServerAction(
  'auth.resetPassword',
  async (data: PasswordResetRequest, context) => {
    return await serverPost('/api/v1/auth/password-reset', data, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: z.object({
      email: z.string().email('Please enter a valid email address')
    }),
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
  async (credentials: LoginRequest, context) => {
    return await serverPost('/api/v1/admin/auth/login', credentials, {
      action: context.action,
      requestId: context.requestId
    });
  },
  {
    validateInput: LoginInputSchema,
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

// Type exports for enhanced actions
export type EnhancedLoginResult = ActionResult<UserProfile>;
export type EnhancedRegisterResult = ActionResult<{ message: string; userId: string }>;
export type EnhancedProfileResult = ActionResult<UserProfile>;