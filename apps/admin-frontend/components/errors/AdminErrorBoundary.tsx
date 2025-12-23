'use client';

import React, { Component, ReactNode } from 'react';

import { PermissionErrorUI } from './PermissionErrorUI';

import { 
  ApiError,
  isPermissionDeniedError,
  isInsufficientTierError,
  isPermissionExpiredError,
  isRateLimitExceededError
} from '@/shared/utils/response-handler';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: any) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  apiError: ApiError | null;
}

/**
 * Error boundary for handling backend permission errors in admin components
 * Displays appropriate error UI for different error types
 */
export class AdminErrorBoundary extends Component<Props, State> {
  /**
   *
   * @param props
   */
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      apiError: null
    };
  }

  /**
   *
   * @param error
   */
  static getDerivedStateFromError(error: Error): State {
    // Check if this is an API error with permission information
    let apiError: ApiError | null = null;
    
    if (error && typeof error === 'object' && 'error' in error) {
      apiError = error as unknown as ApiError;
    }

    return {
      hasError: true,
      error,
      apiError
    };
  }

  /**
   *
   * @param error
   * @param errorInfo
   */
  override componentDidCatch(error: Error, errorInfo: any) {
    // eslint-disable-next-line no-console
    console.error('AdminErrorBoundary caught an error:', error, errorInfo);
    
    // Call optional error handler
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  handleRetry = () => {
    this.setState({
      hasError: false,
      error: null,
      apiError: null
    });
  };

  handleLogin = () => {
    window.location.href = '/login';
  };

  handleSupport = (context?: any) => {
    // Log support request or redirect to support
    // Could implement support ticket creation or redirect
  };

  /**
   *
   */
  override render() {
    if (this.state.hasError) {
      // If we have a structured API error, use the permission error UI
      if (this.state.apiError) {
        return (
          <PermissionErrorUI
            error={this.state.apiError}
            onRetry={this.handleRetry}
            onLogin={this.handleLogin}
            onSupport={this.handleSupport}
            variant="card"
            showRetry={true}
            showSupport={true}
          />
        );
      }

      // For other errors, use fallback or generic error display
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Generic error fallback
      return (
        <div className="p-6 bg-red-50 border border-red-200 rounded-lg">
          <div className="flex items-center space-x-2 mb-4">
            <div className="w-5 h-5 bg-red-500 rounded-full"></div>
            <h3 className="text-lg font-semibold text-red-800">Something went wrong</h3>
          </div>
          <p className="text-red-700 mb-4">
            An unexpected error occurred. Please try refreshing the page.
          </p>
          <div className="flex space-x-2">
            <button
              onClick={this.handleRetry}
              className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
            >
              Try Again
            </button>
            <button
              onClick={() => window.location.reload()}
              className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
            >
              Refresh Page
            </button>
          </div>
          {process.env.NODE_ENV === 'development' && (
            <details className="mt-4 p-3 bg-red-100 rounded">
              <summary className="cursor-pointer text-red-800 font-medium">
                Error Details (Development)
              </summary>
              <pre className="mt-2 text-xs text-red-700 overflow-auto">
                {this.state.error?.toString()}
              </pre>
            </details>
          )}
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * HOC to wrap components with admin error boundary
 * @param Component
 * @param errorBoundaryProps
 */
export function withAdminErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<Props, 'children'>
) {
  return function WrappedComponent(props: P) {
    return (
      <AdminErrorBoundary {...errorBoundaryProps}>
        <Component {...props} />
      </AdminErrorBoundary>
    );
  };
}

export default AdminErrorBoundary;