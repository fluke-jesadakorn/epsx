/**
 * Runtime Environment Variable Validator
 * Validates NEXT_PUBLIC_* variables are available at runtime
 * Works with Cloud Run environment variables (no build-time coupling)
 */

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export interface RequiredEnvVars {
  NEXT_PUBLIC_BACKEND_URL: string;
  NEXT_PUBLIC_APP_URL: string;
  NEXT_PUBLIC_ADMIN_URL: string;
  NEXT_PUBLIC_OAUTH_CLIENT_ID: string;
}

export interface OptionalEnvVars {
  NEXT_PUBLIC_FIREBASE_API_KEY?: string;
  NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN?: string;
  NEXT_PUBLIC_FIREBASE_PROJECT_ID?: string;
  NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET?: string;
  NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID?: string;
  NEXT_PUBLIC_FIREBASE_APP_ID?: string;
  NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID?: string;
}

/**
 * Validate required NEXT_PUBLIC_* environment variables
 */
export function validateRuntimeEnvironment(isDevelopment = false): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Required variables
  const requiredVars: (keyof RequiredEnvVars)[] = [
    'NEXT_PUBLIC_BACKEND_URL',
    'NEXT_PUBLIC_APP_URL', 
    'NEXT_PUBLIC_ADMIN_URL',
    'NEXT_PUBLIC_OAUTH_CLIENT_ID'
  ];

  // Check required variables
  for (const varName of requiredVars) {
    const value = process.env[varName];
    
    if (!value) {
      if (isDevelopment) {
        warnings.push(`${varName} is not set (using development default)`);
      } else {
        errors.push(`${varName} is required in production environment`);
      }
    } else {
      // Validate URL format for URL variables
      if (varName.includes('URL')) {
        try {
          new URL(value);
        } catch {
          errors.push(`${varName} must be a valid URL (current: ${value})`);
        }
      }
      
      // Validate HTTPS in production
      if (!isDevelopment && varName.includes('URL') && !value.startsWith('https://')) {
        errors.push(`${varName} must use HTTPS in production (current: ${value})`);
      }
    }
  }

  // Optional Firebase variables - warn if partially configured
  const firebaseVars: (keyof OptionalEnvVars)[] = [
    'NEXT_PUBLIC_FIREBASE_API_KEY',
    'NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN',
    'NEXT_PUBLIC_FIREBASE_PROJECT_ID',
    'NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET',
    'NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID',
    'NEXT_PUBLIC_FIREBASE_APP_ID',
    'NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID'
  ];

  const configuredFirebaseVars = firebaseVars.filter(varName => !!process.env[varName]);
  
  if (configuredFirebaseVars.length > 0 && configuredFirebaseVars.length < 3) {
    warnings.push(`Firebase partially configured (${configuredFirebaseVars.length}/7 variables). Configure all Firebase variables or none for optimal performance.`);
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  };
}

/**
 * Get runtime environment variables with safe fallbacks
 */
export function getRuntimeEnvironment(isDevelopment = false): RequiredEnvVars & OptionalEnvVars {
  const validation = validateRuntimeEnvironment(isDevelopment);
  
  if (!validation.isValid) {
    throw new Error(`Environment validation failed:\n${validation.errors.join('\n')}`);
  }

  // Log warnings
  if (validation.warnings.length > 0) {
    console.warn('⚠️ Environment warnings:', validation.warnings.join('\n'));
  }

  return {
    // Required variables with development fallbacks
    NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL || (isDevelopment ? 'http://localhost:8080' : ''),
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL || (isDevelopment ? 'http://localhost:3000' : ''),
    NEXT_PUBLIC_ADMIN_URL: process.env.NEXT_PUBLIC_ADMIN_URL || (isDevelopment ? 'http://localhost:3001' : ''),
    NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || (isDevelopment ? 'epsx-frontend' : ''),
    
    // Optional Firebase variables
    NEXT_PUBLIC_FIREBASE_API_KEY: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
    NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
    NEXT_PUBLIC_FIREBASE_PROJECT_ID: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
    NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
    NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
    NEXT_PUBLIC_FIREBASE_APP_ID: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
    NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
  };
}

/**
 * Initialize and validate environment on app startup
 */
export function initializeRuntimeEnvironment(): void {
  const isDevelopment = process.env.NODE_ENV === 'development';
  
  console.log('🔍 Validating runtime environment variables...');
  
  try {
    const env = getRuntimeEnvironment(isDevelopment);
    console.log('✅ Runtime environment validation passed');
    console.log(`🌐 Backend URL: ${env.NEXT_PUBLIC_BACKEND_URL}`);
    console.log(`🎯 App URL: ${env.NEXT_PUBLIC_APP_URL}`);
    console.log(`👥 Admin URL: ${env.NEXT_PUBLIC_ADMIN_URL}`);
    console.log(`🔑 OAuth Client ID: ${env.NEXT_PUBLIC_OAUTH_CLIENT_ID}`);
    
    if (env.NEXT_PUBLIC_FIREBASE_API_KEY) {
      console.log('🔥 Firebase configuration detected');
    }
  } catch (error) {
    console.error('❌ Runtime environment validation failed:', error);
    if (!isDevelopment) {
      throw error; // Fail fast in production
    }
  }
}