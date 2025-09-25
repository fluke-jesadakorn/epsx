'use client';

import { useAuth } from '@/lib/auth';
import { useMemo } from 'react';

// Package tiers in order from lowest to highest
const PACKAGE_TIERS = ['FREE', 'BRONZE', 'SILVER', 'GOLD', 'PLATINUM', 'ENTERPRISE'] as const;
type PackageTier = typeof PACKAGE_TIERS[number];

interface VisualPermissionResult {
  hasPermission: boolean;
  restrictionReason?: string;
  requiredTier?: PackageTier;
  canRequest: boolean;
  isAdmin: boolean;
  currentTier: PackageTier;
  upgradeMessage?: string;
  requestMessage?: string;
}

interface UseVisualPermissionOptions {
  permission?: string;
  requiredTier?: PackageTier;
  feature?: string;
  customCheck?: (user: any) => boolean;
}

/**
 * Hook for checking permissions and providing visual permission information
 * Supports both permission-based and tier-based restrictions
 */
export function useVisualPermission({
  permission,
  requiredTier,
  feature,
  customCheck
}: UseVisualPermissionOptions): VisualPermissionResult {
  const { user, isAuthenticated, isLoading } = useAuth();
  
  const result = useMemo<VisualPermissionResult>(() => {
    // Default result for no user
    if (!user || !isAuthenticated) {
      return {
        hasPermission: false,
        restrictionReason: 'Authentication required',
        canRequest: false,
        isAdmin: false,
        currentTier: 'FREE',
        upgradeMessage: 'Please sign in to access this feature',
        requestMessage: 'Sign in required'
      };
    }

    const userPermissions = Array.isArray(user.permissions) ? user.permissions : [];
    const userTier = 'ENTERPRISE' as PackageTier; // Web3 users are enterprise tier
    const isAdmin = userPermissions.includes('admin:*:*') || 
                   userPermissions.some(p => p.startsWith('admin:'));

    // Custom check takes precedence
    if (customCheck) {
      const hasPermission = customCheck(user);
      return {
        hasPermission,
        restrictionReason: hasPermission ? undefined : 'Custom access restriction',
        canRequest: !isAdmin, // Only non-admins can request permissions
        isAdmin,
        currentTier: userTier,
        upgradeMessage: 'Contact admin for special access',
        requestMessage: 'Request access from administrator'
      };
    }

    // Check tier-based access
    if (requiredTier) {
      const currentTierIndex = PACKAGE_TIERS.indexOf(userTier);
      const requiredTierIndex = PACKAGE_TIERS.indexOf(requiredTier);
      const hasPermission = isAdmin || currentTierIndex >= requiredTierIndex;
      
      return {
        hasPermission,
        restrictionReason: hasPermission ? undefined : `Requires ${requiredTier} tier or higher`,
        requiredTier,
        canRequest: false, // Tier access requires upgrade, not request
        isAdmin,
        currentTier: userTier,
        upgradeMessage: `Upgrade to ${requiredTier} to unlock this feature`,
        requestMessage: `${requiredTier} tier required`
      };
    }

    // Check specific permission
    if (permission) {
      const hasPermission = checkUserPermission(userPermissions, permission);
      
      return {
        hasPermission,
        restrictionReason: hasPermission ? undefined : `Requires permission: ${permission}`,
        canRequest: !isAdmin, // Admins don't need to request permissions
        isAdmin,
        currentTier: userTier,
        upgradeMessage: isAdmin ? undefined : 'Contact admin for permission access',
        requestMessage: `Permission required: ${permission}`
      };
    }

    // Check legacy feature access
    if (feature) {
      const mappedPermission = mapFeatureToPermission(feature);
      if (mappedPermission) {
        const hasPermission = checkUserPermission(userPermissions, mappedPermission);
        
        return {
          hasPermission,
          restrictionReason: hasPermission ? undefined : `Feature "${feature}" access restricted`,
          canRequest: !isAdmin,
          isAdmin,
          currentTier: userTier,
          upgradeMessage: 'Contact admin for feature access',
          requestMessage: `Feature access required: ${feature}`
        };
      }
    }

    // Default: assume access granted if no restrictions specified
    return {
      hasPermission: true,
      canRequest: false,
      isAdmin,
      currentTier: userTier
    };
  }, [user, isAuthenticated, permission, requiredTier, feature, customCheck]);

  return result;
}

/**
 * Check if user has a specific permission
 * Supports wildcard permissions and hierarchical checking
 */
function checkUserPermission(userPermissions: string[], requiredPermission: string): boolean {
  // Check for admin wildcard
  if (userPermissions.includes('admin:*:*')) {
    return true;
  }

  // Check for exact match
  if (userPermissions.includes(requiredPermission)) {
    return true;
  }

  // Check for wildcard matches
  const [platform, resource, action] = requiredPermission.split(':');
  
  if (platform && resource) {
    // Check for platform:resource:* permission
    if (userPermissions.includes(`${platform}:${resource}:*`)) {
      return true;
    }
    
    // Check for platform:*:* permission
    if (userPermissions.includes(`${platform}:*:*`)) {
      return true;
    }
  }

  return false;
}

