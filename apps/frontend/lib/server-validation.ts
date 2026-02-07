'use server';

import { z } from 'zod';

// Import shared validation schemas
import {
  emailSchema,
  passwordSchema,
  nameSchema,
  reasonSchema,
  signInSchema,
  signUpSchema,
  passwordResetSchema,
  passwordChangeSchema,
  contactFormSchema
} from '@/shared/validators/schemas';

// Legacy compatibility schemas (will be migrated to shared)
export const commonSchemas = {
  email: emailSchema,
  password: passwordSchema,
  confirmPassword: z.string(),
  amount: z.string()
    .regex(/^\d+(\.\d{1,2})?$/, 'Please enter a valid amount'),
  phone: z.string()
    .regex(/^\+?[1-9]\d{1,14}$/, 'Please enter a valid phone number'),
  name: nameSchema,
  message: reasonSchema // Use shared reason schema for messages
};

// Form validation helper
export interface ValidationResult<T> {
  success: boolean;
  data?: T;
  fieldErrors?: Record<string, string[]>;
  error?: string;
}

export async function validateFormData<T>(
  schema: z.ZodSchema<T>,
  data: unknown
): Promise<ValidationResult<T>> {
  try {
    const result = schema.safeParse(data);
    
    if (!result.success) {
      const flattenedErrors = result.error.flatten().fieldErrors;
      // Filter out undefined values to match Record<string, string[]> type
      const fieldErrors: Record<string, string[]> = {};
      for (const [key, value] of Object.entries(flattenedErrors)) {
        if (value !== undefined && value !== null && Array.isArray(value)) {
          fieldErrors[key] = value;
        }
      }
      return {
        success: false,
        fieldErrors,
        error: 'Please fix the validation errors'
      };
    }
    
    return {
      success: true,
      data: result.data
    };
  } catch (error) {
    return {
      success: false,
      error: 'Validation failed'
    };
  }
}

// Form data extraction helper
export function extractFormData(formData: FormData): Record<string, string> {
  const data: Record<string, string> = {};
  for (const [key, value] of formData.entries()) {
    if (typeof value === 'string') {
      data[key] = value.trim();
    }
  }
  return data;
}

// Sanitization helpers
export function sanitizeInput(input: string): string {
  return input
    .trim()
    .replace(/[<>]/g, '') // Remove basic HTML tags
    .slice(0, 1000); // Limit length
}

export function sanitizeEmail(email: string): string {
  return email.toLowerCase().trim();
}

// Rate limiting helper (simple in-memory store)
const rateLimitStore = new Map<string, { count: number; resetTime: number }>();

export function checkRateLimit(identifier: string, maxAttempts = 5, windowMs: number = 15 * 60 * 1000): boolean {
  const now = Date.now();
  const record = rateLimitStore.get(identifier);
  
  if (!record || now > record.resetTime) {
    rateLimitStore.set(identifier, { count: 1, resetTime: now + windowMs });
    return true;
  }
  
  if (record.count >= maxAttempts) {
    return false;
  }
  
  record.count++;
  return true;
}

// Use shared form schemas with legacy aliases
export const authSchemas = {
  login: signInSchema, // Use shared sign-in schema
  register: signUpSchema, // Use shared sign-up schema  
  forgotPassword: passwordResetSchema, // Use shared password reset schema
  resetPassword: passwordChangeSchema // Use shared password change schema
};

export const paymentSchemas = {
  createPayment: z.object({
    currency: z.string().min(1, 'Currency is required'),
    amount: commonSchemas.amount,
    payment_method: z.enum(['on_line', 'on_chain']),
    product_name: z.string().min(1, 'Product name is required'),
    notify_url: z.string().url().optional()
  }),
  
  verifyPayment: z.object({
    transactionId: z.string().min(1, 'Transaction ID is required')
  })
};

export const contactSchemas = {
  contact: contactFormSchema.omit({ category: true }) // Use shared contact form schema, remove category field
};