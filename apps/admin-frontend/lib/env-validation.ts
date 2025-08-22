/**
 * Environment Variable Validation for Admin Frontend
 * Validates critical OIDC environment variables at runtime
 */

interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Validate admin OIDC environment variables
 */
export function validateAdminOIDCEnvironment(): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Required variables for admin
  const required = {
    JWT_SECRET: process.env.JWT_SECRET,
    ADMIN_URL: process.env.ADMIN_URL,
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
    OIDC_CLIENT_SECRET: process.env.OIDC_CLIENT_SECRET,
    NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID,
  };

  Object.entries(recommended).forEach(([key, value]) => {
    if (!value) {
      warnings.push(`Missing recommended admin environment variable: ${key} (will use defaults)`);
    }
  });

  // Validate URL formats
  if (required.ADMIN_URL && !isValidUrl(required.ADMIN_URL)) {
    errors.push('ADMIN_URL must be a valid URL');
  }

  if (required.NEXT_PUBLIC_BACKEND_URL && !isValidUrl(required.NEXT_PUBLIC_BACKEND_URL)) {
    errors.push('NEXT_PUBLIC_BACKEND_URL must be a valid URL');
  }

  if (required.NEXT_PUBLIC_ADMIN_URL && !isValidUrl(required.NEXT_PUBLIC_ADMIN_URL)) {
    errors.push('NEXT_PUBLIC_ADMIN_URL must be a valid URL');
  }

  // Check JWT_SECRET length
  if (required.JWT_SECRET && required.JWT_SECRET.length < 32) {
    warnings.push('JWT_SECRET should be at least 32 characters for security');
  }

  // Admin-specific validations
  if (required.NEXT_PUBLIC_ADMIN_URL && required.NEXT_PUBLIC_BACKEND_URL) {
    const adminUrl = new URL(required.NEXT_PUBLIC_ADMIN_URL);
    const backendUrl = new URL(required.NEXT_PUBLIC_BACKEND_URL);
    
    if (adminUrl.hostname === backendUrl.hostname && adminUrl.port === backendUrl.port) {
      warnings.push('Admin and backend URLs should use different ports to avoid conflicts');
    }
  }

  // Check for admin-specific client configuration
  if (!recommended.NEXT_PUBLIC_OAUTH_CLIENT_ID || !recommended.NEXT_PUBLIC_OAUTH_CLIENT_ID.includes('admin')) {
    warnings.push('NEXT_PUBLIC_OAUTH_CLIENT_ID should include "admin" for admin frontend (e.g., "epsx-admin")');
  }

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
 * Development-only admin environment validation
 */
export function validateAdminDevelopmentEnvironment(): void {
  if (process.env.NODE_ENV !== 'development') return;

  console.log('🔍 Validating admin development environment configuration...');

  const adminResults = validateAdminOIDCEnvironment();
  logValidationResults(adminResults, 'Admin OIDC Environment');

  // Additional development checks
  if (!process.env.NEXT_PUBLIC_BACKEND_URL?.includes('localhost')) {
    console.warn('⚠️ Admin Development: NEXT_PUBLIC_BACKEND_URL should point to localhost for local development');
  }

  if (!process.env.ADMIN_URL?.includes('localhost')) {
    console.warn('⚠️ Admin Development: ADMIN_URL should point to localhost for local development');
  }

  // Check that admin port is different from frontend port
  const adminUrl = process.env.NEXT_PUBLIC_ADMIN_URL || process.env.ADMIN_URL;
  if (adminUrl && !adminUrl.includes(':3001')) {
    console.warn('⚠️ Admin Development: Admin should typically run on port 3001');
  }

  // Validate admin module configuration
  if (!process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID?.includes('admin')) {
    console.warn('⚠️ Admin Development: OAuth client ID should be admin-specific (e.g., "epsx-admin")');
  }
}

/**
 * Production-ready admin environment validation
 */
export function validateAdminProductionEnvironment(): ValidationResult {
  const results = validateAdminOIDCEnvironment();
  
  // Additional production checks
  if (process.env.NODE_ENV === 'production') {
    // Ensure HTTPS in production
    if (process.env.NEXT_PUBLIC_ADMIN_URL && !process.env.NEXT_PUBLIC_ADMIN_URL.startsWith('https://')) {
      results.errors.push('NEXT_PUBLIC_ADMIN_URL must use HTTPS in production');
    }
    
    if (process.env.ADMIN_URL && !process.env.ADMIN_URL.startsWith('https://')) {
      results.errors.push('ADMIN_URL must use HTTPS in production');
    }

    // Ensure secure cookie settings
    if (!process.env.JWT_SECRET || process.env.JWT_SECRET.length < 64) {
      results.errors.push('JWT_SECRET must be at least 64 characters in production');
    }
  }
  
  results.isValid = results.errors.length === 0;
  return results;
}