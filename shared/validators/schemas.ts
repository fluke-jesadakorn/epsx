/**
 * UNIFIED VALIDATION SCHEMAS
 * 
 * Consolidates validation schemas for both admin-frontend and frontend applications.
 * Replaces duplicate Zod schema definitions with a single, comprehensive set.
 * 
 * Features:
 * - Common field validation (email, names, dates, etc.)
 * - User management schemas
 * - Permission management schemas  
 * - Platform-aware validation
 * - Reusable schema building blocks
 * - Type-safe validation with Zod
 */

import { z } from 'zod';

// ============================================================================
// BASE FIELD SCHEMAS (Building Blocks)
// ============================================================================

// Email validation with custom error messages
export const emailSchema = z.string()
  .min(1, 'Email is required')
  .email('Please enter a valid email address')
  .max(254, 'Email address is too long')
  .toLowerCase()
  .trim();

// Password validation for authentication forms
export const passwordSchema = z.string()
  .min(8, 'Password must be at least 8 characters')
  .max(128, 'Password is too long')
  .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/, 
    'Password must contain at least one uppercase letter, one lowercase letter, and one number');

// Name fields with reasonable constraints
export const nameSchema = z.string()
  .min(1, 'Name is required')
  .max(50, 'Name is too long')
  .trim()
  .regex(/^[a-zA-Z\s\-'\.]+$/, 'Name can only contain letters, spaces, hyphens, apostrophes, and periods');

// Optional name schema
export const optionalNameSchema = z.string()
  .max(50, 'Name is too long')
  .trim()
  .regex(/^[a-zA-Z\s\-'\.]*$/, 'Name can only contain letters, spaces, hyphens, apostrophes, and periods')
  .optional();

// Display name with more flexibility
export const displayNameSchema = z.string()
  .min(1, 'Display name is required')
  .max(100, 'Display name is too long')
  .trim();

export const optionalDisplayNameSchema = displayNameSchema.optional();

// URL validation
export const urlSchema = z.string()
  .url('Please enter a valid URL')
  .max(2048, 'URL is too long');

export const optionalUrlSchema = urlSchema.optional().or(z.literal(''));

// Date validation
export const dateSchema = z.string()
  .refine((date) => !isNaN(Date.parse(date)), 'Please enter a valid date');

export const optionalDateSchema = dateSchema.optional().or(z.literal(''));

// Time validation (HH:MM format)
export const timeSchema = z.string()
  .regex(/^([0-1]?[0-9]|2[0-3]):[0-5][0-9]$/, 'Please enter a valid time (HH:MM)');

export const optionalTimeSchema = timeSchema.optional().or(z.literal(''));

// Reason/description fields
export const reasonSchema = z.string()
  .min(10, 'Please provide a reason (minimum 10 characters)')
  .max(1000, 'Reason is too long')
  .trim();

export const descriptionSchema = z.string()
  .max(500, 'Description is too long')
  .trim()
  .optional();

// ID validation
export const idSchema = z.string()
  .min(1, 'ID is required')
  .max(100, 'ID is too long');

// Permission string validation
export const permissionSchema = z.string()
  .min(1, 'Permission is required')
  .regex(/^[a-zA-Z0-9_:*-]+$/, 'Invalid permission format');

// Platform validation
export const platformSchema = z.enum(['admin', 'frontend', 'epsx', 'epsx-pay', 'epsx-token'], {
  message: 'Please select a valid platform'
});

// Role validation
export const roleSchema = z.enum(['admin', 'user', 'premium_user'], {
  message: 'Please select a valid role'
});

// ============================================================================
// USER MANAGEMENT SCHEMAS
// ============================================================================

// Base user fields used across different forms
export const baseUserFields = {
  email: emailSchema,
  firstName: optionalNameSchema,
  lastName: optionalNameSchema,
  displayName: optionalDisplayNameSchema,
};

// Create user schema (admin-frontend)
export const createUserSchema = z.object({
  ...baseUserFields,
  role: roleSchema,
  packageTier: z.string().min(1, 'Package tier is required'), // Legacy, will be removed
  permissions: z.array(permissionSchema),
  isActive: z.boolean()
});

// Edit user schema (admin-frontend)  
export const editUserSchema = z.object({
  id: idSchema,
  ...baseUserFields,
  role: roleSchema.optional(),
  packageTier: z.string().optional(), // Legacy, will be removed
  permissions: z.array(permissionSchema).optional(),
  isActive: z.boolean().optional()
});

// Bulk user operations schema
export const bulkUserSchema = z.object({
  userIds: z.array(idSchema).min(1, 'At least one user is required'),
  operation: z.enum(['update_role', 'update_tier', 'assign_permissions', 'activate', 'deactivate', 'delete']),
  role: roleSchema.optional(),
  packageTier: z.string().optional(), // Legacy, will be removed
  permissions: z.array(permissionSchema).optional(),
  reason: reasonSchema
});

// User profile update schema (frontend)
export const updateProfileSchema = z.object({
  firstName: optionalNameSchema,
  lastName: optionalNameSchema,
  displayName: optionalDisplayNameSchema,
});

// ============================================================================
// PERMISSION MANAGEMENT SCHEMAS
// ============================================================================

// Grant permission schema
export const grantPermissionSchema = z.object({
  userId: idSchema,
  permissions: z.array(permissionSchema).min(1, 'At least one permission is required'),
  expiryDate: optionalDateSchema,
  expiryTime: optionalTimeSchema,
  reason: reasonSchema
});

// Revoke permission schema
export const revokePermissionSchema = z.object({
  userId: idSchema,
  permissions: z.array(permissionSchema).min(1, 'At least one permission is required'),
  reason: reasonSchema
});

// Temporary permission schema
export const temporaryPermissionSchema = z.object({
  userId: idSchema,
  permissions: z.array(permissionSchema).min(1, 'At least one permission is required'),
  duration: z.number().min(1, 'Duration must be at least 1 minute').max(525600, 'Duration cannot exceed 1 year'),
  durationType: z.enum(['minutes', 'hours', 'days', 'weeks']),
  reason: reasonSchema
});

// Permission template schema
export const permissionTemplateSchema = z.object({
  name: z.string().min(1, 'Template name is required').max(100, 'Name is too long'),
  description: descriptionSchema,
  permissions: z.array(permissionSchema).min(1, 'At least one permission is required'),
  category: z.string().min(1, 'Category is required'),
  isSystem: z.boolean().default(false)
});

// ============================================================================
// AUTHENTICATION SCHEMAS
// ============================================================================

// Sign in schema
export const signInSchema = z.object({
  email: emailSchema,
  password: z.string().min(1, 'Password is required'), // Don't validate complexity on sign in
  rememberMe: z.boolean().optional()
});

// Sign up schema
export const signUpSchema = z.object({
  email: emailSchema,
  password: passwordSchema,
  confirmPassword: z.string(),
  firstName: nameSchema,
  lastName: nameSchema,
  acceptTerms: z.boolean().refine(val => val === true, 'You must accept the terms and conditions')
}).refine(data => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword']
});

// Password reset schema
export const passwordResetSchema = z.object({
  email: emailSchema
});

// Password change schema
export const passwordChangeSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: passwordSchema,
  confirmPassword: z.string()
}).refine(data => data.newPassword === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword']
});

