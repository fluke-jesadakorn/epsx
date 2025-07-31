"use client";

import React, { Component, ErrorInfo, ReactNode } from 'react';
// Define error types locally to avoid external dependency
export interface AuthError extends Error {
  type: 'auth';
  code?: string;
}

export interface ApiError extends Error {
  type: 'api';
  status?: number;
  code?: string;
}

// Error severity levels
export type ErrorSeverity = 'low' | 'medium' | 'high' | 'critical';

// Error contexts for different UI themes
export type ErrorContext = 'generic' | 'feature' | 'provider' | 'admin' | 'auth' | 'api';

// Error recovery strategies
export type RecoveryStrategy = 'reload' | 'retry' | 'redirect' | 'signout' | 'upgrade' | 'custom';

export interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorId: string;
  retryCount: number;
}

export interface ErrorRecoveryAction {
  type: RecoveryStrategy;
  label: string;
  action: () => void | Promise<void>;
  variant?: 'primary' | 'secondary' | 'destructive';
  disabled?: boolean;
}

export interface ErrorBoundaryProps {
  children: ReactNode;
  
  // Context and theming
  context?: ErrorContext;
  title?: string;
  
  // Recovery options
  enableRetry?: boolean;
  maxRetries?: number;
  recoveryActions?: ErrorRecoveryAction[];
  
  // Custom fallback
  fallback?: (error: Error, retry: () => void) => ReactNode;
  fallbackComponent?: React.ComponentType<{
    error: Error;
    errorInfo: ErrorInfo | null;
    retry: () => void;
    context: ErrorContext;
  }>;
  
  // Logging and monitoring
  onError?: (error: Error, errorInfo: ErrorInfo, errorId: string) => void;
  enableLogging?: boolean;
  enableMonitoring?: boolean;
  
  // Feature-specific options
  featureRequirements?: {
    permission?: string;
    role?: string;
    tokenBalance?: number;
    tier?: string;
  };
  
  // Isolation options
  isolateError?: boolean;
  resetOnPropsChange?: boolean;
  resetKeys?: string[];
}

