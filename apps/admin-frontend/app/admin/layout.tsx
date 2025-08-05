'use client';

import { AdminLayout } from '@/components/layout/AdminLayout';
import { useAdminAuth } from '@/auth/ctx';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';
import { Lock } from 'lucide-react';

export default function AdminRootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { user, loading: authLoading, initialized, navigating } = useAdminAuth();
  const router = useRouter();
  
  // Simple module access check based on user role (temporary fix)
  const hasModuleAccess = (module: string): boolean => {
    if (!user) return false;
    if (user.role === 'super_admin') return true;
    if (user.role === 'admin') return ['admin', 'users', 'analytics', 'billing', 'settings'].includes(module);
    return false;
  };

  useEffect(() => {
    // Only redirect if auth context is fully initialized, not navigating, and no user is found
    if (initialized && !authLoading && !navigating && !user) {
      router.replace('/login');
    }
  }, [user, authLoading, initialized, navigating, router]);

  if (authLoading || !initialized || navigating) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
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
            You don't have permission to access the admin panel. This requires admin-level access.
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