// ============================================================================
// PLAN MANAGEMENT SCHEMAS (Admin)
// ============================================================================

// Create plan schema
export const createPlanSchema = z.object({
  name: z.string().min(1, 'Plan name is required').max(100, 'Name is too long'),
  description: descriptionSchema,
  planType: z.string().min(1, 'Plan type is required'),
  currentPrice: z.number().min(0, 'Price must be 0 or greater'),
  currency: z.string().length(3, 'Currency must be 3 characters (e.g., USD)'),
  targetAudience: z.enum(['web_users', 'api_developers', 'enterprises']),
  billingModel: z.enum(['subscription', 'pay_per_use', 'hybrid']),
  planCategory: z.enum(['standard', 'api', 'enterprise', 'custom']),
  isActive: z.boolean().default(true),
  metadata: z.record(z.string(), z.any()).optional()
});

// Update plan schema
export const updatePlanSchema = createPlanSchema.partial().extend({
  id: z.number().min(1, 'Plan ID is required')
});

// ============================================================================
// NOTIFICATION SCHEMAS (Admin)
// ============================================================================

// Create notification schema
export const createNotificationSchema = z.object({
  title: z.string().min(1, 'Title is required').max(200, 'Title is too long'),
  message: z.string().min(1, 'Message is required').max(1000, 'Message is too long'),
  type: z.enum(['system', 'admin', 'data', 'feature', 'security']),
  priority: z.enum(['low', 'normal', 'high', 'urgent']),
  userId: idSchema.optional(),
  userIds: z.array(idSchema).optional(),
  actionUrl: optionalUrlSchema,
  metadata: z.record(z.string(), z.any()).optional()
});

