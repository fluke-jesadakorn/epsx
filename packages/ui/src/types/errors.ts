// Consolidated error types that eliminate duplication across apps

export type ErrorSeverity = 'low' | 'medium' | 'high' | 'critical';
export type ErrorCategory = 'auth' | 'api' | 'validation' | 'permission' | 'network' | 'feature' | 'system' | 'unknown';

// Unified error codes
export enum ErrorCode {
  // Authentication errors
  AUTH_INVALID_CREDENTIALS = 'AUTH_INVALID_CREDENTIALS',
  AUTH_TOKEN_EXPIRED = 'AUTH_TOKEN_EXPIRED',
  AUTH_TOKEN_INVALID = 'AUTH_TOKEN_INVALID',
  AUTH_USER_NOT_FOUND = 'AUTH_USER_NOT_FOUND',
  AUTH_SESSION_EXPIRED = 'AUTH_SESSION_EXPIRED',
  AUTH_PERMISSION_DENIED = 'AUTH_PERMISSION_DENIED',
  AUTH_ROLE_INSUFFICIENT = 'AUTH_ROLE_INSUFFICIENT',
  AUTH_MFA_REQUIRED = 'AUTH_MFA_REQUIRED',
  AUTH_ACCOUNT_LOCKED = 'AUTH_ACCOUNT_LOCKED',
  AUTH_ACCOUNT_DISABLED = 'AUTH_ACCOUNT_DISABLED',

  // API errors
  API_REQUEST_FAILED = 'API_REQUEST_FAILED',
  API_NETWORK_ERROR = 'API_NETWORK_ERROR',
  API_TIMEOUT = 'API_TIMEOUT',
  API_RATE_LIMITED = 'API_RATE_LIMITED',
  API_SERVER_ERROR = 'API_SERVER_ERROR',
  API_BAD_REQUEST = 'API_BAD_REQUEST',
  API_FORBIDDEN = 'API_FORBIDDEN',
  API_NOT_FOUND = 'API_NOT_FOUND',
  API_CONFLICT = 'API_CONFLICT',
  API_VALIDATION_ERROR = 'API_VALIDATION_ERROR',

  // Permission errors
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  PERMISSION_INSUFFICIENT = 'PERMISSION_INSUFFICIENT',
  PERMISSION_EXPIRED = 'PERMISSION_EXPIRED',
  PERMISSION_INVALID = 'PERMISSION_INVALID',

  // Feature access errors
  FEATURE_NOT_AVAILABLE = 'FEATURE_NOT_AVAILABLE',
  FEATURE_INSUFFICIENT_TOKENS = 'FEATURE_INSUFFICIENT_TOKENS',
  FEATURE_TIER_REQUIRED = 'FEATURE_TIER_REQUIRED',
  FEATURE_SUBSCRIPTION_REQUIRED = 'FEATURE_SUBSCRIPTION_REQUIRED',
  FEATURE_LIMIT_EXCEEDED = 'FEATURE_LIMIT_EXCEEDED',

  // Validation errors
  VALIDATION_REQUIRED_FIELD = 'VALIDATION_REQUIRED_FIELD',
  VALIDATION_INVALID_FORMAT = 'VALIDATION_INVALID_FORMAT',
  VALIDATION_MIN_LENGTH = 'VALIDATION_MIN_LENGTH',
  VALIDATION_MAX_LENGTH = 'VALIDATION_MAX_LENGTH',
  VALIDATION_INVALID_EMAIL = 'VALIDATION_INVALID_EMAIL',
  VALIDATION_PASSWORDS_MISMATCH = 'VALIDATION_PASSWORDS_MISMATCH',

  // System errors
  SYSTEM_MAINTENANCE = 'SYSTEM_MAINTENANCE',
  SYSTEM_OVERLOADED = 'SYSTEM_OVERLOADED',
  SYSTEM_CONFIGURATION_ERROR = 'SYSTEM_CONFIGURATION_ERROR',
  SYSTEM_DATABASE_ERROR = 'SYSTEM_DATABASE_ERROR',
  SYSTEM_STORAGE_ERROR = 'SYSTEM_STORAGE_ERROR',

  // Unknown/Generic
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}

// Base error interface
export interface BaseError {
  code: ErrorCode;
  message: string;
  category: ErrorCategory;
  severity: ErrorSeverity;
  timestamp: string;
  context?: Record<string, any>;
  stack?: string;
  requestId?: string;
}

// Specialized error classes
export class AuthError extends Error implements BaseError {
  public readonly code: ErrorCode;
  public readonly category: ErrorCategory = 'auth';
  public readonly severity: ErrorSeverity;
  public readonly timestamp: string;
  public readonly context?: Record<string, any>;
  public readonly requestId?: string;

