'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/context/shared-auth-provider';

interface ClientAuthGuardProps {
  children: React.ReactNode;
  redirectTo?: string;
}

export function ClientAuthGuard({ children, redirectTo = '/login' }: ClientAuthGuardProps) {
  const { user, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && !user) {
      console.log('ClientAuthGuard: No authenticated user, redirecting to login');
      const currentPath = window.location.pathname;
      const loginUrl = `${redirectTo}?returnUrl=${encodeURIComponent(currentPath)}`;
      router.push(loginUrl);
    }
  }, [user, loading, router, redirectTo]);

  // Show loading while checking auth
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-orange-200 dark:border-orange-800">
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-t-orange-500 dark:border-t-orange-400 absolute top-0 left-0"></div>
        </div>
      </div>
    );
  }

  // Only render children if user is authenticated
  if (!user) {
    return null; // Will redirect in useEffect
  }

  return <>{children}</>;
}
