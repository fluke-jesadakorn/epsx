'use client';

import { useRouter, usePathname } from 'next/navigation';
import { AdminAuthGuard } from '@/components/auth/AdminAuthGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';


export function AdminAuthWrapper({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  
  // Public routes that don't require authentication
  const publicRoutes = ['/login', '/simple-login', '/unauthorized', '/access-denied', '/auth/callback', '/auth/logout'];
  const isPublicRoute = publicRoutes.includes(pathname);
  
  // For public routes, skip auth checks and render without layout
  if (isPublicRoute) {
    return <>{children}</>;
  }

  // For protected routes, use the AdminAuthGuard with layout
  return (
    <AdminAuthGuard>
      <AdminLayout>
        {children}
      </AdminLayout>
    </AdminAuthGuard>
  );
}

export default AdminAuthWrapper;