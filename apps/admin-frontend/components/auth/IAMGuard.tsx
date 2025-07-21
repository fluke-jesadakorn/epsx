'use client';

import { useAdminAuth } from '@/context/admin-auth';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

interface IAMGuardProps {
  children: React.ReactNode;
}

export function IAMGuard({ children }: IAMGuardProps) {
  const { user, loading, isInitialized, isAdmin } = useAdminAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isInitialized || loading) return;

    if (!user) {
      router.replace('/login');
      return;
    }

    if (!isAdmin) {
      router.replace('/unauthorized');
      return;
    }
  }, [user, loading, isInitialized, isAdmin, router]);

  // Show loading while checking auth status
  if (!isInitialized || loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  // Show nothing while redirecting
  if (!user || !isAdmin) {
    return null;
  }

  return <>{children}</>;
}
