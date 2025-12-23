/**
 * UNIFIED PERMISSION RESOLUTION UTILITIES
 * 
 * This file provides utilities to unify permission resolution between:
 * - Tier groups (new dynamic tier system)
 * - Permission groups (existing group system)
 * - Direct permissions (legacy/manual assignments)
 * 
 * All permissions use the same hasPermission() function from shared/config/iam.ts
 * ensuring consistent access control across the entire system.
 */

import { hasAllPermissions, hasAnyPermission, hasPermission } from '../config/iam';
import type {
  TierGroup,
  UnifiedUserPermissions,
  UserTierAssignment
} from '../types/tier-groups';

// ============================================================================
// UNIFIED PERMISSION RESOLUTION
// ============================================================================

export interface PermissionSource {
  id: string;
  name: string;
  type: 'tier_group' | 'permission_group' | 'direct';
  permissions: string[];
  expiresAt?: string;
  isActive: boolean;
}

export interface UserGroupMembership {
  id: string;
  groupId: string;
  groupName: string;
  permissions: string[];
  expiresAt?: string;
  isActive: boolean;
}

/**
 * Resolve all permissions for a user from all sources (tiers, groups, direct)
 * This creates a unified permission array that works with the existing hasPermission() function
 */
export function resolveUnifiedPermissions(
  tierAssignments: UserTierAssignment[] = [],
  tierGroups: TierGroup[] = [],
  groupMemberships: UserGroupMembership[] = [],
  directPermissions: string[] = []
): UnifiedUserPermissions {
  const now = new Date();
  const effectivePermissions = new Set<string>();
  const sources: UnifiedUserPermissions['sources'] = {
    tierGroups: [],
    directGroups: [],
    directPermissions: []
  };
  const expiringPermissions: UnifiedUserPermissions['expiringPermissions'] = [];

  // 1. Process Tier Group Permissions
  for (const assignment of tierAssignments) {
    if (!isActiveTierAssignment(assignment)) continue;

    const tierGroup = tierGroups.find(t => t.id === assignment.tierGroupId);
    if (!tierGroup || !tierGroup.isActive) continue;

    // Add tier group permissions
    const tierSource = {
      tierGroupId: tierGroup.id,
      tierGroupName: tierGroup.name,
      permissions: tierGroup.permissions,
      expiresAt: assignment.expiresAt
    };

    sources.tierGroups.push(tierSource);

    // Add permissions to effective set
    tierGroup.permissions.forEach(permission => {
      effectivePermissions.add(permission);
    });

    // Track expiring permissions
    if (assignment.expiresAt) {
      const expiryDate = new Date(assignment.expiresAt);
      const daysUntilExpiry = Math.ceil((expiryDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));

      tierGroup.permissions.forEach(permission => {
        expiringPermissions.push({
          permission,
          source: `Tier Group: ${tierGroup.name}`,
          expiresAt: assignment.expiresAt!,
          daysUntilExpiry
        });
      });
    }
  }

  // 2. Process Permission Group Memberships
  for (const membership of groupMemberships) {
    if (!membership.isActive) continue;

    // Check if membership has expired
    if (membership.expiresAt && new Date(membership.expiresAt) <= now) continue;

    const groupSource = {
      groupId: membership.groupId,
      groupName: membership.groupName,
      permissions: membership.permissions,
      expiresAt: membership.expiresAt
    };

    sources.directGroups.push(groupSource);

    // Add permissions to effective set
    membership.permissions.forEach(permission => {
      effectivePermissions.add(permission);
    });

    // Track expiring permissions
    if (membership.expiresAt) {
      const expiryDate = new Date(membership.expiresAt);
      const daysUntilExpiry = Math.ceil((expiryDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));

      membership.permissions.forEach(permission => {
        expiringPermissions.push({
          permission,
          source: `Group: ${membership.groupName}`,
          expiresAt: membership.expiresAt!,
          daysUntilExpiry
        });
      });
    }
  }

  // 3. Process Direct Permissions (Legacy)
  directPermissions.forEach(permission => {
    effectivePermissions.add(permission);
  });
  sources.directPermissions = [...directPermissions];

  // Sort expiring permissions by expiry date
  expiringPermissions.sort((a, b) =>
    new Date(a.expiresAt).getTime() - new Date(b.expiresAt).getTime()
  );

  return {
    userId: '', // Will be set by caller
    effectivePermissions: Array.from(effectivePermissions),
    sources,
    expiringPermissions,
    resolvedAt: new Date().toISOString(),
    cacheExpiresAt: new Date(Date.now() + 5 * 60 * 1000).toISOString(), // 5 minutes
    version: 1
  };
}

