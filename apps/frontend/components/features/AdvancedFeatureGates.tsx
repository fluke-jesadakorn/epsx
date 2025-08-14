/**
 * Advanced Server-Side Feature Gates
 * Complex feature gating with multiple conditions
 */
import { ReactNode } from 'react';
import { getAuthUser } from '@/lib/server/auth';
import { ConditionalFeature } from './FeatureGate';

interface FeatureGateProps {
  children: ReactNode;
  fallback?: ReactNode;
}

/**
 * Multiple package tier requirements (any tier)
 */
interface MultiTierFeatureProps extends FeatureGateProps {
  tiers: string[];
}

export async function MultiTierFeature({ 
  tiers, 
  children, 
  fallback 
}: MultiTierFeatureProps) {
  const user = await getAuthUser();
  
  if (!user) {
    return <>{fallback}</>;
  }
  
  const tierHierarchy: Record<string, number> = {
    'FREE': 1,
    'BRONZE': 2,
    'SILVER': 3,
    'GOLD': 4,
    'PLATINUM': 5,
    'ENTERPRISE': 6
  };
  
  const userLevel = tierHierarchy[user.package_tier] || 0;
  const hasAnyTier = tiers.some(tier => {
    const requiredLevel = tierHierarchy[tier] || 1;
    return userLevel >= requiredLevel;
  });
  
  return (
    <ConditionalFeature condition={hasAnyTier} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Multiple permission requirements (any permission)
 */
interface MultiPermissionFeatureProps extends FeatureGateProps {
  permissions: string[];
}

export async function MultiPermissionFeature({ 
  permissions, 
  children, 
  fallback 
}: MultiPermissionFeatureProps) {
  const user = await getAuthUser();
  
  if (!user) {
    return <>{fallback}</>;
  }
  
  const hasAnyPermission = permissions.some(permission => 
    user.permissions.includes(permission) || user.permissions.includes('*')
  );
  
  return (
    <ConditionalFeature condition={hasAnyPermission} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * AND condition feature gate
 */
interface AndFeatureProps extends FeatureGateProps {
  conditions: boolean[];
}

export async function AndFeature({ 
  conditions, 
  children, 
  fallback 
}: AndFeatureProps) {
  const allConditionsTrue = conditions.every(condition => condition === true);
  
  return (
    <ConditionalFeature condition={allConditionsTrue} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * OR condition feature gate
 */
interface OrFeatureProps extends FeatureGateProps {
  conditions: boolean[];
}

export async function OrFeature({ 
  conditions, 
  children, 
  fallback 
}: OrFeatureProps) {
  const anyConditionTrue = conditions.some(condition => condition === true);
  
  return (
    <ConditionalFeature condition={anyConditionTrue} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Time-based feature gate
 */
interface TimeBasedFeatureProps extends FeatureGateProps {
  startDate?: Date;
  endDate?: Date;
}

export async function TimeBasedFeature({ 
  startDate, 
  endDate, 
  children, 
  fallback 
}: TimeBasedFeatureProps) {
  const now = new Date();
  
  let isWithinTimeRange = true;
  
  if (startDate) {
    isWithinTimeRange = isWithinTimeRange && now >= startDate;
  }
  
  if (endDate) {
    isWithinTimeRange = isWithinTimeRange && now <= endDate;
  }
  
  return (
    <ConditionalFeature condition={isWithinTimeRange} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}

/**
 * Environment-based feature gate
 */
interface EnvironmentFeatureProps extends FeatureGateProps {
  environments: string[];
}

export async function EnvironmentFeature({ 
  environments, 
  children, 
  fallback 
}: EnvironmentFeatureProps) {
  const currentEnv = process.env.NODE_ENV || 'development';
  const isAllowedEnv = environments.includes(currentEnv);
  
  return (
    <ConditionalFeature condition={isAllowedEnv} fallback={fallback}>
      {children}
    </ConditionalFeature>
  );
}