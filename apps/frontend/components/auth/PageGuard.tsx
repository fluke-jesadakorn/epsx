import { ReactNode } from 'react';
import { redirect } from 'next/navigation';
import { getCurrentUser, validatePermissions } from '@/lib/auth/server-auth';
import { AccessDenied } from './AccessDenied';

interface PageGuardProps {
  children: ReactNode;
  permissions?: string[];
  requireAuth?: boolean;
  redirectTo?: string;
}

/**
 * Server-side page guard for protecting routes
 * Replaces client-side AuthGuard with server-only validation
 */
export async function PageGuard({
  children,
  permissions = [],
  requireAuth = true,
  redirectTo = '/login',
}: PageGuardProps) {
  // Check authentication
  const user = await getCurrentUser();
  
  if (requireAuth && !user) {
    redirect(redirectTo);
  }
  
  // Check permissions if specified
  if (permissions.length > 0 && user) {
    const validation = await validatePermissions(permissions);
    
    if (!validation.allowed) {
      return (
        <AccessDenied 
          reason={validation.reason}
          requiredPermissions={permissions}
        />
      );
    }
  }
  
  return <>{children}</>;
}

/**
 * Simple wrapper for pages that just need authentication
 */
export async function AuthRequired({ children }: { children: ReactNode }) {
  return (
    <PageGuard requireAuth>
      {children}
    </PageGuard>
  );
}

/**
 * Wrapper for admin-only pages
 */
export async function AdminOnly({ children }: { children: ReactNode }) {
  return (
    <PageGuard permissions={['admin:access']}>
      {children}
    </PageGuard>
  );
}