/**
 * Consolidated ErrorBoundary component that replaces all duplicate implementations
 * Supports multiple contexts, recovery strategies, and monitoring integration
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  private errorId: string = '';
  private resetTimeoutId: number | null = null;

  static defaultProps: Partial<ErrorBoundaryProps> = {
    context: 'generic',
    enableRetry: true,
    maxRetries: 3,
    enableLogging: true,
    enableMonitoring: true,
    isolateError: true,
    resetOnPropsChange: false,
  };

  constructor(props: ErrorBoundaryProps) {
    super(props);
    
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
      retryCount: 0,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    const errorId = `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    return {
      hasError: true,
      error,
      errorId,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({ errorInfo });
    
    // Log error
    if (this.props.enableLogging) {
      this.logError(error, errorInfo);
    }
    
    // Report to monitoring
    if (this.props.enableMonitoring) {
      this.reportError(error, errorInfo);
    }
    
    // Call custom error handler
    if (this.props.onError) {
      this.props.onError(error, errorInfo, this.state.errorId);
    }
  }

  componentDidUpdate(prevProps: ErrorBoundaryProps) {
    const { resetOnPropsChange, resetKeys } = this.props;
    
    if (this.state.hasError && resetOnPropsChange) {
      if (resetKeys) {
        // Check if any reset keys have changed
        const hasChanges = resetKeys.some(key => 
          (prevProps as any)[key] !== (this.props as any)[key]
        );
        
        if (hasChanges) {
          this.resetErrorBoundary();
        }
      } else {
        // Reset on any props change
        this.resetErrorBoundary();
      }
    }
  }

  private logError = (error: Error, errorInfo: ErrorInfo) => {
    const { context } = this.props;
    
    console.group(`🚨 Error Boundary (${context})`);
    console.error('Error:', error);
    console.error('Component Stack:', errorInfo.componentStack);
    console.error('Error ID:', this.state.errorId);
    console.groupEnd();
  };

  private reportError = (error: Error, errorInfo: ErrorInfo) => {
    // Integration with monitoring services (Sentry, etc.)
    if (typeof window !== 'undefined' && window.gtag) {
      window.gtag('event', 'exception', {
        description: error.message,
        fatal: false,
        error_boundary_context: this.props.context,
        error_id: this.state.errorId,
      });
    }
    
    // Additional monitoring integrations can be added here
  };

  private resetErrorBoundary = () => {
    if (this.resetTimeoutId) {
      clearTimeout(this.resetTimeoutId);
    }
    
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
      retryCount: 0,
    });
  };

  private handleRetry = () => {
    const { maxRetries = 3 } = this.props;
    const { retryCount } = this.state;
    
    if (retryCount < maxRetries) {
      this.setState({ retryCount: retryCount + 1 });
      
      // Add a small delay before retry
      this.resetTimeoutId = window.setTimeout(() => {
        this.resetErrorBoundary();
      }, 100);
    }
  };

  private getDefaultRecoveryActions = (): ErrorRecoveryAction[] => {
    const { context, enableRetry, maxRetries = 3 } = this.props;
    const { retryCount } = this.state;
    const canRetry = enableRetry && retryCount < maxRetries;
    
    const actions: ErrorRecoveryAction[] = [];
    
    // Add retry action
    if (canRetry) {
      actions.push({
        type: 'retry',
        label: 'Try Again',
        action: this.handleRetry,
        variant: 'primary',
      });
    }
    
    // Context-specific actions
    switch (context) {
      case 'admin':
        actions.push({
          type: 'reload',
          label: 'Reload Page',
          action: () => window.location.reload(),
          variant: 'secondary',
        });
        break;
        
      case 'auth':
        actions.push({
          type: 'signout',
          label: 'Sign Out',
          action: () => {
            // This would integrate with auth system
            window.location.href = '/auth/signout';
          },
          variant: 'destructive',
        });
        break;
        
      case 'feature':
        if (this.props.featureRequirements) {
          actions.push({
            type: 'upgrade',
            label: 'Upgrade Access',
            action: () => {
              window.location.href = '/upgrade';
            },
            variant: 'primary',
          });
        }
        break;
        
      default:
        actions.push({
          type: 'reload',
          label: 'Reload',
          action: () => window.location.reload(),
          variant: 'secondary',
        });
    }
    
    return actions;
  };

  private parseErrorMessage = (error: Error): { title: string; message: string; severity: ErrorSeverity } => {
    const { context, featureRequirements } = this.props;
    
    // Handle specific error types
    if (error.name === 'AuthError' || (error as any).type === 'auth') {
      return {
        title: 'Authentication Error',
        message: 'Please sign in to continue',
        severity: 'medium',
      };
    }
    
    if (error.name === 'ApiError' || (error as any).type === 'api') {
      return {
        title: 'Service Error',
        message: 'Unable to connect to our services. Please try again.',
        severity: 'medium',
      };
    }
    
    // Feature-specific error parsing
    if (context === 'feature' && featureRequirements) {
      if (error.message.includes('permission') || error.message.includes('Permission')) {
        return {
          title: 'Access Required',
          message: 'You need additional permissions to access this feature.',
          severity: 'low',
        };
      }
      
      if (error.message.includes('token') || error.message.includes('balance')) {
        return {
          title: 'Insufficient Tokens',
          message: 'You need more tokens to use this feature.',
          severity: 'low',
        };
      }
      
      if (error.message.includes('tier') || error.message.includes('upgrade')) {
        return {
          title: 'Upgrade Required',
          message: 'This feature requires a higher subscription tier.',
          severity: 'low',
        };
      }
    }
    
    // Default error parsing
    const title = context === 'admin' ? 'Admin Panel Error' : 'Something went wrong';
    const severity: ErrorSeverity = error.message.includes('network') ? 'medium' : 'high';
    
    return {
      title,
      message: error.message || 'An unexpected error occurred',
      severity,
    };
  };

  private renderErrorUI = () => {
    const { error, errorInfo } = this.state;
    const { context, title, recoveryActions } = this.props;
    
    if (!error) return null;
    
    const { title: errorTitle, message, severity } = this.parseErrorMessage(error);
    const displayTitle = title || errorTitle;
    const actions = recoveryActions || this.getDefaultRecoveryActions();
    
    // Theme classes based on context
    const getThemeClasses = () => {
      const base = "rounded-lg border p-6 shadow-sm";
      
      switch (context) {
        case 'admin':
          return `${base} border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950`;
        case 'feature':
          return `${base} border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-950`;
        case 'auth':
          return `${base} border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-950`;
        default:
          return `${base} border-gray-200 bg-gray-50 dark:border-gray-800 dark:bg-gray-950`;
      }
    };
    
    const getSeverityIcon = () => {
      switch (severity) {
        case 'critical':
          return '🔴';
        case 'high':
          return '🟠';
        case 'medium':
          return '🟡';
        case 'low':
          return '🔵';
        default:
          return '⚠️';
      }
    };
    
    return (
      <div className={getThemeClasses()}>
        <div className="flex items-start space-x-3">
          <div className="text-2xl">{getSeverityIcon()}</div>
          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {displayTitle}
            </h3>
            <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
              {message}
            </p>
            
            {true && (
              <details className="mt-4">
                <summary className="cursor-pointer text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200">
                  Error Details
                </summary>
                <pre className="mt-2 p-3 bg-gray-800 text-gray-100 text-xs rounded overflow-auto max-h-40">
                  {error.stack}
                </pre>
              </details>
            )}
            
            {actions.length > 0 && (
              <div className="mt-4 flex flex-wrap gap-2">
                {actions.map((action, index) => (
                  <button
                    key={index}
                    onClick={action.action}
                    disabled={action.disabled}
                    className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                      action.variant === 'primary'
                        ? 'bg-blue-600 text-white hover:bg-blue-700 disabled:bg-blue-300'
                        : action.variant === 'destructive'
                        ? 'bg-red-600 text-white hover:bg-red-700 disabled:bg-red-300'
                        : 'bg-gray-200 text-gray-900 hover:bg-gray-300 disabled:bg-gray-100'
                    }`}
                  >
                    {action.label}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };

  render() {
    const { hasError, error, errorInfo } = this.state;
    const { children, fallback, fallbackComponent } = this.props;
    
    if (hasError && error) {
      // Custom fallback component
      if (fallbackComponent) {
        const FallbackComponent = fallbackComponent;
        return (
          <FallbackComponent
            error={error}
            errorInfo={errorInfo}
            retry={this.handleRetry}
            context={this.props.context!}
          />
        );
      }
      
      // Custom fallback function
      if (fallback) {
        return fallback(error, this.handleRetry);
      }
      
      // Default error UI
      return this.renderErrorUI();
    }
    
    return children;
  }
}

// HOC for easy wrapping
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<ErrorBoundaryProps, 'children'>
) {
  const WrappedComponent = React.forwardRef<any, P>((props, ref) => {
    return (
      <ErrorBoundary {...errorBoundaryProps}>
        <Component {...(props as P)} ref={ref} />
      </ErrorBoundary>
    );
  });
  
  WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name})`;
  
  return WrappedComponent;
}

// Preset configurations for common use cases
export const ErrorBoundaryPresets = {
  // Generic app-level boundary
  App: (props: Partial<ErrorBoundaryProps> = {}) => (
    <ErrorBoundary context="generic" enableRetry={true} maxRetries={3} {...props} />
  ),
  
  // Admin panel boundary
  Admin: (props: Partial<ErrorBoundaryProps> = {}) => (
    <ErrorBoundary 
      context="admin" 
      title="Admin Panel Error"
      enableRetry={true}
      maxRetries={2}
      {...props} 
    />
  ),
  
  // Feature-specific boundary
  Feature: (featureRequirements: ErrorBoundaryProps['featureRequirements'], props: Partial<ErrorBoundaryProps> = {}) => (
    <ErrorBoundary 
      context="feature"
      featureRequirements={featureRequirements}
      enableRetry={false}
      {...props} 
    />
  ),
  
  // Provider boundary
  Provider: (props: Partial<ErrorBoundaryProps> = {}) => (
    <ErrorBoundary 
      context="provider"
      enableRetry={true}
      maxRetries={1}
      isolateError={true}
      {...props} 
    />
  ),
  
  // Auth boundary
  Auth: (props: Partial<ErrorBoundaryProps> = {}) => (
    <ErrorBoundary 
      context="auth"
      title="Authentication Error"
      enableRetry={false}
      {...props} 
    />
  ),
};