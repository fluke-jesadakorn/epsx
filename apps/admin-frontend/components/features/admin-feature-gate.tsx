/**
 * Server-Side Admin Feature Gates
 * Conditionally render admin features based on JWT authentication
 *
 * PERMISSION REFACTOR: Client-side permission checks are now permissive.
 * Backend (Rust) enforces access control based on user plan/permissions.
 */
import type { ReactNode } from 'react';

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
export function ConditionalFeature({
  condition,
  children,
  fallback = null
}: ConditionalFeatureProps) {
  return condition ? <>{children}</> : <>{fallback}</>;
}

async function authFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={Boolean(user)} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content only for authenticated admin users
 */
export const AdminFeature = authFeature;

/**
 * Show content based on admin module (Permissive for authenticated)
 */
export async function AdminModuleFeature({
  children,
  fallback
}: { module?: string } & FeatureGateProps) {
  return authFeature({ children, fallback });
}

/**
 * Show content only for system administrators (Permissive for authenticated)
 */
export const SystemAdminFeature = authFeature;

/**
 * Show content based on admin permission (Permissive for authenticated)
 */
export async function AdminPermissionFeature({
  children,
  fallback
}: { permission?: string } & FeatureGateProps) {
  return authFeature({ children, fallback });
}

/**
 * User management features (Permissive for authenticated)
 */
export const UserManagementFeature = authFeature;

/**
 * Analytics features (Permissive for authenticated)
 */
export const AnalyticsFeature = authFeature;

/**
 * Billing management features (Permissive for authenticated)
 */
export const BillingFeature = authFeature;

/**
 * Permission management features (Permissive for authenticated)
 */
export const PermissionManagementFeature = authFeature;

/**
 * Module coordination features (Permissive for authenticated)
 */
export const ModuleCoordinatorFeature = authFeature;

/**
 * Development-only admin features
 */
export async function DevAdminFeature({ children, fallback }: FeatureGateProps) {
  const isDev = process.env.NODE_ENV === 'development';
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={isDev && Boolean(user)} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}
