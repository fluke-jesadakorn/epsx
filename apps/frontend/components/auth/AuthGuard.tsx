'use client';

import type { ReactNode } from 'react';
import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/auth-context-improved';
import { useCustomClaims } from '@/hooks/useCustomClaims';
import { UserRole } from '@/types/auth/roles';
import { EmailVerificationNotice } from './EmailVerificationNotice';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ShieldX } from 'lucide-react';

interface AuthGuardProps {
  children: ReactNode;
  requireAuth?: boolean;
  requireEmailVerification?: boolean;
  requiredRole?: UserRole;
  requiredPermissions?: string[];
  redirectTo?: string;
  fallback?: ReactNode;
  showEmailVerification?: boolean;
}

export function AuthGuard({
  children,
  requireAuth = true,
  requireEmailVerification = false,
  requiredRole,
  requiredPermissions = [],
  redirectTo,
  fallback,
  showEmailVerification = true,
}: AuthGuardProps) {
  const { user, loading: authLoading, isInitialized } = useAuth();
  const { loading: claimsLoading, role, permissions } = useCustomClaims();
  const router = useRouter();

  useEffect(() => {
    if (!isInitialized || authLoading || claimsLoading) return;

    if (requireAuth && !user) {
      const loginUrl = redirectTo || `/login?returnUrl=${encodeURIComponent(window.location.pathname)}`;
      router.push(loginUrl);
      return;
    }

    if (!requireAuth && user) {
      router.push(redirectTo || '/dashboard');
      return;
    }

    if (requireEmailVerification && user && !user.emailVerified) {
      if (!showEmailVerification) {
        router.push('/verify-email');
      }
      return;
    }
  }, [user, authLoading, claimsLoading, isInitialized, requireAuth, requireEmailVerification, redirectTo, router, showEmailVerification]);

  // Show loading while auth is initializing
  if (!isInitialized || authLoading || claimsLoading) {
    return fallback || <AuthLoadingFallback />;
  }

  // User must be authenticated
  if (requireAuth && !user) {
    return fallback || <AuthLoadingFallback />;
  }

  // Check if we should redirect unauthenticated users
  if (!requireAuth && user) {
    return fallback || <AuthLoadingFallback />;
  }

  // Check email verification requirement
  if (requireEmailVerification && user && !user.emailVerified) {
    if (showEmailVerification) {
      return <EmailVerificationNotice />;
    }
    return fallback || <AuthLoadingFallback />;
  }

  // If user is authenticated, check role and permission requirements
  if (user) {
    // Check role requirement
    if (requiredRole && role !== requiredRole) {
      return (
        fallback || (
          <div className="min-h-screen flex items-center justify-center p-4">
            <Alert className="border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20 max-w-md">
              <ShieldX className="h-4 w-4 text-red-600 dark:text-red-400" />
              <AlertDescription className="text-red-800 dark:text-red-300">
                You don't have the required role ({requiredRole}) to access this content.
              </AlertDescription>
            </Alert>
          </div>
        )
      );
    }

    // Check permission requirements
    if (requiredPermissions.length > 0) {
      const hasAllPermissions = requiredPermissions.every(permission => {
        return permissions.includes(permission) || 
               permissions.includes('*') ||
               permissions.some(p => p.endsWith(':all') && permission.startsWith(p.split(':')[0] + ':'));
      });

      if (!hasAllPermissions) {
        const missingPermissions = requiredPermissions.filter(permission => {
          return !permissions.includes(permission) && 
                 !permissions.includes('*') &&
                 !permissions.some(p => p.endsWith(':all') && permission.startsWith(p.split(':')[0] + ':'));
        });

        return (
          fallback || (
            <div className="min-h-screen flex items-center justify-center p-4">
              <Alert className="border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20 max-w-md">
                <ShieldX className="h-4 w-4 text-red-600 dark:text-red-400" />
                <AlertDescription className="text-red-800 dark:text-red-300">
                  You don't have the required permissions to access this content.
                  {missingPermissions.length > 0 && (
                    <div className="mt-2 text-sm">
                      Missing: {missingPermissions.join(', ')}
                    </div>
                  )}
                </AlertDescription>
              </Alert>
            </div>
          )
        );
      }
    }
  }

  // All checks passed, render children
  return <>{children}</>;
}

function AuthLoadingFallback() {
  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <LoadingSpinner size="lg" />
        <p className="mt-4 text-muted-foreground">Checking authentication...</p>
      </div>
    </div>
  );
}

// Convenience components for common use cases
export function RequireAuth({ children, fallback }: { children: ReactNode; fallback?: ReactNode }) {
  return <AuthGuard fallback={fallback}>{children}</AuthGuard>;
}

export function RequireEmailVerification({ children, fallback }: { children: ReactNode; fallback?: ReactNode }) {
  return (
    <AuthGuard requireEmailVerification fallback={fallback}>
      {children}
    </AuthGuard>
  );
}

export function RequirePermissions({ 
  children, 
  permissions, 
  fallback 
}: { 
  children: ReactNode; 
  permissions: string[]; 
  fallback?: ReactNode;
}) {
  return (
    <AuthGuard requiredPermissions={permissions} fallback={fallback}>
      {children}
    </AuthGuard>
  );
}

// Higher-order component for protecting pages
export function withAuthGuard<P extends object>(
  Component: React.ComponentType<P>,
  options?: Omit<AuthGuardProps, 'children'>
) {
  return function ProtectedComponent(props: P) {
    return (
      <AuthGuard {...options}>
        <Component {...props} />
      </AuthGuard>
    );
  };
}

// Hook for auth state checks  
export function useAuthGuard() {
  const { user, loading, isInitialized } = useAuth();
  const { role, permissions } = useCustomClaims();
  
  return {
    isAuthenticated: !!user,
    isEmailVerified: user?.emailVerified ?? false,
    isLoading: loading || !isInitialized,
    role,
    permissions,
    user,
  };
}
