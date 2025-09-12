'use client';

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { AlertTriangle, RefreshCw, Home, Bug } from 'lucide-react';
import { safeError, uiLogger } from '@/lib/logger';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  level?: 'global' | 'page' | 'component' | 'feature';
  context?: string;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorId: string;
}

/**
 * Global Error Boundary with comprehensive error handling and reporting
 * Provides different error UIs based on the boundary level (global, page, component, feature)
 */
export class GlobalErrorBoundary extends Component<Props, State> {
  public constructor(props: Props) {
    super(props);
    
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
    };
  }

  public static getDerivedStateFromError(error: Error): Partial<State> {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
      errorId: `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const { onError, level = 'global', context } = this.props;
    
    // Log error with context
    const errorContext = {
      level,
      context,
      errorId: this.state.errorId,
      componentStack: errorInfo.componentStack,
      errorBoundary: this.constructor.name,
    };

    safeError(`Error caught by ${level} boundary`, error);
    uiLogger.error('Component error boundary triggered', error, errorContext);

    // Call custom error handler if provided
    if (onError) {
      try {
        onError(error, errorInfo);
      } catch (handlerError) {
        uiLogger.error('Error handler failed', handlerError);
      }
    }

    // Report to external error tracking service in production
    if (typeof window !== 'undefined' && process.env.NODE_ENV === 'production') {
      // Here you could integrate with Sentry, LogRocket, etc.
      this.reportErrorToService(error, errorInfo, errorContext);
    }

    this.setState({ errorInfo });
  }

  private reportErrorToService(error: Error, errorInfo: ErrorInfo, context: any) {
    // Placeholder for external error reporting service
    // Example: Sentry.captureException(error, { contexts: { errorBoundary: context } });
    uiLogger.info('Error reported to external service', { error: error.message, context });
  }

  private handleRetry = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
    });
  };

  private handleReload = () => {
    window.location.reload();
  };

  private handleGoHome = () => {
    window.location.href = '/';
  };

  private renderGlobalError() {
    const { error, errorId } = this.state;
    
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <Card className="max-w-md w-full mx-4 p-8">
          <div className="text-center">
            <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full bg-red-100 dark:bg-red-900/20">
              <AlertTriangle className="h-8 w-8 text-red-500" />
            </div>
            
            <h1 className="mb-4 text-2xl font-bold text-gray-900 dark:text-white">
              Something went wrong
            </h1>
            
            <p className="mb-6 text-gray-600 dark:text-gray-300">
              We encountered an unexpected error. Our team has been notified and is working on a fix.
            </p>
            
            {process.env.NODE_ENV === 'development' && error && (
              <details className="mb-6 text-left">
                <summary className="cursor-pointer text-sm text-gray-500 hover:text-gray-700">
                  Technical Details
                </summary>
                <pre className="mt-2 overflow-auto rounded bg-gray-100 p-4 text-xs dark:bg-gray-800">
                  {error.message}
                  {'\n\n'}
                  {error.stack}
                </pre>
              </details>
            )}
            
            <div className="space-y-3">
              <Button onClick={this.handleReload} className="w-full">
                <RefreshCw className="mr-2 h-4 w-4" />
                Reload Page
              </Button>
              
              <Button 
                onClick={this.handleGoHome} 
                variant="outline" 
                className="w-full"
              >
                <Home className="mr-2 h-4 w-4" />
                Go to Home
              </Button>
            </div>
            
            <p className="mt-4 text-xs text-gray-500">
              Error ID: {errorId}
            </p>
          </div>
        </Card>
      </div>
    );
  }

  private renderPageError() {
    const { error, errorId } = this.state;
    
    return (
      <div className="flex min-h-[400px] items-center justify-center">
        <Card className="max-w-sm w-full mx-4 p-6">
          <div className="text-center">
            <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-orange-100 dark:bg-orange-900/20">
              <Bug className="h-6 w-6 text-orange-500" />
            </div>
            
            <h2 className="mb-3 text-lg font-semibold text-gray-900 dark:text-white">
              Page Error
            </h2>
            
            <p className="mb-4 text-sm text-gray-600 dark:text-gray-300">
              This page encountered an error and couldn't load properly.
            </p>
            
            <div className="space-y-2">
              <Button onClick={this.handleRetry} size="sm" className="w-full">
                <RefreshCw className="mr-2 h-4 w-4" />
                Try Again
              </Button>
              
              <Button 
                onClick={this.handleGoHome} 
                variant="outline" 
                size="sm" 
                className="w-full"
              >
                <Home className="mr-2 h-4 w-4" />
                Go Home
              </Button>
            </div>
            
            {process.env.NODE_ENV === 'development' && (
              <p className="mt-3 text-xs text-gray-400">
                Error: {error?.message} | ID: {errorId}
              </p>
            )}
          </div>
        </Card>
      </div>
    );
  }

  private renderComponentError() {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/10">
        <div className="flex items-center">
          <AlertTriangle className="h-5 w-5 text-red-500 mr-3" />
          <div className="flex-1">
            <h3 className="text-sm font-medium text-red-800 dark:text-red-200">
              Component Error
            </h3>
            <p className="text-xs text-red-600 dark:text-red-300 mt-1">
              This component failed to render. Please refresh or try again.
            </p>
          </div>
          <Button 
            onClick={this.handleRetry} 
            size="sm" 
            variant="outline"
            className="border-red-300 text-red-700 hover:bg-red-100"
          >
            Retry
          </Button>
        </div>
      </div>
    );
  }

  private renderFeatureError() {
    const { context } = this.props;
    
    return (
      <div className="rounded-lg border border-yellow-200 bg-yellow-50 p-4 dark:border-yellow-800 dark:bg-yellow-900/10">
        <div className="flex items-center">
          <Bug className="h-5 w-5 text-yellow-500 mr-3" />
          <div className="flex-1">
            <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
              {context ? `${context} Error` : 'Feature Error'}
            </h3>
            <p className="text-xs text-yellow-600 dark:text-yellow-300 mt-1">
              This feature is temporarily unavailable.
            </p>
          </div>
          <Button 
            onClick={this.handleRetry} 
            size="sm" 
            variant="outline"
            className="border-yellow-300 text-yellow-700 hover:bg-yellow-100"
          >
            Retry
          </Button>
        </div>
      </div>
    );
  }

  public render() {
    const { hasError } = this.state;
    const { children, fallback, level = 'global' } = this.props;

    if (hasError) {
      // If custom fallback is provided, use it
      if (fallback) {
        return fallback;
      }

      // Otherwise, render appropriate error UI based on level
      switch (level) {
        case 'global':
          return this.renderGlobalError();
        case 'page':
          return this.renderPageError();
        case 'component':
          return this.renderComponentError();
        case 'feature':
          return this.renderFeatureError();
        default:
          return this.renderGlobalError();
      }
    }

    return children;
  }
}

// Convenience components for different levels
export const PageErrorBoundary: React.FC<Omit<Props, 'level'>> = (props) => (
  <GlobalErrorBoundary {...props} level="page" />
);

export const ComponentErrorBoundary: React.FC<Omit<Props, 'level'>> = (props) => (
  <GlobalErrorBoundary {...props} level="component" />
);

export const FeatureErrorBoundary: React.FC<Omit<Props, 'level'>> = (props) => (
  <GlobalErrorBoundary {...props} level="feature" />
);

// Hook to trigger error boundaries for testing
export function useErrorBoundaryTrigger() {
  return {
    triggerError: (message = 'Test error') => {
      throw new Error(message);
    }
  };
}