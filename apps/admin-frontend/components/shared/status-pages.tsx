'use client';

import { cn } from '@/design-system';
import {
  AlertTriangle,
  ArrowLeft,
  FileQuestion,
  Home,
  RefreshCw,
  RotateCcw,
  ShieldX,
} from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import React from 'react';

/**
 * Unified Status Page Components
 * Consistent design for error, not-found, and access-denied pages
 */

interface StatusPageLayoutProps {
  children: React.ReactNode;
  className?: string;
}

function StatusPageLayout({ children, className }: StatusPageLayoutProps) {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center min-h-[60vh] p-6 sm:p-8 lg:p-12',
      className
    )}>
      {children}
    </div>
  );
}

/**
 * Not Found Page Component
 */

interface NotFoundContentProps {
  title?: string;
  message?: string;
  showHomeLink?: boolean;
  showBackButton?: boolean;
}

export function NotFoundContent({
  title = 'Page Not Found',
  message = 'The page you\'re looking for doesn\'t exist or has been moved.',
  showHomeLink = true,
  showBackButton = true,
}: NotFoundContentProps) {
  const router = useRouter();

  return (
    <StatusPageLayout>
      <div className="relative">
        {/* Background decoration */}
        <div className="absolute inset-0 -z-10">
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-gradient-to-r from-primary/20 to-secondary/20 rounded-full blur-3xl" />
        </div>

        {/* Icon */}
        <div className="flex justify-center mb-6">
          <div className="relative">
            <div className="w-24 h-24 sm:w-28 sm:h-28 bg-gradient-to-br from-primary/20 to-secondary/20 rounded-3xl flex items-center justify-center border-2 border-primary/30">
              <FileQuestion className="w-12 h-12 sm:w-14 sm:h-14 text-primary" />
            </div>
            <div className="absolute -top-2 -right-2 w-8 h-8 bg-gradient-to-r from-primary to-secondary rounded-xl flex items-center justify-center text-white font-bold text-lg">
              ?
            </div>
          </div>
        </div>

        {/* Error code */}
        <div className="text-center mb-4">
          <span className="text-7xl sm:text-8xl font-black bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
            404
          </span>
        </div>

        {/* Title and message */}
        <div className="text-center max-w-md mx-auto">
          <h1 className="text-2xl sm:text-3xl font-bold text-foreground mb-3">
            {title}
          </h1>
          <p className="text-base sm:text-lg text-muted-foreground mb-8">
            {message}
          </p>
        </div>

        {/* Actions */}
        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          {showHomeLink && (
            <Link
              href="/"
              className="inline-flex items-center justify-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity"
            >
              <Home className="w-5 h-5" />
              Go Home
            </Link>
          )}
          {showBackButton && (
            <button
              onClick={() => router.back()}
              className="inline-flex items-center justify-center gap-2 px-6 py-3 bg-muted text-foreground rounded-2xl font-semibold hover:bg-muted/80 transition-colors"
            >
              <ArrowLeft className="w-5 h-5" />
              Go Back
            </button>
          )}
        </div>
      </div>
    </StatusPageLayout>
  );
}

/**
 * Error Page Component
 */

interface ErrorContentProps {
  title?: string;
  message?: string;
  errorId?: string;
  onReset?: () => void;
  showHomeLink?: boolean;
  showBackButton?: boolean;
}

