/**
 * Runtime Configuration Validator
 * Centralizes hardcoded values and provides runtime validation
 * Ensures configuration consistency across development and production
 */

import { z } from 'zod';
import { logger, devLog, safeError } from '@/lib/utils/logging';
import { getBackendUrl } from '../../../../shared/utils/url-resolver';

// Configuration schemas for validation
const apiEndpointsSchema = z.object({
  backend: z.string().url('Backend URL must be a valid URL'),
  auth: z.string().url('Auth URL must be a valid URL'),
  analytics: z.string().url('Analytics URL must be a valid URL'),
  notifications: z.string().url('Notifications URL must be a valid URL'),
  payments: z.string().url('Payments URL must be a valid URL').optional(),
});

const featureFlagsSchema = z.object({
  enabledFeatures: z.array(z.string()),
  experimentalFeatures: z.array(z.string()).default([]),
  rolloutPercentages: z.record(z.string(), z.number().min(0).max(100)),
});

const performanceConfigSchema = z.object({
  cacheTimeout: z.number().min(1000).max(3600000), // 1s to 1h
  requestTimeout: z.number().min(1000).max(60000), // 1s to 1m
  retryAttempts: z.number().min(1).max(10),
  throttleDelay: z.number().min(0).max(5000),
  batchSize: z.number().min(1).max(1000),
});

const securityConfigSchema = z.object({
  allowedOrigins: z.array(z.string()),
  csrfProtection: z.boolean(),
  sessionTimeout: z.number().min(300000).max(86400000), // 5m to 24h
  tokenRefreshThreshold: z.number().min(60000).max(3600000), // 1m to 1h
});

// Centralized configuration constants
export const CONFIG_CONSTANTS = {
  // API Configuration
  API: {
    ENDPOINTS: {
      BACKEND: getBackendUrl('client'),
      AUTH: '/api/auth',
      ANALYTICS: '/api/analytics',
      NOTIFICATIONS: '/api/notifications',
      PAYMENTS: '/api/payments',
    },
    TIMEOUTS: {
      DEFAULT: 30000,
      AUTH: 15000,
      UPLOAD: 60000,
      ANALYTICS: 45000,
    },
    RETRY: {
      ATTEMPTS: 3,
      DELAY: 1000,
      BACKOFF_MULTIPLIER: 2,
    }
  },

  // UI Configuration
  UI: {
    PAGINATION: {
      DEFAULT_PAGE_SIZE: 20,
      MAX_PAGE_SIZE: 100,
      PAGE_SIZE_OPTIONS: [10, 20, 50, 100],
    },
    THEME: {
      DEFAULT: 'system' as const,
      OPTIONS: ['light', 'dark', 'system'] as const,
    },
    DEBOUNCE: {
      SEARCH: 300,
      FORM_VALIDATION: 500,
      SCROLL: 100,
      RESIZE: 250,
    },
    ANIMATION: {
      DURATION: {
        FAST: 150,
        NORMAL: 300,
        SLOW: 500,
      },
      EASING: 'ease-in-out',
    }
  },

  // Performance Configuration
  PERFORMANCE: {
    CACHE: {
      ANALYTICS_TTL: 300000, // 5 minutes
      USER_DATA_TTL: 600000,  // 10 minutes
      STATIC_DATA_TTL: 3600000, // 1 hour
    },
    LAZY_LOADING: {
      ROOT_MARGIN: '50px',
      THRESHOLD: 0.1,
    },
    VIRTUAL_SCROLLING: {
      ITEM_HEIGHT: 60,
      BUFFER_SIZE: 5,
      OVERSCAN: 3,
    }
  },

  // Security Configuration
  SECURITY: {
    SESSION: {
      TIMEOUT: 3600000, // 1 hour
      REFRESH_THRESHOLD: 300000, // 5 minutes
      MAX_IDLE_TIME: 1800000, // 30 minutes
    },
    VALIDATION: {
      MIN_PASSWORD_LENGTH: 8,
      MAX_LOGIN_ATTEMPTS: 5,
      LOCKOUT_DURATION: 900000, // 15 minutes
    },
    PERMISSIONS: {
      CACHE_DURATION: 300000, // 5 minutes
      REFRESH_INTERVAL: 600000, // 10 minutes
    }
  },

  // Feature Flags
  FEATURES: {
    ANALYTICS_EXPORT: true,
    REAL_TIME_UPDATES: true,
    ADVANCED_FILTERS: true,
    MOBILE_OPTIMIZATIONS: true,
    PERFORMANCE_MONITORING: process.env.NODE_ENV === 'development',
  },

  // Business Logic Constants
  BUSINESS: {
    STOCK_RANKING: {
      MIN_MARKET_CAP: 1000000, // $1M
      MAX_RESULTS: 1000,
      UPDATE_FREQUENCY: 900000, // 15 minutes
    },
    ANALYTICS: {
      SUPPORTED_COUNTRIES: ['US', 'CA', 'UK', 'DE', 'FR', 'JP', 'AU'],
      SUPPORTED_SECTORS: [
        'Technology',
        'Healthcare',
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ],
      GROWTH_THRESHOLDS: {
        HIGH: 20,
        MEDIUM: 10,
        LOW: 5,
      }
    }
  }
} as const;