  constructor(
    code: ErrorCode,
    message: string,
    options: {
      severity?: ErrorSeverity;
      context?: Record<string, any>;
      requestId?: string;
      cause?: Error;
    } = {}
  ) {
    super(message);
    this.name = 'AuthError';
    this.code = code;
    this.severity = options.severity || 'medium';
    this.timestamp = new Date().toISOString();
    this.context = options.context;
    this.requestId = options.requestId;
    
    if (options.cause) {
      this.cause = options.cause;
      this.stack = options.cause.stack;
    }
  }

  // Helper methods for specific auth error types
  static invalidCredentials(context?: Record<string, any>): AuthError {
    return new AuthError(
      ErrorCode.AUTH_INVALID_CREDENTIALS,
      'Invalid email or password',
      { severity: 'medium', context }
    );
  }

  static tokenExpired(context?: Record<string, any>): AuthError {
    return new AuthError(
      ErrorCode.AUTH_TOKEN_EXPIRED,
      'Your session has expired. Please sign in again.',
      { severity: 'medium', context }
    );
  }

  static permissionDenied(permission: string, context?: Record<string, any>): AuthError {
    return new AuthError(
      ErrorCode.AUTH_PERMISSION_DENIED,
      `You don't have permission to ${permission}`,
      { severity: 'high', context: { permission, ...context } }
    );
  }

  static roleInsufficient(requiredRole: string, userRole: string, context?: Record<string, any>): AuthError {
    return new AuthError(
      ErrorCode.AUTH_ROLE_INSUFFICIENT,
      `This action requires ${requiredRole} role. Your role: ${userRole}`,
      { severity: 'high', context: { requiredRole, userRole, ...context } }
    );
  }
}

export class ApiError extends Error implements BaseError {
  public readonly code: ErrorCode;
  public readonly category: ErrorCategory = 'api';
  public readonly severity: ErrorSeverity;
  public readonly timestamp: string;
  public readonly context?: Record<string, any>;
  public readonly requestId?: string;
  public readonly status?: number;
  public readonly url?: string;

  constructor(
    code: ErrorCode,
    message: string,
    options: {
      severity?: ErrorSeverity;
      context?: Record<string, any>;
      requestId?: string;
      status?: number;
      url?: string;
      cause?: Error;
    } = {}
  ) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.severity = options.severity || 'medium';
    this.timestamp = new Date().toISOString();
    this.context = options.context;
    this.requestId = options.requestId;
    this.status = options.status;
    this.url = options.url;
    
    if (options.cause) {
      this.cause = options.cause;
      this.stack = options.cause.stack;
    }
  }

  // Helper methods for common API errors
  static networkError(url: string, context?: Record<string, any>): ApiError {
    return new ApiError(
      ErrorCode.API_NETWORK_ERROR,
      'Network connection failed. Please check your internet connection.',
      { severity: 'high', url, context }
    );
  }

  static timeout(url: string, context?: Record<string, any>): ApiError {
    return new ApiError(
      ErrorCode.API_TIMEOUT,
      'Request timed out. Please try again.',
      { severity: 'medium', url, context }
    );
  }

  static serverError(status: number, url: string, context?: Record<string, any>): ApiError {
    return new ApiError(
      ErrorCode.API_SERVER_ERROR,
      `Server error (${status}). Please try again later.`,
      { severity: 'high', status, url, context }
    );
  }

  static rateLimited(retryAfter?: number, context?: Record<string, any>): ApiError {
    const message = retryAfter 
      ? `Too many requests. Please try again in ${retryAfter} seconds.`
      : 'Too many requests. Please try again later.';
      
    return new ApiError(
      ErrorCode.API_RATE_LIMITED,
      message,
      { severity: 'medium', context: { retryAfter, ...context } }
    );
  }
}

export class FeatureError extends Error implements BaseError {
  public readonly code: ErrorCode;
  public readonly category: ErrorCategory = 'feature';
  public readonly severity: ErrorSeverity;
  public readonly timestamp: string;
  public readonly context?: Record<string, any>;
  public readonly requestId?: string;
  public readonly featureName?: string;
  public readonly requirement?: string;

  constructor(
    code: ErrorCode,
    message: string,
    options: {
      severity?: ErrorSeverity;
      context?: Record<string, any>;
      requestId?: string;
      featureName?: string;
      requirement?: string;
      cause?: Error;
    } = {}
  ) {
    super(message);
    this.name = 'FeatureError';
    this.code = code;
    this.severity = options.severity || 'low';
    this.timestamp = new Date().toISOString();
    this.context = options.context;
    this.requestId = options.requestId;
    this.featureName = options.featureName;
    this.requirement = options.requirement;
    
    if (options.cause) {
      this.cause = options.cause;
      this.stack = options.cause.stack;
    }
  }

