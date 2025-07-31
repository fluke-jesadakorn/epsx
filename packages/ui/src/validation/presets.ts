import { z } from 'zod';
import { ValidationSchemas } from './schemas.js';
import { ValidationUtils, createValidator } from './validators.js';
import { FormHelpers } from './form-helpers.js';
import { InputSanitizer } from './sanitizers.js';

/**
 * Validation presets for common use cases
 * Provides ready-to-use validation configurations
 */

// Common field validation presets
export const FieldValidators = {
  // Basic fields
  email: createValidator(ValidationSchemas.base.email),
  password: createValidator(ValidationSchemas.base.password),
  phone: createValidator(ValidationSchemas.base.phone),
  url: createValidator(ValidationSchemas.base.url),
  uuid: createValidator(ValidationSchemas.base.uuid),
  
  // Text fields with common configurations
  shortText: createValidator(ValidationSchemas.base.text({ max: 100 })),
  mediumText: createValidator(ValidationSchemas.base.text({ max: 500 })),
  longText: createValidator(ValidationSchemas.base.text({ max: 2000 })),
  
  // Names and identifiers
  displayName: createValidator(ValidationSchemas.base.name()),
  username: createValidator(
    z.string()
      .min(3, 'Username must be at least 3 characters')
      .max(30, 'Username must be less than 30 characters')
      .regex(/^[a-zA-Z0-9_-]+$/, 'Username can only contain letters, numbers, underscores, and hyphens')
      .transform(str => str.toLowerCase())
  ),
  
  // Numbers with common ranges
  age: createValidator(ValidationSchemas.base.number({ min: 13, max: 120, integer: true })),
  rating: createValidator(ValidationSchemas.base.number({ min: 1, max: 5, integer: true })),
  percentage: createValidator(ValidationSchemas.base.number({ min: 0, max: 100 })),
  
  // Dates
  birthDate: createValidator(
    ValidationSchemas.base.date
      .refine(date => date <= new Date(), 'Birth date cannot be in the future')
      .refine(date => {
        const age = new Date().getFullYear() - date.getFullYear();
        return age >= 13 && age <= 120;
      }, 'Invalid birth date')
  ),
  
  futureDate: createValidator(
    ValidationSchemas.base.date
      .refine(date => date > new Date(), 'Date must be in the future')
  ),
  
  // Files
  avatar: createValidator(ValidationSchemas.base.file({
    required: false,
    maxSize: 5 * 1024 * 1024, // 5MB
    allowedTypes: ['image/jpeg', 'image/png', 'image/webp'],
  })),
  
  document: createValidator(ValidationSchemas.base.file({
    maxSize: 10 * 1024 * 1024, // 10MB
    allowedTypes: ['application/pdf', 'application/msword', 'text/plain'],
  })),
};

// Form validation presets
export const FormValidators = {
  // Authentication forms
  login: createValidator(ValidationSchemas.auth.login),
  register: createValidator(ValidationSchemas.auth.register),
  passwordReset: createValidator(ValidationSchemas.auth.passwordReset),
  changePassword: createValidator(ValidationSchemas.auth.changePassword),
  
  // Profile forms
  profile: createValidator(ValidationSchemas.profile.profile),
  settings: createValidator(ValidationSchemas.profile.settings),
  
  // Contact forms
  contact: createValidator(ValidationSchemas.contact.contact),
  support: createValidator(ValidationSchemas.contact.supportTicket),
  
  // Payment forms
  paymentMethod: createValidator(ValidationSchemas.payment.paymentMethod),
  subscription: createValidator(ValidationSchemas.payment.subscription),
  
  // Admin forms
  userCreate: createValidator(ValidationSchemas.admin.userCreate),
  userUpdate: createValidator(ValidationSchemas.admin.userUpdate),
  systemSettings: createValidator(ValidationSchemas.admin.systemSettings),
};

// Sanitization presets
export const SanitizationPresets = {
  // User input sanitization
  userInput: (input: string) => InputSanitizer.sanitizeText(input, {
    stripHtml: true,
    stripScripts: true,
    normalizeWhitespace: true,
    maxLength: 1000,
  }),
  
  // Rich text with safe HTML
  richText: (input: string) => InputSanitizer.sanitizeText(input, {
    stripHtml: true,
    allowedTags: ['p', 'br', 'strong', 'em', 'u', 'ol', 'ul', 'li', 'a'],
    stripScripts: true,
    normalizeWhitespace: true,
  }),
  
  // Search queries
  searchQuery: (input: string) => InputSanitizer.sanitizeText(input, {
    stripHtml: true,
    stripScripts: true,
    normalizeWhitespace: true,
    maxLength: 200,
    removeSpecialChars: true,
  }),
  
  // Usernames
  username: (input: string) => InputSanitizer.sanitizeText(input, {
    alphanumericOnly: true,
    toLowerCase: true,
    maxLength: 30,
  }),
  
  // Display text
  displayText: (input: string) => InputSanitizer.sanitizeForDisplay(input),
  
  // URLs
  url: (input: string) => InputSanitizer.sanitizeUrl(input),
  
  // Emails
  email: (input: string) => InputSanitizer.sanitizeEmail(input, {
    toLowerCase: true,
    removePlusAddressing: false,
  }),
  
  // Filenames
  filename: (input: string) => InputSanitizer.sanitizeFilename(input),
  
  // Phone numbers
  phone: (input: string) => InputSanitizer.sanitizePhone(input, {
    digitsOnly: true,
    format: 'e164',
  }),
};

