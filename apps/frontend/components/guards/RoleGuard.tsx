/**
 * Server-Side Role Guard
 * Protects components/pages based on JWT user role
 */
import { ReactNode } from 'react';
import { requireRole, hasRole, type EPSXJWTPayload } from '@/lib/server/auth';

interface RoleGuardProps {
  role: string;
  children: ReactNode;
  redirectTo?: string;
}

export default async function RoleGuard({ 
  role, 
  children, 
  redirectTo 
}: RoleGuardProps) {
  await requireRole(role, redirectTo);
  return <>{children}</>;
}

interface WithRoleProps extends RoleGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithRole({ 
  role, 
  children, 
  redirectTo 
}: WithRoleProps) {
  const user = await requireRole(role, redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalRoleProps {
  role: string;
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalRole({ 
  role, 
  children, 
  fallback = null 
}: ConditionalRoleProps) {
  const hasRequiredRole = await hasRole(role);
  return <>{hasRequiredRole ? children : fallback}</>;
}