  // Helper methods for feature access errors
  static insufficientTokens(
    featureName: string, 
    required: number, 
    available: number, 
    context?: Record<string, any>
  ): FeatureError {
    return new FeatureError(
      ErrorCode.FEATURE_INSUFFICIENT_TOKENS,
      `${featureName} requires ${required} tokens. You have ${available} tokens.`,
      { 
        severity: 'low', 
        featureName, 
        requirement: `${required} tokens`,
        context: { required, available, ...context } 
      }
    );
  }

  static tierRequired(
    featureName: string, 
    requiredTier: string, 
    userTier: string, 
    context?: Record<string, any>
  ): FeatureError {
    return new FeatureError(
      ErrorCode.FEATURE_TIER_REQUIRED,
      `${featureName} requires ${requiredTier} tier. Your tier: ${userTier}`,
      { 
        severity: 'low', 
        featureName, 
        requirement: `${requiredTier} tier`,
        context: { requiredTier, userTier, ...context } 
      }
    );
  }

  static limitExceeded(
    featureName: string, 
    limit: number, 
    usage: number, 
    context?: Record<string, any>
  ): FeatureError {
    return new FeatureError(
      ErrorCode.FEATURE_LIMIT_EXCEEDED,
      `${featureName} usage limit exceeded. Limit: ${limit}, Used: ${usage}`,
      { 
        severity: 'medium', 
        featureName, 
        requirement: `under ${limit} usage`,
        context: { limit, usage, ...context } 
      }
    );
  }
}

export class ValidationError extends Error implements BaseError {
  public readonly code: ErrorCode;
  public readonly category: ErrorCategory = 'validation';
  public readonly severity: ErrorSeverity = 'low';
  public readonly timestamp: string;
  public readonly context?: Record<string, any>;
  public readonly requestId?: string;
  public readonly field?: string;
  public readonly value?: any;

  constructor(
    code: ErrorCode,
    message: string,
    options: {
      context?: Record<string, any>;
      requestId?: string;
      field?: string;
      value?: any;
      cause?: Error;
    } = {}
  ) {
    super(message);
    this.name = 'ValidationError';
    this.code = code;
    this.timestamp = new Date().toISOString();
    this.context = options.context;
    this.requestId = options.requestId;
    this.field = options.field;
    this.value = options.value;
    
    if (options.cause) {
      this.cause = options.cause;
      this.stack = options.cause.stack;
    }
  }

  // Helper methods for validation errors
  static required(field: string, context?: Record<string, any>): ValidationError {
    return new ValidationError(
      ErrorCode.VALIDATION_REQUIRED_FIELD,
      `${field} is required`,
      { field, context }
    );
  }

  static invalidFormat(field: string, expectedFormat: string, context?: Record<string, any>): ValidationError {
    return new ValidationError(
      ErrorCode.VALIDATION_INVALID_FORMAT,
      `${field} must be in ${expectedFormat} format`,
      { field, context: { expectedFormat, ...context } }
    );
  }

  static minLength(field: string, minLength: number, actualLength: number, context?: Record<string, any>): ValidationError {
    return new ValidationError(
      ErrorCode.VALIDATION_MIN_LENGTH,
      `${field} must be at least ${minLength} characters. Got ${actualLength}`,
      { field, context: { minLength, actualLength, ...context } }
    );
  }
}

// Type guards for error identification
export function isAuthError(error: unknown): error is AuthError {
  return error instanceof AuthError || (error instanceof Error && error.name === 'AuthError');
}

export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError || (error instanceof Error && error.name === 'ApiError');
}

export function isFeatureError(error: unknown): error is FeatureError {
  return error instanceof FeatureError || (error instanceof Error && error.name === 'FeatureError');
}

export function isValidationError(error: unknown): error is ValidationError {
  return error instanceof ValidationError || (error instanceof Error && error.name === 'ValidationError');
}

// Error severity mapping
export function getErrorSeverity(error: unknown): ErrorSeverity {
  if (isAuthError(error) || isApiError(error) || isFeatureError(error) || isValidationError(error)) {
    return error.severity;
  }
  
  // Default severity for unknown errors
  return 'medium';
}

// Error category mapping
export function getErrorCategory(error: unknown): ErrorCategory {
  if (isAuthError(error)) return 'auth';
  if (isApiError(error)) return 'api';
  if (isFeatureError(error)) return 'feature';
  if (isValidationError(error)) return 'validation';
  
  // Try to infer from error message
  const message = error instanceof Error ? error.message.toLowerCase() : '';
  
  if (message.includes('network') || message.includes('connection')) return 'network';
  if (message.includes('permission') || message.includes('unauthorized')) return 'permission';
  if (message.includes('server') || message.includes('database')) return 'system';
  
  return 'unknown';
}

// Error formatting utilities
export function formatErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  
  if (typeof error === 'string') {
    return error;
  }
  
  return 'An unexpected error occurred';
}

export function createErrorId(): string {
  return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}