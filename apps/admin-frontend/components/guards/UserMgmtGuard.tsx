/**
 * Server-Side User Management Guard
 * Protects user management components based on JWT admin modules
 */
import { ReactNode } from 'react';
import { requireUserManagement, canManageUsers, type EPSXJWTPayload } from '@/lib/server/auth';

interface UserMgmtGuardProps {
  children: ReactNode;
  redirectTo?: string;
}

export default async function UserMgmtGuard({ 
  children, 
  redirectTo 
}: UserMgmtGuardProps) {
  await requireUserManagement(redirectTo);
  return <>{children}</>;
}

interface WithUserMgmtProps extends UserMgmtGuardProps {
  children: (user: EPSXJWTPayload) => ReactNode;
}

export async function WithUserMgmt({ 
  children, 
  redirectTo 
}: WithUserMgmtProps) {
  const user = await requireUserManagement(redirectTo);
  return <>{children(user)}</>;
}

interface ConditionalUserMgmtProps {
  children: ReactNode;
  fallback?: ReactNode;
}

export async function ConditionalUserMgmt({ 
  children, 
  fallback = null 
}: ConditionalUserMgmtProps) {
  const canManage = await canManageUsers();
  return <>{canManage ? children : fallback}</>;
}