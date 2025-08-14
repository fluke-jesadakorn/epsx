/**
 * Server-Side Feature Gates for Frontend
 * Conditionally render features based on JWT authentication
 */
import { ReactNode } from 'react';
import { getAuthUser, hasPermission, hasPackageTier, hasRole } from '@/lib/server/auth';

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
 * Show content only for authenticated users
 */
export async function AuthenticatedFeature({ children, fallback }: FeatureGateProps) {
  const user = await getAuthUser();
  return (
    <ConditionalFeature condition={!!user} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on permission
 */
interface PermissionFeatureProps extends FeatureGateProps {
  permission: string;
}

export async function PermissionFeature({ 
  permission, 
  children, 
  fallback 
}: PermissionFeatureProps) {
  const hasRequiredPermission = await hasPermission(permission);
  return (
    <ConditionalFeature condition={hasRequiredPermission} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on package tier
 */
interface PackageTierFeatureProps extends FeatureGateProps {
  tier: string;
}

export async function PackageTierFeature({ 
  tier, 
  children, 
  fallback 
}: PackageTierFeatureProps) {
  const hasRequiredTier = await hasPackageTier(tier);
  return (
    <ConditionalFeature condition={hasRequiredTier} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Show content based on user role
 */
interface RoleFeatureProps extends FeatureGateProps {
  role: string;
}

export async function RoleFeature({ 
  role, 
  children, 
  fallback 
}: RoleFeatureProps) {
  const hasRequiredRole = await hasRole(role);
  return (
    <ConditionalFeature condition={hasRequiredRole} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Premium features (Bronze tier and above)
 */
export async function PremiumFeature({ children, fallback }: FeatureGateProps) {
  return (
    <PackageTierFeature tier="BRONZE" fallback={fallback}>
      {children}
    </PackageTierFeature>
  );
}

/**
 * Enterprise features (Enterprise tier only)
 */
export async function EnterpriseFeature({ children, fallback }: FeatureGateProps) {
  return (
    <PackageTierFeature tier="ENTERPRISE" fallback={fallback}>
      {children}
    </PackageTierFeature>
  );
}

/**
 * Development-only features
 */
export async function DevFeature({ children, fallback }: FeatureGateProps) {
  const isDev = process.env.NODE_ENV === 'development';
  return (
    <ConditionalFeature condition={isDev} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}