// Broadcast notification schema
export const broadcastNotificationSchema = z.object({
  title: z.string().min(1, 'Title is required').max(200, 'Title is too long'),
  message: z.string().min(1, 'Message is required').max(1000, 'Message is too long'),
  type: z.enum(['system', 'admin', 'data', 'feature', 'security']),
  priority: z.enum(['low', 'normal', 'high', 'urgent']),
  userIds: z.array(idSchema).optional(),
  allUsers: z.boolean().default(false)
});

// ============================================================================
// API KEY MANAGEMENT SCHEMAS (Admin)
// ============================================================================

// Create API key schema
export const createApiKeySchema = z.object({
  clientName: z.string().min(1, 'Client name is required').max(100, 'Name is too long'),
  clientDescription: descriptionSchema,
  clientContactEmail: emailSchema.optional(),
  allowedModules: z.array(z.object({
    moduleId: idSchema,
    moduleName: z.string(),
    accessLevel: z.enum(['bronze', 'silver', 'gold', 'platinum', 'enterprise']),
    customQuotas: z.record(z.string(), z.any()).optional()
  })).min(1, 'At least one module is required'),
  ipRestrictions: z.array(z.string().regex(/^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$/, 'Invalid IP address')).optional(),
  expiresAt: optionalDateSchema,
  rateLimits: z.record(z.string(), z.number()).optional()
});

// ============================================================================
// FORM FIELD SCHEMAS (UI Components)
// ============================================================================

// Search/filter schemas
export const searchSchema = z.object({
  query: z.string().max(200, 'Search query is too long').trim(),
  filters: z.record(z.string(), z.any()).optional(),
  sortBy: z.string().optional(),
  sortOrder: z.enum(['asc', 'desc']).optional(),
  page: z.number().min(1).optional(),
  limit: z.number().min(1).max(100).optional()
});

// Contact form schema
export const contactFormSchema = z.object({
  name: nameSchema,
  email: emailSchema,
  subject: z.string().min(1, 'Subject is required').max(200, 'Subject is too long'),
  message: z.string().min(10, 'Message must be at least 10 characters').max(2000, 'Message is too long'),
  category: z.enum(['general', 'support', 'sales', 'technical']).optional()
});

// Feedback form schema
export const feedbackSchema = z.object({
  rating: z.number().min(1, 'Rating is required').max(5, 'Rating must be between 1 and 5'),
  feedback: z.string().min(10, 'Feedback must be at least 10 characters').max(1000, 'Feedback is too long'),
  category: z.enum(['bug', 'feature', 'improvement', 'other']),
  email: emailSchema.optional()
});

// ============================================================================
// TYPE INFERENCE HELPERS
// ============================================================================

// Infer types from schemas for TypeScript
export type CreateUserForm = z.infer<typeof createUserSchema>;
export type EditUserForm = z.infer<typeof editUserSchema>;
export type BulkUserForm = z.infer<typeof bulkUserSchema>;
export type UpdateProfileForm = z.infer<typeof updateProfileSchema>;

export type GrantPermissionForm = z.infer<typeof grantPermissionSchema>;
export type RevokePermissionForm = z.infer<typeof revokePermissionSchema>;
export type TemporaryPermissionForm = z.infer<typeof temporaryPermissionSchema>;
export type PermissionTemplateForm = z.infer<typeof permissionTemplateSchema>;

