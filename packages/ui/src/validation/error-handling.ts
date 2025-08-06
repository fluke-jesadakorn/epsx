import type { ValidationResult } from './validators.js';
import type { z } from 'zod';


// Error types for validation
export interface ValidationError {
  field: string;
  message: string;
  code: string;
  value?: unknown;
}

export interface ValidationErrorSummary {
  hasErrors: boolean;
  totalErrors: number;
  generalErrors: string[];
  fieldErrors: ValidationError[];
  formattedMessage: string;
}

// Error formatting options
export interface ErrorFormattingOptions {
  /** Include field names in error messages */
  includeFieldNames?: boolean;
  /** Format field names (e.g., camelCase to Title Case) */
  formatFieldNames?: boolean;
  /** Maximum number of errors to display */
  maxErrors?: number;
  /** Separator for multiple errors */
  separator?: string;
  /** Custom field name mappings */
  fieldNameMap?: Record<string, string>;
}

// Error display options
export interface ErrorDisplayOptions {
  /** Show inline field errors */
  showFieldErrors?: boolean;
  /** Show general form errors */
  showGeneralErrors?: boolean;
  /** Group errors by field */
  groupByField?: boolean;
  /** Custom error templates */
  templates?: {
    fieldError?: (error: ValidationError) => string;
    generalError?: (error: string) => string;
    summary?: (summary: ValidationErrorSummary) => string;
  };
}

/**
 * Validation error handling utilities
 * Consolidates error processing and formatting from multiple implementations
 */
export class ValidationErrorHandler {
  /**
   * Transform Zod errors into a standardized format
   * Consolidates Zod error handling from multiple locations
   */
  static transformZodError(
    error: z.ZodError,
    options: ErrorFormattingOptions = {}
  ): ValidationError[] {
    const {
      includeFieldNames = true,
      formatFieldNames = true,
      fieldNameMap = {},
    } = options;

    return error.issues.map((issue) => {
      const fieldPath = issue.path.join('.');
      let fieldName = fieldPath;

      // Apply custom field name mapping
      if (fieldNameMap[fieldPath]) {
        fieldName = fieldNameMap[fieldPath];
      } else if (formatFieldNames) {
        fieldName = this.formatFieldName(fieldPath);
      }

      // Customize error message based on error type
      let message = issue.message;
      
      if (includeFieldNames && fieldName) {
        // Check if message already includes field name
        if (!message.toLowerCase().includes(fieldName.toLowerCase())) {
          message = `${fieldName}: ${message}`;
        }
      }

      return {
        field: fieldPath,
        message,
        code: issue.code,
        value: issue.path.length > 0 ? this.getNestedValue(issue.path) : undefined,
      };
    });
  }

  /**
   * Format field names for display (camelCase to Title Case)
   */
  static formatFieldName(fieldName: string): string {
    if (!fieldName) return '';

    return fieldName
      .replace(/([A-Z])/g, ' $1') // Add space before capitals
      .replace(/^./, (str) => str.toUpperCase()) // Capitalize first letter
      .replace(/[._-]/g, ' ') // Replace separators with spaces
      .replace(/\s+/g, ' ') // Normalize multiple spaces
      .trim();
  }

  /**
   * Create error summary from validation result
   */
  static createErrorSummary(
    validation: ValidationResult,
    options: ErrorFormattingOptions = {}
  ): ValidationErrorSummary {
    const {
      maxErrors = 10,
      separator = '; ',
      includeFieldNames = true,
    } = options;

    const generalErrors: string[] = [];
    const fieldErrors: ValidationError[] = [];

    // Add general error if present
    if (validation.error) {
      generalErrors.push(validation.error);
    }

    // Process field errors
    if (validation.fieldErrors) {
      for (const [field, messages] of Object.entries(validation.fieldErrors)) {
        for (const message of messages.slice(0, maxErrors)) {
          fieldErrors.push({
            field,
            message: includeFieldNames ? `${this.formatFieldName(field)}: ${message}` : message,
            code: 'field_error',
          });
        }
      }
    }

    const totalErrors = generalErrors.length + fieldErrors.length;
    const hasErrors = totalErrors > 0;

    // Create formatted message
    let formattedMessage = '';
    if (hasErrors) {
      const allMessages = [
        ...generalErrors,
        ...fieldErrors.slice(0, maxErrors).map(e => e.message),
      ];
      formattedMessage = allMessages.join(separator);
    }

    return {
      hasErrors,
      totalErrors,
      generalErrors,
      fieldErrors,
      formattedMessage,
    };
  }

