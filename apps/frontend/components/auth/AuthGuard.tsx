'use client';

import { useAuth } from '@/context/auth-context';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

interface AuthGuardProps {
  children: React.ReactNode;
  requireAuth?: boolean;
}

export function AuthGuard({ children, requireAuth = false }: AuthGuardProps) {
  const { user, loading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && requireAuth && !user && typeof window !== 'undefined') {
      const currentPath = window.location.pathname + window.location.search;
      const loginUrl = `/login?returnUrl=${encodeURIComponent(currentPath)}`;
      router.push(loginUrl);
    }
  }, [user, loading, requireAuth, router]);

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

  // If auth is required and user is not authenticated, show nothing (redirect is happening)
  if (requireAuth && !user) {
    return null;
  }

  // Render children
  return <>{children}</>;
}