// Validation workflows for complex forms
export const ValidationWorkflows = {
  /**
   * Multi-step form validation
   */
  multiStep: {
    validateStep: <T>(stepSchema: z.ZodSchema<T>, data: unknown, stepIndex: number) => {
      const result = ValidationUtils.validate(stepSchema, data);
      return {
        ...result,
        stepIndex,
        isComplete: result.success,
      };
    },
    
    validateAllSteps: <T>(schemas: z.ZodSchema<any>[], data: unknown[]) => {
      const results = schemas.map((schema, index) => 
        ValidationWorkflows.multiStep.validateStep(schema, data[index], index)
      );
      
      const hasErrors = results.some(result => !result.success);
      const completedSteps = results.filter(result => result.isComplete).length;
      
      return {
        success: !hasErrors,
        results,
        completedSteps,
        totalSteps: schemas.length,
        isComplete: completedSteps === schemas.length,
      };
    },
  },
  
  /**
   * Conditional validation based on field values
   */
  conditional: {
    validateWhen: <T>(
      schema: z.ZodSchema<T>,
      data: unknown,
      condition: (data: any) => boolean
    ) => {
      if (!condition(data)) {
        return { success: true, skipped: true };
      }
      
      return { ...ValidationUtils.validate(schema, data), skipped: false };
    },
    
    validateOneOf: <T>(
      schemas: { condition: (data: any) => boolean; schema: z.ZodSchema<T> }[],
      data: unknown
    ) => {
      for (const { condition, schema } of schemas) {
        if (condition(data)) {
          return ValidationUtils.validate(schema, data);
        }
      }
      
      return {
        success: false,
        error: 'No matching validation condition found',
      };
    },
  },
  
  /**
   * Server-side validation workflow
   */
  serverSide: {
    processFormData: async <T>(
      formData: FormData,
      schema: z.ZodSchema<T>,
      options: {
        rateLimitKey?: string;
        sanitize?: boolean;
        maxFileSize?: number;
      } = {}
    ) => {
      const { rateLimitKey, sanitize = true, maxFileSize = 10 * 1024 * 1024 } = options;
      
      return FormHelpers.processServerActionForm(formData, schema, {
        rateLimitKey,
        extractionOptions: {
          maxFileSize,
        },
        transform: sanitize ? (data: any) => {
          const sanitized: any = {};
          for (const [key, value] of Object.entries(data)) {
            if (typeof value === 'string') {
              sanitized[key] = SanitizationPresets.userInput(value);
            } else {
              sanitized[key] = value;
            }
          }
          return sanitized;
        } : undefined,
      });
    },
  },
  
  /**
   * Batch validation workflow
   */
  batch: {
    validateArray: <T>(
      items: unknown[],
      schema: z.ZodSchema<T>
    ) => {
      return ValidationUtils.validateBatch(items, (item) => 
        ValidationUtils.validate(schema, item)
      );
    },
    
    validateObjects: <T>(
      objects: Record<string, unknown>[],
      schema: z.ZodSchema<T>
    ) => {
      const results = objects.map((obj, index) => ({
        index,
        result: ValidationUtils.validate(schema, obj),
      }));
      
      const successful = results.filter(r => r.result.success);
      const failed = results.filter(r => !r.result.success);
      
      return {
        success: failed.length === 0,
        total: objects.length,
        successful: successful.length,
        failed: failed.length,
        results,
        data: successful.map(r => r.result.data).filter(Boolean),
        errors: failed.map(r => ({
          index: r.index,
          error: r.result.error,
          fieldErrors: r.result.fieldErrors,
        })),
      };
    },
  },
};

// Quick validation helpers
export const QuickValidators = {
  // Simple boolean checks
  isEmail: (value: string) => ValidationUtils.isValidEmail(value),
  isPhone: (value: string) => ValidationUtils.isValidPhone(value),
  isUrl: (value: string) => ValidationUtils.isValidUrl(value),
  isUuid: (value: string) => ValidationUtils.isValidUUID(value),
  isCreditCard: (value: string) => ValidationUtils.isValidCreditCard(value),
  isSlug: (value: string) => ValidationUtils.isValidSlug(value),
  
  // String checks
  isEmpty: (value: string) => !value || value.trim().length === 0,
  isNotEmpty: (value: string) => value && value.trim().length > 0,
  isAlphanumeric: (value: string) => /^[a-zA-Z0-9]+$/.test(value),
  isNumeric: (value: string) => /^\d+$/.test(value),
  isAlpha: (value: string) => /^[a-zA-Z]+$/.test(value),
  
  // Length checks
  hasMinLength: (value: string, min: number) => value.length >= min,
  hasMaxLength: (value: string, max: number) => value.length <= max,
  hasExactLength: (value: string, length: number) => value.length === length,
  
  // Password strength
  isWeakPassword: (password: string) => ValidationUtils.isPasswordWeak(password),
  getPasswordStrength: (password: string) => ValidationUtils.analyzePasswordStrength(password),
  
  // File validation
  isValidFile: (file: File, options: { maxSize?: number; allowedTypes?: string[] } = {}) => {
    const result = ValidationUtils.validateFile(file, options);
    return result.success;
  },
};

// Export all presets as a consolidated object
export const ValidationPresets = {
  fields: FieldValidators,
  forms: FormValidators,
  sanitization: SanitizationPresets,
  workflows: ValidationWorkflows,
  quick: QuickValidators,
} as const;