'use client';

import { useRouter } from 'next/navigation';
import { useAdminAuth } from '@/auth/ctx';
import { useEffect } from 'react';

export default function RootPage() {
  const router = useRouter();
  const { initialized, loading, user, navigating } = useAdminAuth();

  useEffect(() => {
    // Only redirect after auth is initialized and not navigating to prevent loops
    if (initialized && !loading && !navigating) {
      if (user) {
        router.replace('/admin');
      } else {
        router.replace('/login');
      }
    }
  }, [initialized, loading, user, navigating, router]);

  // Show loading while auth initializes or navigating
  if (!initialized || loading || navigating) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return null;
}
