/**
 * Validation Utilities
 * Environment validation, input validation, and schema validation utilities
 */

// ============================================================================
// Environment Validation
// ============================================================================

export interface EnvironmentVariable {
  key: string;
  required: boolean;
  defaultValue?: string;
  validator?: (value: string) => boolean;
  description?: string;
}

export class EnvironmentValidator {
  private variables: EnvironmentVariable[] = [];
  private validated = false;

  add(variable: EnvironmentVariable): void {
    this.variables.push(variable);
    this.validated = false;
  }

  validate(): { valid: boolean; missing: string[]; invalid: string[]; warnings: string[] } {
    const missing: string[] = [];
    const invalid: string[] = [];
    const warnings: string[] = [];

    for (const variable of this.variables) {
      const value = process.env[variable.key];

      if (!value && variable.required) {
        missing.push(variable.key);
        continue;
      }

      if (value && variable.validator && !variable.validator(value)) {
        invalid.push(variable.key);
        continue;
      }

      if (!value && !variable.required && !variable.defaultValue) {
        warnings.push(`Optional environment variable ${variable.key} is not set`);
      }
    }

    this.validated = true;

    return {
      valid: missing.length === 0 && invalid.length === 0,
      missing,
      invalid,
      warnings
    };
  }

  getVariable(key: string): string | undefined {
    if (!this.validated) {
      this.validate();
    }

    const variable = this.variables.find(v => v.key === key);
    return process.env[key] || variable?.defaultValue;
  }

  isValid(): boolean {
    const result = this.validate();
    return result.valid;
  }

  generateTemplate(): string {
    const lines: string[] = ['# Environment Variables Configuration'];

    for (const variable of this.variables) {
      lines.push('');

      if (variable.description) {
        lines.push(`# ${variable.description}`);
      }

      lines.push(`# Required: ${variable.required}`);

      if (variable.defaultValue) {
        lines.push(`# Default: ${variable.defaultValue}`);
      }

      lines.push(`${variable.key}=${variable.defaultValue || ''}`);
    }

    return lines.join('\n');
  }
}

// ============================================================================
// Input Validation
// ============================================================================

export interface ValidationRule {
  required?: boolean;
  minLength?: number;
  maxLength?: number;
  min?: number;
  max?: number;
  pattern?: RegExp;
  custom?: (value: unknown) => boolean | string;
  type?: 'string' | 'number' | 'boolean' | 'email' | 'url' | 'date';
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  value?: unknown;
}

