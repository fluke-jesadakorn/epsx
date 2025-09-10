import { ReactNode } from 'react';
import { headers } from 'next/headers';
import { UnifiedAuth } from '@/lib/auth/unified-auth';
import { redirect } from 'next/navigation';
import { PancakeAdminLayout } from '@/components/layout/PancakeAdminLayout';
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
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50">
          {children}
        </div>
      </ClientProviders>
    );
  }
  
  // OIDC Migration: Validate authentication using OIDC tokens
  const session = await UnifiedAuth.getSession();
  
  if (!session?.user) {
    console.log('❌ AdminAuthWrapper: No valid session found');
    redirect('/login');
  }
  
  // Check admin access using structured permissions system
  const hasAdminAccess = UnifiedAuth.hasAdminAccess(session.user);
  
  if (!hasAdminAccess) {
    console.warn('⚠️ AdminAuthWrapper: User lacks admin permissions', {
      user: session.user?.email,
      permissions: session.user?.permissions,
      required: 'admin:*:* or admin:{resource}:{action}'
    });
    redirect('/access-denied?reason=insufficient_admin_permissions');
  }
  
  console.log('✅ AdminAuthWrapper: Admin authentication successful', {
    user: session.user?.email,
    permissions: session.user?.permissions?.filter(p => p.startsWith('admin:')).length || 0,
    hasAdminAccess
  });
  
  // For protected routes, render with authentication and layout
  try {
    return (
      <ClientProviders>
        <PancakeAdminLayout user={session.user}>
          {children}
        </PancakeAdminLayout>
      </ClientProviders>
    );
  } catch (error) {
    console.error('Critical AdminAuthWrapper error:', error);
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
}

export default AdminAuthWrapper;