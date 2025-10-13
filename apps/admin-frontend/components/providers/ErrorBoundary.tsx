'use client';

import { Component, ReactNode } from 'react';

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  _error?: Error; // Add _error for backward compatibility
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

/**
 *
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  /**
   *
   * @param props
   */
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: undefined, _error: undefined };
  }

  /**
   *
   * @param error
   */
  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error, _error: error };
  }

  /**
   *
   * @param error
   * @param errorInfo
   */
  componentDidCatch(error: Error, errorInfo: any) {
    // eslint-disable-next-line no-console
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    
    // Update state to include _error for backward compatibility
    this.setState({ _error: error });
  }

  /**
   *
   */
  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
          <div className="text-center space-y-4 max-w-md">
            <div className="text-6xl mb-4">⚠️</div>
            <h1 className="text-2xl font-bold">Something went wrong</h1>
            <p className="text-gray-300">
              An error occurred while rendering the admin interface.
            </p>
            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details className="text-left bg-gray-800 p-4 rounded text-sm">
                <summary className="cursor-pointer mb-2 text-yellow-400">
                  Error Details (Development)
                </summary>
                <pre className="whitespace-pre-wrap text-red-300">
                  {this.state._error.message}
                  {'\n'}
                  {this.state.error.stack}
                </pre>
              </details>
            )}
            <div className="space-x-4">
              <button
                onClick={() => window.location.reload()}
                className="bg-yellow-500 text-black px-4 py-2 rounded hover:bg-yellow-600"
              >
                Reload Page
              </button>
              <a
                href="/login"
                className="inline-block bg-gray-700 text-white px-4 py-2 rounded hover:bg-gray-600"
              >
                Return to Login
              </a>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}