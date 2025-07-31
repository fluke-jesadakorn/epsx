'use client';

import { ReactNode } from 'react';
import { redirect } from 'next/navigation';
import { useUnifiedAuth } from '../providers/unified-auth';

export interface AuthGuardProps {
  children: ReactNode;
  fallback?: ReactNode;
  redirectTo?: string;
  permission?: string;
  profile?: string;
  role?: string;
  requireAuth?: boolean;
}

export function AuthGuard({
  children,
  fallback = <div>Loading...</div>,
  redirectTo,
  permission,
  profile,
  role,
  requireAuth = true,
}: AuthGuardProps) {
  const { isLoading, isAuthenticated, checkAccess } = useUnifiedAuth();

  // Show loading state
  if (isLoading) {
    return <>{fallback}</>;
  }

  // Check authentication requirement
  if (requireAuth && !isAuthenticated) {
    if (redirectTo) {
      redirect(redirectTo);
    }
    return null;
  }

  // Check access permissions
  if (permission || profile || role) {
    const hasAccess = checkAccess(permission || '', profile, role);
    if (!hasAccess) {
      if (redirectTo) {
        redirect(redirectTo);
      }
      return null;
    }
  }

  return <>{children}</>;
}

export interface PermissionGuardProps {
  children: ReactNode;
  permission: string;
  fallback?: ReactNode;
  profile?: string;
  role?: string;
}

export function PermissionGuard({
  children,
  permission,
  fallback = null,
  profile,
  role,
}: PermissionGuardProps) {
  const { checkAccess } = useUnifiedAuth();

  const hasAccess = checkAccess(permission, profile, role);

  if (!hasAccess) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}

export interface RoleGuardProps {
  children: ReactNode;
  role: string;
  fallback?: ReactNode;
}

export function RoleGuard({
  children,
  role,
  fallback = null,
}: RoleGuardProps) {
  const { hasRole } = useUnifiedAuth();

  if (!hasRole(role)) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}

export interface AdminGuardProps {
  children: ReactNode;
  fallback?: ReactNode;
  redirectTo?: string;
}

export function AdminGuard({
  children,
  fallback = <div>Access denied</div>,
  redirectTo,
}: AdminGuardProps) {
  const { isAdmin, isLoading, isAuthenticated } = useUnifiedAuth();

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (!isAuthenticated) {
    if (redirectTo) {
      redirect(redirectTo);
    }
    return <>{fallback}</>;
  }

  if (!isAdmin) {
    if (redirectTo) {
      redirect(redirectTo);
    }
    return <>{fallback}</>;
  }

  return <>{children}</>;
}