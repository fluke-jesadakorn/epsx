'use client';

import { AdminLayout } from '@/components/layout/AdminLayout';
import { useAdminAuth } from '@/auth/ctx';
import { useModuleAuth } from '@/auth/module-ctx';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';
import { Lock } from 'lucide-react';

export default function AdminRootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { user, loading: authLoading, init } = useAdminAuth();
  const { hasModuleAccess, canPerformAction, loading: moduleLoading } = useModuleAuth();
  const router = useRouter();

  useEffect(() => {
    // Only redirect if auth context is fully initialized and no user is found
    if (init && !authLoading && !user) {
      router.replace('/login');
    }
  }, [user, authLoading, init, router]);

  if (authLoading || moduleLoading || !init) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!user) {
    return null;
  }

  // Check if user has admin access - allow super_admin to bypass
  const canAccessAdmin = user?.roles?.includes('super_admin') || 
                         user?.claims?.role === 'super_admin' ||
                         hasModuleAccess('admin') || 
                         canPerformAction('admin', 'view');

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