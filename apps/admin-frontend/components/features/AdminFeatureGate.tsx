/**
 * Server-Side Admin Feature Gates
 * Conditionally render admin features based on JWT authentication
 * 
 * PERMISSION REFACTOR: Client-side permission checks are now permissive.
 * Backend (Rust) enforces access control based on user plan/permissions.
 */
import { ReactNode } from 'react';

import {
  getAuthUser
} from '@/lib/server/auth';

interface FeatureGateProps {
  children: ReactNode;
  fallback?: ReactNode;
}

interface ConditionalFeatureProps extends FeatureGateProps {
  condition: boolean;
}

/**
 * Base conditional feature component
 */
export async function ConditionalFeature({
  condition,
  children,
  fallback = null
}: ConditionalFeatureProps) {
  return condition ? <>{children}</> : <>{fallback}</>;
}

/**
 * Show content only for authenticated admin users
 */
export async function AdminFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on admin module (Permissive for authenticated)
 */
export async function AdminModuleFeature({
  children,
  fallback
}: { module?: string } & FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content only for system administrators (Permissive for authenticated)
 */
export async function SystemAdminFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on admin permission (Permissive for authenticated)
 */
export async function AdminPermissionFeature({
  children,
  fallback
}: { permission?: string } & FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * User management features (Permissive for authenticated)
 */
export async function UserManagementFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Analytics features (Permissive for authenticated)
 */
export async function AnalyticsFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Billing management features (Permissive for authenticated)
 */
export async function BillingFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Permission management features (Permissive for authenticated)
 */
export async function PermissionManagementFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Module coordination features (Permissive for authenticated)
 */
export async function ModuleCoordinatorFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Development-only admin features
 */
export async function DevAdminFeature({ children, fallback }: FeatureGateProps) {
  const isDev = process.env.NODE_ENV === 'development';
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={isDev && !!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}