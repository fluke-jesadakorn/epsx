"use client";

import type { BaseError, ErrorSeverity, ErrorCategory } from '../types/errors.js';

export interface ErrorMonitoringConfig {
  enableConsoleLogging?: boolean;
  enableAnalytics?: boolean;
  enableSentry?: boolean;
  enableCustomReporting?: boolean;
  environment?: 'development' | 'staging' | 'production';
  userId?: string;
  sessionId?: string;
  buildVersion?: string;
}

export interface ErrorContext {
  component?: string;
  action?: string;
  url?: string;
  userAgent?: string;
  timestamp?: string;
  userId?: string;
  sessionId?: string;
  buildVersion?: string;
  customData?: Record<string, unknown>;
}

export interface ErrorReportData {
  error: Error | BaseError;
  context: ErrorContext;
  severity: ErrorSeverity;
  category: ErrorCategory;
  errorId: string;
  fingerprint?: string;
}

/**
 * Consolidated error monitoring system that replaces scattered monitoring implementations
 * Supports multiple reporting backends and provides consistent error tracking
 */
export class ErrorMonitor {
  private config: Required<ErrorMonitoringConfig>;
  private isInitialized = false;

  constructor(config: ErrorMonitoringConfig = {}) {
    this.config = {
      enableConsoleLogging: config.enableConsoleLogging ?? true,
      enableAnalytics: config.enableAnalytics ?? true,
      enableSentry: config.enableSentry ?? false,
      enableCustomReporting: config.enableCustomReporting ?? false,
      environment: config.environment ?? 'development',
      userId: config.userId ?? '',
      sessionId: config.sessionId ?? this.generateSessionId(),
      buildVersion: config.buildVersion ?? '',
    };
  }

  /**
   * Initialize the error monitor with global error handlers
   */
  initialize(): void {
    if (this.isInitialized) return;

    // Global error handler
    if (typeof window !== 'undefined') {
      window.addEventListener('error', (event) => {
        this.reportError(event.error, {
          component: 'global',
          action: 'unhandled_error',
          url: window.location.href,
          customData: {
            filename: event.filename,
            lineno: event.lineno,
            colno: event.colno,
          },
        });
      });

      // Unhandled promise rejection handler
      window.addEventListener('unhandledrejection', (event) => {
        this.reportError(
          new Error(`Unhandled Promise Rejection: ${event.reason}`),
          {
            component: 'global',
            action: 'unhandled_promise_rejection',
            url: window.location.href,
            customData: {
              reason: event.reason,
              promise: event.promise,
            },
          }
        );
      });
    }

    this.isInitialized = true;
  }

  /**
   * Update configuration
   */
  updateConfig(updates: Partial<ErrorMonitoringConfig>): void {
    this.config = { ...this.config, ...updates };
  }

  /**
   * Set user context
   */
  setUser(userId: string, additionalData?: Record<string, unknown>): void {
    this.config.userId = userId;
    
    // Update Sentry user context if enabled
    if (this.config.enableSentry && typeof window !== 'undefined' && window.Sentry) {
      window.Sentry.setUser({ id: userId, ...additionalData });
    }
  }

  /**
   * Set custom context data
   */
  setContext(key: string, data: unknown): void {
    // Update Sentry context if enabled
    if (this.config.enableSentry && typeof window !== 'undefined' && window.Sentry) {
      window.Sentry.setContext(key, data);
    }
  }

  /**
   * Main error reporting method
   */
  reportError(
    error: Error | BaseError,
    context: Partial<ErrorContext> = {},
    options: {
      severity?: ErrorSeverity;
      category?: ErrorCategory;
      fingerprint?: string;
    } = {}
  ): string {
    const errorId = this.generateErrorId();
    const enhancedContext = this.enhanceContext(context);
    const severity = options.severity || this.inferSeverity(error);
    const category = options.category || this.inferCategory(error);

    const reportData: ErrorReportData = {
      error,
      context: enhancedContext,
      severity,
      category,
      errorId,
      fingerprint: options.fingerprint || this.generateFingerprint(error),
    };

    // Console logging
    if (this.config.enableConsoleLogging) {
      this.logToConsole(reportData);
    }

    // Analytics reporting
    if (this.config.enableAnalytics) {
      this.reportToAnalytics(reportData);
    }

    // Sentry reporting
    if (this.config.enableSentry) {
      this.reportToSentry(reportData);
    }

    // Custom reporting
    if (this.config.enableCustomReporting) {
      this.reportToCustomBackend(reportData);
    }

    return errorId;
  }

