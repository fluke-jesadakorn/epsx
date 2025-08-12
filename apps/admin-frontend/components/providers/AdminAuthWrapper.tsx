'use client';

import { usePathname } from 'next/navigation';
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

  // For protected routes, use AdminLayout directly
  // Authentication is now handled by middleware with HTTP-only cookies
  return (
    <AdminLayout>
      {children}
    </AdminLayout>
  );
}

export default AdminAuthWrapper;