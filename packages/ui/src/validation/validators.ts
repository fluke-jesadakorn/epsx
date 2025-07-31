import { z } from 'zod';
import { ValidationConfig, ValidationMessages } from './schemas.js';

// Validation result types (consolidates different result patterns)
export interface ValidationResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  fieldErrors?: Record<string, string[]>;
  warnings?: string[];
}

export interface AsyncValidationResult<T = any> extends ValidationResult<T> {
  loading?: boolean;
}

// Password strength analysis (consolidates 4+ password strength implementations)
export interface PasswordStrengthResult {
  score: number; // 0-4 (very weak to very strong)
  feedback: string[];
  requirements: {
    minLength: boolean;
    hasUppercase: boolean;
    hasLowercase: boolean;
    hasNumbers: boolean;
    hasSpecialChars: boolean;
  };
  estimatedCrackTime: string;
  isWeak: boolean;
}

// Rate limiting validation
export interface RateLimitResult {
  allowed: boolean;
  remaining: number;
  resetTime: Date;
  retryAfter?: number;
}

/**
 * Consolidated validation utility class that replaces scattered validation functions
 * Eliminates duplicate implementations across packages and apps
 */
export class ValidationUtils {
  /**
   * Validate data against a Zod schema with enhanced error handling
   * Consolidates schema validation from multiple packages
   */
  static validate<T>(
    schema: z.ZodSchema<T>,
    data: unknown,
    options: {
      transform?: boolean;
      stripUnknown?: boolean;
      abortEarly?: boolean;
    } = {}
  ): ValidationResult<T> {
    const { transform = true, stripUnknown = true, abortEarly = false } = options;

    try {
      let processedSchema = schema;

      if (stripUnknown && schema instanceof z.ZodObject) {
        processedSchema = schema.strict() as z.ZodSchema<T>;
      }

      const result = processedSchema.parse(data);

      return {
        success: true,
        data: result,
      };
    } catch (error) {
      if (error instanceof z.ZodError) {
        const fieldErrors: Record<string, string[]> = {};
        const errorMessages: string[] = [];

        for (const issue of error.issues) {
          const path = issue.path.join('.');
          const message = issue.message;

          if (path) {
            if (!fieldErrors[path]) {
              fieldErrors[path] = [];
            }
            fieldErrors[path].push(message);
          } else {
            errorMessages.push(message);
          }

          if (abortEarly) break;
        }

        return {
          success: false,
          error: errorMessages.length > 0 ? errorMessages[0] : 'Validation failed',
          fieldErrors: Object.keys(fieldErrors).length > 0 ? fieldErrors : undefined,
        };
      }

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Validation failed',
      };
    }
  }

  /**
   * Async validation with debouncing and loading states
   */
  static async validateAsync<T>(
    schema: z.ZodSchema<T>,
    data: unknown,
    options: {
      debounceMs?: number;
      signal?: AbortSignal;
    } = {}
  ): Promise<AsyncValidationResult<T>> {
    const { debounceMs = 0, signal } = options;

    if (debounceMs > 0) {
      await new Promise((resolve) => setTimeout(resolve, debounceMs));
    }

    if (signal?.aborted) {
      return {
        success: false,
        error: 'Validation cancelled',
        loading: false,
      };
    }

    return {
      ...this.validate(schema, data),
      loading: false,
    };
  }

  /**
   * Email validation (consolidates 5+ email validation implementations)
   */
  static isValidEmail(email: string): boolean {
    if (!email || typeof email !== 'string') return false;
    
    // Basic format check
    if (!ValidationConfig.email.pattern.test(email)) return false;
    
    // Length check
    if (email.length > ValidationConfig.email.maxLength) return false;
    
    // Additional checks for common mistakes
    const trimmed = email.trim().toLowerCase();
    
    // Check for consecutive dots
    if (trimmed.includes('..')) return false;
    
    // Check for valid TLD
    const parts = trimmed.split('@');
    if (parts.length !== 2) return false;
    
    const [localPart, domain] = parts;
    
    // Local part checks
    if (localPart.length === 0 || localPart.length > 64) return false;
    if (localPart.startsWith('.') || localPart.endsWith('.')) return false;
    
    // Domain checks
    if (domain.length === 0 || domain.length > 253) return false;
    if (domain.startsWith('.') || domain.endsWith('.')) return false;
    if (domain.includes('..')) return false;
    
    return true;
  }

  /**
   * Phone number validation (consolidates phone validation patterns)
   */
  static isValidPhone(phone: string): boolean {
    if (!phone || typeof phone !== 'string') return false;
    
    // Remove all non-digit characters except +
    const cleaned = phone.replace(/[^\d+]/g, '');
    
    // Check E.164 format
    return ValidationConfig.phone.pattern.test(cleaned);
  }

  /**
   * URL validation with protocol and domain checks
   */
  static isValidUrl(url: string): boolean {
    if (!url || typeof url !== 'string') return false;
    
    try {
      const parsed = new URL(url);
      return ValidationConfig.url.protocols.includes(parsed.protocol.slice(0, -1));
    } catch {
      return false;
    }
  }

  /**
   * Password strength analysis (consolidates multiple password strength implementations)
   */
  static analyzePasswordStrength(password: string): PasswordStrengthResult {
    if (!password || typeof password !== 'string') {
      return {
        score: 0,
        feedback: ['Password is required'],
        requirements: {
          minLength: false,
          hasUppercase: false,
          hasLowercase: false,
          hasNumbers: false,
          hasSpecialChars: false,
        },
        estimatedCrackTime: 'Instant',
        isWeak: true,
      };
    }

    const requirements = {
      minLength: password.length >= ValidationConfig.password.minLength,
      hasUppercase: /[A-Z]/.test(password),
      hasLowercase: /[a-z]/.test(password),
      hasNumbers: /\d/.test(password),
      hasSpecialChars: new RegExp(`[${ValidationConfig.password.specialChars.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}]`).test(password),
    };

    const feedback: string[] = [];
    let score = 0;

    // Length scoring
    if (password.length >= 12) score += 2;
    else if (password.length >= 8) score += 1;
    else feedback.push('Use at least 8 characters');

    // Character variety scoring
    if (requirements.hasUppercase) score += 1;
    else feedback.push('Add uppercase letters');

    if (requirements.hasLowercase) score += 1;
    else feedback.push('Add lowercase letters');

    if (requirements.hasNumbers) score += 1;
    else feedback.push('Add numbers');

    if (requirements.hasSpecialChars) score += 1;
    else feedback.push('Add special characters');

    // Common patterns penalty
    const commonPatterns = [
      /(.)\1{2,}/, // Repeated characters
      /123|abc|qwerty|password/i, // Common sequences
      /^(.{1,2})\1+$/, // Simple repeated patterns
    ];

    for (const pattern of commonPatterns) {
      if (pattern.test(password)) {
        score = Math.max(0, score - 1);
        if (pattern === commonPatterns[0]) feedback.push('Avoid repeated characters');
        else if (pattern === commonPatterns[1]) feedback.push('Avoid common words and sequences');
        else feedback.push('Avoid simple patterns');
        break;
      }
    }

    // Ensure score is within bounds
    score = Math.max(0, Math.min(4, score));

    // Estimate crack time (simplified)
    const crackTimes = [
      'Instant', // 0
      'Seconds', // 1
      'Minutes', // 2
      'Hours', // 3
      'Years', // 4
    ];

    const isWeak = score < 3;

    if (feedback.length === 0 && score >= 3) {
      feedback.push('Strong password!');
    }

    return {
      score,
      feedback,
      requirements,
      estimatedCrackTime: crackTimes[score],
      isWeak,
    };
  }

  /**
   * Check if password meets minimum requirements
   */
  static isPasswordWeak(password: string): boolean {
    const analysis = this.analyzePasswordStrength(password);
    return analysis.isWeak;
  }

  /**
   * Validate file upload (consolidates file validation patterns)
   */
  static validateFile(
    file: File,
    options: {
      maxSize?: number;
      allowedTypes?: string[];
      allowedExtensions?: string[];
    } = {}
  ): ValidationResult<File> {
    const { maxSize, allowedTypes, allowedExtensions } = options;

    // Size check
    if (maxSize && file.size > maxSize) {
      return {
        success: false,
        error: `File size (${this.formatFileSize(file.size)}) exceeds maximum allowed size (${this.formatFileSize(maxSize)})`,
      };
    }

    // Type check
    if (allowedTypes?.length) {
      const isAllowed = allowedTypes.some(type => {
        if (type.endsWith('/*')) {
          return file.type.startsWith(type.slice(0, -1));
        }
        return file.type === type;
      });

      if (!isAllowed) {
        return {
          success: false,
          error: `File type (${file.type}) is not allowed. Allowed types: ${allowedTypes.join(', ')}`,
        };
      }
    }

    // Extension check
    if (allowedExtensions?.length) {
      const extension = file.name.split('.').pop()?.toLowerCase();
      if (!extension || !allowedExtensions.includes(extension)) {
        return {
          success: false,
          error: `File extension (.${extension}) is not allowed. Allowed extensions: ${allowedExtensions.join(', ')}`,
        };
      }
    }

    return {
      success: true,
      data: file,
    };
  }

  /**
   * Rate limiting validation (consolidates rate limiting patterns)
   */
  static checkRateLimit(
    key: string,
    maxRequests: number,
    windowMs: number,
    storage: Map<string, { count: number; resetTime: number }> = new Map()
  ): RateLimitResult {
    const now = Date.now();
    const existing = storage.get(key);

    if (!existing || now > existing.resetTime) {
      // Reset window
      const resetTime = now + windowMs;
      storage.set(key, { count: 1, resetTime });
      
      return {
        allowed: true,
        remaining: maxRequests - 1,
        resetTime: new Date(resetTime),
      };
    }

    if (existing.count >= maxRequests) {
      return {
        allowed: false,
        remaining: 0,
        resetTime: new Date(existing.resetTime),
        retryAfter: Math.ceil((existing.resetTime - now) / 1000),
      };
    }

    existing.count += 1;
    storage.set(key, existing);

    return {
      allowed: true,
      remaining: maxRequests - existing.count,
      resetTime: new Date(existing.resetTime),
    };
  }

  /**
   * Credit card validation (basic Luhn algorithm)
   */
  static isValidCreditCard(cardNumber: string): boolean {
    if (!cardNumber || typeof cardNumber !== 'string') return false;
    
    // Remove spaces and non-digits
    const cleaned = cardNumber.replace(/\D/g, '');
    
    // Check length (13-19 digits for most cards)
    if (cleaned.length < 13 || cleaned.length > 19) return false;
    
    // Luhn algorithm
    let sum = 0;
    let isEven = false;
    
    for (let i = cleaned.length - 1; i >= 0; i--) {
      let digit = parseInt(cleaned.charAt(i));
      
      if (isEven) {
        digit *= 2;
        if (digit > 9) {
          digit -= 9;
        }
      }
      
      sum += digit;
      isEven = !isEven;
    }
    
    return sum % 10 === 0;
  }

  /**
   * UUID validation
   */
  static isValidUUID(uuid: string): boolean {
    if (!uuid || typeof uuid !== 'string') return false;
    
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    return uuidRegex.test(uuid);
  }

  /**
   * Slug validation (URL-friendly strings)
   */
  static isValidSlug(slug: string): boolean {
    if (!slug || typeof slug !== 'string') return false;
    
    const slugRegex = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
    return slugRegex.test(slug) && slug.length <= 100;
  }

  /**
   * Generate a slug from text
   */
  static generateSlug(text: string): string {
    if (!text || typeof text !== 'string') return '';
    
    return text
      .toLowerCase()
      .trim()
      .replace(/[^\w\s-]/g, '') // Remove special characters
      .replace(/[\s_-]+/g, '-') // Replace spaces and underscores with hyphens
      .replace(/^-+|-+$/g, ''); // Remove leading/trailing hyphens
  }

  /**
   * Format file size for display
   */
  static formatFileSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = bytes;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${Math.round(size * 100) / 100}${units[unitIndex]}`;
  }

  /**
   * Deep validation for nested objects
   */
  static validateNested<T>(
    data: unknown,
    validators: Record<string, (value: any) => ValidationResult>
  ): ValidationResult<T> {
    if (!data || typeof data !== 'object') {
      return {
        success: false,
        error: 'Data must be an object',
      };
    }

    const fieldErrors: Record<string, string[]> = {};
    const validatedData: any = {};
    
    for (const [key, validator] of Object.entries(validators)) {
      const value = (data as any)[key];
      const result = validator(value);
      
      if (result.success) {
        validatedData[key] = result.data;
      } else {
        fieldErrors[key] = [result.error || 'Invalid value'];
      }
    }

    if (Object.keys(fieldErrors).length > 0) {
      return {
        success: false,
        error: 'Validation failed',
        fieldErrors,
      };
    }

    return {
      success: true,
      data: validatedData as T,
    };
  }

  /**
   * Batch validation for arrays
   */
  static validateBatch<T>(
    items: unknown[],
    validator: (item: unknown) => ValidationResult<T>
  ): ValidationResult<T[]> {
    const results: T[] = [];
    const errors: string[] = [];
    
    for (let i = 0; i < items.length; i++) {
      const result = validator(items[i]);
      
      if (result.success && result.data) {
        results.push(result.data);
      } else {
        errors.push(`Item ${i}: ${result.error || 'Invalid'}`);
      }
    }

    if (errors.length > 0) {
      return {
        success: false,
        error: `Batch validation failed: ${errors.join(', ')}`,
      };
    }

    return {
      success: true,
      data: results,
    };
  }
}

// Convenience functions for common validations
export const isValidEmail = ValidationUtils.isValidEmail;
export const isValidPhone = ValidationUtils.isValidPhone;
export const isValidUrl = ValidationUtils.isValidUrl;
export const isPasswordWeak = ValidationUtils.isPasswordWeak;
export const analyzePasswordStrength = ValidationUtils.analyzePasswordStrength;
export const validateFile = ValidationUtils.validateFile;
export const isValidCreditCard = ValidationUtils.isValidCreditCard;
export const isValidUUID = ValidationUtils.isValidUUID;
export const isValidSlug = ValidationUtils.isValidSlug;
export const generateSlug = ValidationUtils.generateSlug;
export const formatFileSize = ValidationUtils.formatFileSize;

// Validator factory functions
export const createValidator = <T>(schema: z.ZodSchema<T>) => {
  return (data: unknown) => ValidationUtils.validate(schema, data);
};

export const createAsyncValidator = <T>(schema: z.ZodSchema<T>) => {
  return (data: unknown, options?: { debounceMs?: number; signal?: AbortSignal }) =>
    ValidationUtils.validateAsync(schema, data, options);
};