  /**
   * Format errors for display in forms
   */
  static formatForForm(
    validation: ValidationResult,
    options: ErrorDisplayOptions = {}
  ): {
    generalErrors: string[];
    fieldErrors: Record<string, string>;
    hasErrors: boolean;
  } {
    const {
      showFieldErrors = true,
      showGeneralErrors = true,
      templates,
    } = options;

    const generalErrors: string[] = [];
    const fieldErrors: Record<string, string> = {};

    // Process general errors
    if (showGeneralErrors && validation.error) {
      const formatted = templates?.generalError 
        ? templates.generalError(validation.error)
        : validation.error;
      generalErrors.push(formatted);
    }

    // Process field errors
    if (showFieldErrors && validation.fieldErrors) {
      for (const [field, messages] of Object.entries(validation.fieldErrors)) {
        const message = messages[0]; // Take first error for each field
        const formatted = templates?.fieldError 
          ? templates.fieldError({ field, message, code: 'field_error' })
          : message;
        fieldErrors[field] = formatted;
      }
    }

    return {
      generalErrors,
      fieldErrors,
      hasErrors: !validation.success,
    };
  }

  /**
   * Create user-friendly error messages
   */
  static createUserFriendlyMessage(
    validation: ValidationResult,
    context: {
      operation?: string; // e.g., 'login', 'registration', 'profile update'
      entity?: string; // e.g., 'account', 'profile', 'payment'
    } = {}
  ): string {
    const { operation = 'operation', entity = 'form' } = context;

    if (validation.success) {
      return `${operation} completed successfully.`;
    }

    const summary = this.createErrorSummary(validation);

    if (summary.totalErrors === 1) {
      return summary.formattedMessage;
    }

    if (summary.totalErrors <= 3) {
      return `Please fix the following issues with your ${entity}: ${summary.formattedMessage}`;
    }

    return `Please fix ${summary.totalErrors} issues with your ${entity}. ${summary.generalErrors[0] || summary.fieldErrors[0]?.message || 'Check the form for errors.'}`;
  }

  /**
   * Extract first error message for quick display
   */
  static getFirstErrorMessage(validation: ValidationResult): string | null {
    if (validation.success) return null;

    // Return general error first
    if (validation.error) return validation.error;

    // Return first field error
    if (validation.fieldErrors) {
      for (const messages of Object.values(validation.fieldErrors)) {
        if (messages.length > 0) return messages[0];
      }
    }

    return 'Validation failed';
  }

  /**
   * Check if specific field has errors
   */
  static hasFieldError(validation: ValidationResult, fieldName: string): boolean {
    return !!(validation.fieldErrors?.[fieldName]?.length);
  }

  /**
   * Get error messages for specific field
   */
  static getFieldErrors(validation: ValidationResult, fieldName: string): string[] {
    return validation.fieldErrors?.[fieldName] || [];
  }

  /**
   * Get first error message for specific field
   */
  static getFirstFieldError(validation: ValidationResult, fieldName: string): string | null {
    const errors = this.getFieldErrors(validation, fieldName);
    return errors.length > 0 ? errors[0] : null;
  }

