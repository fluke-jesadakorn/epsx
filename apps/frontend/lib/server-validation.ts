'use server';

import { z } from 'zod';

// Common validation schemas
export const commonSchemas = {
  email: z.string().email('Please enter a valid email address'),
  password: z.string()
    .min(8, 'Password must be at least 8 characters long')
    .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
    .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
    .regex(/[0-9]/, 'Password must contain at least one number'),
  confirmPassword: z.string(),
  amount: z.string()
    .regex(/^\d+(\.\d{1,2})?$/, 'Please enter a valid amount'),
  phone: z.string()
    .regex(/^\+?[1-9]\d{1,14}$/, 'Please enter a valid phone number'),
  name: z.string()
    .min(2, 'Name must be at least 2 characters long')
    .max(50, 'Name must be less than 50 characters'),
  message: z.string()
    .min(10, 'Message must be at least 10 characters long')
    .max(1000, 'Message must be less than 1000 characters')
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
      const fieldErrors = result.error.flatten().fieldErrors;
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

export function checkRateLimit(identifier: string, maxAttempts: number = 5, windowMs: number = 15 * 60 * 1000): boolean {
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

// Common form schemas
export const authSchemas = {
  login: z.object({
    email: commonSchemas.email,
    password: z.string().min(1, 'Password is required')
  }),
  
  register: z.object({
    email: commonSchemas.email,
    password: commonSchemas.password,
    confirmPassword: commonSchemas.confirmPassword,
    name: commonSchemas.name.optional()
  }).refine(data => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ["confirmPassword"]
  }),
  
  forgotPassword: z.object({
    email: commonSchemas.email
  }),
  
  resetPassword: z.object({
    token: z.string().min(1, 'Reset token is required'),
    password: commonSchemas.password,
    confirmPassword: commonSchemas.confirmPassword
  }).refine(data => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ["confirmPassword"]
  })
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
  contact: z.object({
    name: commonSchemas.name,
    email: commonSchemas.email,
    phone: commonSchemas.phone.optional(),
    message: commonSchemas.message
  })
};