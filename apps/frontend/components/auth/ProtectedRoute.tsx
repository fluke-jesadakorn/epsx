// ============================================================================
// PROTECTED ROUTE COMPONENT
// Handles authentication flow for protected pages with OpenID + Web3
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Simple authentication check - no permission logic
 * - Backend makes all authorization decisions
 * - Redirect to auth page if not authenticated
 * - Display loading states during authentication checks
 */

'use client';

import { useSharedAuth } from '@/shared/components/auth/Provider';
import { Loader2 } from 'lucide-react';
import { usePathname, useRouter } from 'next/navigation';
import React, { useEffect, useState } from 'react';

interface ProtectedRouteProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  redirectTo?: string;
  requireAuth?: boolean;
}

export function ProtectedRoute({
  children,
  fallback,
  redirectTo = '/auth/signin',
  requireAuth = true,
}: ProtectedRouteProps) {
  const router = useRouter();
  const pathname = usePathname();
  const { isAuthenticated, isLoading, user } = useSharedAuth();
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    const checkAuthentication = async () => {
      if (isLoading) {
        // Still loading authentication state
        return;
      }

      setIsChecking(false);

      if (requireAuth && !isAuthenticated) {
        console.log('User not authenticated - page will handle auth UI', {
          current_path: pathname,
        });
        // No redirect - let page handle authentication UI
        return;
      }

      if (isAuthenticated && user) {
        console.log('User authenticated, allowing access', {
          wallet_address: user.wallet_address,
          path: pathname,
        });
      }
    };

    checkAuthentication();
  }, [isAuthenticated, isLoading, requireAuth, pathname, user]);

  // Show loading state while checking authentication
  if (isLoading || isChecking) {
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="flex flex-col items-center space-y-4">
          <Loader2 className="h-8 w-8 animate-spin text-blue-500" />
          <p className="text-sm text-gray-600">Checking authentication...</p>
        </div>
      </div>
    );
  }

  // If authentication is required but user is not authenticated, show fallback or children
  // (Page will handle showing auth UI)
  if (requireAuth && !isAuthenticated) {
    if (fallback) {
      return <>{fallback}</>;
    }
  }

  // Render protected content
  return <>{children}</>;
}

// HOC version for backward compatibility
export function withProtectedRoute<P extends object>(
  Component: React.ComponentType<P>,
  options?: {
    redirectTo?: string;
    requireAuth?: boolean;
  }
) {
  const ProtectedComponent = (props: P) => {
    return (
      <ProtectedRoute
        redirectTo={options?.redirectTo}
        requireAuth={options?.requireAuth}
      >
        <Component {...props} />
      </ProtectedRoute>
    );
  };

  ProtectedComponent.displayName = `withProtectedRoute(${Component.displayName || Component.name})`;

  return ProtectedComponent;
}

// Simple authentication guard hook
export function useAuthGuard(requireAuth: boolean = true) {
  const { isAuthenticated, isLoading, user } = useSharedAuth();

  return {
    isAuthenticated,
    isLoading,
    user,
    shouldRender: !requireAuth || isAuthenticated,
  };
}

export default ProtectedRoute;
