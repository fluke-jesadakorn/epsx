/**
 * Server-Side Permission Guard
 * Protects components/pages based on JWT permissions
 */
import { ReactNode } from 'react';
import { requirePermission, hasPermission, type EPSXJWTPayload } from '@/lib/server/auth';

interface PermissionGuardProps {
  permission: string;
  children: ReactNode;
  redirectTo?: string;
}

export default async function PermissionGuard({ 
  permission, 
  children, 
  redirectTo 
}: PermissionGuardProps) {
  await requirePermission(permission, redirectTo);
  return <>{children}</>;
}

interface WithPermissionProps extends PermissionGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithPermission({ 
  permission, 
  children, 
  redirectTo 
}: WithPermissionProps) {
  const user = await requirePermission(permission, redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalPermissionProps {
  permission: string;
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalPermission({ 
  permission, 
  children, 
  fallback = null 
}: ConditionalPermissionProps) {
  const hasRequiredPermission = await hasPermission(permission);
  return <>{hasRequiredPermission ? children : fallback}</>;
}