'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

interface AdminAuthGuardProps {
  children: React.ReactNode;
}

/**
 * Client-side authentication guard for admin routes
 * Checks for valid OIDC tokens and admin privileges
 */
export function AdminAuthGuard({ children }: AdminAuthGuardProps) {
  const router = useRouter();
  const pathname = usePathname();
  const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null);
  const [isAuthorized, setIsAuthorized] = useState<boolean | null>(null);

  useEffect(() => {
    checkAuthentication();
  }, [pathname]);

  const checkAuthentication = () => {
    try {
      // Check for admin tokens
      const accessToken = sessionStorage.getItem('admin_access_token');
      const userInfo = sessionStorage.getItem('admin_user');
      
      if (!accessToken || !userInfo) {
        // No authentication - redirect to login with callback
        const loginUrl = `/login?callbackUrl=${encodeURIComponent(pathname)}`;
        router.push(loginUrl);
        return;
      }

      // Check token expiration
      const expiresAt = sessionStorage.getItem('admin_expires_at');
      if (expiresAt && Date.now() > parseInt(expiresAt)) {
        // Token expired - clear and redirect to login
        clearTokens();
        const loginUrl = `/login?callbackUrl=${encodeURIComponent(pathname)}`;
        router.push(loginUrl);
        return;
      }

      setIsAuthenticated(true);

      // Check admin authorization
      const user = JSON.parse(userInfo);
      const isAdmin = ['admin', 'super_admin', 'moderator'].includes(user.role);
      
      if (!isAdmin) {
        // Not authorized - redirect to unauthorized page
        router.push('/unauthorized');
        return;
      }

      setIsAuthorized(true);
    } catch (error) {
      console.error('Authentication check failed:', error);
      // Clear invalid tokens and redirect to login
      clearTokens();
      const loginUrl = `/login?callbackUrl=${encodeURIComponent(pathname)}`;
      router.push(loginUrl);
    }
  };

  const clearTokens = () => {
    sessionStorage.removeItem('admin_access_token');
    sessionStorage.removeItem('admin_id_token');
    sessionStorage.removeItem('admin_token_type');
    sessionStorage.removeItem('admin_expires_in');
    sessionStorage.removeItem('admin_scope');
    sessionStorage.removeItem('admin_expires_at');
    sessionStorage.removeItem('admin_refresh_token');
    sessionStorage.removeItem('admin_user');
  };

  // Show loading while checking authentication
  if (isAuthenticated === null || isAuthorized === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <LoadingSpinner />
          <p className="mt-4 text-sm text-muted-foreground">
            Verifying administrative access...
          </p>
        </div>
      </div>
    );
  }

  // Render children if authenticated and authorized
  return <>{children}</>;
}

export default AdminAuthGuard;