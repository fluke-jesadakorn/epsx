/**
 * Server-Side Admin Feature Gates
 * Conditionally render admin features based on JWT authentication
 */
import { ReactNode } from 'react';
import { 
  getAuthUser, 
  hasPermission, 
  isSystemAdmin,
  canManageUsers,
  canAccessAnalytics,
  hasPlatformPermission
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
  
  // Check permissions system
  const hasAdminPermissions = user?.permissions?.some(p => 
    p.includes(':manage') || 
    p.includes(':admin') || 
    p === '*'
  ) || false;
  
  const hasAdmin = hasAdminPermissions;
  
  return (
    <ConditionalFeature condition={!!hasAdmin} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on specific admin module
 */
interface AdminModuleFeatureProps extends FeatureGateProps {
  module: string;
}

export async function AdminModuleFeature({ 
  module, 
  children, 
  fallback 
}: AdminModuleFeatureProps) {
  const hasModule = await hasAdminModule(module);
  return (
    <ConditionalFeature condition={hasModule} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content only for system administrators
 */
export async function SystemAdminFeature({ children, fallback }: FeatureGateProps) {
  const isSysAdmin = await isSystemAdmin();
  return (
    <ConditionalFeature condition={isSysAdmin} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on admin permission
 */
interface AdminPermissionFeatureProps extends FeatureGateProps {
  permission: string;
}

export async function AdminPermissionFeature({ 
  permission, 
  children, 
  fallback 
}: AdminPermissionFeatureProps) {
  const hasRequiredPermission = await hasPermission(permission);
  return (
    <ConditionalFeature condition={hasRequiredPermission} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * User management features
 */
export async function UserManagementFeature({ children, fallback }: FeatureGateProps) {
  const canManage = await canManageUsers();
  return (
    <ConditionalFeature condition={canManage} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Analytics features
 */
export async function AnalyticsFeature({ children, fallback }: FeatureGateProps) {
  const canAccess = await canAccessAnalytics();
  return (
    <ConditionalFeature condition={canAccess} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Billing management features
 */
export async function BillingFeature({ children, fallback }: FeatureGateProps) {
  const hasBillingPermission = await hasPermission('epsx:billing:manage');
  const hasLegacyModule = await hasAdminModule('billing_admin');
  
  return (
    <ConditionalFeature condition={hasBillingPermission || hasLegacyModule} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Permission management features
 */
export async function PermissionManagementFeature({ children, fallback }: FeatureGateProps) {
  const hasPermissionPermission = await hasPermission('epsx:permissions:manage');
  const hasLegacyModule = await hasAdminModule('permission_admin');
  
  return (
    <ConditionalFeature condition={hasPermissionPermission || hasLegacyModule} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Module coordination features
 */
export async function ModuleCoordinatorFeature({ children, fallback }: FeatureGateProps) {
  const hasPackagePermission = await hasPermission('epsx:packages:manage');
  const hasLegacyModule = await hasAdminModule('package_coordinator');
  
  return (
    <ConditionalFeature condition={hasPackagePermission || hasLegacyModule} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Development-only admin features
 */
export async function DevAdminFeature({ children, fallback }: FeatureGateProps) {
  const isDev = process.env.NODE_ENV === 'development';
  const isAdmin = await isSystemAdmin();
  return (
    <ConditionalFeature condition={isDev && isAdmin} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}