/**
 * Environment Variable Validation for Frontend
 * Validates critical OIDC environment variables at runtime
 */

interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Validate OIDC environment variables
 */
export function validateOIDCEnvironment(): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Required variables
  const required = {
    JWT_SECRET: process.env.JWT_SECRET,
    APP_URL: process.env.APP_URL,
    NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL,
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL,
  };

  // Check required variables
  Object.entries(required).forEach(([key, value]) => {
    if (!value) {
      errors.push(`Missing required environment variable: ${key}`);
    }
  });

  // Optional but recommended variables
  const recommended = {
    OIDC_CLIENT_ID: process.env.OIDC_CLIENT_ID,
    JWT_SECRET: process.env.JWT_SECRET,
    NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID,
  };

  Object.entries(recommended).forEach(([key, value]) => {
    if (!value) {
      warnings.push(`Missing recommended environment variable: ${key} (will use defaults)`);
    }
  });

  // Validate URL formats
  if (required.APP_URL && !isValidUrl(required.APP_URL)) {
    errors.push('APP_URL must be a valid URL');
  }

  if (required.NEXT_PUBLIC_BACKEND_URL && !isValidUrl(required.NEXT_PUBLIC_BACKEND_URL)) {
    errors.push('NEXT_PUBLIC_BACKEND_URL must be a valid URL');
  }

  if (required.NEXT_PUBLIC_APP_URL && !isValidUrl(required.NEXT_PUBLIC_APP_URL)) {
    errors.push('NEXT_PUBLIC_APP_URL must be a valid URL');
  }

  // Check JWT_SECRET length if provided
  if (recommended.JWT_SECRET && recommended.JWT_SECRET.length < 32) {
    warnings.push('JWT_SECRET should be at least 32 characters for security');
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  };
}

/**
 * Validate admin environment variables
 */
export function validateAdminEnvironment(): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Required variables for admin
  const required = {
    JWT_SECRET: process.env.JWT_SECRET,
    APP_URL: process.env.APP_URL,
    NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL,
    NEXT_PUBLIC_ADMIN_URL: process.env.NEXT_PUBLIC_ADMIN_URL,
  };

  // Check required variables
  Object.entries(required).forEach(([key, value]) => {
    if (!value) {
      errors.push(`Missing required admin environment variable: ${key}`);
    }
  });

  // Admin-specific recommended variables
  const recommended = {
    OIDC_CLIENT_ID: process.env.OIDC_CLIENT_ID,
    NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID,
  };

  Object.entries(recommended).forEach(([key, value]) => {
    if (!value) {
      warnings.push(`Missing recommended admin environment variable: ${key}`);
    }
  });

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  };
}

/**
 * Helper function to validate URL format
 */
function isValidUrl(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

/**
 * Log validation results
 */
export function logValidationResults(results: ValidationResult, context: string = 'Environment'): void {
  if (results.isValid) {
    console.log(`✅ ${context} validation passed`);
  } else {
    console.error(`❌ ${context} validation failed:`);
    results.errors.forEach(error => console.error(`  - ${error}`));
  }

  if (results.warnings.length > 0) {
    console.warn(`⚠️ ${context} warnings:`);
    results.warnings.forEach(warning => console.warn(`  - ${warning}`));
  }
}

/**
 * Development-only environment validation
 */
export function validateDevelopmentEnvironment(): void {
  if (process.env.NODE_ENV !== 'development') return;

  console.log('🔍 Validating development environment configuration...');

  const frontendResults = validateOIDCEnvironment();
  logValidationResults(frontendResults, 'Frontend OIDC Environment');

  // Additional development checks
  if (!process.env.NEXT_PUBLIC_BACKEND_URL?.includes('localhost')) {
    console.warn('⚠️ Development: NEXT_PUBLIC_BACKEND_URL should point to localhost for local development');
  }

  if (!process.env.APP_URL?.includes('localhost')) {
    console.warn('⚠️ Development: APP_URL should point to localhost for local development');
  }
}