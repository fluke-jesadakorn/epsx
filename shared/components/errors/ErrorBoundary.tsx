'use client'

/**
 * SHARED Error Boundary
 * Generic error boundary with customizable fallback UI
 * 
 * Single source of truth for handling React errors
 */

import { Component, ReactNode } from 'react'

// ============================================================================
// TYPES
// ============================================================================

export interface ErrorBoundaryState {
    hasError: boolean
    error?: Error
}

export interface ErrorBoundaryProps {
    children: ReactNode
    fallback?: ReactNode | ((error: Error) => ReactNode)
    onError?: (error: Error, errorInfo: React.ErrorInfo) => void
}

// ============================================================================
// SHARED ERROR BOUNDARY
// ============================================================================

export class SharedErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props)
        this.state = { hasError: false, error: undefined }
    }

    static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error }
    }

    override componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
         
        console.error('ErrorBoundary caught an error:', error, errorInfo)

        if (this.props.onError) {
            this.props.onError(error, errorInfo)
        }
    }

    handleReset = () => {
        this.setState({ hasError: false, error: undefined })
    }

    override render() {
        if (this.state.hasError && this.state.error) {
            // Custom fallback provided
            if (this.props.fallback) {
                if (typeof this.props.fallback === 'function') {
                    return this.props.fallback(this.state.error)
                }
                return this.props.fallback
            }

            // Default fallback UI
            return (
                <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
                    <div className="text-center space-y-4 max-w-md">
                        <div className="text-6xl mb-4">⚠️</div>
                        <h1 className="text-2xl font-bold">Something went wrong</h1>
                        <p className="text-gray-300">
                            An error occurred while rendering the application.
                        </p>
                        {process.env.NODE_ENV === 'development' && (
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
                        </div>
                    </div>
                </div>
            )
        }

        return this.props.children
    }
}
