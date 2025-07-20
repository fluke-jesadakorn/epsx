'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useIAM } from '@/hooks/useIAM';
import { Loader2 } from 'lucide-react';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requiredPermission?: string;
  requiredRole?: string;
  fallbackUrl?: string;
}

export function ProtectedRoute({ 
  children, 
  requiredPermission, 
  requiredRole,
  fallbackUrl = '/login' 
}: ProtectedRouteProps) {
  const { user, role, hasPermission, loading } = useIAM();
  const router = useRouter();

  useEffect(() => {
    if (!loading) {
      if (!user) {
        router.push(fallbackUrl);
        return;
      }

      if (requiredRole && role?.id !== requiredRole) {
        router.push(fallbackUrl);
        return;
      }

      if (requiredPermission && !hasPermission(requiredPermission)) {
        router.push(fallbackUrl);
        return;
      }
    }
  }, [user, role, loading, requiredPermission, requiredRole, fallbackUrl, router]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Loader2 className="h-8 w-8 animate-spin" />
      </div>
    );
  }

  if (!user) {
    return null;
  }

  if (requiredRole && role?.id !== requiredRole) {
    return null;
  }

  if (requiredPermission && !hasPermission(requiredPermission)) {
    return null;
  }

  return <>{children}</>;
}