export type SignInForm = z.infer<typeof signInSchema>;
export type SignUpForm = z.infer<typeof signUpSchema>;
export type PasswordResetForm = z.infer<typeof passwordResetSchema>;
export type PasswordChangeForm = z.infer<typeof passwordChangeSchema>;

export type CreatePlanForm = z.infer<typeof createPlanSchema>;
export type UpdatePlanForm = z.infer<typeof updatePlanSchema>;

export type CreateNotificationForm = z.infer<typeof createNotificationSchema>;
export type BroadcastNotificationForm = z.infer<typeof broadcastNotificationSchema>;

export type CreateApiKeyForm = z.infer<typeof createApiKeySchema>;

export type SearchForm = z.infer<typeof searchSchema>;
export type ContactForm = z.infer<typeof contactFormSchema>;
export type FeedbackForm = z.infer<typeof feedbackSchema>;

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Helper function to create conditional validation
 */
export const createConditionalSchema = <T>(
  condition: (data: any) => boolean,
  schema: z.ZodSchema<T>,
  fallback: z.ZodSchema<T>
) => {
  return z.any().superRefine((data, ctx) => {
    const targetSchema = condition(data) ? schema : fallback;
    const result = targetSchema.safeParse(data);
    
    if (!result.success) {
      result.error.issues.forEach(issue => {
        ctx.addIssue({
          ...issue,
          path: issue.path
        });
      });
    }
  });
};

/**
 * Helper to validate array of unique strings
 */
export const uniqueStringArraySchema = (minLength = 0) => 
  z.array(z.string())
    .min(minLength, minLength > 0 ? `At least ${minLength} item${minLength !== 1 ? 's' : ''} required` : undefined)
    .refine(arr => new Set(arr).size === arr.length, 'Duplicate values are not allowed');

/**
 * Helper to create platform-specific validation
 */
export const createPlatformSchema = (platform: 'admin' | 'frontend') => {
  const baseSchema = z.object({
    platform: z.literal(platform)
  });
  
  return {
    extend: <T extends z.ZodRawShape>(shape: T) => baseSchema.extend(shape),
    merge: <T extends z.ZodTypeAny>(schema: T) => baseSchema.merge(schema)
  };
};

// ============================================================================
// CUSTOM VALIDATION FUNCTIONS
// ============================================================================

/**
 * Validate permission format: platform:resource:action or platform:resource:action:timestamp
 */
export const isValidPermission = (permission: string): boolean => {
  return /^[a-zA-Z0-9_]+:[a-zA-Z0-9_*]+:[a-zA-Z0-9_*]+(?::\d+)?$/.test(permission);
};

/**
 * Validate embedded timestamp permission
 */
export const isValidEmbeddedPermission = (permission: string): boolean => {
  const parts = permission.split(':');
  if (parts.length !== 4) return false;
  
  const timestamp = parseInt(parts[3], 10);
  return !isNaN(timestamp) && timestamp > 1000000000; // Valid Unix timestamp
};

/**
 * Validate email domain
 */
export const isValidEmailDomain = (email: string, allowedDomains: string[]): boolean => {
  const domain = email.split('@')[1];
  return allowedDomains.includes(domain);
};

/**
 * Validate strong password
 */
export const isStrongPassword = (password: string): boolean => {
  return /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$/.test(password);
};

/**
 * Validate phone number (international format)
 */
export const isValidPhoneNumber = (phone: string): boolean => {
  return /^\+[1-9]\d{1,14}$/.test(phone);
};

// ============================================================================
// ERROR MESSAGE HELPERS
// ============================================================================

/**
 * Get user-friendly error messages
 */
export const getErrorMessage = (error: z.ZodError): string => {
  const firstError = error.issues[0];
  if (!firstError) return 'Validation error';
  
  return firstError.message;
};

/**
 * Get all error messages by field
 */
export const getErrorMessages = (error: z.ZodError): Record<string, string[]> => {
  const messages: Record<string, string[]> = {};
  
  error.issues.forEach(issue => {
    const path = issue.path.join('.');
    if (!messages[path]) messages[path] = [];
    messages[path].push(issue.message);
  });
  
  return messages;
};