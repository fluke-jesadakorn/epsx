/**
 * Unified Authentication Provider - Wallet Authentication
 * Uses wallet-based authentication with SIWE instead of OIDC
 * Maintains same interface for compatibility with existing code
 */

import { headers } from 'next/headers';
import { redirect } from 'next/navigation';
import { ReactNode } from 'react';

import { ClientProviders } from './ClientProviders';

import { MainLayout } from '@/components/layout/MainLayout';
import { UnifiedAuth } from '@/lib/auth/auth';

interface AuthProviderProps {
  children: ReactNode;
  requireAuth?: boolean;
  requireAdmin?: boolean;
  layout?: boolean;
}

/**
 * Unified Authentication Provider
 * Handles both server-side authentication and client-side auth context
 * Supports both layout-wrapped and standalone authentication
 * @param root0
 * @param root0.children
 * @param root0.requireAuth
 * @param root0.requireAdmin
 * @param root0.layout
 */
export async function AuthProvider({
  children,
  requireAuth = true,
  requireAdmin = true,
  layout = true
}: AuthProviderProps) {
  // Get the current pathname from headers
  const headersList = await headers();
  const pathname = headersList.get('x-pathname') || '';

  // Public routes that don't require authentication
  const publicRoutes = [
    '/auth',
    '/api/auth/callback/epsx-backend',
    '/api/auth/login',
    '/api/auth/logout',
    '/auth/error',
    '/unauthorized',
    '/access-denied'
  ];

  const isPublicRoute = publicRoutes.some(route => pathname.startsWith(route));

  // For public routes, render without authentication or layout
  if (isPublicRoute) {
    return (
      <ClientProviders>
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50">
          {children}
        </div>
      </ClientProviders>
    );
  }

  // Check authentication if required
  if (requireAuth) {
    // Wallet Authentication: Validate authentication using wallet signatures
    const session = await UnifiedAuth.getSession();

    if (!session?.user) {
      console.log('⛔ AuthProvider: No session user found, redirecting to /auth');
      const returnUrl = encodeURIComponent(pathname);
      redirect(`/auth?return_url=${returnUrl}&reason=no-session`);
    }

    // Backend validates all permissions - don't check on frontend
    // Just let authenticated users through - backend will return 403 if no access

    // Render with layout if requested
    if (layout && session.user) {
      try {
        return (
          <ClientProviders>
            <MainLayout user={session.user as any}>
              {children}
            </MainLayout>
          </ClientProviders>
        );
      } catch (_error) {

        console.error('Critical AuthProvider error:', _error);
        return <AuthError />;
      }
    } else {
      // Render without layout (for pages that manage their own layout)
      return (
        <ClientProviders>
          {children}
        </ClientProviders>
      );
    }
  }

  // No authentication required - render without auth checks
  return (
    <ClientProviders>
      {layout ? (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50">
          {children}
        </div>
      ) : (
        children
      )}
    </ClientProviders>
  );
}

/**
 * Error component for authentication failures
 */
function AuthError() {
  return (
    <ClientProviders>
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 flex items-center justify-center">
        <div className="text-center space-y-4">
          <div className="h-16 w-16 bg-gradient-to-r from-red-400 to-pink-500 rounded-3xl flex items-center justify-center mx-auto">
            <span className="text-white text-2xl">⚠️</span>
          </div>
          <h1 className="text-xl font-bold text-gray-900">Authentication Error</h1>
          <p className="text-gray-600">Failed to load admin interface</p>
          <a
            href="/auth"
            className="inline-block bg-gradient-to-r from-yellow-400 to-orange-500 text-white px-6 py-3 rounded-2xl font-semibold shadow-lg focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-yellow-500"
          >
            Return to Login
          </a>
        </div>
      </div>
    </ClientProviders>
  );
}

/**
 * Higher-order component for protecting routes with admin access
 * @param Component
 * @deprecated Use AuthProvider with requireAdmin=true instead
 */
export function withAdminAuth<T extends {}>(Component: React.ComponentType<T>) {
  return async function AdminProtectedComponent(props: T) {
    return (
      <AuthProvider requireAuth={true} requireAdmin={true} layout={false}>
        <Component {...props} />
      </AuthProvider>
    );
  };
}

/**
 * Higher-order component for protecting routes with basic auth
 * @param Component
 * @deprecated Use AuthProvider with requireAuth=true instead  
 */
export function withAuth<T extends {}>(Component: React.ComponentType<T>) {
  return async function AuthProtectedComponent(props: T) {
    return (
      <AuthProvider requireAuth={true} requireAdmin={false} layout={false}>
        <Component {...props} />
      </AuthProvider>
    );
  };
}

export default AuthProvider;