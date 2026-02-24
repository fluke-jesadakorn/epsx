/**
 * Runtime Environment Variable Validator
 * Validates NEXT_PUBLIC_* variables are available at runtime
 * Works with Cloud Run environment variables (no build-time coupling)
 * Modernized with centralized URL resolver for fallbacks
 */

import { logger } from './logger';
import { URLContext, getAdminUrl, getBackendUrl, getFrontendUrl } from './url-resolver';

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
  // Firebase legacy configuration
  NEXT_PUBLIC_FIREBASE_API_KEY?: string;
  NEXT_PUBLIC_FIREBASE_PROJECT_ID?: string;
  NEXT_PUBLIC_FIREBASE_APP_ID?: string;
}

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
    (process.env['NODE_ENV'] === 'production' && typeof window === 'undefined' && process.env['PORT'] === undefined);
}

/**
 * Validate required NEXT_PUBLIC_* environment variables
 */
export function validateRuntimeEnvironment(isDevelopment = false): ValidationResult {
  const errors: string[] = [];
  const warnings: string[] = [];

  // Skip strict validation during build phase
  const skipStrictValidation = isDevelopment || isBuildPhase();

  // Required variables
  const requiredVars: (keyof RequiredEnvVars)[] = [
    'NEXT_PUBLIC_BACKEND_URL',
    'NEXT_PUBLIC_APP_URL',
    'NEXT_PUBLIC_ADMIN_URL'
  ];

  // Check required variables
  for (const varName of requiredVars) {
    validateEnvVar({
      varName,
      value: process.env[varName],
      skipStrictValidation,
      errors,
      warnings
    });
  }

  // Special handling for NEXT_PUBLIC_OAUTH_CLIENT_ID
  // Use dynamic key lookup to bypass webpack build-time inlining — reads from actual runtime env
  const oauthKey: keyof NodeJS.ProcessEnv = 'NEXT_PUBLIC_OAUTH_CLIENT_ID';
  validateOauthClientId(process.env[oauthKey], skipStrictValidation, errors);

  // Optional Web3 variables - validate blockchain network
  // Use dynamic key lookup to bypass webpack build-time inlining
  const blockchainKey: keyof NodeJS.ProcessEnv = 'NEXT_PUBLIC_BLOCKCHAIN_NETWORK';
  validateBlockchainNetwork(process.env[blockchainKey], errors);

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  };
}

function validateEnvVar(params: {
  varName: string,
  value: string | undefined,
  skipStrictValidation: boolean,
  errors: string[],
  warnings: string[]
}): void {
  const { varName, value, skipStrictValidation, errors, warnings } = params;

  if (value === undefined || value === '') {
    if (skipStrictValidation) {
      warnings.push(`${varName} is not set (using development default)`);
    } else {
      errors.push(`${varName} is required in production environment`);
    }
    return;
  }

  // Validate URL format for URL variables
  if (varName.includes('URL')) {
    try {
      new URL(value);
    } catch {
      errors.push(`${varName} must be a valid URL (current: ${value})`);
    }

    // Validate HTTPS in production (skip during build phase)
    if (!skipStrictValidation && !value.startsWith('https://')) {
      errors.push(`${varName} must use HTTPS in production (current: ${value})`);
    }
  }
}

function validateOauthClientId(
  value: string | undefined,
  skipStrictValidation: boolean,
  errors: string[]
): void {
  // We silence the warning in development because we have a safe code-level default
  if ((value === undefined || value === '') && !skipStrictValidation) {
    errors.push('NEXT_PUBLIC_OAUTH_CLIENT_ID is required in production environment');
  }
}

function validateBlockchainNetwork(
  value: string | undefined,
  errors: string[]
): void {
  if (value !== undefined && value !== '' && !['mainnet', 'testnet'].includes(value)) {
    errors.push(`NEXT_PUBLIC_BLOCKCHAIN_NETWORK must be 'mainnet' or 'testnet' (current: ${value})`);
  }
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
    logger.warn('⚠️ Environment warnings:', validation.warnings.join('\n'));
  }

  // Use variable keys to bypass webpack build-time inlining for NEXT_PUBLIC_* vars
  // that must be read from actual runtime environment
  const env = process.env as Record<string, string | undefined>;
  const oauthKey = 'NEXT_PUBLIC_OAUTH_CLIENT_ID';
  const blockchainKey = 'NEXT_PUBLIC_BLOCKCHAIN_NETWORK';

  return {
    // Required variables with centralized URL resolver fallbacks
    NEXT_PUBLIC_BACKEND_URL: process.env.NEXT_PUBLIC_BACKEND_URL ?? getBackendUrl(URLContext.CLIENT),
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL ?? getFrontendUrl(URLContext.CLIENT),
    NEXT_PUBLIC_ADMIN_URL: process.env.NEXT_PUBLIC_ADMIN_URL ?? getAdminUrl(URLContext.CLIENT),
    NEXT_PUBLIC_OAUTH_CLIENT_ID: env[oauthKey] ?? (isDevelopment ? 'epsx-frontend' : ''),

    // Optional Web3 variables
    NEXT_PUBLIC_BLOCKCHAIN_NETWORK: env[blockchainKey] ?? 'testnet',
    NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID ?? 'epsx-web3-frontend',

    // Optional Firebase variables
    NEXT_PUBLIC_FIREBASE_API_KEY: process.env['NEXT_PUBLIC_FIREBASE_API_KEY'],
    NEXT_PUBLIC_FIREBASE_PROJECT_ID: process.env['NEXT_PUBLIC_FIREBASE_PROJECT_ID'],
    NEXT_PUBLIC_FIREBASE_APP_ID: process.env['NEXT_PUBLIC_FIREBASE_APP_ID']
  };
}

/**
 * Initialize and validate environment on app startup
 */
export function initializeRuntimeEnvironment(): void {
  const isDevelopment = process.env.NODE_ENV === 'development';

  if (isDevelopment) {
    logger.info('🔍 Validating runtime environment variables...');
  }

  try {
    const env = getRuntimeEnvironment(isDevelopment);
    if (isDevelopment) {
      logger.info('✅ Runtime environment validation passed');
      logger.info(`🌐 Backend URL: ${env.NEXT_PUBLIC_BACKEND_URL}`);
      logger.info(`🎯 App URL: ${env.NEXT_PUBLIC_APP_URL}`);
      logger.info(`👥 Admin URL: ${env.NEXT_PUBLIC_ADMIN_URL}`);
      logger.info(`🔑 OAuth Client ID: ${env.NEXT_PUBLIC_OAUTH_CLIENT_ID}`);
      logger.info(`⛓️ Blockchain Network: ${env.NEXT_PUBLIC_BLOCKCHAIN_NETWORK}`);
      logger.info(`🔗 WalletConnect Project ID: ${env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID}`);
    }
  } catch (error) {
    logger.error('❌ Runtime environment validation failed:', error);
    if (!isDevelopment) {
      throw error; // Fail fast in production
    }
  }
}