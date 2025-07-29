import { z } from 'zod';
import type { ValidationResult, ValidationError, Schema } from './types';
import { SchemaValidationError } from '../error-handling';

export class Validator {
  static validate<T>(schema: Schema<T>, data: unknown): ValidationResult<T> {
    try {
      const result = schema.parse(data);
      return { success: true, data: result };
    } catch (error) {
      if (error instanceof z.ZodError) {
        const errors: ValidationError[] = error.errors.map(err => ({
          field: err.path.join('.'),
          message: err.message,
          code: err.code
        }));
        return { success: false, errors };
      }
      
      return {
        success: false,
        errors: [{
          field: 'unknown',
          message: 'Validation failed',
          code: 'VALIDATION_ERROR'
        }]
      };
    }
  }

  static async validateAsync<T>(schema: Schema<T>, data: unknown): Promise<ValidationResult<T>> {
    try {
      const result = await schema.parseAsync(data);
      return { success: true, data: result };
    } catch (error) {
      if (error instanceof z.ZodError) {
        const errors: ValidationError[] = error.errors.map(err => ({
          field: err.path.join('.'),
          message: err.message,
          code: err.code
        }));
        return { success: false, errors };
      }
      
      return {
        success: false,
        errors: [{
          field: 'unknown',
          message: 'Validation failed',
          code: 'VALIDATION_ERROR'
        }]
      };
    }
  }

  static throwOnError<T>(schema: Schema<T>, data: unknown): T {
    const result = this.validate(schema, data);
    if (!result.success) {
      throw new SchemaValidationError(
        'Validation failed',
        result.errors[0]?.field,
        result.errors.map(e => e.message),
        { errors: result.errors }
      );
    }
    return result.data;
  }
}

// Common validation schemas
export const CommonSchemas = {
  email: z.string().email(),
  password: z.string().min(8),
  uuid: z.string().uuid(),
  url: z.string().url(),
  phoneNumber: z.string().regex(/^\+?[\d\s\-\(\)]+$/),
  positiveNumber: z.number().positive(),
  nonEmptyString: z.string().min(1),
} as const;