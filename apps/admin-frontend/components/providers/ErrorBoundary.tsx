'use client'

/**
 * Error Boundary (Admin Frontend)
 * Wrapper using shared component with admin-specific fallback UI
 */

import { SharedErrorBoundary } from '@/shared/components/errors/ErrorBoundary'
import { Component, ReactNode } from 'react'

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
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = { hasError: false, error: undefined }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error }
  }

  override componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // eslint-disable-next-line no-console
    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

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
      )
    }

    return this.props.children
  }
}

// Re-export SharedErrorBoundary for cases where the shared version is preferred
export { SharedErrorBoundary }
