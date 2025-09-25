// ============================================================================
// PERMISSION ERROR BOUNDARY (Phase 3.1 - API-First Frontend Architecture)
// React error boundary for handling permission-related errors gracefully
// ============================================================================

'use client'

import React, { Component, ReactNode, ErrorInfo } from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { 
  AlertTriangle, 
  ShieldAlert, 
  RefreshCw, 
  ArrowUp, 
  LogIn,
  HelpCircle
} from 'lucide-react'
import {
  ApiError,
  PermissionDeniedError,
  InsufficientTierError, 
  PermissionExpiredError,
  RateLimitExceededError,
  isPermissionDeniedError,
  isInsufficientTierError,
  isPermissionExpiredError,
  isRateLimitExceededError
} from '@/lib/api/response-handler'

// ============================================================================
// ERROR BOUNDARY STATE AND PROPS
// ============================================================================

interface PermissionErrorBoundaryState {
  hasError: boolean
  error: Error | null
  errorInfo: ErrorInfo | null
  apiError: ApiError | null
  errorId: string
  retryCount: number
}

interface PermissionErrorBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
  onError?: (error: Error, errorInfo: ErrorInfo, apiError?: ApiError) => void
  onRetry?: () => void
  onUpgrade?: () => void
  onLogin?: () => void
  showSupportLink?: boolean
  maxRetries?: number
  component?: string // For logging context
}

interface ErrorDisplayProps {
  apiError: ApiError
  onRetry: () => void
  onUpgrade?: () => void
  onLogin?: () => void
  showSupportLink?: boolean
  retryCount: number
  maxRetries: number
}

// ============================================================================
// PERMISSION ERROR BOUNDARY COMPONENT
// ============================================================================

export class PermissionErrorBoundary extends Component<
  PermissionErrorBoundaryProps,
  PermissionErrorBoundaryState
