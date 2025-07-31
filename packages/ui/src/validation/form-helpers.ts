import { z } from 'zod';
import { ValidationResult, ValidationUtils } from './validators.js';

// Form data extraction utilities (consolidates FormData processing patterns)
export interface FormDataExtractionOptions {
  /** Remove empty string values */
  trimEmpty?: boolean;
  /** Convert string numbers to numbers */
  parseNumbers?: boolean;
  /** Convert 'true'/'false' strings to booleans */
  parseBooleans?: boolean;
  /** Fields to parse as arrays (comma-separated or multiple values) */
  parseArrays?: string[];
  /** Maximum file size for file uploads */
  maxFileSize?: number;
  /** Allowed file types for file uploads */
  allowedFileTypes?: string[];
}

export interface FormValidationOptions<T> {
  /** Schema to validate against */
  schema: z.ZodSchema<T>;
  /** Abort validation on first error */
  abortEarly?: boolean;
  /** Transform data before validation */
  transform?: (data: any) => any;
  /** Custom field validators */
  customValidators?: Record<string, (value: any) => ValidationResult>;
}

/**
 * Form helper utilities that consolidate form processing patterns
 * Replaces scattered FormData extraction and validation code
 */
export class FormHelpers {
  /**
   * Extract and process FormData into a plain object
   * Consolidates form data processing from multiple implementations
   */
  static extractFormData(
    formData: FormData,
    options: FormDataExtractionOptions = {}
  ): Record<string, any> {
    const {
      trimEmpty = true,
      parseNumbers = true,
      parseBooleans = true,
      parseArrays = [],
      maxFileSize,
      allowedFileTypes,
    } = options;

    const result: Record<string, any> = {};

    for (const [key, value] of formData.entries()) {
      if (value instanceof File) {
        // Handle file uploads
        if (value.size === 0 && value.name === '') {
          // Empty file input
          continue;
        }

        // Validate file if options provided
        if (maxFileSize || allowedFileTypes) {
          const fileValidation = ValidationUtils.validateFile(value, {
            maxSize: maxFileSize,
            allowedTypes: allowedFileTypes,
          });

          if (!fileValidation.success) {
            throw new Error(fileValidation.error);
          }
        }

        if (parseArrays.includes(key)) {
          if (!result[key]) result[key] = [];
          (result[key] as File[]).push(value);
        } else {
          result[key] = value;
        }
      } else {
        // Handle regular form values
        let processedValue: any = value;

        // Trim and handle empty strings
        if (typeof processedValue === 'string') {
          processedValue = processedValue.trim();
          if (trimEmpty && processedValue === '') {
            continue;
          }
        }

        // Parse numbers
        if (parseNumbers && typeof processedValue === 'string') {
          const numberValue = Number(processedValue);
          if (!isNaN(numberValue) && isFinite(numberValue)) {
            processedValue = numberValue;
          }
        }

        // Parse booleans
        if (parseBooleans && typeof processedValue === 'string') {
          if (processedValue === 'true') processedValue = true;
          else if (processedValue === 'false') processedValue = false;
        }

        // Handle arrays
        if (parseArrays.includes(key)) {
          if (!result[key]) result[key] = [];
          
          if (typeof processedValue === 'string' && processedValue.includes(',')) {
            // Split comma-separated values
            const arrayValues = processedValue
              .split(',')
              .map(v => v.trim())
              .filter(v => v);
            result[key].push(...arrayValues);
          } else {
            result[key].push(processedValue);
          }
        } else {
          result[key] = processedValue;
        }
      }
    }

    return result;
  }