// Runtime validator class
class RuntimeConfigValidator {
  private validationErrors: string[] = [];
  private warnings: string[] = [];

  constructor() {
    this.validateConfiguration();
  }

  private validateConfiguration() {
    devLog('Starting runtime configuration validation');

    // Validate API endpoints
    this.validateApiEndpoints();
    
    // Validate feature flags
    this.validateFeatureFlags();
    
    // Validate performance settings
    this.validatePerformanceConfig();
    
    // Validate security settings
    this.validateSecurityConfig();

    // Validate environment-specific requirements
    this.validateEnvironmentRequirements();

    // Report results
    this.reportValidationResults();
  }

  private validateApiEndpoints() {
    try {
      const endpoints = {
        backend: CONFIG_CONSTANTS.API.ENDPOINTS.BACKEND,
        auth: CONFIG_CONSTANTS.API.ENDPOINTS.AUTH,
        analytics: CONFIG_CONSTANTS.API.ENDPOINTS.ANALYTICS,
        notifications: CONFIG_CONSTANTS.API.ENDPOINTS.NOTIFICATIONS,
        payments: CONFIG_CONSTANTS.API.ENDPOINTS.PAYMENTS,
      };

      apiEndpointsSchema.parse(endpoints);
      devLog('API endpoints validation passed');
    } catch (error) {
      if (error instanceof z.ZodError) {
        this.validationErrors.push(`API endpoints validation failed: ${error.errors.map(e => e.message).join(', ')}`);
      }
    }
  }

  private validateFeatureFlags() {
    try {
      const featureFlags = {
        enabledFeatures: Object.entries(CONFIG_CONSTANTS.FEATURES)
          .filter(([_, enabled]) => enabled)
          .map(([feature]) => feature),
        experimentalFeatures: [],
        rolloutPercentages: {
          analytics_export: 100,
          real_time_updates: 100,
          mobile_optimizations: 100,
        }
      };

      featureFlagsSchema.parse(featureFlags);
      devLog('Feature flags validation passed');
    } catch (error) {
      if (error instanceof z.ZodError) {
        this.validationErrors.push(`Feature flags validation failed: ${error.errors.map(e => e.message).join(', ')}`);
      }
    }
  }

  private validatePerformanceConfig() {
    try {
      const performanceConfig = {
        cacheTimeout: CONFIG_CONSTANTS.PERFORMANCE.CACHE.ANALYTICS_TTL,
        requestTimeout: CONFIG_CONSTANTS.API.TIMEOUTS.DEFAULT,
        retryAttempts: CONFIG_CONSTANTS.API.RETRY.ATTEMPTS,
        throttleDelay: CONFIG_CONSTANTS.UI.DEBOUNCE.SEARCH,
        batchSize: CONFIG_CONSTANTS.UI.PAGINATION.DEFAULT_PAGE_SIZE,
      };

      performanceConfigSchema.parse(performanceConfig);
      devLog('Performance configuration validation passed');
    } catch (error) {
      if (error instanceof z.ZodError) {
        this.validationErrors.push(`Performance config validation failed: ${error.errors.map(e => e.message).join(', ')}`);
      }
    }
  }

  private validateSecurityConfig() {
    try {
      const securityConfig = {
        allowedOrigins: ['https://epsx.io', 'https://admin.epsx.io', 'http://localhost:3000'],
        csrfProtection: true,
        sessionTimeout: CONFIG_CONSTANTS.SECURITY.SESSION.TIMEOUT,
        tokenRefreshThreshold: CONFIG_CONSTANTS.SECURITY.SESSION.REFRESH_THRESHOLD,
      };

      securityConfigSchema.parse(securityConfig);
      devLog('Security configuration validation passed');
    } catch (error) {
      if (error instanceof z.ZodError) {
        this.validationErrors.push(`Security config validation failed: ${error.errors.map(e => e.message).join(', ')}`);
      }
    }
  }

