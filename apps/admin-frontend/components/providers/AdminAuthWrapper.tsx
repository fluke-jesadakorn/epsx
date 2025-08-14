import { ReactNode } from 'react';
import { headers } from 'next/headers';
import { getSessionFromJWT } from '@/lib/server/jwt';
import { redirect } from 'next/navigation';
import { AdminLayoutServer } from '@/components/layout/AdminLayoutServer';
import { ClientProviders } from './ClientProviders';

interface AdminAuthWrapperProps {
  children: ReactNode;
}

/**
 * Server-side Authentication Wrapper
 * Handles authentication on the server and determines if layout is needed
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
  
  // For protected routes, validate authentication
  const sessionData = await getSessionFromJWT();
  
  if (!sessionData?.isAuthenticated || !sessionData?.user) {
    redirect('/login');
  }
  
  // Check if user has admin access
  const userAdminModules = sessionData.user.admin_modules || [];
  if (userAdminModules.length === 0) {
    redirect('/access-denied?reason=insufficient_admin_access');
  }
  
  // For protected routes, render with authentication and layout
  return (
    <ClientProviders>
      <AdminLayoutServer>
        {children}
      </AdminLayoutServer>
    </ClientProviders>
  );
}

export default AdminAuthWrapper;