  /**
   * Validate form data with enhanced error handling
   * Consolidates form validation patterns from multiple components
   */
  static validateForm<T>(
    data: any,
    options: FormValidationOptions<T>
  ): ValidationResult<T> {
    const { schema, abortEarly = false, transform, customValidators = {} } = options;

    // Apply transformation if provided
    let processedData = transform ? transform(data) : data;

    // Run custom validators first
    const customErrors: Record<string, string[]> = {};
    
    for (const [field, validator] of Object.entries(customValidators)) {
      if (field in processedData) {
        const result = validator(processedData[field]);
        if (!result.success) {
          customErrors[field] = [result.error || 'Invalid value'];
          if (abortEarly) break;
        }
      }
    }

    if (Object.keys(customErrors).length > 0) {
      return {
        success: false,
        error: 'Custom validation failed',
        fieldErrors: customErrors,
      };
    }

    // Run schema validation
    return ValidationUtils.validate(schema, processedData, { abortEarly });
  }

  /**
   * Process server action form data
   * Consolidates server action form handling patterns
   */
  static async processServerActionForm<T>(
    formData: FormData,
    schema: z.ZodSchema<T>,
    options: {
      rateLimitKey?: string;
      maxRequests?: number;
      windowMs?: number;
      extractionOptions?: FormDataExtractionOptions;
      transform?: (data: any) => any;
    } = {}
  ): Promise<ValidationResult<T>> {
    const {
      rateLimitKey,
      maxRequests = 10,
      windowMs = 60000, // 1 minute
      extractionOptions = {},
      transform,
    } = options;

    try {
      // Rate limiting check
      if (rateLimitKey) {
        const rateLimitResult = ValidationUtils.checkRateLimit(
          rateLimitKey,
          maxRequests,
          windowMs
        );

        if (!rateLimitResult.allowed) {
          return {
            success: false,
            error: `Too many requests. Try again in ${rateLimitResult.retryAfter} seconds.`,
          };
        }
      }

      // Extract form data
      const extractedData = this.extractFormData(formData, extractionOptions);

      // Validate with schema
      return this.validateForm(extractedData, { schema, transform });
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Form processing failed',
      };
    }
  }

  /**
   * Create form field validator for React Hook Form
   * Provides integration with React Hook Form for consistent validation
   */
  static createFieldValidator<T>(schema: z.ZodSchema<T>) {
    return {
      validate: (value: any) => {
        const result = ValidationUtils.validate(schema, value);
        return result.success ? true : result.error || 'Invalid value';
      },
    };
  }

  /**
   * Create async field validator with debouncing
   * Useful for expensive validations like API calls
   */
  static createAsyncFieldValidator<T>(
    schema: z.ZodSchema<T>,
    debounceMs: number = 300
  ) {
    return {
      validate: async (value: any) => {
        const result = await ValidationUtils.validateAsync(schema, value, {
          debounceMs,
        });
        return result.success ? true : result.error || 'Invalid value';
      },
    };
  }

  /**
   * Generate form field props for consistent styling and behavior
   */
  static generateFieldProps(
    name: string,
    validation: ValidationResult,
    options: {
      required?: boolean;
      disabled?: boolean;
      placeholder?: string;
    } = {}
  ) {
    const { required = false, disabled = false, placeholder } = options;
    const hasError = !validation.success;
    const errorMessage = validation.fieldErrors?.[name]?.[0] || validation.error;

    return {
      name,
      required,
      disabled,
      placeholder,
      'aria-invalid': hasError,
      'aria-describedby': hasError ? `${name}-error` : undefined,
      className: hasError ? 'error' : '',
      error: hasError,
      errorMessage,
    };
  }

  /**
   * Create form submission handler with loading and error states
   */
  static createSubmissionHandler<T>(
    onSubmit: (data: T) => Promise<void> | void,
    options: {
      onSuccess?: (data: T) => void;
      onError?: (error: string) => void;
      onValidationError?: (errors: Record<string, string[]>) => void;
    } = {}
  ) {
    const { onSuccess, onError, onValidationError } = options;

    return async (data: T, validation: ValidationResult<T>) => {
      if (!validation.success) {
        if (validation.fieldErrors && onValidationError) {
          onValidationError(validation.fieldErrors);
        } else if (validation.error && onError) {
          onError(validation.error);
        }
        return;
      }

      try {
        await onSubmit(data);
        if (onSuccess) {
          onSuccess(data);
        }
      } catch (error) {
        if (onError) {
          onError(error instanceof Error ? error.message : 'Submission failed');
        }
      }
    };
  }

  /**
   * Handle file upload validation and processing
   */
  static processFileUploads(
    files: FileList | File[],
    options: {
      maxFiles?: number;
      maxSize?: number;
      allowedTypes?: string[];
      resizeImages?: boolean;
      maxWidth?: number;
      maxHeight?: number;
    } = {}
  ): ValidationResult<File[]> {
    const {
      maxFiles = 10,
      maxSize,
      allowedTypes,
      resizeImages = false,
      maxWidth = 1920,
      maxHeight = 1080,
    } = options;

    const fileArray = Array.from(files);

    // Check file count
    if (fileArray.length > maxFiles) {
      return {
        success: false,
        error: `Too many files. Maximum allowed: ${maxFiles}`,
      };
    }

    // Validate each file
    const validFiles: File[] = [];
    const errors: string[] = [];

    for (let i = 0; i < fileArray.length; i++) {
      const file = fileArray[i];
      const validation = ValidationUtils.validateFile(file, {
        maxSize,
        allowedTypes,
      });

      if (validation.success && validation.data) {
        validFiles.push(validation.data);
      } else {
        errors.push(`File ${i + 1}: ${validation.error}`);
      }
    }

    if (errors.length > 0) {
      return {
        success: false,
        error: errors.join(', '),
      };
    }

    return {
      success: true,
      data: validFiles,
    };
  }

  /**
   * Create form reset handler that clears validation state
   */
  static createResetHandler(
    resetForm: () => void,
    clearValidation: () => void,
    options: {
      onReset?: () => void;
      confirmReset?: boolean;
      confirmMessage?: string;
    } = {}
  ) {
    const {
      onReset,
      confirmReset = false,
      confirmMessage = 'Are you sure you want to reset the form?',
    } = options;

    return () => {
      if (confirmReset && !confirm(confirmMessage)) {
        return;
      }

      resetForm();
      clearValidation();

      if (onReset) {
        onReset();
      }
    };
  }

  /**
   * Generate validation summary for display
   */
  static generateValidationSummary(
    validation: ValidationResult,
    options: {
      showFieldErrors?: boolean;
      maxErrors?: number;
    } = {}
  ): {
    hasErrors: boolean;
    errorCount: number;
    generalError?: string;
    fieldErrors: Array<{ field: string; message: string }>;
    summary: string;
  } {
    const { showFieldErrors = true, maxErrors = 10 } = options;

    const hasErrors = !validation.success;
    const fieldErrors: Array<{ field: string; message: string }> = [];

    if (validation.fieldErrors && showFieldErrors) {
      for (const [field, messages] of Object.entries(validation.fieldErrors)) {
        for (const message of messages.slice(0, maxErrors)) {
          fieldErrors.push({ field, message });
        }
      }
    }

    let errorCount = fieldErrors.length;
    if (validation.error) errorCount++;

    let summary = '';
    if (hasErrors) {
      if (errorCount === 1) {
        summary = 'There is 1 error that needs to be fixed.';
      } else {
        summary = `There are ${errorCount} errors that need to be fixed.`;
      }
    }

    return {
      hasErrors,
      errorCount,
      generalError: validation.error,
      fieldErrors,
      summary,
    };
  }
}

// Convenience exports
export const extractFormData = FormHelpers.extractFormData;
export const validateForm = FormHelpers.validateForm;
export const processServerActionForm = FormHelpers.processServerActionForm;
export const createFieldValidator = FormHelpers.createFieldValidator;
export const createAsyncFieldValidator = FormHelpers.createAsyncFieldValidator;
export const generateFieldProps = FormHelpers.generateFieldProps;
export const createSubmissionHandler = FormHelpers.createSubmissionHandler;
export const processFileUploads = FormHelpers.processFileUploads;
export const createResetHandler = FormHelpers.createResetHandler;
export const generateValidationSummary = FormHelpers.generateValidationSummary;