  private validateEnvironmentRequirements() {
    const env = process.env.NODE_ENV;
    
    if (env === 'production') {
      // Production-specific validations
      if (!process.env.NEXT_PUBLIC_BACKEND_URL) {
        this.validationErrors.push('NEXT_PUBLIC_BACKEND_URL is required in production');
      }
      
      if (!process.env.NEXTAUTH_SECRET) {
        this.validationErrors.push('NEXTAUTH_SECRET is required in production');
      }

      // Check for development artifacts
      if (CONFIG_CONSTANTS.FEATURES.PERFORMANCE_MONITORING) {
        this.warnings.push('Performance monitoring is enabled in production');
      }
    } else if (env === 'development') {
      // Development-specific validations
      if (!CONFIG_CONSTANTS.FEATURES.PERFORMANCE_MONITORING) {
        this.warnings.push('Performance monitoring is disabled in development');
      }
    }

    // Validate required browser features
    if (typeof window !== 'undefined') {
      this.validateBrowserSupport();
    }
  }

  private validateBrowserSupport() {
    const requiredFeatures = [
      'localStorage',
      'sessionStorage',
      'fetch',
      'Promise',
      'IntersectionObserver',
    ];

    const missingFeatures = requiredFeatures.filter(feature => {
      switch (feature) {
        case 'localStorage':
        case 'sessionStorage':
          return !window[feature];
        case 'fetch':
          return typeof fetch === 'undefined';
        case 'Promise':
          return typeof Promise === 'undefined';
        case 'IntersectionObserver':
          return typeof IntersectionObserver === 'undefined';
        default:
          return false;
      }
    });

    if (missingFeatures.length > 0) {
      this.warnings.push(`Browser missing required features: ${missingFeatures.join(', ')}`);
    }
  }

  private reportValidationResults() {
    if (this.validationErrors.length > 0) {
      logger.error('Configuration validation failed', { errors: this.validationErrors });
      
      if (process.env.NODE_ENV === 'production') {
        // In production, throw error to prevent startup with invalid config
        throw new Error(`Configuration validation failed: ${this.validationErrors.join('; ')}`);
      }
    }

    if (this.warnings.length > 0) {
      logger.warn('Configuration warnings', { warnings: this.warnings });
    }

    if (this.validationErrors.length === 0) {
      devLog('All configuration validations passed');
    }
  }

  public getValidationErrors(): string[] {
    return this.validationErrors;
  }

  public getWarnings(): string[] {
    return this.warnings;
  }

  public isValid(): boolean {
    return this.validationErrors.length === 0;
  }
}

// Singleton validator instance
export const configValidator = new RuntimeConfigValidator();

// Configuration getter with validation
export function getConfig<T>(path: string): T {
  const pathParts = path.split('.');
  let current: unknown = CONFIG_CONSTANTS;
  
  for (const part of pathParts) {
    if (current && typeof current === 'object' && part in current) {
      current = current[part];
    } else {
      throw new Error(`Configuration path '${path}' not found`);
    }
  }
  
  return current as T;
}

// Type-safe configuration getters
export const getApiConfig = () => CONFIG_CONSTANTS.API;
export const getUIConfig = () => CONFIG_CONSTANTS.UI;
export const getPerformanceConfig = () => CONFIG_CONSTANTS.PERFORMANCE;
export const getSecurityConfig = () => CONFIG_CONSTANTS.SECURITY;
export const getFeatureConfig = () => CONFIG_CONSTANTS.FEATURES;
export const getBusinessConfig = () => CONFIG_CONSTANTS.BUSINESS;

// Environment-specific configuration
export function getEnvironmentConfig() {
  const env = process.env.NODE_ENV || 'development';
  
  return {
    isDevelopment: env === 'development',
    isProduction: env === 'production',
    isTest: env === 'test',
    enableDebugMode: env === 'development',
    enablePerformanceMonitoring: CONFIG_CONSTANTS.FEATURES.PERFORMANCE_MONITORING,
    logLevel: env === 'development' ? 'debug' : 'warn',
  };
}

// Feature flag checker
export function isFeatureEnabled(feature: keyof typeof CONFIG_CONSTANTS.FEATURES): boolean {
  return CONFIG_CONSTANTS.FEATURES[feature];
}

// Configuration update utility (for dynamic configuration)
export function updateConfig(path: string, value: unknown) {
  const pathParts = path.split('.');
  let current: unknown = CONFIG_CONSTANTS;
  
  for (let i = 0; i < pathParts.length - 1; i++) {
    const part = pathParts[i];
    if (current && typeof current === 'object' && part in current) {
      current = current[part];
    } else {
      throw new Error(`Configuration path '${path}' not found`);
    }
  }
  
  const lastPart = pathParts[pathParts.length - 1];
  if (current && typeof current === 'object') {
    current[lastPart] = value;
    devLog(`Configuration updated: ${path} = ${JSON.stringify(value)}`);
  } else {
    throw new Error(`Cannot update configuration at '${path}'`);
  }
}