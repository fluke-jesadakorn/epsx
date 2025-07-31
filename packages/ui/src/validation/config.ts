/**
 * Validation configuration that consolidates validation settings
 * Provides centralized configuration for all validation behavior
 */

// Global validation configuration
export interface ValidationGlobalConfig {
  /** Default validation behavior */
  defaults: {
    /** Abort validation on first error */
    abortEarly: boolean;
    /** Transform data during validation */
    transform: boolean;
    /** Strip unknown properties */
    stripUnknown: boolean;
    /** Enable async validation */
    enableAsync: boolean;
    /** Default debounce time for async validation (ms) */
    debounceMs: number;
  };

  /** Password requirements */
  password: {
    minLength: number;
    maxLength: number;
    requireUppercase: boolean;
    requireLowercase: boolean;
    requireNumbers: boolean;
    requireSpecialChars: boolean;
    specialChars: string;
    preventCommonPatterns: boolean;
    minStrengthScore: number; // 0-4
  };

  /** Email configuration */
  email: {
    maxLength: number;
    allowInternational: boolean;
    normalizeDomains: boolean;
    blockDisposable: boolean;
    blockedDomains: string[];
  };

  /** File upload limits */
  file: {
    maxSize: number;
    maxFiles: number;
    allowedTypes: string[];
    allowedExtensions: string[];
    scanForMalware: boolean;
  };

  /** Rate limiting */
  rateLimit: {
    enabled: boolean;
    defaultWindow: number; // milliseconds
    defaultMaxRequests: number;
    storage: 'memory' | 'redis' | 'database';
  };

  /** Security settings */
  security: {
    enableXssProtection: boolean;
    enableSqlInjectionProtection: boolean;
    maxInputLength: number;
    allowedHtmlTags: string[];
    sanitizeInputs: boolean;
  };

  /** Localization */
  localization: {
    defaultLocale: string;
    supportedLocales: string[];
    messageTemplates: Record<string, Record<string, string>>;
  };

  /** Logging and monitoring */
  monitoring: {
    logValidationErrors: boolean;
    logLevel: 'debug' | 'info' | 'warn' | 'error';
    enableMetrics: boolean;
    trackPerformance: boolean;
  };
}

// Default configuration
export const defaultValidationConfig: ValidationGlobalConfig = {
  defaults: {
    abortEarly: false,
    transform: true,
    stripUnknown: true,
    enableAsync: true,
    debounceMs: 300,
  },

  password: {
    minLength: 8,
    maxLength: 128,
    requireUppercase: true,
    requireLowercase: true,
    requireNumbers: true,
    requireSpecialChars: true,
    specialChars: '!@#$%^&*()_+-=[]{}|;:,.<>?',
    preventCommonPatterns: true,
    minStrengthScore: 3,
  },

  email: {
    maxLength: 254,
    allowInternational: true,
    normalizeDomains: true,
    blockDisposable: false,
    blockedDomains: [],
  },

  file: {
    maxSize: 10 * 1024 * 1024, // 10MB
    maxFiles: 5,
    allowedTypes: ['image/*', 'application/pdf', 'text/*'],
    allowedExtensions: ['jpg', 'jpeg', 'png', 'gif', 'pdf', 'txt', 'doc', 'docx'],
    scanForMalware: false,
  },

  rateLimit: {
    enabled: true,
    defaultWindow: 60 * 1000, // 1 minute
    defaultMaxRequests: 10,
    storage: 'memory',
  },

  security: {
    enableXssProtection: true,
    enableSqlInjectionProtection: true,
    maxInputLength: 10000,
    allowedHtmlTags: ['p', 'br', 'strong', 'em', 'u', 'ol', 'ul', 'li', 'a'],
    sanitizeInputs: true,
  },

  localization: {
    defaultLocale: 'en',
    supportedLocales: ['en', 'es', 'fr', 'de', 'it', 'pt', 'ja', 'ko', 'zh'],
    messageTemplates: {
      en: {
        required: 'This field is required',
        email: 'Please enter a valid email address',
        password: 'Password does not meet requirements',
        minLength: 'Must be at least {min} characters',
        maxLength: 'Must be less than {max} characters',
        fileSize: 'File size must be less than {size}',
        fileType: 'File type not allowed',
      },
    },
  },

  monitoring: {
    logValidationErrors: true,
    logLevel: 'warn',
    enableMetrics: false,
    trackPerformance: false,
  },
};

// Validation context for different environments
export interface ValidationContext {
  environment: 'development' | 'staging' | 'production';
  userRole?: string;
  features?: string[];
  locale?: string;
  rateLimit?: {
    key: string;
    maxRequests: number;
    windowMs: number;
  };
}

/**
 * Validation configuration manager
 * Provides centralized configuration management for validation behavior
 */
export class ValidationConfigManager {
  private static instance: ValidationConfigManager;
  private config: ValidationGlobalConfig;
  private context: ValidationContext;

  private constructor() {
    this.config = { ...defaultValidationConfig };
    this.context = {
      environment: 'development',
      locale: 'en',
    };
  }

  /**
   * Get singleton instance
   */
  static getInstance(): ValidationConfigManager {
    if (!ValidationConfigManager.instance) {
      ValidationConfigManager.instance = new ValidationConfigManager();
    }
    return ValidationConfigManager.instance;
  }

  /**
   * Update configuration
   */
  updateConfig(updates: Partial<ValidationGlobalConfig>): void {
    this.config = this.deepMerge(this.config, updates);
  }