export function ErrorContent({
  title = 'Something Went Wrong',
  message = 'An unexpected error occurred. Please try again.',
  errorId,
  onReset,
  showHomeLink = true,
  showBackButton = true,
}: ErrorContentProps) {
  const router = useRouter();

  return (
    <StatusPageLayout>
      <div className="w-full max-w-lg">
        {/* Header */}
        <div className="flex items-center gap-4 mb-6">
          <div className="w-14 h-14 sm:w-16 sm:h-16 bg-destructive/10 rounded-2xl flex items-center justify-center border border-destructive/20">
            <AlertTriangle className="w-7 h-7 sm:w-8 sm:h-8 text-destructive" />
          </div>
          <div>
            <h1 className="text-2xl sm:text-3xl font-bold text-foreground">
              {title}
            </h1>
            <p className="text-muted-foreground">Error encountered</p>
          </div>
        </div>

        {/* Error details */}
        <div className="bg-card rounded-2xl border border-border p-6 mb-6 shadow-lg">
          <p className="text-foreground mb-3">{message}</p>
          {errorId && (
            <p className="text-xs text-muted-foreground font-mono bg-muted px-3 py-2 rounded-lg">
              Error ID: {errorId}
            </p>
          )}
        </div>

        {/* Actions */}
        <div className="space-y-3">
          {onReset && (
            <button
              onClick={onReset}
              className="w-full inline-flex items-center justify-center gap-2 px-6 py-4 bg-primary text-primary-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity"
            >
              <RefreshCw className="w-5 h-5" />
              Try Again
            </button>
          )}
          {showHomeLink && (
            <Link
              href="/"
              className="w-full inline-flex items-center justify-center gap-2 px-6 py-4 bg-secondary text-secondary-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity"
            >
              <Home className="w-5 h-5" />
              Go Home
            </Link>
          )}
          {showBackButton && (
            <button
              onClick={() => router.back()}
              className="w-full inline-flex items-center justify-center gap-2 px-4 py-3 text-muted-foreground hover:text-foreground transition-colors font-medium"
            >
              <ArrowLeft className="w-5 h-5" />
              Go Back
            </button>
          )}
        </div>
      </div>
    </StatusPageLayout>
  );
}

/**
 * Access Denied Page Component
 */

interface AccessDeniedContentProps {
  title?: string;
  reason?: string;
  route?: string;
  context?: string;
  permission?: string;
  showLoginButton?: boolean;
  showHomeButton?: boolean;
}

export function AccessDeniedContent({
  title = 'Access Denied',
  reason = 'You don\'t have permission to access this resource.',
  route,
  context,
  permission,
  showLoginButton = true,
  showHomeButton = true,
}: AccessDeniedContentProps) {
  return (
    <StatusPageLayout>
      <div className="w-full max-w-lg">
        {/* Icon */}
        <div className="flex justify-center mb-6">
          <div className="w-20 h-20 sm:w-24 sm:h-24 bg-destructive/10 rounded-3xl flex items-center justify-center border-2 border-destructive/20">
            <ShieldX className="w-10 h-10 sm:w-12 sm:h-12 text-destructive" />
          </div>
        </div>

        {/* Title */}
        <div className="text-center mb-6">
          <h1 className="text-2xl sm:text-3xl font-bold text-foreground mb-2">
            {title}
          </h1>
          <p className="text-base sm:text-lg text-muted-foreground">
            {reason}
          </p>
        </div>

        {/* Error details card */}
        <div className="bg-card rounded-2xl border border-border shadow-lg overflow-hidden mb-6">
          <div className="p-6">
            <h3 className="text-sm font-semibold text-foreground mb-4 flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-destructive" />
              Error Details
            </h3>
            <div className="space-y-3 text-sm">
              {route && (
                <div className="flex justify-between items-start gap-4">
                  <span className="text-muted-foreground shrink-0">Requested Route:</span>
                  <code className="text-foreground bg-muted px-2 py-1 rounded text-right break-all">
                    {decodeURIComponent(route)}
                  </code>
                </div>
              )}
              {context && (
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground">Context:</span>
                  <span className="text-foreground capitalize">{context}</span>
                </div>
              )}
              {permission && (
                <div className="flex justify-between items-start gap-4">
                  <span className="text-muted-foreground shrink-0">Required Permission:</span>
                  <code className="text-foreground bg-muted px-2 py-1 rounded text-right break-all">
                    {decodeURIComponent(permission)}
                  </code>
                </div>
              )}
            </div>
          </div>

          {context === 'admin' && (
            <div className="border-t border-border bg-primary/5 p-4">
              <p className="text-sm text-foreground">
                <span className="font-medium">Admin Access Required:</span> Only authorized administrators can access this panel.
                Contact your system administrator if you believe this is an error.
              </p>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex flex-col sm:flex-row gap-3">
          {showLoginButton && (
            <Link
              href="/auth"
              className="flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-destructive text-destructive-foreground rounded-2xl font-semibold hover:opacity-90 transition-opacity"
            >
              <RotateCcw className="w-5 h-5" />
              Login Again
            </Link>
          )}
          {showHomeButton && (
            <Link
              href="/"
              className="flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-muted text-foreground rounded-2xl font-semibold hover:bg-muted/80 transition-colors"
            >
              <Home className="w-5 h-5" />
              Go Home
            </Link>
          )}
        </div>
      </div>
    </StatusPageLayout>
  );
}

export default {
  NotFoundContent,
  ErrorContent,
  AccessDeniedContent,
};
