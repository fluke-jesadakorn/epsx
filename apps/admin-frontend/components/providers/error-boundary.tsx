'use client'

import '@/lib/polyfills';

/**
 * Error Boundary (Admin Frontend)
 * Wrapper using shared component with admin-specific fallback UI
 */

import type { ReactNode } from 'react';
import { Component } from 'react';

import { SharedErrorBoundary } from '@/shared/components/errors/error-boundary';

interface ErrorBoundaryState {
  hasError: boolean
  error?: Error
}

interface ErrorBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
}

/**
 * Admin-specific Error Boundary with admin-themed fallback
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  /**
   *
   * @param props
   */
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = { hasError: false, error: undefined }
  }

  /**
   *
   * @param error
   */
  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error }
  }

  /**
   *
   * @param error
   * @param errorInfo
   */
  override componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {

    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  /**
   *
   */
  override render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      // Admin-specific fallback UI
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
                  {this.state.error.message}
                  {'\n'}
                  {this.state.error.stack}
                </pre>
              </details>
            )}
            <div className="space-x-4">
              <button
                onClick={() => window.location.reload()}
                className="bg-yellow-500 text-gray-900 px-4 py-2 rounded hover:bg-yellow-600"
              >
                Reload Page
              </button>
              <a
                href="/auth"
                className="inline-block bg-gray-700 text-white px-4 py-2 rounded hover:bg-gray-600"
              >
                Return to Login
              </a>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

// Re-export SharedErrorBoundary for cases where the shared version is preferred
export { SharedErrorBoundary };

