/**
 * Runtime Environment Variable Validator
 * Validates NEXT_PUBLIC_* variables are available at runtime
 * Works with Cloud Run environment variables (no build-time coupling)
 * Modernized with centralized URL resolver for fallbacks
 */

import { URLContext, getAdminUrl, getBackendUrl, getFrontendUrl } from '@/shared/utils/url-resolver';

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
  // Web3 configuration
  NEXT_PUBLIC_BLOCKCHAIN_NETWORK?: string;
  NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID?: string;
}

/**
 * Validate required NEXT_PUBLIC_* environment variables
 */
/**
 * Detect if we're in the Next.js build phase
 * During `next build`, we should skip strict validation since env vars
 * will be provided at actual runtime (e.g., Cloud Run, Docker)
 */
function isBuildPhase(): boolean {
  // NEXT_PHASE is set during build
  return process.env['NEXT_PHASE'] === 'phase-production-build' ||
    // Fallback: check if we're in a CI/build environment
    process.env['CI'] === 'true' ||
    // Another indicator: building but no server running
    (process.env['NODE_ENV'] === 'production' && typeof window === 'undefined' && !process.env['PORT']);
}

export function validateRuntimeEnvironment(isDevelopment = false): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Skip strict validation during build phase
  const skipStrictValidation = isDevelopment || isBuildPhase();

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
      if (skipStrictValidation) {
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

      // Validate HTTPS in production (skip during build phase)
      if (!skipStrictValidation && varName.includes('URL') && !value.startsWith('https://')) {
        errors.push(`${varName} must use HTTPS in production (current: ${value})`);
      }
    }
  }

  // Optional Web3 variables - validate blockchain network
  const blockchainNetwork = process.env['NEXT_PUBLIC_BLOCKCHAIN_NETWORK'];
  if (blockchainNetwork && !['mainnet', 'testnet'].includes(blockchainNetwork)) {
    errors.push(`NEXT_PUBLIC_BLOCKCHAIN_NETWORK must be 'mainnet' or 'testnet' (current: ${blockchainNetwork})`);
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
    // Required variables with centralized URL resolver fallbacks
    NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL || (isDevelopment ? getBackendUrl(URLContext.CLIENT) : ''),
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL || (isDevelopment ? getFrontendUrl(URLContext.CLIENT) : ''),
    NEXT_PUBLIC_ADMIN_URL: process.env.NEXT_PUBLIC_ADMIN_URL || (isDevelopment ? getAdminUrl(URLContext.CLIENT) : ''),
    NEXT_PUBLIC_OAUTH_CLIENT_ID: process.env.NEXT_PUBLIC_OAUTH_CLIENT_ID || (isDevelopment ? 'epsx-frontend' : ''),

    // Optional Web3 variables
    NEXT_PUBLIC_BLOCKCHAIN_NETWORK: process.env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK || 'testnet',
    NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'epsx-web3-frontend'
  };
}

/**
 * Initialize and validate environment on app startup
 */
export function initializeRuntimeEnvironment(): void {
  const isDevelopment = process.env.NODE_ENV === 'development';

  if (isDevelopment) {
    console.log('🔍 Validating runtime environment variables...');
  }

  try {
    const env = getRuntimeEnvironment(isDevelopment);
    if (isDevelopment) {
      console.log('✅ Runtime environment validation passed');
      console.log(`🌐 Backend URL: ${env.NEXT_PUBLIC_BACKEND_URL}`);
      console.log(`🎯 App URL: ${env.NEXT_PUBLIC_APP_URL}`);
      console.log(`👥 Admin URL: ${env.NEXT_PUBLIC_ADMIN_URL}`);
      console.log(`🔑 OAuth Client ID: ${env.NEXT_PUBLIC_OAUTH_CLIENT_ID}`);
      console.log(`⛓️ Blockchain Network: ${env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK}`);
      console.log(`🔗 WalletConnect Project ID: ${env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID}`);
    }
  } catch (error) {
    console.error('❌ Runtime environment validation failed:', error);
    if (!isDevelopment) {
      throw error; // Fail fast in production
    }
  }
}