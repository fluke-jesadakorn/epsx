/**
 * Unified Authentication Provider - Wallet Authentication
 * Uses wallet-based authentication with SIWE instead of OIDC
 * Maintains same interface for compatibility with existing code
 */

import { ReactNode } from 'react';
import { headers } from 'next/headers';
import { UnifiedAuth } from '@/lib/auth/wallet-auth';
import { redirect } from 'next/navigation';
import { PancakeAdminLayout } from '@/components/layout/PancakeAdminLayout';
import { ClientProviders } from './ClientProviders';

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
    '/login',
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
      console.log('❌ AuthProvider: No valid wallet session found');
      redirect('/login');
    }
    
    // Check admin access if required
    if (requireAdmin) {
      const hasAdminAccess = UnifiedAuth.hasAdminAccess(session.user);
      
      if (!hasAdminAccess) {
        console.warn('⚠️ AuthProvider: Wallet lacks admin permissions', {
          wallet_address: session.user?.wallet_address,
          email: session.user?.email,
          permissions: session.user?.permissions,
          required: 'admin:*:* or admin:{resource}:{action}'
        });
        redirect('/access-denied?reason=insufficient_admin_permissions');
      }
      
      console.log('✅ AuthProvider: Admin wallet authentication successful', {
        wallet_address: session.user?.wallet_address,
        email: session.user?.email,
        permissions: session.user?.permissions?.filter(p => p.startsWith('admin:')).length || 0,
        hasAdminAccess
      });
    }
    
    // Render with layout if requested
    if (layout && session.user) {
      try {
        return (
          <ClientProviders>
            <PancakeAdminLayout user={session.user}>
              {children}
            </PancakeAdminLayout>
          </ClientProviders>
        );
      } catch (error) {
        console.error('Critical AuthProvider error:', error);
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
            href="/login" 
            className="inline-block bg-gradient-to-r from-yellow-400 to-orange-500 text-white px-6 py-3 rounded-2xl font-semibold hover:from-yellow-500 hover:to-orange-600 transition-all duration-200 shadow-lg"
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