  /**
   * Report performance issues
   */
  reportPerformanceIssue(
    metric: string,
    value: number,
    threshold: number,
    context: Partial<ErrorContext> = {}
  ): void {
    const error = new Error(`Performance issue: ${metric} (${value}) exceeded threshold (${threshold})`);
    
    this.reportError(error, {
      ...context,
      action: 'performance_issue',
      customData: {
        metric,
        value,
        threshold,
        exceedBy: value - threshold,
      },
    }, {
      severity: 'medium',
      category: 'system',
    });
  }

  /**
   * Report user feedback about errors
   */
  reportUserFeedback(
    errorId: string,
    feedback: string,
    userEmail?: string,
    context: Partial<ErrorContext> = {}
  ): void {
    const error = new Error(`User feedback for error ${errorId}: ${feedback}`);
    
    this.reportError(error, {
      ...context,
      action: 'user_feedback',
      customData: {
        errorId,
        feedback,
        userEmail,
      },
    }, {
      severity: 'low',
      category: 'unknown',
    });
  }

  /**
   * Create error monitoring hook for React components
   */
  createErrorHook() {
    return (error: Error, errorInfo?: unknown) => {
      this.reportError(error, {
        component: 'react_component',
        action: 'component_error',
        customData: {
          componentStack: errorInfo?.componentStack,
          errorBoundary: true,
        },
      });
    };
  }

  private generateErrorId(): string {
    return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateFingerprint(error: Error | BaseError): string {
    // Create a fingerprint based on error type and message
    const message = error.message || 'unknown';
    const type = (error as Error).name || 'Error';
    const stack = error.stack?.split('\n')[1] || '';
    
    return `${type}:${message}:${stack}`.replace(/[^a-zA-Z0-9]/g, '_').substring(0, 100);
  }

  private enhanceContext(context: Partial<ErrorContext>): ErrorContext {
    const enhanced: ErrorContext = {
      timestamp: new Date().toISOString(),
      url: typeof window !== 'undefined' ? window.location.href : '',
      userAgent: typeof window !== 'undefined' ? navigator.userAgent : '',
      userId: this.config.userId,
      sessionId: this.config.sessionId,
      buildVersion: this.config.buildVersion,
      ...context,
    };

    return enhanced;
  }

  private inferSeverity(error: Error | BaseError): ErrorSeverity {
    // Check if it's a BaseError with severity
    if ('severity' in error && error.severity) {
      return error.severity;
    }

    // Infer from error type or message
    const message = error.message?.toLowerCase() || '';
    const name = ((error as Error).name || '').toLowerCase();

    if (name.includes('auth') || message.includes('unauthorized') || message.includes('forbidden')) {
      return 'medium';
    }

    if (message.includes('network') || message.includes('timeout') || message.includes('connection')) {
      return 'medium';
    }

    if (message.includes('critical') || message.includes('fatal') || message.includes('crash')) {
      return 'critical';
    }

    if (message.includes('warning') || message.includes('deprecat')) {
      return 'low';
    }

    return 'high'; // Default for unknown errors
  }

  private inferCategory(error: Error | BaseError): ErrorCategory {
    // Check if it's a BaseError with category
    if ('category' in error && error.category) {
      return error.category;
    }

    // Infer from error type or message
    const message = error.message?.toLowerCase() || '';
    const name = ((error as Error).name || '').toLowerCase();

    if (name.includes('auth') || message.includes('auth') || message.includes('unauthorized')) {
      return 'auth';
    }

    if (name.includes('api') || message.includes('api') || message.includes('fetch') || message.includes('request')) {
      return 'api';
    }

    if (message.includes('network') || message.includes('connection') || message.includes('timeout')) {
      return 'network';
    }

    if (message.includes('permission') || message.includes('access') || message.includes('forbidden')) {
      return 'permission';
    }

    if (message.includes('validation') || message.includes('invalid') || message.includes('required')) {
      return 'validation';
    }

    if (message.includes('feature') || message.includes('tier') || message.includes('token')) {
      return 'feature';
    }

    return 'unknown';
  }

  private logToConsole(reportData: ErrorReportData): void {
    const { error, context, severity, category, errorId } = reportData;
    
    const severityColors = {
      low: '#3b82f6',      // blue
      medium: '#f59e0b',   // amber
      high: '#ef4444',     // red
      critical: '#dc2626', // dark red
    };

    console.group(`🚨 Error Report (${severity.toUpperCase()})`);
    console.log(`%cError ID: ${errorId}`, `color: ${severityColors[severity]}`);
    console.log(`%cCategory: ${category}`, `color: ${severityColors[severity]}`);
    console.error('Error:', error);
    console.log('Context:', context);
    console.log('Stack:', error.stack);
    console.groupEnd();
  }

  private reportToAnalytics(reportData: ErrorReportData): void {
    if (typeof window === 'undefined') return;

    // Google Analytics 4
    if (window.gtag) {
      window.gtag('event', 'exception', {
        description: reportData.error.message,
        fatal: reportData.severity === 'critical',
        error_id: reportData.errorId,
        error_category: reportData.category,
        error_severity: reportData.severity,
        custom_map: {
          component: reportData.context.component,
          action: reportData.context.action,
        },
      });
    }

    // Custom analytics
    if (window.analytics) {
      window.analytics.track('Error Occurred', {
        errorId: reportData.errorId,
        errorMessage: reportData.error.message,
        errorName: (reportData.error as Error).name || 'Error',
        category: reportData.category,
        severity: reportData.severity,
        context: reportData.context,
        fingerprint: reportData.fingerprint,
      });
    }
  }

  private reportToSentry(reportData: ErrorReportData): void {
    if (typeof window === 'undefined' || !window.Sentry) return;

    // Type-safe Sentry integration with proper interface
    interface SentryScope {
      setTag: (key: string, value: string) => void;
      setLevel: (level: string) => void;
      setFingerprint: (fingerprint: string[]) => void;
      setContext: (key: string, context: Record<string, unknown>) => void;
    }

    interface SentryInstance {
      withScope: (callback: (scope: SentryScope) => void) => void;
      captureException: (error: Error) => void;
    }

    const sentry = window.Sentry as unknown as SentryInstance;
    
    sentry.withScope((scope: SentryScope) => {
      scope.setTag('error_id', reportData.errorId);
      scope.setTag('category', reportData.category);
      scope.setLevel(this.mapSeverityToSentryLevel(reportData.severity));
      scope.setFingerprint([reportData.fingerprint || 'default']);
      
      // Set context data
      scope.setContext('error_context', reportData.context);
      
      // Capture the error
      sentry.captureException(reportData.error as Error);
    });
  }

  private async reportToCustomBackend(reportData: ErrorReportData): Promise<void> {
    try {
      await fetch('/api/errors/report', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          errorId: reportData.errorId,
          message: reportData.error.message,
          name: (reportData.error as Error).name || 'Error',
          stack: reportData.error.stack,
          category: reportData.category,
          severity: reportData.severity,
          context: reportData.context,
          fingerprint: reportData.fingerprint,
          environment: this.config.environment,
        }),
      });
    } catch (error) {
      // Fail silently for error reporting errors
      console.warn('Failed to report error to custom backend:', error);
    }
  }

  private mapSeverityToSentryLevel(severity: ErrorSeverity): 'debug' | 'info' | 'warning' | 'error' | 'fatal' {
    switch (severity) {
      case 'low':
        return 'info';
      case 'medium':
        return 'warning';
      case 'high':
        return 'error';
      case 'critical':
        return 'fatal';
      default:
        return 'error';
    }
  }
}

