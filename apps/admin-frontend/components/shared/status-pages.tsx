'use client';

import { cn } from '@/design-system';
import { logoutAction } from '@/lib/auth/auth-actions';
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
import React, { useCallback } from 'react';
import { useDisconnect } from 'wagmi';

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
            <div className="w-24 h-24 sm:w-28 sm:h-28 bg-gradient-to-br from-primary to-secondary rounded-3xl flex items-center justify-center border-2 border-primary/50 shadow-lg">
              <FileQuestion className="w-12 h-12 sm:w-14 sm:h-14 text-white" />
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
              className="inline-flex items-center justify-center gap-2 px-6 py-3 bg-gradient-to-r from-purple-500 to-orange-500 text-white rounded-2xl font-semibold shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30 hover-lift transition-all"
            >
              <Home className="w-5 h-5" />
              Go Home
            </Link>
          )}
          {showBackButton && (
            <button
              onClick={() => router.back()}
              className="inline-flex items-center justify-center gap-2 px-6 py-3 bg-muted/30 border border-border/20 text-foreground rounded-2xl font-semibold hover:bg-muted/50 transition-colors"
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
          <div className="w-14 h-14 sm:w-16 sm:h-16 bg-red-600 rounded-2xl flex items-center justify-center border border-red-400 shadow-lg shadow-red-500/30">
            <AlertTriangle className="w-7 h-7 sm:w-8 sm:h-8 text-white" />
          </div>
          <div>
            <h1 className="text-2xl sm:text-3xl font-bold text-foreground">
              {title}
            </h1>
            <p className="text-muted-foreground">Error encountered</p>
          </div>
        </div>

        {/* Error details */}
        <div className="bg-muted/30 rounded-2xl border border-border/20 p-6 mb-6 shadow-lg">
          <p className="text-foreground mb-3">{message}</p>
          {errorId !== undefined && errorId !== '' && (
            <p className="text-xs text-muted-foreground font-mono bg-muted/30 border border-border/20 px-3 py-2 rounded-lg">
              Error ID: {errorId}
            </p>
          )}
        </div>

        {/* Actions */}
        <div className="space-y-3">
          {onReset && (
            <button
              onClick={onReset}
              className="w-full inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-purple-500 to-orange-500 text-white rounded-2xl font-semibold shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30 hover-lift transition-all"
            >
              <RefreshCw className="w-5 h-5" />
              Try Again
            </button>
          )}
          {showHomeLink && (
            <Link
              href="/"
              className="w-full inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-purple-500 to-orange-500 text-white rounded-2xl font-semibold shadow-lg shadow-purple-500/20 hover:shadow-xl hover:shadow-purple-500/30 hover-lift transition-all"
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
  detail?: string;
  showLoginButton?: boolean;
  showHomeButton?: boolean;
}

interface ErrorDetailsProps {
  route?: string;
  context?: string;
  permission?: string;
  detail?: string;
}

function ErrorDetailsCard({ route, context, permission, detail }: ErrorDetailsProps) {
  return (
    <div className="bg-muted/30 rounded-2xl border border-border/20 shadow-lg overflow-hidden mb-6">
      <div className="p-6">
        <h3 className="text-sm font-semibold text-foreground mb-4 flex items-center gap-2">
          <AlertTriangle className="w-4 h-4 text-destructive" />
          Error Details
        </h3>
        <div className="space-y-3 text-sm">
          {route !== undefined && route !== '' && (
            <div className="flex justify-between items-start gap-4">
              <span className="text-muted-foreground shrink-0">Requested Route:</span>
              <code className="text-foreground bg-muted/30 border border-border/20 px-2 py-1 rounded text-right break-all">{decodeURIComponent(route)}</code>
            </div>
          )}
          {context !== undefined && context !== '' && (
            <div className="flex justify-between items-center">
              <span className="text-muted-foreground">Context:</span>
              <span className="text-foreground capitalize">{context}</span>
            </div>
          )}
          {permission !== undefined && permission !== '' && (
            <div className="flex justify-between items-start gap-4">
              <span className="text-muted-foreground shrink-0">Required Permission:</span>
              <code className="text-foreground bg-muted/30 border border-border/20 px-2 py-1 rounded text-right break-all">{decodeURIComponent(permission)}</code>
            </div>
          )}
          {detail !== undefined && detail !== '' && (
            <div className="flex justify-between items-start gap-4 border-t border-border/20 pt-3 mt-1">
              <span className="text-muted-foreground shrink-0">Backend Detail:</span>
              <span className="text-foreground text-right">{detail}</span>
            </div>
          )}
        </div>
      </div>
      {context === 'admin' && (
        <div className="border-t border-border/20 bg-gradient-to-r from-purple-500/10 to-orange-500/10 p-4">
          <p className="text-sm text-foreground">
            <span className="font-medium">Admin Access Required:</span> Only authorized administrators can access this panel.
            Contact your system administrator if you believe this is an error.
          </p>
        </div>
      )}
    </div>
  );
}

export function AccessDeniedContent({
  title = 'Access Denied',
  reason = 'You don\'t have permission to access this resource.',
  route,
  context,
  permission,
  detail,
  showLoginButton = true,
  showHomeButton = true,
}: AccessDeniedContentProps) {
  const router = useRouter();
  const { disconnect } = useDisconnect();

  const handleReauth = useCallback(async () => {
    try { disconnect(); } catch { /* WalletConnect origin check may fail in dev */ }
    const returnPath = route !== undefined ? decodeURIComponent(route) : undefined;
    await logoutAction(returnPath);
    router.replace('/auth');
  }, [disconnect, route, router]);

  return (
    <StatusPageLayout>
      <div className="w-full max-w-lg">
        <div className="flex justify-center mb-6">
          <div className="w-20 h-20 sm:w-24 sm:h-24 bg-gradient-to-br from-red-500 to-red-600 rounded-3xl flex items-center justify-center border-2 border-red-400/30 shadow-lg shadow-red-500/30">
            <ShieldX className="w-10 h-10 sm:w-12 sm:h-12 text-white" />
          </div>
        </div>

        <div className="text-center mb-6">
          <h1 className="text-2xl sm:text-3xl font-bold text-foreground mb-2">{title}</h1>
          <p className="text-base sm:text-lg text-muted-foreground">{reason}</p>
        </div>

        <ErrorDetailsCard route={route} context={context} permission={permission} detail={detail} />

        <div className="flex flex-col sm:flex-row gap-3">
          {showLoginButton && (
            <button
              onClick={() => { void handleReauth(); }}
              className="flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-gradient-to-r from-red-500 to-red-600 text-white rounded-2xl font-semibold shadow-lg shadow-red-500/20 hover:shadow-xl hover:shadow-red-500/30 hover-lift transition-all"
            >
              <RotateCcw className="w-5 h-5" />
              Go to Auth
            </button>
          )}
          {showHomeButton && (
            <button
              onClick={() => router.back()}
              className="flex-1 inline-flex items-center justify-center gap-2 px-6 py-4 bg-muted/30 border border-border/20 text-foreground rounded-2xl font-semibold hover:bg-muted/50 transition-colors"
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

export default {
  NotFoundContent,
  ErrorContent,
  AccessDeniedContent,
};
