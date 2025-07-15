'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';

import { useAuth } from '@/context/auth-context-improved';
import { LoadingSpinner } from '@/components/ui/loading-spinner';

interface AuthGuardProps {
  children: React.ReactNode;
  requireAuth?: boolean;
  requireEmailVerification?: boolean;
  redirectTo?: string;
  fallback?: React.ReactNode;
}

export function AuthGuard({ 
  children, 
  requireAuth = true,
  requireEmailVerification = false,
  redirectTo,
  fallback 
}: AuthGuardProps) {
  const { user, loading, isInitialized } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isInitialized || loading) return;

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
      router.push('/verify-email');
      return;
    }
  }, [user, loading, isInitialized, requireAuth, requireEmailVerification, redirectTo, router]);

  // Show loading while checking auth state
  if (!isInitialized || loading) {
    return fallback || <AuthLoadingFallback />;
  }

  // Prevent flash of content for auth redirects
  if (requireAuth && !user) {
    return fallback || <AuthLoadingFallback />;
  }

  if (!requireAuth && user) {
    return fallback || <AuthLoadingFallback />;
  }

  if (requireEmailVerification && user && !user.emailVerified) {
    return fallback || <AuthLoadingFallback />;
  }

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
  
  return {
    isAuthenticated: !!user,
    isEmailVerified: user?.emailVerified ?? false,
    isLoading: loading || !isInitialized,
    user,
  };
}
