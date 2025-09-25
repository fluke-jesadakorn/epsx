/**
 * Enhanced Admin Error Boundary
 * Provides comprehensive error handling for admin interface
 * Prevents admin interface crashes and provides recovery options
 */

'use client';

import React, { Component, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home, Shield, Bug } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  context?: 'page' | 'component' | 'feature' | 'critical';
  featureName?: string;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorId: string | null;
  retryCount: number;
}

export class AdminErrorBoundary extends Component<Props, State> {
  private maxRetries = 3;

  constructor(props: Props) {
    super(props);
    this.state = { 
      hasError: false, 
      error: null,
      errorId: null,
      retryCount: 0
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    const errorId = `admin_err_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
    
    return {
      hasError: true,
      error,
      errorId
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    const { context = 'component', featureName } = this.props;
    
    // Enhanced error logging for admin interface
    console.error('🚨 Admin Error Boundary caught an error:', {
      error: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      errorId: this.state.errorId,
      context,
      featureName,
      retryCount: this.state.retryCount,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
      adminSession: this.getAdminSessionInfo()
    });

    // Call custom error handler
    this.props.onError?.(error, errorInfo);

    // Report critical admin errors
    if (context === 'critical') {
      this.reportCriticalError(error, errorInfo);
    }
  }

  private getAdminSessionInfo = () => {
    try {
      // Get basic admin session info for debugging (no sensitive data)
      return {
        hasAdminToken: !!document.cookie.includes('admin_access_token'),
        currentPath: window.location.pathname,
        referrer: document.referrer
      };
    } catch {
      return { error: 'Could not retrieve admin session info' };
    }
  };

  private reportCriticalError = (error: Error, errorInfo: React.ErrorInfo) => {
    // For critical admin errors, we might want special handling
    console.error('🔥 CRITICAL ADMIN ERROR:', {
      error: error.message,
      context: this.props.context,
      feature: this.props.featureName,
      componentStack: errorInfo.componentStack,
      errorId: this.state.errorId
    });
  };

  private handleReset = () => {
    const newRetryCount = this.state.retryCount + 1;
    
    if (newRetryCount <= this.maxRetries) {
      this.setState({ 
        hasError: false, 
        error: null,
        errorId: null,
        retryCount: newRetryCount
      });
    } else {
      // Too many retries, reload the page
      this.handleReload();
    }
  };

  private handleReload = () => {
    window.location.reload();
  };

  private handleGoToDashboard = () => {
    window.location.href = '/admin';
  };

  private getErrorSeverity = (): 'low' | 'medium' | 'high' | 'critical' => {
    const { context } = this.props;
    const { retryCount } = this.state;
    
    if (context === 'critical' || retryCount >= this.maxRetries) return 'critical';
    if (context === 'page' || retryCount >= 2) return 'high';
    if (context === 'feature' || retryCount >= 1) return 'medium';
    return 'low';
  };

  private getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      case 'high': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/20 dark:text-orange-400';
      case 'medium': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
      default: return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400';
    }
  };

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      const { context, featureName } = this.props;
      const severity = this.getErrorSeverity();
      const canRetry = this.state.retryCount < this.maxRetries;

      // Minimal error UI for components
      if (context === 'component') {
        return (
          <div className="border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/10 rounded-lg p-4 m-2">
            <div className="flex items-center gap-2 text-red-700 dark:text-red-400 mb-2">
              <AlertTriangle className="w-4 h-4" />
              <span className="text-sm font-medium">Component Error</span>
            </div>
            <p className="text-xs text-red-600 dark:text-red-400 mb-3">
              {featureName ? `${featureName} failed to load` : 'This component encountered an error'}
            </p>
            {canRetry && (
              <Button size="sm" variant="outline" onClick={this.handleReset}>
                <RefreshCw className="w-3 h-3 mr-1" />
                Retry
              </Button>
            )}
          </div>
        );
      }

      // Full error UI for page/feature/critical errors
      return (
        <div className="min-h-[400px] flex items-center justify-center p-4">
          <Card className="w-full max-w-lg">
            <CardHeader className="text-center">
              <div className="mx-auto w-16 h-16 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mb-4">
                {context === 'critical' ? (
                  <Shield className="w-8 h-8 text-red-600 dark:text-red-400" />
                ) : (
                  <AlertTriangle className="w-8 h-8 text-red-600 dark:text-red-400" />
                )}
              </div>
              
              <div className="space-y-2">
                <Badge className={this.getSeverityColor(severity)}>
                  {severity.toUpperCase()} ERROR
                </Badge>
                
                <CardTitle className="text-xl font-semibold">
                  {context === 'critical' && 'Critical Admin Error'}
                  {context === 'page' && 'Page Error'}
                  {context === 'feature' && `${featureName || 'Feature'} Error`}
                </CardTitle>
                
                <CardDescription>
                  {context === 'critical' && 'A critical admin function has failed. This may affect system operations.'}
                  {context === 'page' && 'This admin page encountered an error and cannot be displayed.'}
                  {context === 'feature' && `The ${featureName || 'feature'} is temporarily unavailable.`}
                </CardDescription>
              </div>
            </CardHeader>
            
            <CardContent className="space-y-4">
              {/* Error ID for support */}
              {this.state.errorId && (
                <div className="text-xs text-gray-500 dark:text-gray-400 font-mono bg-gray-100 dark:bg-gray-800 p-2 rounded">
                  Error ID: {this.state.errorId}
                  {this.state.retryCount > 0 && ` (Attempt ${this.state.retryCount + 1})`}
                </div>
              )}

              {/* Development error details */}
              {process.env.NODE_ENV === 'development' && this.state.error && (
                <details className="text-xs">
                  <summary className="cursor-pointer text-gray-600 dark:text-gray-400 mb-2 flex items-center gap-1">
                    <Bug className="w-3 h-3" />
                    Error Details (Development)
                  </summary>
                  <pre className="whitespace-pre-wrap bg-red-50 dark:bg-red-900/10 text-red-700 dark:text-red-300 p-2 rounded text-xs overflow-auto max-h-40 border">
                    {this.state.error.message}
                    {'\n\n'}
                    {this.state.error.stack}
                  </pre>
                </details>
              )}

              {/* Recovery actions */}
              <div className="space-y-2">
                {canRetry ? (
                  <Button onClick={this.handleReset} className="w-full">
                    <RefreshCw className="w-4 h-4 mr-2" />
                    Try Again ({this.maxRetries - this.state.retryCount} attempts left)
                  </Button>
                ) : (
                  <Button onClick={this.handleReload} className="w-full">
                    <RefreshCw className="w-4 h-4 mr-2" />
                    Reload Page
                  </Button>
                )}
                
                <div className="grid grid-cols-2 gap-2">
                  <Button 
                    variant="outline" 
                    onClick={this.handleGoToDashboard}
                    className="text-sm"
                  >
                    <Home className="w-4 h-4 mr-1" />
                    Dashboard
                  </Button>
                  
                  <Button 
                    variant="outline" 
                    onClick={this.handleReload}
                    className="text-sm"
                  >
                    Reload
                  </Button>
                </div>
              </div>

              <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
                If this problem persists, please contact the system administrator with the error ID above.
              </p>
            </CardContent>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}

// Convenience wrapper for admin components
export function withAdminErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  context: Props['context'] = 'component',
  featureName?: string
) {
  const WrappedComponent = (props: P) => (
    <AdminErrorBoundary context={context} featureName={featureName}>
      <Component {...props} />
    </AdminErrorBoundary>
  );

  WrappedComponent.displayName = `withAdminErrorBoundary(${Component.displayName || Component.name})`;
  
  return WrappedComponent;
}