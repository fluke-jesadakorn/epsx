'use client';

import { useAuth } from '@/lib/auth';
import { useMemo } from 'react';
import { hasPermission, hasAnyPermission } from '@/types/permissions';

// Package tiers in order from lowest to highest
const PACKAGE_TIERS = ['FREE', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'] as const;
type PackageTier = typeof PACKAGE_TIERS[number];

interface VisualPermissionResult {
  hasPermission: boolean;
  restrictionReason?: string;
  requiredTier?: PackageTier;
  canUpgrade: boolean;
  isLoggedIn: boolean;
  currentTier: PackageTier;
  upgradeMessage?: string;
  loginMessage?: string;
  nextTier?: PackageTier;
  tierGap?: number;
}

interface UseVisualPermissionOptions {
  permission?: string;
  requiredTier?: PackageTier;
  feature?: string;
  customCheck?: (user: any) => boolean;
  requireAuth?: boolean;
}

/**
 * Hook for checking permissions and providing visual permission information
 * Optimized for frontend user features and package tiers
 */
export function useVisualPermission({
  permission,
  requiredTier,
  feature,
  customCheck,
  requireAuth = true
}: UseVisualPermissionOptions): VisualPermissionResult {
  const { user, isAuthenticated, isLoading } = useAuth();
  
  const result = useMemo<VisualPermissionResult>(() => {
    // Show loading state while authentication is being determined
    if (isLoading) {
      return {
        hasPermission: false,
        restrictionReason: 'Loading...',
        canUpgrade: false,
        isLoggedIn: false,
        currentTier: 'FREE'
      };
    }

    // Handle unauthenticated users
    if (requireAuth && !isAuthenticated) {
      return {
        hasPermission: false,
        restrictionReason: 'Login required',
        canUpgrade: false,
        isLoggedIn: false,
        currentTier: 'FREE',
        upgradeMessage: 'Sign in to access premium features',
        loginMessage: 'Please sign in to continue'
      };
    }

    const userTier = (user?.package_tier || 'FREE') as PackageTier;
    const userPermissions = Array.isArray(user?.permissions) ? user.permissions : [];
    const currentTierIndex = PACKAGE_TIERS.indexOf(userTier);

    // Custom check takes precedence
    if (customCheck) {
      const hasPermission = customCheck(user);
      const canUpgrade = isAuthenticated && !hasPermission && currentTierIndex < PACKAGE_TIERS.length - 1;
      
      return {
        hasPermission,
        restrictionReason: hasPermission ? undefined : 'Feature access restricted',
        canUpgrade,
        isLoggedIn: isAuthenticated,
        currentTier: userTier,
        nextTier: canUpgrade ? PACKAGE_TIERS[currentTierIndex + 1] : undefined,
        upgradeMessage: canUpgrade ? 'Upgrade your plan to access this feature' : undefined
      };
    }

    // Check tier-based access
    if (requiredTier) {
      const requiredTierIndex = PACKAGE_TIERS.indexOf(requiredTier);
      const hasPermission = currentTierIndex >= requiredTierIndex;
      const tierGap = requiredTierIndex - currentTierIndex;
      const canUpgrade = isAuthenticated && !hasPermission && tierGap > 0;
      
      return {
        hasPermission,
        restrictionReason: hasPermission ? undefined : `${requiredTier} tier required`,
        requiredTier,
        canUpgrade,
        isLoggedIn: isAuthenticated,
        currentTier: userTier,
        nextTier: canUpgrade ? PACKAGE_TIERS[currentTierIndex + 1] : undefined,
        tierGap: tierGap > 0 ? tierGap : 0,
        upgradeMessage: canUpgrade ? `Upgrade to ${requiredTier} to unlock this feature` : undefined,
        loginMessage: !isAuthenticated ? 'Sign in to check your tier access' : undefined
      };
    }

    // Check specific permission
    if (permission && user) {
      const hasPermissionResult = hasPermission(user, permission);
      
      return {
        hasPermission: hasPermissionResult,
        restrictionReason: hasPermissionResult ? undefined : `Permission required: ${permission}`,
        canUpgrade: isAuthenticated && !hasPermissionResult && canUpgradeForPermission(permission, userTier),
        isLoggedIn: isAuthenticated,
        currentTier: userTier,
        nextTier: isAuthenticated && !hasPermissionResult ? getNextTierForPermission(permission, userTier) : undefined,
        upgradeMessage: !hasPermissionResult && isAuthenticated ? getUpgradeMessageForPermission(permission, userTier) : undefined,
        loginMessage: !isAuthenticated ? 'Sign in to access this feature' : undefined
      };
    }

    // Check legacy feature access
    if (feature) {
      const mappedTier = mapFeatureToTier(feature);
      if (mappedTier) {
        return useVisualPermission({ requiredTier: mappedTier });
      }
      
      const mappedPermission = mapFeatureToPermission(feature);
      if (mappedPermission) {
        return useVisualPermission({ permission: mappedPermission });
      }
    }

    // Default: assume access granted if no restrictions specified
    return {
      hasPermission: true,
      canUpgrade: false,
      isLoggedIn: isAuthenticated,
      currentTier: userTier
    };
  }, [user, isAuthenticated, isLoading, permission, requiredTier, feature, customCheck, requireAuth]);

  return result;
}

/**
 * Hook variant for multiple permissions (any of them)
 */
export function useVisualPermissions(permissions: string[]): VisualPermissionResult & { 
  grantedPermissions: string[];
  deniedPermissions: string[];
} {
  const { user, isAuthenticated } = useAuth();
  
  return useMemo(() => {
    const baseResult = useVisualPermission({ permission: permissions[0] });
    
    if (!user || !isAuthenticated) {
      return {
        ...baseResult,
        grantedPermissions: [],
        deniedPermissions: permissions
      };
    }

    const grantedPermissions: string[] = [];
    const deniedPermissions: string[] = [];

    permissions.forEach(permission => {
      if (hasPermission(user, permission)) {
        grantedPermissions.push(permission);
      } else {
        deniedPermissions.push(permission);
      }
    });

    const hasAnyPermissionResult = grantedPermissions.length > 0;

    return {
      ...baseResult,
      hasPermission: hasAnyPermissionResult,
      restrictionReason: hasAnyPermissionResult ? undefined : `Requires any of: ${deniedPermissions.join(', ')}`,
      grantedPermissions,
      deniedPermissions
    };
  }, [user, isAuthenticated, permissions]);
}

/**
 * Hook for checking tier-based access with upgrade paths
 */
export function useVisualTierAccess(requiredTier: PackageTier): VisualPermissionResult & {
  upgradePath: PackageTier[];
  costDifference?: string;
} {
  const baseResult = useVisualPermission({ requiredTier });
  
  return useMemo(() => {
    const currentTierIndex = PACKAGE_TIERS.indexOf(baseResult.currentTier);
    const requiredTierIndex = PACKAGE_TIERS.indexOf(requiredTier);
    
    const upgradePath = currentTierIndex < requiredTierIndex 
      ? PACKAGE_TIERS.slice(currentTierIndex + 1, requiredTierIndex + 1)
      : [];

    return {
      ...baseResult,
      upgradePath
    };
  }, [baseResult, requiredTier]);
}

/**
 * Hook for analytics-specific features
 */
export function useAnalyticsVisualAccess(): {
  viewAccess: VisualPermissionResult;
  exportAccess: VisualPermissionResult;
  advancedAccess: VisualPermissionResult;
  realtimeAccess: VisualPermissionResult;
} {
  const viewAccess = useVisualPermission({ permission: 'epsx:analytics:view' });
  const exportAccess = useVisualPermission({ permission: 'epsx:analytics:export', requiredTier: 'SILVER' });
  const advancedAccess = useVisualPermission({ permission: 'epsx:analytics:advanced', requiredTier: 'GOLD' });
  const realtimeAccess = useVisualPermission({ permission: 'epsx:realtime:access', requiredTier: 'PLATINUM' });

  return {
    viewAccess,
    exportAccess,
    advancedAccess,
    realtimeAccess
  };
}

/**
 * Utility functions
 */

function canUpgradeForPermission(permission: string, currentTier: PackageTier): boolean {
  const tierPermissionMap: Record<string, PackageTier> = {
    'epsx:analytics:view': 'FREE',
    'epsx:analytics:export': 'SILVER', 
    'epsx:analytics:advanced': 'GOLD',
    'epsx:realtime:access': 'PLATINUM',
    'epsx:notifications:premium': 'GOLD',
    'epsx:profile:advanced': 'SILVER'
  };

  const requiredTier = tierPermissionMap[permission];
  if (!requiredTier) return false;

  const currentIndex = PACKAGE_TIERS.indexOf(currentTier);
  const requiredIndex = PACKAGE_TIERS.indexOf(requiredTier);
  
  return currentIndex < requiredIndex;
}

function getNextTierForPermission(permission: string, currentTier: PackageTier): PackageTier | undefined {
  const tierPermissionMap: Record<string, PackageTier> = {
    'epsx:analytics:export': 'SILVER',
    'epsx:analytics:advanced': 'GOLD', 
    'epsx:realtime:access': 'PLATINUM',
    'epsx:notifications:premium': 'GOLD',
    'epsx:profile:advanced': 'SILVER'
  };

  const requiredTier = tierPermissionMap[permission];
  if (!requiredTier) return undefined;

  const currentIndex = PACKAGE_TIERS.indexOf(currentTier);
  const requiredIndex = PACKAGE_TIERS.indexOf(requiredTier);
  
  if (currentIndex < requiredIndex) {
    return PACKAGE_TIERS[currentIndex + 1];
  }

  return undefined;
}

function getUpgradeMessageForPermission(permission: string, currentTier: PackageTier): string | undefined {
  const nextTier = getNextTierForPermission(permission, currentTier);
  if (!nextTier) return undefined;

  const featureMessages: Record<string, string> = {
    'epsx:analytics:export': 'Export analytics data',
    'epsx:analytics:advanced': 'Use advanced analytics features',
    'epsx:realtime:access': 'Access real-time data streams', 
    'epsx:notifications:premium': 'Get premium notifications',
    'epsx:profile:advanced': 'Access advanced profile settings'
  };

  const featureMessage = featureMessages[permission] || 'Access this feature';
  return `Upgrade to ${nextTier} to ${featureMessage.toLowerCase()}`;
}

function mapFeatureToTier(feature: string): PackageTier | null {
  const featureTierMap: Record<string, PackageTier> = {
    'analytics-export': 'SILVER',
    'advanced-analytics': 'GOLD', 
    'realtime-data': 'PLATINUM',
    'premium-notifications': 'GOLD',
    'advanced-profile': 'SILVER'
  };

  return featureTierMap[feature] || null;
}

function mapFeatureToPermission(feature: string): string | null {
  const featurePermissionMap: Record<string, string> = {
    'view-analytics': 'epsx:analytics:view',
    'export-data': 'epsx:analytics:export',
    'advanced-filters': 'epsx:analytics:advanced',
    'realtime-updates': 'epsx:realtime:access',
    'premium-notifications': 'epsx:notifications:premium',
    'profile-management': 'epsx:profile:manage'
  };

  return featurePermissionMap[feature] || null;
}