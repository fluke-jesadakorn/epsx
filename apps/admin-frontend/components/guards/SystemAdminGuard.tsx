/**
 * Server-Side System Admin Guard
 * Protects system admin components based on JWT admin modules
 */
import { ReactNode } from 'react';
import { requireSystemAdmin, isSystemAdmin, type EPSXJWTPayload } from '@/lib/server/auth';

interface SystemAdminGuardProps {
  children: ReactNode;
  redirectTo?: string;
}

export default async function SystemAdminGuard({ 
  children, 
  redirectTo 
}: SystemAdminGuardProps) {
  await requireSystemAdmin(redirectTo);
  return <>{children}</>;
}

interface WithSystemAdminProps extends SystemAdminGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithSystemAdmin({ 
  children, 
  redirectTo 
}: WithSystemAdminProps) {
  const user = await requireSystemAdmin(redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalSystemAdminProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalSystemAdmin({ 
  children, 
  fallback = null 
}: ConditionalSystemAdminProps) {
  const isAdmin = await isSystemAdmin();
  return <>{isAdmin ? children : fallback}</>;
}