// Global error monitor instance
let globalErrorMonitor: ErrorMonitor | null = null;

/**
 * Get or create the global error monitor instance
 */
export function getErrorMonitor(config?: ErrorMonitoringConfig): ErrorMonitor {
  if (!globalErrorMonitor) {
    globalErrorMonitor = new ErrorMonitor(config);
    globalErrorMonitor.initialize();
  } else if (config) {
    globalErrorMonitor.updateConfig(config);
  }
  
  return globalErrorMonitor;
}

/**
 * Quick error reporting function
 */
export function reportError(
  error: Error | BaseError,
  context?: Partial<ErrorContext>,
  options?: {
    severity?: ErrorSeverity;
    category?: ErrorCategory;
  }
): string {
  const monitor = getErrorMonitor();
  return monitor.reportError(error, context, options);
}

/**
 * React hook for error monitoring
 */
export function useErrorMonitoring(config?: ErrorMonitoringConfig): { reportError: (error: Error, context?: Partial<ErrorContext>) => void; reportMessage: (message: string, level?: string, context?: Partial<ErrorContext>) => void; reportPerformanceIssue: (metric: string, value: number, context?: Partial<ErrorContext>) => void } {
  const monitor = getErrorMonitor(config);
  
  return {
    reportError: (error: Error, context?: Partial<ErrorContext>) => 
      monitor.reportError(error, context),
    reportPerformanceIssue: (metric: string, value: number, threshold: number) =>
      monitor.reportPerformanceIssue(metric, value, threshold),
    setUser: (userId: string, data?: Record<string, unknown>) => 
      monitor.setUser(userId, data),
    setContext: (key: string, data: unknown) => 
      monitor.setContext(key, data),
  };
}

// TypeScript declaration for global objects
declare global {
  interface Window {
    gtag?: (...args: unknown[]) => void;
    analytics?: {
      track: (event: string, properties?: Record<string, unknown>) => void;
    };
    Sentry?: {
      captureException: (error: Error) => void;
      withScope: (callback: (scope: unknown) => void) => void;
      setUser: (user: Record<string, unknown>) => void;
      setContext: (key: string, data: unknown) => void;
    };
  }
}