  /**
   * Set validation context
   */
  setContext(context: Partial<ValidationContext>): void {
    this.context = { ...this.context, ...context };
  }

  /**
   * Get current configuration
   */
  getConfig(): ValidationGlobalConfig {
    return { ...this.config };
  }

  /**
   * Get current context
   */
  getContext(): ValidationContext {
    return { ...this.context };
  }

  /**
   * Get configuration for specific validation type
   */
  getPasswordConfig() {
    return { ...this.config.password };
  }

  getEmailConfig() {
    return { ...this.config.email };
  }

  getFileConfig() {
    return { ...this.config.file };
  }

  getRateLimitConfig() {
    return { ...this.config.rateLimit };
  }

  getSecurityConfig() {
    return { ...this.config.security };
  }

  /**
   * Get localized message
   */
  getMessage(key: string, params: Record<string, any> = {}): string {
    const locale = this.context.locale || this.config.localization.defaultLocale;
    const messages = this.config.localization.messageTemplates[locale] || 
                     this.config.localization.messageTemplates[this.config.localization.defaultLocale];
    
    let message = messages?.[key] || key;

    // Replace parameters in message
    for (const [param, value] of Object.entries(params)) {
      message = message.replace(new RegExp(`{${param}}`, 'g'), String(value));
    }

    return message;
  }

  /**
   * Check if feature is enabled
   */
  isFeatureEnabled(feature: string): boolean {
    return this.context.features?.includes(feature) ?? false;
  }

  /**
   * Get environment-specific configuration
   */
  getEnvironmentConfig() {
    const baseConfig = { ...this.config };

    // Adjust configuration based on environment
    switch (this.context.environment) {
      case 'development':
        baseConfig.monitoring.logLevel = 'debug';
        baseConfig.monitoring.logValidationErrors = true;
        break;
      
      case 'staging':
        baseConfig.monitoring.logLevel = 'info';
        baseConfig.monitoring.enableMetrics = true;
        break;
      
      case 'production':
        baseConfig.monitoring.logLevel = 'error';
        baseConfig.monitoring.enableMetrics = true;
        baseConfig.monitoring.trackPerformance = true;
        baseConfig.security.enableXssProtection = true;
        baseConfig.security.enableSqlInjectionProtection = true;
        break;
    }

    return baseConfig;
  }

  /**
   * Deep merge objects
   */
  private deepMerge(target: any, source: any): any {
    const result = { ...target };

    for (const key in source) {
      if (source[key] !== null && typeof source[key] === 'object' && !Array.isArray(source[key])) {
        result[key] = this.deepMerge(target[key] || {}, source[key]);
      } else {
        result[key] = source[key];
      }
    }

    return result;
  }

  /**
   * Reset to default configuration
   */
  reset(): void {
    this.config = { ...defaultValidationConfig };
    this.context = {
      environment: 'development',
      locale: 'en',
    };
  }

  /**
   * Export configuration for debugging
   */
  exportConfig(): string {
    return JSON.stringify({
      config: this.config,
      context: this.context,
    }, null, 2);
  }

  /**
   * Import configuration from JSON
   */
  importConfig(configJson: string): void {
    try {
      const imported = JSON.parse(configJson);
      if (imported.config) {
        this.updateConfig(imported.config);
      }
      if (imported.context) {
        this.setContext(imported.context);
      }
    } catch (error) {
      throw new Error('Invalid configuration JSON');
    }
  }
}

// Convenience functions
export function getValidationConfig(): ValidationGlobalConfig {
  return ValidationConfigManager.getInstance().getConfig();
}

export function updateValidationConfig(updates: Partial<ValidationGlobalConfig>): void {
  ValidationConfigManager.getInstance().updateConfig(updates);
}

export function setValidationContext(context: Partial<ValidationContext>): void {
  ValidationConfigManager.getInstance().setContext(context);
}

export function getValidationMessage(key: string, params?: Record<string, any>): string {
  return ValidationConfigManager.getInstance().getMessage(key, params);
}

// Configuration presets for different use cases
export const ValidationConfigPresets = {
  /** Strict configuration for high-security applications */
  strict: {
    password: {
      minLength: 12,
      requireUppercase: true,
      requireLowercase: true,
      requireNumbers: true,
      requireSpecialChars: true,
      preventCommonPatterns: true,
      minStrengthScore: 4,
    },
    security: {
      enableXssProtection: true,
      enableSqlInjectionProtection: true,
      maxInputLength: 5000,
      sanitizeInputs: true,
    },
    rateLimit: {
      enabled: true,
      defaultMaxRequests: 5,
      defaultWindow: 60 * 1000,
    },
  },

  /** Relaxed configuration for development */
  relaxed: {
    password: {
      minLength: 6,
      requireUppercase: false,
      requireLowercase: false,
      requireNumbers: false,
      requireSpecialChars: false,
      preventCommonPatterns: false,
      minStrengthScore: 1,
    },
    security: {
      enableXssProtection: false,
      enableSqlInjectionProtection: false,
      maxInputLength: 50000,
      sanitizeInputs: false,
    },
    rateLimit: {
      enabled: false,
    },
  },

  /** Performance-optimized configuration */
  performance: {
    defaults: {
      abortEarly: true,
      debounceMs: 100,
    },
    monitoring: {
      trackPerformance: true,
      enableMetrics: true,
    },
  },
} as const;