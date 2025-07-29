import { z } from 'zod';

export type ValidationResult<T> = {
  success: true;
  data: T;
} | {
  success: false;
  errors: ValidationError[];
};

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

export type Schema<T> = z.ZodType<T>;

export interface ValidatedData<T> {
  data: T;
  isValid: boolean;
  errors?: ValidationError[];
}