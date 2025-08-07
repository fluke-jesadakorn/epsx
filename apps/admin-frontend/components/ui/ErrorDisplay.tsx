import React from 'react';
import { AlertTriangle, RefreshCw, Home, LogOut } from 'lucide-react';
import { Button } from '@epsx/ui';

export interface ErrorDisplayProps {
  error: string | Error | null;
  context?: 'loading' | 'permission' | 'network' | 'server' | 'auth' | 'validation';
  title?: string;
  onRetry?: () => void;
  onGoBack?: () => void;
  onSignOut?: () => void;
  showDetails?: boolean;
  className?: string;
}

export function ErrorDisplay({
  error,
  context = 'server',
  title,
  onRetry,
  onGoBack,
  onSignOut,
  showDetails = false,
  className = ''
}: ErrorDisplayProps) {
  if (!error) return null;

  const errorMessage = typeof error === 'string' ? error : error.message || 'An unexpected error occurred';
  const errorStack = typeof error === 'object' && error?.stack ? error.stack : undefined;

  // Context-specific styling and content
  const getContextConfig = () => {
    switch (context) {
      case 'auth':
        return {
          title: title || 'Authentication Required',
          message: 'Please sign in to continue accessing this feature.',
          icon: <LogOut className="h-8 w-8" />,
          bgColor: 'bg-blue-50 dark:bg-blue-950/50',
          borderColor: 'border-blue-200 dark:border-blue-800',
          iconColor: 'text-blue-600 dark:text-blue-400',
          textColor: 'text-blue-900 dark:text-blue-100'
        };
      case 'permission':
        return {
          title: title || 'Access Denied',
          message: 'You don\'t have permission to access this resource.',
          icon: <AlertTriangle className="h-8 w-8" />,
          bgColor: 'bg-amber-50 dark:bg-amber-950/50',
          borderColor: 'border-amber-200 dark:border-amber-800',
          iconColor: 'text-amber-600 dark:text-amber-400',
          textColor: 'text-amber-900 dark:text-amber-100'
        };
      case 'network':
        return {
          title: title || 'Connection Error',
          message: 'Unable to connect to the server. Please check your internet connection.',
          icon: <RefreshCw className="h-8 w-8" />,
          bgColor: 'bg-orange-50 dark:bg-orange-950/50',
          borderColor: 'border-orange-200 dark:border-orange-800',
          iconColor: 'text-orange-600 dark:text-orange-400',
          textColor: 'text-orange-900 dark:text-orange-100'
        };
      case 'validation':
        return {
          title: title || 'Invalid Data',
          message: 'Please check your input and try again.',
          icon: <AlertTriangle className="h-8 w-8" />,
          bgColor: 'bg-yellow-50 dark:bg-yellow-950/50',
          borderColor: 'border-yellow-200 dark:border-yellow-800',
          iconColor: 'text-yellow-600 dark:text-yellow-400',
          textColor: 'text-yellow-900 dark:text-yellow-100'
        };
      case 'loading':
        return {
          title: title || 'Loading Error',
          message: 'Failed to load the requested data.',
          icon: <RefreshCw className="h-8 w-8" />,
          bgColor: 'bg-gray-50 dark:bg-gray-950/50',
          borderColor: 'border-gray-200 dark:border-gray-800',
          iconColor: 'text-gray-600 dark:text-gray-400',
          textColor: 'text-gray-900 dark:text-gray-100'
        };
      default: // server
        return {
          title: title || 'Server Error',
          message: 'A server error occurred. Please try again later.',
          icon: <AlertTriangle className="h-8 w-8" />,
          bgColor: 'bg-red-50 dark:bg-red-950/50',
          borderColor: 'border-red-200 dark:border-red-800',
          iconColor: 'text-red-600 dark:text-red-400',
          textColor: 'text-red-900 dark:text-red-100'
        };
    }
  };

  const config = getContextConfig();

  return (
    <div className={`rounded-lg border p-6 ${config.bgColor} ${config.borderColor} ${className}`}>
      <div className="flex items-start space-x-4">
        {/* Icon */}
        <div className={`flex-shrink-0 ${config.iconColor}`}>
          {config.icon}
        </div>
        
        {/* Content */}
        <div className="flex-1 min-w-0">
          <h3 className={`text-lg font-semibold ${config.textColor}`}>
            {config.title}
          </h3>
          
          <div className="mt-2 space-y-2">
            <p className={`text-sm ${config.textColor.replace('900', '700').replace('100', '300')}`}>
              {config.message}
            </p>
            
            {errorMessage !== config.message && (
              <p className={`text-sm ${config.textColor.replace('900', '600').replace('100', '400')}`}>
                {errorMessage}
              </p>
            )}
          </div>

          {/* Error details (development/debug) */}
          {showDetails && errorStack && (
            <details className="mt-4">
              <summary className="cursor-pointer text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200">
                Technical Details
              </summary>
              <pre className="mt-2 p-3 bg-gray-900 text-gray-100 text-xs rounded-md overflow-auto max-h-40 whitespace-pre-wrap">
                {errorStack}
              </pre>
            </details>
          )}

          {/* Action buttons */}
          <div className="mt-4 flex flex-wrap gap-2">
            {onRetry && (
              <Button
                onClick={onRetry}
                variant="default"
                size="sm"
                className="flex items-center gap-2"
              >
                <RefreshCw className="h-4 w-4" />
                Try Again
              </Button>
            )}
            
            {onGoBack && (
              <Button
                onClick={onGoBack}
                variant="outline"
                size="sm"
                className="flex items-center gap-2"
              >
                <Home className="h-4 w-4" />
                Go Back
              </Button>
            )}
            
            {onSignOut && context === 'auth' && (
              <Button
                onClick={onSignOut}
                variant="destructive"
                size="sm"
                className="flex items-center gap-2"
              >
                <LogOut className="h-4 w-4" />
                Sign Out
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// Preset error displays for common scenarios
export const ErrorDisplayPresets = {
  // Network connectivity error
  Network: (onRetry?: () => void, className?: string) => (
    <ErrorDisplay
      error="Connection failed"
      context="network"
      onRetry={onRetry}
      className={className}
    />
  ),

  // Authentication required
  Auth: (onSignOut?: () => void, className?: string) => (
    <ErrorDisplay
      error="Authentication required"
      context="auth"
      onSignOut={onSignOut}
      className={className}
    />
  ),

  // Permission denied
  Permission: (onGoBack?: () => void, className?: string) => (
    <ErrorDisplay
      error="Access denied"
      context="permission"
      onGoBack={onGoBack}
      className={className}
    />
  ),

  // Server error
  Server: (onRetry?: () => void, className?: string) => (
    <ErrorDisplay
      error="Server error occurred"
      context="server"
      onRetry={onRetry}
      className={className}
    />
  ),

  // Loading error
  Loading: (onRetry?: () => void, className?: string) => (
    <ErrorDisplay
      error="Failed to load data"
      context="loading"
      onRetry={onRetry}
      className={className}
    />
  )
};