import { ReactNode } from 'react';
import { headers } from 'next/headers';
import { adminOIDCAuth } from '@/lib/admin-client';
import { redirect } from 'next/navigation';
import { AdminLayoutServer } from '@/components/layout/AdminLayoutServer';
import { ClientProviders } from './ClientProviders';

interface AdminAuthWrapperProps {
  children: ReactNode;
}

/**
 * Server-side Authentication Wrapper
 * OIDC Migration: Uses OIDC tokens instead of legacy JWT
 * Handles admin authentication and determines if layout is needed
 */
export async function AdminAuthWrapper({ children }: AdminAuthWrapperProps) {
  // Get the current pathname from headers
  const headersList = await headers();
  const pathname = headersList.get('x-pathname') || '';
  
  // Public routes that don't require authentication or layout
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
        <div className="min-h-screen bg-gray-50">
          {children}
        </div>
      </ClientProviders>
    );
  }
  
  // OIDC Migration: Validate authentication using OIDC tokens
  const adminSession = await adminOIDCAuth.getSession();
  
  if (!adminSession.isAuthenticated || !adminSession.user) {
    console.log('❌ AdminAuthWrapper: No valid OIDC session found');
    redirect('/login');
  }
  
  // Check admin access using structured permissions system
  if (!adminSession.hasAdminAccess) {
    console.warn('⚠️ AdminAuthWrapper: User lacks admin permissions', {
      user: adminSession.user?.email,
      permissions: adminSession.user?.permissions,
      required: 'admin:*:* or admin:{resource}:{action}',
      error: adminSession.error
    });
    redirect('/access-denied?reason=insufficient_admin_permissions');
  }
  
  console.log('✅ AdminAuthWrapper: Admin OIDC authentication successful', {
    user: adminSession.user?.email,
    permissions: adminSession.user?.permissions.filter(p => p.startsWith('admin:')).length,
    hasAdminAccess: adminSession.hasAdminAccess
  });
  
  // For protected routes, render with authentication and layout
  try {
    return (
      <ClientProviders>
        <AdminLayoutServer>
          {children}
        </AdminLayoutServer>
      </ClientProviders>
    );
  } catch (error) {
    console.error('Critical AdminAuthWrapper error:', error);
    return (
      <ClientProviders>
        <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
          <div className="text-center space-y-4">
            <h1 className="text-xl font-bold">Authentication Error</h1>
            <p className="text-gray-300">Failed to load admin interface</p>
            <a 
              href="/login" 
              className="inline-block bg-yellow-500 text-black px-4 py-2 rounded hover:bg-yellow-600"
            >
              Return to Login
            </a>
          </div>
        </div>
      </ClientProviders>
    );
  }
}

export default AdminAuthWrapper;