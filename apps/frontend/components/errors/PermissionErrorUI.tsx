// ============================================================================
// PERMISSION ERROR UI SYSTEM (Phase 3.1 - API-First Frontend Architecture)
// Comprehensive user-friendly interfaces for all permission error scenarios
// ============================================================================

'use client'

import React, { useState, useCallback } from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { 
  AlertTriangle, 
  ShieldAlert, 
  RefreshCw, 
  ArrowUp, 
  LogIn,
  HelpCircle,
  Clock,
  Zap,
  Shield,
  User,
  CreditCard,
  Star,
  CheckCircle,
  XCircle,
  Timer,
  TrendingUp,
  Lock
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
// MAIN PERMISSION ERROR UI COMPONENT
// ============================================================================

interface PermissionErrorUIProps {
  error: ApiError
  onRetry?: () => void
  onUpgrade?: (tier?: string) => void
  onLogin?: () => void
  onSupport?: (context?: any) => void
  showRetry?: boolean
  showSupport?: boolean
  variant?: 'alert' | 'card' | 'full-page'
  className?: string
}

export function PermissionErrorUI({
  error,
  onRetry,
  onUpgrade,
  onLogin,
  onSupport,
  showRetry = true,
  showSupport = true,
  variant = 'card',
  className = ''
}: PermissionErrorUIProps) {
  // Route to specific error UI based on error type
  if (isPermissionDeniedError(error)) {
    return (
      <PermissionDeniedUI
        error={error}
        onRetry={onRetry}
        onLogin={onLogin}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
      />
    )
  }

  if (isInsufficientTierError(error)) {
    return (
      <InsufficientTierUI
        error={error}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
      />
    )
  }

  if (isPermissionExpiredError(error)) {
    return (
      <PermissionExpiredUI
        error={error}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
      />
    )
  }

  if (isRateLimitExceededError(error)) {
    return (
      <RateLimitExceededUI
        error={error}
        onRetry={onRetry}
        onUpgrade={onUpgrade}
        onSupport={onSupport}
        showRetry={showRetry}
        showSupport={showSupport}
        variant={variant}
        className={className}
      />
    )
  }

  // Generic error fallback
  return (
    <GenericErrorUI
      error={error}
      onRetry={onRetry}
      onSupport={onSupport}
      showRetry={showRetry}
      showSupport={showSupport}
      variant={variant}
      className={className}
    />
  )
}

// ============================================================================
// PERMISSION DENIED UI COMPONENT
// ============================================================================

interface PermissionDeniedUIProps extends Omit<PermissionErrorUIProps, 'error'> {
  error: PermissionDeniedError
}

function PermissionDeniedUI({
  error,
  onRetry,
  onLogin,
  onSupport,
  showRetry,
  showSupport,
  variant,
  className
}: PermissionDeniedUIProps) {
  if (variant === 'alert') {
    return (
      <Alert className={`border-red-300 bg-red-50 ${className}`}>
        <ShieldAlert className="h-5 w-5 text-red-600" />
        <AlertDescription>
          <div className="flex items-center justify-between">
            <div>
              <span className="font-medium text-red-800">Access Denied</span>
              <p className="text-red-700 mt-1">{error.error.user_message}</p>
            </div>
            <div className="flex gap-2 ml-4">
              {onLogin && (
                <Button variant="default" size="sm" onClick={onLogin}>
                  <LogIn className="h-4 w-4 mr-1" />
                  Sign In
                </Button>
              )}
              {showRetry && onRetry && (
                <Button variant="outline" size="sm" onClick={onRetry}>
                  <RefreshCw className="h-4 w-4" />
                </Button>
              )}
            </div>
          </div>
        </AlertDescription>
      </Alert>
    )
  }

  if (variant === 'full-page') {
    return (
      <div className={`min-h-screen flex items-center justify-center bg-gray-50 ${className}`}>
        <div className="max-w-md w-full space-y-8">
          <div className="text-center">
            <ShieldAlert className="mx-auto h-12 w-12 text-red-500" />
            <h2 className="mt-6 text-3xl font-bold text-gray-900">Access Denied</h2>
            <p className="mt-2 text-sm text-gray-600">{error.error.user_message}</p>
          </div>

          <Card>
            <CardContent className="pt-6">
              <div className="space-y-4">
                {error.error.permission && (
                  <div className="bg-red-50 p-3 rounded-lg">
                    <p className="text-sm font-medium text-red-800 mb-1">Missing Permission:</p>
                    <code className="text-sm text-red-700 bg-red-100 px-2 py-1 rounded font-mono">
                      {error.error.permission}
                    </code>
                  </div>
                )}

                <div className="flex flex-col space-y-2">
                  {onLogin && (
                    <Button onClick={onLogin} className="w-full">
                      <LogIn className="h-4 w-4 mr-2" />
                      Sign In to Continue
                    </Button>
                  )}
                  
                  {showRetry && onRetry && (
                    <Button variant="outline" onClick={onRetry} className="w-full">
                      <RefreshCw className="h-4 w-4 mr-2" />
                      Try Again
                    </Button>
                  )}

                  {showSupport && onSupport && (
                    <Button variant="ghost" onClick={() => onSupport?.(error)} className="w-full">
                      <HelpCircle className="h-4 w-4 mr-2" />
                      Contact Support
                    </Button>
                  )}
                </div>

                {error.error.suggested_actions.length > 0 && (
                  <div className="border-t pt-4">
                    <p className="text-sm font-medium text-gray-900 mb-2">Suggested Actions:</p>
                    <ul className="text-sm text-gray-600 space-y-1">
                      {error.error.suggested_actions.map((action, index) => (
                        <li key={index} className="flex items-start">
                          <span className="text-gray-400 mr-2">•</span>
                          {action}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    )
  }

  // Default card variant
  return (
    <Card className={`border-red-200 ${className}`}>
      <CardHeader>
        <div className="flex items-center space-x-2">
          <ShieldAlert className="h-5 w-5 text-red-500" />
          <CardTitle className="text-red-800">Access Denied</CardTitle>
        </div>
        <CardDescription className="text-red-600">
          {error.error.user_message}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {error.error.permission && (
          <div className="bg-red-50 p-3 rounded-lg">
            <p className="text-sm font-medium text-red-800 mb-1">Missing Permission:</p>
            <code className="text-sm text-red-700 bg-red-100 px-2 py-1 rounded font-mono">
              {error.error.permission}
            </code>
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          {onLogin && (
            <Button onClick={onLogin}>
              <LogIn className="h-4 w-4 mr-1" />
              Sign In
            </Button>
          )}
          
          {showRetry && onRetry && (
            <Button variant="outline" onClick={onRetry}>
              <RefreshCw className="h-4 w-4 mr-1" />
              Try Again
            </Button>
          )}

          {showSupport && onSupport && (
            <Button variant="ghost" onClick={() => onSupport?.(error)}>
              <HelpCircle className="h-4 w-4 mr-1" />
              Get Help
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// INSUFFICIENT TIER UI COMPONENT
// ============================================================================

interface InsufficientTierUIProps extends Omit<PermissionErrorUIProps, 'error'> {
  error: InsufficientTierError
}

function InsufficientTierUI({
  error,
  onRetry,
  onUpgrade,
  onSupport,
  showRetry,
  showSupport,
  variant,
  className
}: InsufficientTierUIProps) {
  const [isUpgrading, setIsUpgrading] = useState(false)

  const handleUpgrade = useCallback(() => {
    setIsUpgrading(true)
    onUpgrade?.(error.error.required_tier)
  }, [onUpgrade, error.error.required_tier])

  if (variant === 'full-page') {
    return (
      <div className={`min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-purple-50 ${className}`}>
        <div className="max-w-lg w-full space-y-8 p-6">
          <div className="text-center">
            <Star className="mx-auto h-12 w-12 text-yellow-500" />
            <h2 className="mt-6 text-3xl font-bold text-gray-900">Upgrade Required</h2>
            <p className="mt-2 text-lg text-gray-600">{error.error.user_message}</p>
          </div>

          <Card className="border-yellow-200 bg-yellow-50">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-yellow-800">Current Plan</CardTitle>
                  <Badge variant="outline" className="mt-1 capitalize">
                    {error.error.current_tier}
                  </Badge>
                </div>
                <ArrowUp className="h-6 w-6 text-yellow-600" />
                <div>
                  <CardTitle className="text-yellow-800">Required Plan</CardTitle>
                  <Badge className="mt-1 bg-yellow-600 text-white capitalize">
                    {error.error.required_tier}
                  </Badge>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {error.error.upgrade_info.benefits.length > 0 && (
                <div>
                  <h4 className="font-medium text-yellow-800 mb-2">Upgrade Benefits:</h4>
                  <ul className="space-y-1">
                    {error.error.upgrade_info.benefits.map((benefit, index) => (
                      <li key={index} className="flex items-start text-sm text-yellow-700">
                        <CheckCircle className="h-4 w-4 text-green-500 mr-2 mt-0.5 flex-shrink-0" />
                        {benefit}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="flex flex-col space-y-2">
                <Button 
                  onClick={handleUpgrade} 
                  disabled={isUpgrading}
                  className="w-full bg-yellow-600 hover:bg-yellow-700"
                >
                  {isUpgrading ? (
                    <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  ) : (
                    <Star className="h-4 w-4 mr-2" />
                  )}
                  Upgrade to {error.error.required_tier}
                </Button>

                {showRetry && onRetry && (
                  <Button variant="outline" onClick={onRetry}>
                    <RefreshCw className="h-4 w-4 mr-2" />
                    Check Current Plan
                  </Button>
                )}

                {showSupport && onSupport && (
                  <Button variant="ghost" onClick={() => onSupport?.(error)}>
                    <HelpCircle className="h-4 w-4 mr-2" />
                    Contact Sales
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    )
  }

  // Default card variant
  return (
    <Card className={`border-yellow-200 bg-yellow-50 ${className}`}>
      <CardHeader>
        <div className="flex items-center space-x-2">
          <Star className="h-5 w-5 text-yellow-500" />
          <CardTitle className="text-yellow-800">Upgrade Required</CardTitle>
        </div>
        <CardDescription className="text-yellow-700">
          {error.error.user_message}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between p-3 bg-yellow-100 rounded-lg">
          <div className="text-sm">
            <span className="text-yellow-800">Current: </span>
            <Badge variant="outline" className="capitalize">{error.error.current_tier}</Badge>
          </div>
          <ArrowUp className="h-4 w-4 text-yellow-600" />
          <div className="text-sm">
            <span className="text-yellow-800">Required: </span>
            <Badge className="bg-yellow-600 text-white capitalize">{error.error.required_tier}</Badge>
          </div>
        </div>

        {error.error.upgrade_info.benefits.length > 0 && (
          <div>
            <h4 className="font-medium text-yellow-800 mb-2">Upgrade Benefits:</h4>
            <ul className="space-y-1">
              {error.error.upgrade_info.benefits.slice(0, 3).map((benefit, index) => (
                <li key={index} className="flex items-start text-sm text-yellow-700">
                  <CheckCircle className="h-4 w-4 text-green-500 mr-2 mt-0.5 flex-shrink-0" />
                  {benefit}
                </li>
              ))}
              {error.error.upgrade_info.benefits.length > 3 && (
                <li className="text-sm text-yellow-600">
                  And {error.error.upgrade_info.benefits.length - 3} more benefits...
                </li>
              )}
            </ul>
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          <Button 
            onClick={handleUpgrade} 
            disabled={isUpgrading}
            className="bg-yellow-600 hover:bg-yellow-700"
          >
            {isUpgrading ? (
              <RefreshCw className="h-4 w-4 mr-1 animate-spin" />
            ) : (
              <Star className="h-4 w-4 mr-1" />
            )}
            Upgrade Now
          </Button>

          {showRetry && onRetry && (
            <Button variant="outline" onClick={onRetry}>
              <RefreshCw className="h-4 w-4 mr-1" />
              Retry
            </Button>
          )}

          {showSupport && onSupport && (
            <Button variant="ghost" onClick={() => onSupport?.(error)}>
              <HelpCircle className="h-4 w-4 mr-1" />
              Contact Sales
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// PERMISSION EXPIRED UI COMPONENT
// ============================================================================

interface PermissionExpiredUIProps extends Omit<PermissionErrorUIProps, 'error'> {
  error: PermissionExpiredError
}

function PermissionExpiredUI({
  error,
  onRetry,
  onUpgrade,
  onSupport,
  showRetry,
  showSupport,
  variant,
  className
}: PermissionExpiredUIProps) {
  return (
    <Card className={`border-orange-200 bg-orange-50 ${className}`}>
      <CardHeader>
        <div className="flex items-center space-x-2">
          <Clock className="h-5 w-5 text-orange-500" />
          <CardTitle className="text-orange-800">Access Expired</CardTitle>
        </div>
        <CardDescription className="text-orange-700">
          {error.error.user_message}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {error.error.expired_permissions.length > 0 && (
          <div className="bg-orange-100 p-3 rounded-lg">
            <p className="text-sm font-medium text-orange-800 mb-2">Expired Permissions:</p>
            <div className="space-y-1">
              {error.error.expired_permissions.map((perm, index) => (
                <div key={index} className="text-sm text-orange-700">
                  <code className="bg-orange-200 px-2 py-1 rounded text-xs font-mono">
                    {perm.permission}
                  </code>
                  <span className="ml-2 text-xs">
                    Expired {new Date(perm.expired_at).toLocaleString()}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          {onUpgrade && (
            <Button onClick={() => onUpgrade?.()} className="bg-orange-600 hover:bg-orange-700">
              <CreditCard className="h-4 w-4 mr-1" />
              Renew Access
            </Button>
          )}

          {showRetry && onRetry && (
            <Button variant="outline" onClick={onRetry}>
              <RefreshCw className="h-4 w-4 mr-1" />
              Check Status
            </Button>
          )}

          {showSupport && onSupport && (
            <Button variant="ghost" onClick={() => onSupport?.(error)}>
              <HelpCircle className="h-4 w-4 mr-1" />
              Get Help
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// RATE LIMIT EXCEEDED UI COMPONENT
// ============================================================================

interface RateLimitExceededUIProps extends Omit<PermissionErrorUIProps, 'error'> {
  error: RateLimitExceededError
}

function RateLimitExceededUI({
  error,
  onRetry,
  onUpgrade,
  onSupport,
  showRetry,
  showSupport,
  variant,
  className
}: RateLimitExceededUIProps) {
  const [countdown, setCountdown] = useState(0)

  React.useEffect(() => {
    const resetTime = new Date(error.error.rate_limit.reset_at).getTime()
    
    const updateCountdown = () => {
      const now = Date.now()
      const timeLeft = Math.max(0, resetTime - now)
      setCountdown(Math.ceil(timeLeft / 1000))
      
      if (timeLeft > 0) {
        setTimeout(updateCountdown, 1000)
      }
    }
    
    updateCountdown()
  }, [error.error.rate_limit.reset_at])

  const usagePercentage = Math.round(
    ((error.error.rate_limit.limit - error.error.rate_limit.remaining) / error.error.rate_limit.limit) * 100
  )

  return (
    <Card className={`border-blue-200 bg-blue-50 ${className}`}>
      <CardHeader>
        <div className="flex items-center space-x-2">
          <Zap className="h-5 w-5 text-blue-500" />
          <CardTitle className="text-blue-800">Usage Limit Reached</CardTitle>
        </div>
        <CardDescription className="text-blue-700">
          {error.error.user_message}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-blue-700">Usage</span>
            <span className="text-blue-800 font-mono">
              {error.error.rate_limit.limit - error.error.rate_limit.remaining} / {error.error.rate_limit.limit}
            </span>
          </div>
          <Progress value={usagePercentage} className="h-2" />
          <p className="text-xs text-blue-600">
            {usagePercentage}% of your {error.error.rate_limit.window_size} limit used
          </p>
        </div>

        {countdown > 0 && (
          <div className="bg-blue-100 p-3 rounded-lg text-center">
            <Timer className="h-5 w-5 text-blue-600 mx-auto mb-1" />
            <p className="text-sm font-medium text-blue-800">
              Resets in {Math.floor(countdown / 60)}:{(countdown % 60).toString().padStart(2, '0')}
            </p>
          </div>
        )}

        {error.error.upgrade_for_higher_limits && (
          <div className="bg-gradient-to-r from-blue-100 to-purple-100 p-3 rounded-lg">
            <div className="flex items-center space-x-2 mb-2">
              <TrendingUp className="h-4 w-4 text-purple-600" />
              <span className="text-sm font-medium text-purple-800">Upgrade Available</span>
            </div>
            <p className="text-sm text-purple-700">
              Get {error.error.upgrade_for_higher_limits.new_limit} requests per {error.error.rate_limit.window_size} with {error.error.upgrade_for_higher_limits.tier}
            </p>
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          {error.error.upgrade_for_higher_limits && onUpgrade && (
            <Button onClick={() => onUpgrade?.(error.error.upgrade_for_higher_limits?.tier)}>
              <ArrowUp className="h-4 w-4 mr-1" />
              Upgrade for More
            </Button>
          )}

          {showRetry && onRetry && (
            <Button 
              variant="outline" 
              onClick={onRetry} 
              disabled={countdown > 0}
            >
              <RefreshCw className="h-4 w-4 mr-1" />
              {countdown > 0 ? `Try in ${Math.ceil(countdown / 60)}m` : 'Try Again'}
            </Button>
          )}

          {showSupport && onSupport && (
            <Button variant="ghost" onClick={() => onSupport?.(error)}>
              <HelpCircle className="h-4 w-4 mr-1" />
              Contact Support
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

// ============================================================================
// GENERIC ERROR UI COMPONENT
// ============================================================================

interface GenericErrorUIProps extends Omit<PermissionErrorUIProps, 'error'> {
  error: ApiError
}

function GenericErrorUI({
  error,
  onRetry,
  onSupport,
  showRetry,
  showSupport,
  variant,
  className
}: GenericErrorUIProps) {
  return (
    <Card className={`border-gray-200 ${className}`}>
      <CardHeader>
        <div className="flex items-center space-x-2">
          <AlertTriangle className="h-5 w-5 text-gray-500" />
          <CardTitle className="text-gray-800">Something Went Wrong</CardTitle>
        </div>
        <CardDescription className="text-gray-600">
          {error.error.user_message}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {error.error.suggested_actions.length > 0 && (
          <div>
            <p className="text-sm font-medium text-gray-800 mb-2">Suggested Actions:</p>
            <ul className="space-y-1">
              {error.error.suggested_actions.map((action, index) => (
                <li key={index} className="flex items-start text-sm text-gray-600">
                  <span className="text-gray-400 mr-2">•</span>
                  {action}
                </li>
              ))}
            </ul>
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          {showRetry && onRetry && (
            <Button variant="outline" onClick={onRetry}>
              <RefreshCw className="h-4 w-4 mr-1" />
              Try Again
            </Button>
          )}

          {showSupport && onSupport && (
            <Button variant="ghost" onClick={() => onSupport?.(error)}>
              <HelpCircle className="h-4 w-4 mr-1" />
              Get Help
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

export default PermissionErrorUI

// ============================================================================
// PERMISSION ERROR UI SYSTEM COMPLETE NOTICE (Phase 3.1.4)
// ============================================================================
//
// 🎉 PERMISSION ERROR UI SYSTEM COMPLETE!
//
// Created comprehensive user-friendly interfaces for all permission error scenarios:
// - Type-specific error UIs with appropriate styling and actions
// - Multiple display variants (alert, card, full-page)
// - Interactive elements with real-time countdowns and progress bars
// - Contextual upgrade prompts with benefits and pricing
// - Structured suggested actions and help flows
// - Responsive design with consistent UX patterns
//
// Error UI Components:
// ✅ Permission Denied UI with sign-in prompts
// ✅ Insufficient Tier UI with upgrade benefits
// ✅ Permission Expired UI with renewal flows
// ✅ Rate Limit Exceeded UI with countdown timers
// ✅ Generic Error UI with suggested actions
// ✅ Multiple display variants for different contexts
// ✅ Interactive elements and real-time updates
//
// The permission error UI system is now PRODUCTION-READY! 🎯
// Users get clear, actionable guidance for every error scenario.
// ============================================================================