  /**
   * Create error object for API responses
   */
  static createApiErrorResponse(
    validation: ValidationResult,
    statusCode: number = 400
  ): {
    success: false;
    error: string;
    statusCode: number;
    fieldErrors?: Record<string, string[]>;
    timestamp: string;
  } {
    return {
      success: false,
      error: validation.error || 'Validation failed',
      statusCode,
      fieldErrors: validation.fieldErrors,
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * Merge multiple validation results
   */
  static mergeValidationResults(
    results: ValidationResult[]
  ): ValidationResult {
    const allFieldErrors: Record<string, string[]> = {};
    const generalErrors: string[] = [];
    let hasSuccess = true;

    for (const result of results) {
      if (!result.success) {
        hasSuccess = false;

        // Collect general errors
        if (result.error) {
          generalErrors.push(result.error);
        }

        // Collect field errors
        if (result.fieldErrors) {
          for (const [field, messages] of Object.entries(result.fieldErrors)) {
            if (!allFieldErrors[field]) {
              allFieldErrors[field] = [];
            }
            allFieldErrors[field].push(...messages);
          }
        }
      }
    }

    if (hasSuccess) {
      return { success: true };
    }

    return {
      success: false,
      error: generalErrors.length > 0 ? generalErrors.join('; ') : undefined,
      fieldErrors: Object.keys(allFieldErrors).length > 0 ? allFieldErrors : undefined,
    };
  }

  /**
   * Filter errors by field pattern
   */
  static filterErrorsByField(
    validation: ValidationResult,
    fieldPattern: string | RegExp
  ): ValidationResult {
    if (validation.success || !validation.fieldErrors) {
      return validation;
    }

    const regex = fieldPattern instanceof RegExp 
      ? fieldPattern 
      : new RegExp(fieldPattern);

    const filteredFieldErrors: Record<string, string[]> = {};

    for (const [field, messages] of Object.entries(validation.fieldErrors)) {
      if (regex.test(field)) {
        filteredFieldErrors[field] = messages;
      }
    }

    return {
      success: Object.keys(filteredFieldErrors).length === 0,
      error: validation.error,
      fieldErrors: filteredFieldErrors,
    };
  }

  /**
   * Convert validation result to form state for React Hook Form
   */
  static toFormState(validation: ValidationResult): { isValid: boolean; errors: Record<string, unknown> } {
    if (validation.success) {
      return {
        isValid: true,
        errors: {},
      };
    }

    const errors: Record<string, { message: string }> = {};

    if (validation.fieldErrors) {
      for (const [field, messages] of Object.entries(validation.fieldErrors)) {
        errors[field] = { message: messages[0] };
      }
    }

    return {
      isValid: false,
      errors,
      generalError: validation.error,
    };
  }

  /**
   * Helper to get nested value from object path
   */
  private static getNestedValue(_path: (string | number)[]): unknown {
    // This would need access to the original data object
    // For now, return undefined
    return undefined;
  }

  /**
   * Create validation error from simple message
   */
  static createError(message: string, field?: string): ValidationResult {
    if (field) {
      return {
        success: false,
        fieldErrors: {
          [field]: [message],
        },
      };
    }

    return {
      success: false,
      error: message,
    };
  }

  /**
   * Create success result
   */
  static createSuccess<T>(data?: T): ValidationResult<T> {
    return {
      success: true,
      data,
    };
  }
}

// Convenience exports
export const transformZodError = ValidationErrorHandler.transformZodError;
export const formatFieldName = ValidationErrorHandler.formatFieldName;
export const createErrorSummary = ValidationErrorHandler.createErrorSummary;
export const formatForForm = ValidationErrorHandler.formatForForm;
export const createUserFriendlyMessage = ValidationErrorHandler.createUserFriendlyMessage;
export const getFirstErrorMessage = ValidationErrorHandler.getFirstErrorMessage;
export const hasFieldError = ValidationErrorHandler.hasFieldError;
export const getFieldErrors = ValidationErrorHandler.getFieldErrors;
export const getFirstFieldError = ValidationErrorHandler.getFirstFieldError;
export const createApiErrorResponse = ValidationErrorHandler.createApiErrorResponse;
export const mergeValidationResults = ValidationErrorHandler.mergeValidationResults;
export const filterErrorsByField = ValidationErrorHandler.filterErrorsByField;
export const toFormState = ValidationErrorHandler.toFormState;
export const createError = ValidationErrorHandler.createError;
export const createSuccess = ValidationErrorHandler.createSuccess;

// Error message templates
export const ErrorTemplates = {
  required: (field: string) => `${field} is required`,
  email: (field: string) => `${field} must be a valid email address`,
  password: (field: string) => `${field} must meet security requirements`,
  minLength: (field: string, min: number) => `${field} must be at least ${min} characters`,
  maxLength: (field: string, max: number) => `${field} must be less than ${max} characters`,
  pattern: (field: string, pattern: string) => `${field} format is invalid (expected: ${pattern})`,
  custom: (field: string, message: string) => `${field}: ${message}`,
} as const;