/**
 * Map legacy feature names to structured permissions
 */
function mapFeatureToPermission(feature: string): string | null {
  const featureMap: Record<string, string> = {
    'user-management': 'admin:users:manage',
    'analytics-access': 'admin:analytics:view',
    'system-admin': 'admin:system:manage',
    'security-management': 'admin:security:manage',
    'content-management': 'admin:content:manage',
    'notification-management': 'admin:notifications:manage'
  };

  return featureMap[feature] || null;
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

    const userPermissions = Array.isArray(user.permissions) ? user.permissions : [];
    const grantedPermissions: string[] = [];
    const deniedPermissions: string[] = [];

    permissions.forEach(permission => {
      if (checkUserPermission(userPermissions, permission)) {
        grantedPermissions.push(permission);
      } else {
        deniedPermissions.push(permission);
      }
    });

    const hasPermission = grantedPermissions.length > 0;

    return {
      ...baseResult,
      hasPermission,
      restrictionReason: hasPermission ? undefined : `Requires any of: ${deniedPermissions.join(', ')}`,
      grantedPermissions,
      deniedPermissions
    };
  }, [user, isAuthenticated, permissions]);
}

/**
 * Hook for checking tier-based access with upgrade suggestions
 */
export function useVisualTierAccess(requiredTier: PackageTier): VisualPermissionResult & {
  canUpgrade: boolean;
  nextTier?: PackageTier;
  tierGap: number;
} {
  const baseResult = useVisualPermission({ requiredTier });
  
  return useMemo(() => {
    const currentTierIndex = PACKAGE_TIERS.indexOf(baseResult.currentTier);
    const requiredTierIndex = PACKAGE_TIERS.indexOf(requiredTier);
    const tierGap = requiredTierIndex - currentTierIndex;
    
    const canUpgrade = !baseResult.isAdmin && tierGap > 0;
    const nextTier = canUpgrade && currentTierIndex < PACKAGE_TIERS.length - 1 
      ? PACKAGE_TIERS[currentTierIndex + 1] 
      : undefined;

    return {
      ...baseResult,
      canUpgrade,
      nextTier,
      tierGap
    };
  }, [baseResult, requiredTier]);
}

/**
 * Utility hook for admin-specific checks
 */
export function useAdminVisualPermission(requiredAdminLevel: 'basic' | 'full' = 'basic'): VisualPermissionResult {
  const permission = requiredAdminLevel === 'full' ? 'admin:*:*' : 'admin:access:basic';
  return useVisualPermission({ permission });
}

/**
 * Hook for checking multiple conditions (permission AND tier)
 */
export function useComplexVisualPermission({
  permission,
  requiredTier,
  requireBoth = false // If true, both permission AND tier required. If false, either is sufficient.
}: {
  permission?: string;
  requiredTier?: PackageTier;
  requireBoth?: boolean;
}): VisualPermissionResult {
  const permissionResult = useVisualPermission({ permission });
  const tierResult = useVisualPermission({ requiredTier });
  
  return useMemo(() => {
    if (!permission && !requiredTier) {
      return { 
        hasPermission: true, 
        canRequest: false, 
        isAdmin: permissionResult.isAdmin,
        currentTier: permissionResult.currentTier
      };
    }

    if (permission && !requiredTier) return permissionResult;
    if (!permission && requiredTier) return tierResult;

    // Both specified
    const hasPermission = requireBoth 
      ? permissionResult.hasPermission && tierResult.hasPermission
      : permissionResult.hasPermission || tierResult.hasPermission;

    let restrictionReason: string | undefined;
    let upgradeMessage: string | undefined;

    if (!hasPermission) {
      if (requireBoth) {
        const reasons = [
          !permissionResult.hasPermission ? permissionResult.restrictionReason : null,
          !tierResult.hasPermission ? tierResult.restrictionReason : null
        ].filter(Boolean);
        restrictionReason = reasons.join(' AND ');
      } else {
        restrictionReason = `Requires either ${permissionResult.restrictionReason} OR ${tierResult.restrictionReason}`;
      }
    }

    return {
      hasPermission,
      restrictionReason,
      requiredTier,
      canRequest: permissionResult.canRequest || tierResult.canRequest,
      isAdmin: permissionResult.isAdmin,
      currentTier: permissionResult.currentTier,
      upgradeMessage: tierResult.upgradeMessage || permissionResult.upgradeMessage,
      requestMessage: permissionResult.requestMessage
    };
  }, [permissionResult, tierResult, requireBoth, permission, requiredTier]);
}