export class InputValidator {
  validate(value: unknown, rules: ValidationRule): ValidationResult {
    const errors: string[] = [];

    // Required check
    if (rules.required && (value === undefined || value === null || value === '')) {
      errors.push('This field is required');
      return { valid: false, errors };
    }

    // Skip other validations if value is empty and not required
    if (!rules.required && (value === undefined || value === null || value === '')) {
      return { valid: true, errors: [], value };
    }

    // Type validation
    if (rules.type) {
      const typeError = this.validateType(value, rules.type);
      if (typeError) {
        errors.push(typeError);
      }
    }

    // String validations
    if (typeof value === 'string') {
      if (rules.minLength && value.length < rules.minLength) {
        errors.push(`Minimum length is ${rules.minLength} characters`);
      }

      if (rules.maxLength && value.length > rules.maxLength) {
        errors.push(`Maximum length is ${rules.maxLength} characters`);
      }

      if (rules.pattern && !rules.pattern.test(value)) {
        errors.push('Invalid format');
      }
    }

    // Number validations
    if (typeof value === 'number') {
      if (rules.min !== undefined && value < rules.min) {
        errors.push(`Minimum value is ${rules.min}`);
      }

      if (rules.max !== undefined && value > rules.max) {
        errors.push(`Maximum value is ${rules.max}`);
      }
    }

    // Custom validation
    if (rules.custom) {
      const customResult = rules.custom(value);
      if (typeof customResult === 'string') {
        errors.push(customResult);
      } else if (!customResult) {
        errors.push('Custom validation failed');
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      value: this.convertType(value, rules.type)
    };
  }

  private validateType(value: unknown, type: ValidationRule['type']): string | null {
    switch (type) {
      case 'string':
        return typeof value !== 'string' ? 'Must be a string' : null;

      case 'number':
        return isNaN(Number(value)) ? 'Must be a number' : null;

      case 'boolean':
        return typeof value !== 'boolean' && value !== 'true' && value !== 'false' ? 'Must be a boolean' : null;

      case 'email': {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return !emailRegex.test(String(value)) ? 'Must be a valid email address' : null;
      }

      case 'url':
        try {
          new URL(String(value));
          return null;
        } catch {
          return 'Must be a valid URL';
        }

      case 'date': {
        const date = new Date(value as string | number | Date);
        return isNaN(date.getTime()) ? 'Must be a valid date' : null;
      }

      default:
        return null;
    }
  }

  private convertType(value: unknown, type?: ValidationRule['type']): unknown {
    if (!type) return value;

    switch (type) {
      case 'number':
        return Number(value);

      case 'boolean':
        if (typeof value === 'boolean') return value;
        return value === 'true' || value === true;

      case 'date':
        return new Date(value as string | number | Date);

      default:
        return value;
    }
  }

  validateObject(obj: Record<string, unknown>, schema: Record<string, ValidationRule>): ValidationResult {
    const errors: string[] = [];
    const validatedValues: Record<string, unknown> = {};

    for (const [key, rules] of Object.entries(schema)) {
      const value = obj[key];
      const result = this.validate(value, rules);

      if (!result.valid) {
        errors.push(...result.errors.map(error => `${key}: ${error}`));
      } else if (result.value !== undefined) {
        validatedValues[key] = result.value;
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      value: validatedValues
    };
  }
}

// ============================================================================
// Common Validation Patterns
// ============================================================================

export const commonPatterns = {
  email: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
  phone: /^\+?[\d\s\-()]+$/,
  url: /^https?:\/\/.+/,
  slug: /^[a-z0-9]+(?:-[a-z0-9]+)*$/,
  username: /^[a-zA-Z0-9_-]{3,20}$/,
  password: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$/,
  hexColor: /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/,
  ipAddress: /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
  creditCard: /^\d{4}\s?\d{4}\s?\d{4}\s?\d{4}$/,
  postalCode: /^\d{5}(-\d{4})?$/
};

export const commonRules = {
  email: {
    required: true,
    type: 'email' as const,
    maxLength: 254
  },

  password: {
    required: true,
    type: 'string' as const,
    minLength: 8,
    pattern: commonPatterns.password,
    custom: (value: string) => {
      if (!/(?=.*[a-z])/.test(value)) return 'Must contain at least one lowercase letter';
      if (!/(?=.*[A-Z])/.test(value)) return 'Must contain at least one uppercase letter';
      if (!/(?=.*\d)/.test(value)) return 'Must contain at least one number';
      if (!/(?=.*[@$!%*?&])/.test(value)) return 'Must contain at least one special character';
      return true;
    }
  },

  username: {
    required: true,
    type: 'string' as const,
    minLength: 3,
    maxLength: 20,
    pattern: commonPatterns.username
  },

  url: {
    required: false,
    type: 'url' as const
  },

  phone: {
    required: false,
    type: 'string' as const,
    pattern: commonPatterns.phone
  }
};

// ============================================================================
// Form Validation Helper
// ============================================================================

export class FormValidator {
  private validator = new InputValidator();
  private schema: Record<string, ValidationRule> = {};

  setSchema(schema: Record<string, ValidationRule>): void {
    this.schema = schema;
  }

  addField(name: string, rules: ValidationRule): void {
    this.schema[name] = rules;
  }

  removeField(name: string): void {
    delete this.schema[name];
  }

  validate(formData: Record<string, unknown>): ValidationResult {
    return this.validator.validateObject(formData, this.schema);
  }

  validateField(fieldName: string, value: unknown): ValidationResult {
    const rules = this.schema[fieldName];
    if (!rules) {
      return { valid: true, errors: [] };
    }

    return this.validator.validate(value, rules);
  }

  getFieldRules(fieldName: string): ValidationRule | undefined {
    return this.schema[fieldName];
  }

  getRequiredFields(): string[] {
    return Object.entries(this.schema)
      .filter(([, rules]) => rules.required)
      .map(([name]) => name);
  }
}

// ============================================================================
// Environment Setup
// ============================================================================

export const envValidator = new EnvironmentValidator();

// Add common environment variables
envValidator.add({
  key: 'NEXT_PUBLIC_BACKEND_URL',
  required: true,
  validator: (value) => value.startsWith('http'),
  description: 'Backend API URL'
});

envValidator.add({
  key: 'NEXTAUTH_SECRET',
  required: true,
  validator: (value) => value.length >= 32,
  description: 'NextAuth secret key (minimum 32 characters)'
});

envValidator.add({
  key: 'NODE_ENV',
  required: true,
  defaultValue: 'development',
  validator: (value) => ['development', 'production', 'test'].includes(value),
  description: 'Node environment'
});

// ============================================================================
// Exports
// ============================================================================

export const inputValidator = new InputValidator();