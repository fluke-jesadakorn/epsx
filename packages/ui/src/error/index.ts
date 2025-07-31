// Error boundary components
export * from '../components/error-boundary';

// Error types and utilities
export * from '../types/errors';

// Error monitoring
export * from '../utils/error-monitoring';

// Convenience re-exports for common patterns
export {
  ErrorBoundary,
  withErrorBoundary,
  ErrorBoundaryPresets,
  type ErrorBoundaryProps,
  type ErrorRecoveryAction,
  type ErrorSeverity,
  type ErrorContext as ErrorBoundaryContext,
} from '../components/error-boundary';

export {
  AuthError,
  ApiError,
  FeatureError,
  ValidationError,
  ErrorCode,
  type BaseError,
  isAuthError,
  isApiError,
  isFeatureError,
  isValidationError,
  getErrorSeverity,
  getErrorCategory,
  formatErrorMessage,
  createErrorId,
} from '../types/errors';

export {
  ErrorMonitor,
  getErrorMonitor,
  reportError,
  useErrorMonitoring,
  type ErrorMonitoringConfig,
  type ErrorContext,
  type ErrorReportData,
} from '../utils/error-monitoring';