// ============================================================================
// UNIFIED PERMISSION CHECKING FUNCTIONS
// ============================================================================

/**
 * Check if user has permission using unified permission resolution
 * Uses the same hasPermission() function as the existing system
 */
export function hasUnifiedPermission(
  unifiedPermissions: UnifiedUserPermissions,
  requiredPermission: string
): boolean {
  return hasPermission(unifiedPermissions.effectivePermissions, requiredPermission);
}

/**
 * Check if user has any of the required permissions using unified resolution
 */
export function hasAnyUnifiedPermission(
  unifiedPermissions: UnifiedUserPermissions,
  requiredPermissions: string[]
): boolean {
  return hasAnyPermission(unifiedPermissions.effectivePermissions, requiredPermissions);
}

/**
 * Check if user has all required permissions using unified resolution
 */
export function hasAllUnifiedPermissions(
  unifiedPermissions: UnifiedUserPermissions,
  requiredPermissions: string[]
): boolean {
  return hasAllPermissions(unifiedPermissions.effectivePermissions, requiredPermissions);
}

// ============================================================================
// TIER GROUP UTILITIES
// ============================================================================

/**
 * Check if a tier assignment is currently active
 */
export function isActiveTierAssignment(assignment: UserTierAssignment): boolean {
  if (!assignment.isActive || assignment.status !== 'active') return false;

  if (assignment.expiresAt) {
    return new Date(assignment.expiresAt) > new Date();
  }

  return true;
}

/**
 * Get the highest tier group a user has access to
 * Useful for display purposes and upgrade prompts
 */
export function getHighestTierGroup(
  tierAssignments: UserTierAssignment[],
  tierGroups: TierGroup[]
): TierGroup | null {
  const activeTiers = tierAssignments
    .filter(isActiveTierAssignment)
    .map(assignment => tierGroups.find(t => t.id === assignment.tierGroupId))
    .filter(Boolean) as TierGroup[];

  if (activeTiers.length === 0) return null;

  // Sort by price (highest first) then by display order
  return activeTiers.sort((a, b) => {
    if (a.price !== b.price) return b.price - a.price;
    return a.displayOrder - b.displayOrder;
  })[0] || null;
}

/**
 * Get all active tier groups for a user
 */
export function getActiveTierGroups(
  tierAssignments: UserTierAssignment[],
  tierGroups: TierGroup[]
): TierGroup[] {
  return tierAssignments
    .filter(isActiveTierAssignment)
    .map(assignment => tierGroups.find(t => t.id === assignment.tierGroupId))
    .filter(Boolean) as TierGroup[];
}

/**
 * Check if user has access to a specific tier level
 * Compatible with legacy tier checking
 */
export function hasMinimumTier(
  unifiedPermissions: UnifiedUserPermissions,
  minimumTier: 'FREE' | 'BASIC' | 'PRO' | 'PREMIUM' | 'ENTERPRISE'
): boolean {
  // Map tier requirements to permission patterns
  const tierPermissionMap = {
    'FREE': ['epsx:analytics:view'],
    'BASIC': ['epsx:analytics:export'],
    'PRO': ['epsx:analytics:advanced', 'epsx:rankings:view:25'],
    'PREMIUM': ['epsx:analytics:premium', 'epsx:rankings:view:50'],
    'ENTERPRISE': ['epsx:*:*', 'epsx:rankings:view:unlimited']
  };

  const requiredPermissions = tierPermissionMap[minimumTier];
  return hasAnyUnifiedPermission(unifiedPermissions, requiredPermissions);
}

