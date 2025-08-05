import { z } from 'zod';
import { fmtFileSize } from '@epsx/shared-utils/formatting';

// Validation configuration (consolidated from scattered implementations)
export const ValidationConfig = {
  email: {
    maxLength: 254,
    pattern: /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/,
  },
  password: {
    minLength: 8,
    maxLength: 128,
    requireUppercase: true,
    requireLowercase: true,
    requireNumbers: true,
    requireSpecialChars: true,
    specialChars: '!@#$%^&*()_+-=[]{}|;:,.<>?',
  },
  phone: {
    pattern: /^\+?[1-9]\d{1,14}$/, // E.164 format
  },
  url: {
    protocols: ['http', 'https'],
  },
  text: {
    minLength: 1,
    maxLength: 1000,
  },
  name: {
    minLength: 1,
    maxLength: 100,
    pattern: /^[a-zA-Z\s'-]+$/,
  },
} as const;

// Validation messages (consolidated from scattered implementations)
export const ValidationMessages = {
  required: (field: string) => `${field} is required`,
  email: {
    invalid: 'Please enter a valid email address',
    maxLength: `Email must be less than ${ValidationConfig.email.maxLength} characters`,
  },
  password: {
    minLength: `Password must be at least ${ValidationConfig.password.minLength} characters`,
    maxLength: `Password must be less than ${ValidationConfig.password.maxLength} characters`,
    requireUppercase: 'Password must contain at least one uppercase letter',
    requireLowercase: 'Password must contain at least one lowercase letter',
    requireNumbers: 'Password must contain at least one number',
    requireSpecialChars: `Password must contain at least one special character (${ValidationConfig.password.specialChars})`,
    weak: 'Password is too weak',
    mismatch: 'Passwords do not match',
  },
  phone: {
    invalid: 'Please enter a valid phone number',
  },
  url: {
    invalid: 'Please enter a valid URL',
  },
  text: {
    minLength: (min: number) => `Must be at least ${min} characters`,
    maxLength: (max: number) => `Must be less than ${max} characters`,
  },
  name: {
    invalid: 'Name can only contain letters, spaces, hyphens, and apostrophes',
    minLength: `Name must be at least ${ValidationConfig.name.minLength} characters`,
    maxLength: `Name must be less than ${ValidationConfig.name.maxLength} characters`,
  },
  file: {
    required: 'Please select a file',
    invalidType: (types: string[]) => `File must be one of: ${types.join(', ')}`,
    tooLarge: (maxSize: string) => `File size must be less than ${maxSize}`,
  },
} as const;

// Base field schemas (consolidates email validation from 5+ implementations)
export const BaseSchemas = {
  // Email validation (replaces 5+ duplicate implementations)
  email: z
    .string()
    .min(1, ValidationMessages.required('Email'))
    .max(ValidationConfig.email.maxLength, ValidationMessages.email.maxLength)
    .email(ValidationMessages.email.invalid)
    .transform((email) => email.toLowerCase().trim()),

  // Password validation (consolidates 4+ duplicate implementations)
  password: z
    .string()
    .min(ValidationConfig.password.minLength, ValidationMessages.password.minLength)
    .max(ValidationConfig.password.maxLength, ValidationMessages.password.maxLength)
    .refine(
      (password: string) => !ValidationConfig.password.requireUppercase || /[A-Z]/.test(password),
      ValidationMessages.password.requireUppercase
    )
    .refine(
      (password: string) => !ValidationConfig.password.requireLowercase || /[a-z]/.test(password),
      ValidationMessages.password.requireLowercase
    )
    .refine(
      (password: string) => !ValidationConfig.password.requireNumbers || /\d/.test(password),
      ValidationMessages.password.requireNumbers
    )
    .refine(
      (password: string) => {
        if (!ValidationConfig.password.requireSpecialChars) return true;
        const specialCharsRegex = new RegExp(`[${ValidationConfig.password.specialChars.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}]`);
        return specialCharsRegex.test(password);
      },
      ValidationMessages.password.requireSpecialChars
    ),

  // Confirm password (used in multiple forms)
  confirmPassword: (passwordField: string = 'password') =>
    z.string().min(1, ValidationMessages.required('Confirm Password')),

  // Phone validation
  phone: z
    .string()
    .optional()
    .refine(
      (phone: string | undefined) => !phone || ValidationConfig.phone.pattern.test(phone),
      ValidationMessages.phone.invalid
    )
    .transform((phone: string | undefined) => phone?.replace(/\s+/g, '') || undefined),

  // URL validation
  url: z
    .string()
    .url(ValidationMessages.url.invalid)
    .refine(
      (url: string) => {
        try {
          const parsed = new URL(url);
          return ValidationConfig.url.protocols.includes(parsed.protocol.slice(0, -1) as 'http' | 'https');
        } catch {
          return false;
        }
      },
      ValidationMessages.url.invalid
    ),

  // Text fields
  text: (options: { min?: number; max?: number; required?: boolean } = {}) => {
    const { min = ValidationConfig.text.minLength, max = ValidationConfig.text.maxLength, required = true } = options;
    
    let schema = z.string();
    
    if (required) {
      schema = schema.min(min, ValidationMessages.text.minLength(min));
    } else {
      schema = schema.optional();
    }
    
    return schema
      .refine((text: string | undefined) => !text || text.length <= max, ValidationMessages.text.maxLength(max))
      .transform((text: string | undefined) => text?.trim() || (required ? '' : undefined));
  },

  // Name fields (person names, display names, etc.)
  name: (options: { required?: boolean } = {}) => {
    const { required = true } = options;
    
    let schema = z.string();
    
    if (required) {
      schema = schema.min(ValidationConfig.name.minLength, ValidationMessages.name.minLength);
    } else {
      schema = schema.optional();
    }
    
    return schema
      .refine(
        (name: string | undefined) => !name || ValidationConfig.name.pattern.test(name),
        ValidationMessages.name.invalid
      )
      .refine(
        (name: string | undefined) => !name || name.length <= ValidationConfig.name.maxLength,
        ValidationMessages.name.maxLength
      )
      .transform((name: string | undefined) => name?.trim() || (required ? '' : undefined));
  },

  // UUID validation
  uuid: z.string().uuid('Invalid ID format'),

  // Boolean with string coercion
  boolean: z
    .union([z.boolean(), z.string()])
    .transform((val: boolean | string) => val === true || val === 'true'),

  // Number with string coercion
  number: (options: { min?: number; max?: number; integer?: boolean } = {}) => {
    const { min, max, integer = false } = options;
    
    let schema = z
      .union([z.number(), z.string()])
      .transform((val: number | string) => {
        const num = typeof val === 'string' ? parseFloat(val) : val;
        return isNaN(num) ? undefined : num;
      })
      .refine((val: number | undefined) => val !== undefined, 'Must be a valid number');
    
    if (min !== undefined) {
      schema = schema.refine((val: number | undefined) => val! >= min, `Must be at least ${min}`);
    }
    
    if (max !== undefined) {
      schema = schema.refine((val: number | undefined) => val! <= max, `Must be at most ${max}`);
    }
    
    if (integer) {
      schema = schema.refine((val: number | undefined) => Number.isInteger(val), 'Must be a whole number');
    }
    
    return schema;
  },

  // Date validation
  date: z
    .union([z.date(), z.string()])
    .transform((val: Date | string) => {
      if (val instanceof Date) return val;
      const date = new Date(val);
      return isNaN(date.getTime()) ? undefined : date;
    })
    .refine((val: Date | undefined) => val !== undefined, 'Must be a valid date'),

  // File validation
  file: (options: {
    required?: boolean;
    maxSize?: number;
    allowedTypes?: string[];
  } = {}) => {
    const { required = false, maxSize, allowedTypes } = options;
    
    let schema = z.instanceof(File);
    
    if (!required) {
      schema = schema.optional();
    }
    
    if (maxSize) {
      schema = schema.refine(
        (file: File | undefined) => !file || file.size <= maxSize,
        ValidationMessages.file.tooLarge(fmtFileSize(maxSize))
      );
    }
    
    if (allowedTypes?.length) {
      schema = schema.refine(
        (file: File | undefined) => !file || allowedTypes.includes(file.type),
        ValidationMessages.file.invalidType(allowedTypes)
      );
    }
    
    return schema;
  },
} as const;

// Authentication schemas (consolidates auth form validation)
export const AuthSchemas = {
  // Login form
  login: z.object({
    email: BaseSchemas.email,
    password: z.string().min(1, ValidationMessages.required('Password')),
    rememberMe: BaseSchemas.boolean.optional(),
  }),

  // Registration form
  register: z
    .object({
      email: BaseSchemas.email,
      password: BaseSchemas.password,
      confirmPassword: z.string().min(1, ValidationMessages.required('Confirm Password')),
      displayName: BaseSchemas.name({ required: false }),
      terms: BaseSchemas.boolean.refine((val: boolean) => val === true, 'You must accept the terms and conditions'),
    })
    .refine(
      (data: { password: string; confirmPassword: string }) => data.password === data.confirmPassword,
      {
        message: ValidationMessages.password.mismatch,
        path: ['confirmPassword'],
      }
    ),

  // Password reset request
  passwordResetRequest: z.object({
    email: BaseSchemas.email,
  }),

  // Password reset form
  passwordReset: z
    .object({
      token: z.string().min(1, ValidationMessages.required('Reset token')),
      password: BaseSchemas.password,
      confirmPassword: z.string().min(1, ValidationMessages.required('Confirm Password')),
    })
    .refine(
      (data: { password: string; confirmPassword: string }) => data.password === data.confirmPassword,
      {
        message: ValidationMessages.password.mismatch,
        path: ['confirmPassword'],
      }
    ),

  // Change password form
  changePassword: z
    .object({
      currentPassword: z.string().min(1, ValidationMessages.required('Current Password')),
      newPassword: BaseSchemas.password,
      confirmPassword: z.string().min(1, ValidationMessages.required('Confirm Password')),
    })
    .refine(
      (data: { newPassword: string; confirmPassword: string }) => data.newPassword === data.confirmPassword,
      {
        message: ValidationMessages.password.mismatch,
        path: ['confirmPassword'],
      }
    ),
} as const;

// User profile schemas
export const ProfileSchemas = {
  // Basic profile update
  profile: z.object({
    displayName: BaseSchemas.name({ required: false }),
    email: BaseSchemas.email,
    phone: BaseSchemas.phone,
    bio: BaseSchemas.text({ max: 500, required: false }),
    website: BaseSchemas.url.optional(),
    avatar: BaseSchemas.file({
      required: false,
      maxSize: 5 * 1024 * 1024, // 5MB
      allowedTypes: ['image/jpeg', 'image/png', 'image/webp'],
    }),
  }),

  // Settings update
  settings: z.object({
    notifications: z.object({
      email: BaseSchemas.boolean,
      push: BaseSchemas.boolean,
      marketing: BaseSchemas.boolean,
    }),
    privacy: z.object({
      profileVisibility: z.enum(['public', 'private']),
      showEmail: BaseSchemas.boolean,
    }),
    preferences: z.object({
      theme: z.enum(['light', 'dark', 'system']),
      language: z.string().min(2).max(5),
      timezone: z.string(),
    }),
  }),
} as const;

// Payment schemas (consolidates payment validation)
export const PaymentSchemas = {
  // Payment method
  paymentMethod: z.object({
    type: z.enum(['card', 'paypal', 'crypto']),
    cardNumber: z.string().optional(),
    expiryMonth: BaseSchemas.number({ min: 1, max: 12, integer: true }).optional(),
    expiryYear: BaseSchemas.number({ min: new Date().getFullYear(), integer: true }).optional(),
    cvv: z.string().min(3).max(4).optional(),
    billingAddress: z.object({
      street: BaseSchemas.text(),
      city: BaseSchemas.text(),
      state: BaseSchemas.text(),
      zipCode: z.string().min(5).max(10),
      country: z.string().length(2),
    }).optional(),
  }),

  // Subscription
  subscription: z.object({
    planId: BaseSchemas.uuid,
    paymentMethodId: BaseSchemas.uuid,
    couponCode: z.string().optional(),
  }),
} as const;

// Contact/Support schemas
export const ContactSchemas = {
  // Contact form
  contact: z.object({
    name: BaseSchemas.name(),
    email: BaseSchemas.email,
    subject: BaseSchemas.text({ max: 200 }),
    message: BaseSchemas.text({ min: 10, max: 2000 }),
    category: z.enum(['general', 'technical', 'billing', 'feature-request']),
    priority: z.enum(['low', 'medium', 'high']).optional(),
  }),

  // Support ticket
  supportTicket: z.object({
    title: BaseSchemas.text({ max: 200 }),
    description: BaseSchemas.text({ min: 20, max: 5000 }),
    category: z.enum(['bug', 'feature', 'question', 'billing']),
    priority: z.enum(['low', 'medium', 'high', 'urgent']),
    attachments: z.array(BaseSchemas.file({
      maxSize: 10 * 1024 * 1024, // 10MB
      allowedTypes: ['image/*', 'application/pdf', 'text/*'],
    })).optional(),
  }),
} as const;

// Admin schemas (consolidates admin validation)
export const AdminSchemas = {
  // User management
  userCreate: z.object({
    email: BaseSchemas.email,
    displayName: BaseSchemas.name(),
    role: z.enum(['user', 'admin', 'super_admin']),
    permissions: z.array(z.string()).optional(),
    isActive: BaseSchemas.boolean,
  }),

  userUpdate: z.object({
    displayName: BaseSchemas.name({ required: false }),
    role: z.enum(['user', 'admin', 'super_admin']).optional(),
    permissions: z.array(z.string()).optional(),
    isActive: BaseSchemas.boolean.optional(),
  }),

  // System settings
  systemSettings: z.object({
    siteName: BaseSchemas.text({ max: 100 }),
    siteDescription: BaseSchemas.text({ max: 500 }),
    maintenance: z.object({
      enabled: BaseSchemas.boolean,
      message: BaseSchemas.text({ required: false }),
      allowedRoles: z.array(z.string()).optional(),
    }),
    features: z.object({
      registration: BaseSchemas.boolean,
      passwordReset: BaseSchemas.boolean,
      socialLogin: BaseSchemas.boolean,
    }),
  }),
} as const;


// Export all schemas as a consolidated collection
export const ValidationSchemas = {
  base: BaseSchemas,
  auth: AuthSchemas,
  profile: ProfileSchemas,
  payment: PaymentSchemas,
  contact: ContactSchemas,
  admin: AdminSchemas,
} as const;

// Type exports for schema inference
export type LoginFormData = z.infer<typeof AuthSchemas.login>;
export type RegisterFormData = z.infer<typeof AuthSchemas.register>;
export type ProfileFormData = z.infer<typeof ProfileSchemas.profile>;
export type ContactFormData = z.infer<typeof ContactSchemas.contact>;
export type PaymentMethodData = z.infer<typeof PaymentSchemas.paymentMethod>;
export type SubscriptionData = z.infer<typeof PaymentSchemas.subscription>;