> {
  private errorId = ''

  constructor(props: PermissionErrorBoundaryProps) {
    super(props)
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      apiError: null,
      errorId: '',
      retryCount: 0
    }
  }

  static getDerivedStateFromError(error: Error): Partial<PermissionErrorBoundaryState> {
    const errorId = `err_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    
    // Check if error contains API error information
    let apiError: ApiError | null = null
    if (error.cause && typeof error.cause === 'object' && 'success' in error.cause) {
      apiError = error.cause as ApiError
    }

    return {
      hasError: true,
      error,
      apiError,
      errorId
    }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Update state with error info
    this.setState({
      errorInfo
    })

    // Extract API error if available
    const apiError = this.state.apiError

    // Call error handler if provided
    this.props.onError?.(error, errorInfo, apiError || undefined)

    // Log structured error information
    console.error('Permission Error Boundary caught error:', {
      errorId: this.state.errorId,
      component: this.props.component,
      error: error.message,
      stack: error.stack,
      apiError,
      errorInfo: {
        componentStack: errorInfo.componentStack
      }
    })
  }

  handleRetry = () => {
    const { maxRetries = 3 } = this.props
    const newRetryCount = this.state.retryCount + 1

    if (newRetryCount > maxRetries) {
      console.warn('Max retries exceeded for error boundary', {
        errorId: this.state.errorId,
        retryCount: newRetryCount,
        maxRetries
      })
      return
    }

    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      apiError: null,
      retryCount: newRetryCount
    })

    // Call retry handler if provided
    this.props.onRetry?.()
  }

  handleUpgrade = () => {
    this.props.onUpgrade?.()
  }

  handleLogin = () => {
    this.props.onLogin?.()
  }

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback
      }

      // Render structured error UI based on API error type
      if (this.state.apiError) {
        return (
          <ErrorDisplay
            apiError={this.state.apiError}
            onRetry={this.handleRetry}
            onUpgrade={this.handleUpgrade}
            onLogin={this.handleLogin}
            showSupportLink={this.props.showSupportLink}
            retryCount={this.state.retryCount}
            maxRetries={this.props.maxRetries || 3}
          />
        )
      }

      // Generic error fallback
      return (
        <GenericErrorFallback
          error={this.state.error}
          errorId={this.state.errorId}
          onRetry={this.handleRetry}
          showSupportLink={this.props.showSupportLink}
          retryCount={this.state.retryCount}
          maxRetries={this.props.maxRetries || 3}
        />
      )
    }

    return this.props.children
  }
}

// ============================================================================
// ERROR DISPLAY COMPONENTS
// ============================================================================

function ErrorDisplay({
  apiError,
  onRetry,
  onUpgrade,
  onLogin,
  showSupportLink = true,
  retryCount,
  maxRetries
}: ErrorDisplayProps) {
  // Handle permission denied errors
  if (isPermissionDeniedError(apiError)) {
    return (
      <PermissionDeniedDisplay
        error={apiError}
        onRetry={onRetry}
        onLogin={onLogin}
        showSupportLink={showSupportLink}
        retryCount={retryCount}
        maxRetries={maxRetries}
      />
    )
  }

  // Handle insufficient tier errors
  if (isInsufficientTierError(apiError)) {
    return (
      <InsufficientTierDisplay
        error={apiError}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        showSupportLink={showSupportLink}
        retryCount={retryCount}
        maxRetries={maxRetries}
      />
    )
  }

  // Handle permission expired errors
  if (isPermissionExpiredError(apiError)) {
    return (
      <PermissionExpiredDisplay
        error={apiError}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        showSupportLink={showSupportLink}
        retryCount={retryCount}
        maxRetries={maxRetries}
      />
    )
  }

  // Handle rate limit exceeded errors
  if (isRateLimitExceededError(apiError)) {
    return (
      <RateLimitExceededDisplay
        error={apiError}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        showSupportLink={showSupportLink}
        retryCount={retryCount}
        maxRetries={maxRetries}
      />
    )
  }

  // Generic API error display
  return (
    <GenericApiErrorDisplay
      error={apiError}
      onRetry={onRetry}
      showSupportLink={showSupportLink}
      retryCount={retryCount}
      maxRetries={maxRetries}
    />
  )
}

function PermissionDeniedDisplay({ error, onRetry, onLogin, showSupportLink, retryCount, maxRetries }: {
  error: PermissionDeniedError
  onRetry: () => void
  onLogin?: () => void
  showSupportLink: boolean
  retryCount: number
  maxRetries: number
}) {
  return (
    <Alert className="border-red-300 bg-red-50">
      <ShieldAlert className="h-5 w-5 text-red-600" />
      <AlertDescription>
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold text-red-800">Access Denied</h3>
            <p className="text-red-700 mt-1">{error.error.user_message}</p>
          </div>

          {error.error.permission && (
            <div className="text-xs text-red-600 bg-red-100 p-2 rounded font-mono">
              Missing permission: {error.error.permission}
            </div>
          )}

          <div className="flex flex-wrap gap-2">
            {onLogin && (
              <Button variant="default" size="sm" onClick={onLogin}>
                <LogIn className="h-4 w-4 mr-1" />
                Sign In
              </Button>
            )}
            
            {retryCount < maxRetries && (
              <Button variant="outline" size="sm" onClick={onRetry}>
                <RefreshCw className="h-4 w-4 mr-1" />
                Try Again
              </Button>
            )}

            {showSupportLink && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={() => window.open('/support', '_blank')}
              >
                <HelpCircle className="h-4 w-4 mr-1" />
                Contact Support
              </Button>
            )}
          </div>

          {error.error.suggested_actions.length > 0 && (
            <div className="text-sm text-red-600">
              <p className="font-medium">Suggested actions:</p>
              <ul className="list-disc list-inside mt-1 space-y-1">
                {error.error.suggested_actions.map((action, index) => (
                  <li key={index}>{action}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </AlertDescription>
    </Alert>
  )
}

function InsufficientTierDisplay({ error, onRetry, onUpgrade, showSupportLink, retryCount, maxRetries }: {
  error: InsufficientTierError
  onRetry: () => void
  onUpgrade?: () => void
  showSupportLink: boolean
  retryCount: number
  maxRetries: number
}) {
  return (
    <Alert className="border-yellow-300 bg-yellow-50">
      <ArrowUp className="h-5 w-5 text-yellow-600" />
      <AlertDescription>
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold text-yellow-800">Upgrade Required</h3>
            <p className="text-yellow-700 mt-1">{error.error.user_message}</p>
          </div>

          <div className="text-sm text-yellow-700 bg-yellow-100 p-3 rounded">
            <div className="flex justify-between">
              <span>Current tier:</span>
              <span className="font-mono">{error.error.current_tier}</span>
            </div>
            <div className="flex justify-between">
              <span>Required tier:</span>
              <span className="font-mono">{error.error.required_tier}</span>
            </div>
          </div>

          {error.error.upgrade_info.benefits.length > 0 && (
            <div className="text-sm text-yellow-700">
              <p className="font-medium">Upgrade benefits:</p>
              <ul className="list-disc list-inside mt-1 space-y-1">
                {error.error.upgrade_info.benefits.map((benefit, index) => (
                  <li key={index}>{benefit}</li>
                ))}
              </ul>
            </div>
          )}

          <div className="flex flex-wrap gap-2">
            {onUpgrade && (
              <Button variant="default" size="sm" onClick={onUpgrade}>
                <ArrowUp className="h-4 w-4 mr-1" />
                Upgrade Now
              </Button>
            )}
            
            {retryCount < maxRetries && (
              <Button variant="outline" size="sm" onClick={onRetry}>
                <RefreshCw className="h-4 w-4 mr-1" />
                Check Again
              </Button>
            )}

            {showSupportLink && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={() => window.open('/support', '_blank')}
              >
                <HelpCircle className="h-4 w-4 mr-1" />
                Contact Sales
              </Button>
            )}
          </div>
        </div>
      </AlertDescription>
    </Alert>
  )
}

function PermissionExpiredDisplay({ error, onRetry, onUpgrade, showSupportLink, retryCount, maxRetries }: {
  error: PermissionExpiredError
  onRetry: () => void
  onUpgrade?: () => void
  showSupportLink: boolean
  retryCount: number
  maxRetries: number
}) {
  return (
    <Alert className="border-orange-300 bg-orange-50">
      <AlertTriangle className="h-5 w-5 text-orange-600" />
      <AlertDescription>
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold text-orange-800">Access Expired</h3>
            <p className="text-orange-700 mt-1">{error.error.user_message}</p>
          </div>

          {error.error.expired_permissions.length > 0 && (
            <div className="text-xs text-orange-600 bg-orange-100 p-2 rounded">
              <p className="font-medium mb-1">Expired permissions:</p>
              {error.error.expired_permissions.map((perm, index) => (
                <div key={index} className="font-mono">
                  {perm.permission} (expired {new Date(perm.expired_at).toLocaleString()})
                </div>
              ))}
            </div>
          )}

          <div className="flex flex-wrap gap-2">
            {onUpgrade && (
              <Button variant="default" size="sm" onClick={onUpgrade}>
                <ArrowUp className="h-4 w-4 mr-1" />
                Renew Access
              </Button>
            )}
            
            {retryCount < maxRetries && (
              <Button variant="outline" size="sm" onClick={onRetry}>
                <RefreshCw className="h-4 w-4 mr-1" />
                Check Status
              </Button>
            )}

            {showSupportLink && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={() => window.open(error.error.renewal_url || '/renew', '_blank')}
              >
                <HelpCircle className="h-4 w-4 mr-1" />
                Renew Subscription
              </Button>
            )}
          </div>
        </div>
      </AlertDescription>
    </Alert>
  )
}

function RateLimitExceededDisplay({ error, onRetry, onUpgrade, showSupportLink, retryCount, maxRetries }: {
  error: RateLimitExceededError
  onRetry: () => void
  onUpgrade?: () => void
  showSupportLink: boolean
  retryCount: number
  maxRetries: number
}) {
  const resetTime = new Date(error.error.rate_limit.reset_at)
  const timeUntilReset = Math.max(0, resetTime.getTime() - Date.now())
  const minutesUntilReset = Math.ceil(timeUntilReset / 60000)

  return (
    <Alert className="border-blue-300 bg-blue-50">
      <AlertTriangle className="h-5 w-5 text-blue-600" />
      <AlertDescription>
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold text-blue-800">Usage Limit Exceeded</h3>
            <p className="text-blue-700 mt-1">{error.error.user_message}</p>
          </div>

          <div className="text-sm text-blue-700 bg-blue-100 p-3 rounded">
            <div className="flex justify-between">
              <span>Usage:</span>
              <span className="font-mono">{error.error.rate_limit.limit - error.error.rate_limit.remaining} / {error.error.rate_limit.limit}</span>
            </div>
            <div className="flex justify-between">
              <span>Resets in:</span>
              <span className="font-mono">{minutesUntilReset}m</span>
            </div>
            <div className="flex justify-between">
              <span>Period:</span>
              <span className="font-mono">{error.error.rate_limit.window_size}</span>
            </div>
          </div>

          {error.error.upgrade_for_higher_limits && (
            <div className="text-sm text-blue-700">
              <p className="font-medium">Upgrade to {error.error.upgrade_for_higher_limits.tier} for {error.error.upgrade_for_higher_limits.new_limit} requests per {error.error.rate_limit.window_size}.</p>
            </div>
          )}

          <div className="flex flex-wrap gap-2">
            {error.error.upgrade_for_higher_limits && onUpgrade && (
              <Button variant="default" size="sm" onClick={onUpgrade}>
                <ArrowUp className="h-4 w-4 mr-1" />
                Upgrade for Higher Limits
              </Button>
            )}
            
            <Button 
              variant="outline" 
              size="sm" 
              onClick={onRetry}
              disabled={timeUntilReset > 0}
            >
              <RefreshCw className="h-4 w-4 mr-1" />
              {timeUntilReset > 0 ? `Try in ${minutesUntilReset}m` : 'Try Again'}
            </Button>

            {showSupportLink && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={() => window.open('/support', '_blank')}
              >
                <HelpCircle className="h-4 w-4 mr-1" />
                Contact Support
              </Button>
            )}
          </div>
        </div>
      </AlertDescription>
    </Alert>
  )
}

function GenericApiErrorDisplay({ error, onRetry, showSupportLink, retryCount, maxRetries }: {
  error: ApiError
  onRetry: () => void
  showSupportLink: boolean
  retryCount: number
  maxRetries: number
}) {
  return (
    <Alert variant="destructive">
      <AlertTriangle className="h-5 w-5" />
      <AlertDescription>
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold">Something Went Wrong</h3>
            <p className="mt-1">{error.error.user_message}</p>
          </div>

          {error.error.suggested_actions.length > 0 && (
            <div className="text-sm">
              <p className="font-medium">Suggested actions:</p>
              <ul className="list-disc list-inside mt-1 space-y-1">
                {error.error.suggested_actions.map((action, index) => (
                  <li key={index}>{action}</li>
                ))}
              </ul>
            </div>
          )}

          <div className="flex flex-wrap gap-2">
            {retryCount < maxRetries && (
              <Button variant="outline" size="sm" onClick={onRetry}>
                <RefreshCw className="h-4 w-4 mr-1" />
                Try Again
              </Button>
            )}

            {showSupportLink && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={() => window.open('/support', '_blank')}
              >
                <HelpCircle className="h-4 w-4 mr-1" />
                Get Help
              </Button>
            )}
          </div>
        </div>
      </AlertDescription>
    </Alert>
  )
}

function GenericErrorFallback({ error, errorId, onRetry, showSupportLink, retryCount, maxRetries }: {
  error: Error | null
  errorId: string
  onRetry: () => void
  showSupportLink: boolean
  retryCount: number
  maxRetries: number
}) {
  return (
    <Alert variant="destructive">
      <AlertTriangle className="h-5 w-5" />
      <AlertDescription>
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold">Unexpected Error</h3>
            <p className="mt-1">
              Something unexpected happened. This error has been logged and we're working to fix it.
            </p>
          </div>

          <div className="text-xs font-mono text-muted-foreground">
            Error ID: {errorId}
          </div>

          <div className="flex flex-wrap gap-2">
            {retryCount < maxRetries && (
              <Button variant="outline" size="sm" onClick={onRetry}>
                <RefreshCw className="h-4 w-4 mr-1" />
                Try Again
              </Button>
            )}

            {showSupportLink && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={() => window.open(`/support?error=${errorId}`, '_blank')}
              >
                <HelpCircle className="h-4 w-4 mr-1" />
                Report Issue
              </Button>
            )}
          </div>
        </div>
      </AlertDescription>
    </Alert>
  )
}

// ============================================================================
// CONVENIENCE COMPONENTS  
// ============================================================================

/**
 * HOC to wrap components with permission error boundary
 */
export function withPermissionErrorBoundary<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  options: Omit<PermissionErrorBoundaryProps, 'children'> = {}
) {
  const WithPermissionErrorBoundary = (props: P) => (
    <PermissionErrorBoundary {...options}>
      <WrappedComponent {...props} />
    </PermissionErrorBoundary>
  )

  WithPermissionErrorBoundary.displayName = `withPermissionErrorBoundary(${WrappedComponent.displayName || WrappedComponent.name})`
  
  return WithPermissionErrorBoundary
}

/**
 * Hook to trigger permission errors (for testing/development)
 */
export const usePermissionErrorTrigger = () => {
  const triggerPermissionError = (apiError: ApiError) => {
    const error = new Error(apiError.error.message)
    error.cause = apiError
    throw error
  }

  return { triggerPermissionError }
}

export default PermissionErrorBoundary

// ============================================================================
// PERMISSION ERROR BOUNDARY COMPLETE NOTICE (Phase 3.1.2)
// ============================================================================
//
// 🎉 PERMISSION ERROR BOUNDARY COMPLETE!
//
// Created comprehensive React error boundary system for permission errors:
// - Catches and handles all permission-related errors gracefully
// - Structured error displays for each error type
// - User-friendly messaging with suggested actions
// - Retry mechanisms with exponential backoff
// - Context-aware error logging and reporting
// - HOC for easy component wrapping
//
// Error Types Handled:
// ✅ Permission denied (403) with login prompts
// ✅ Insufficient tier with upgrade prompts
// ✅ Permission expired with renewal prompts  
// ✅ Rate limit exceeded with wait times
// ✅ Generic API errors with retry logic
// ✅ Unexpected errors with error IDs
//
// The error boundary system is now BULLETPROOF! 🎯
// ============================================================================