// ============================================================================
// PERMISSION EXPIRY UTILITIES
// ============================================================================

/**
 * Get permissions expiring within specified days
 */
export function getExpiringPermissions(
  unifiedPermissions: UnifiedUserPermissions,
  withinDays: number = 7
): UnifiedUserPermissions['expiringPermissions'] {
  return unifiedPermissions.expiringPermissions.filter(
    perm => perm.daysUntilExpiry <= withinDays && perm.daysUntilExpiry >= 0
  );
}

/**
 * Get permissions that have already expired
 */
export function getExpiredPermissions(
  unifiedPermissions: UnifiedUserPermissions
): UnifiedUserPermissions['expiringPermissions'] {
  return unifiedPermissions.expiringPermissions.filter(
    perm => perm.daysUntilExpiry < 0
  );
}

/**
 * Check if user needs permission renewal warning
 */
export function needsRenewalWarning(
  unifiedPermissions: UnifiedUserPermissions,
  warningThreshold: number = 7
): boolean {
  const expiring = getExpiringPermissions(unifiedPermissions, warningThreshold);
  return expiring.length > 0;
}

// ============================================================================
// LEGACY COMPATIBILITY
// ============================================================================

/**
 * Convert legacy package tier to tier group permissions check
 * This allows gradual migration from old tier checking to new system
 */
export function legacyTierToPermissionCheck(
  legacyTier: string,
  unifiedPermissions: UnifiedUserPermissions
): boolean {
  switch (legacyTier) {
    case 'FREE':
      return hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:view');
    case 'BASIC':
      return hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:export');
    case 'PRO':
      return hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:advanced');
    case 'PREMIUM':
      return hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:premium');
    case 'ENTERPRISE':
      return hasUnifiedPermission(unifiedPermissions, 'epsx:*:*');
    default:
      return false;
  }
}

/**
 * Get tier display name from permissions for legacy compatibility
 */
export function getTierDisplayFromPermissions(
  unifiedPermissions: UnifiedUserPermissions
): string {
  if (hasUnifiedPermission(unifiedPermissions, 'epsx:*:*')) {
    return 'ENTERPRISE';
  } else if (hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:premium')) {
    return 'PREMIUM';
  } else if (hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:advanced')) {
    return 'PRO';
  } else if (hasUnifiedPermission(unifiedPermissions, 'epsx:analytics:export')) {
    return 'BASIC';
  } else {
    return 'FREE';
  }
}

// ============================================================================
// CACHE UTILITIES
// ============================================================================

/**
 * Check if unified permissions cache is still valid
 */
export function isPermissionCacheValid(unifiedPermissions: UnifiedUserPermissions): boolean {
  return new Date(unifiedPermissions.cacheExpiresAt) > new Date();
}

/**
 * Generate cache key for user permissions
 */
export function generatePermissionCacheKey(userId: string): string {
  return `unified_permissions:${userId}:v${Date.now()}`;
}

/**
 * Create a minimal permissions object for caching
 */
export function createCacheablePermissions(
  unifiedPermissions: UnifiedUserPermissions
): Pick<UnifiedUserPermissions, 'userId' | 'effectivePermissions' | 'resolvedAt' | 'cacheExpiresAt'> {
  return {
    userId: unifiedPermissions.userId,
    effectivePermissions: unifiedPermissions.effectivePermissions,
    resolvedAt: unifiedPermissions.resolvedAt,
    cacheExpiresAt: unifiedPermissions.cacheExpiresAt
  };
}