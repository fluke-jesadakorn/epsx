'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { LoadingSpinner } from '@/components/common/LoadingSpinner';

interface FrontendAuthGuardProps {
  children: React.ReactNode;
}

/**
 * Client-side authentication guard for frontend routes
 * Checks for valid OIDC tokens
 */
export function FrontendAuthGuard({ children }: FrontendAuthGuardProps) {
  const router = useRouter();
  const pathname = usePathname();
  const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null);

  useEffect(() => {
    checkAuthentication();
  }, [pathname]);

  const checkAuthentication = () => {
    try {
      // Check for tokens
      const accessToken = sessionStorage.getItem('access_token');
      const userInfo = sessionStorage.getItem('user');
      
      if (!accessToken || !userInfo) {
        // No authentication - redirect to login with callback
        const loginUrl = `/login?callbackUrl=${encodeURIComponent(pathname)}`;
        router.push(loginUrl);
        return;
      }

      // Check token expiration
      const expiresAt = sessionStorage.getItem('expires_at');
      if (expiresAt && Date.now() > parseInt(expiresAt)) {
        // Token expired - clear and redirect to login
        clearTokens();
        const loginUrl = `/login?callbackUrl=${encodeURIComponent(pathname)}`;
        router.push(loginUrl);
        return;
      }

      setIsAuthenticated(true);
    } catch (error) {
      console.error('Authentication check failed:', error);
      // Clear invalid tokens and redirect to login
      clearTokens();
      const loginUrl = `/login?callbackUrl=${encodeURIComponent(pathname)}`;
      router.push(loginUrl);
    }
  };

  const clearTokens = () => {
    sessionStorage.removeItem('access_token');
    sessionStorage.removeItem('id_token');
    sessionStorage.removeItem('token_type');
    sessionStorage.removeItem('expires_in');
    sessionStorage.removeItem('scope');
    sessionStorage.removeItem('expires_at');
    sessionStorage.removeItem('refresh_token');
    sessionStorage.removeItem('user');
  };

  // Show loading while checking authentication
  if (isAuthenticated === null) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <LoadingSpinner />
          <p className="mt-4 text-sm text-muted-foreground">
            Verifying authentication...
          </p>
        </div>
      </div>
    );
  }

  // Render children if authenticated
  return <>{children}</>;
}

export default FrontendAuthGuard;