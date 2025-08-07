'use client';

import { useAdminAuth } from '@/lib/auth/ctx';
import { useRouter, usePathname } from 'next/navigation';
import { useEffect, useState } from 'react';
import { Lock } from 'lucide-react';
import { AdminLayout } from '@/components/layout/AdminLayout';

export function AdminAuthWrapper({ children }: { children: React.ReactNode }) {
  const { user, loading: authLoading, initialized, navigating } = useAdminAuth();
  const router = useRouter();
  const pathname = usePathname();
  const [loadingTimeout, setLoadingTimeout] = useState(false);
  
  // Public routes that don't require authentication
  const publicRoutes = ['/login', '/unauthorized', '/access-denied'];
  const isPublicRoute = publicRoutes.includes(pathname);
  
  // Simple module access check based on user role (temporary fix)
  const hasModuleAccess = (module: string): boolean => {
    if (!user) return false;
    if (user.role === 'super_admin') return true;
    if (user.role === 'admin') return ['admin', 'users', 'analytics', 'billing', 'settings'].includes(module);
    return false;
  };

  // Add timeout for loading state to prevent infinite loading
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (authLoading && !initialized) {
        console.warn('Authentication timeout reached, redirecting to login');
        setLoadingTimeout(true);
      }
    }, 10000); // 10 second timeout

    return () => clearTimeout(timeout);
  }, [authLoading, initialized]);

  useEffect(() => {
    // Only redirect if auth context is fully initialized, not navigating, no user is found, and not on a public route
    if (initialized && !authLoading && !navigating && !user && !isPublicRoute) {
      router.replace('/login');
    }
  }, [user, authLoading, initialized, navigating, router, isPublicRoute]);

  // For public routes, skip auth checks and render without layout
  if (isPublicRoute) {
    console.log('AdminAuthWrapper: Rendering public route', pathname);
    return <>{children}</>;
  }

  // Handle loading timeout
  if (loadingTimeout) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="text-center max-w-md">
          <Lock className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">Authentication Timeout</h2>
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            Authentication is taking longer than expected. Please try logging in again.
          </p>
          <button
            onClick={() => router.replace('/login')}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Go to Login
          </button>
        </div>
      </div>
    );
  }

  // Add timeout for loading state to prevent infinite loading
  if (authLoading || !initialized) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <div className="text-gray-600 dark:text-gray-400">Loading authentication...</div>
        </div>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  // Check if user has admin access - allow super_admin to bypass module restrictions
  const canAccessAdmin = user?.role === 'super_admin' || 
                         user?.role === 'admin' || 
                         hasModuleAccess('admin');

  if (!canAccessAdmin) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="text-center max-w-md">
          <Lock className="w-16 h-16 text-gray-400 mx-auto mb-4" />
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">Access Denied</h2>
          <p className="text-gray-600 dark:text-gray-400 mb-4">
            You don&apos;t have permission to access the admin panel. This requires admin-level access.
          </p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
            <h3 className="font-medium text-blue-900 dark:text-blue-300 mb-2">Contact Administrator</h3>
            <p className="text-sm text-blue-700 dark:text-blue-400">
              Please contact your system administrator to request admin access.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <AdminLayout>
      {children}
    </AdminLayout>
  );
}