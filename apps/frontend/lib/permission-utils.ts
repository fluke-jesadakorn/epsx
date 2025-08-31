// ============================================================================
// FRONTEND PERMISSION UTILITIES - DYNAMIC RANKING SYSTEM WITH EMBEDDED TIMESTAMPS
// ============================================================================
// Utilities for parsing structured permissions and extracting ranking limits
// Format: "epsx:rankings:view:N" where N is the numeric limit
// Format with timestamp: "epsx:rankings:view:N:unix_timestamp" or "epsx:rankings:view:unlimited:unix_timestamp"

import type { UserLevelType } from '@/app/constants/packages';
import { 
  filterValidPermissions, 
  parsePermissionWithTimestamp, 
  isPermissionValidWithTime,
  createTimestampedPermission,
  hasPermissionWithTime,
  type TimestampedPermission
} from '@/types/permissions';

// ============================================================================
// RANKING LIMIT EXTRACTION
// ============================================================================

/**
 * Extract ranking limit from permissions array with embedded timestamp support
 * Parses permissions like "epsx:rankings:view:25" or "epsx:rankings:view:25:1703980800" to return the numeric limit
 * Automatically filters out expired permissions
 */
export function extractRankingLimitFromPermissions(permissions: string[]): number {
  // First filter out expired permissions
  const validPermissions = filterValidPermissions(permissions);
  
  for (const perm of validPermissions) {
    if (perm.startsWith('epsx:rankings:view:')) {
      const parts = perm.split(':');
      
      // Handle both formats: epsx:rankings:view:25 and epsx:rankings:view:25:timestamp
      const limitPart = parts[3];
      
      if (limitPart === 'unlimited') {
        return -1; // -1 represents unlimited access
      }
      
      const parsed = parseInt(limitPart, 10);
      if (!isNaN(parsed)) {
        return parsed;
      }
    }
  }
  return 5; // Default fallback to basic access (Bronze level)
}

/**
 * Check if user has specific permission with embedded timestamp validation
 * Automatically filters out expired permissions
 */
export function hasPermission(permissions: string[], required: string): boolean {
  // First filter out expired permissions
  const validPermissions = filterValidPermissions(permissions);
  
  // Check for exact match first
  for (const perm of validPermissions) {
    const { basePermission } = parsePermissionWithTimestamp(perm);
    if (basePermission === required) {
      return true;
    }
  }
  
  // Check for wildcard matches
  for (const perm of validPermissions) {
    const { basePermission } = parsePermissionWithTimestamp(perm);
    
    if (basePermission.endsWith(':*:*') || basePermission.endsWith(':*')) {
      const permParts = basePermission.split(':');
      const requiredParts = required.split(':');
      
      // Admin wildcard check
      if (basePermission === 'admin:*:*') {
        return true;
      }
      
      // Platform wildcard check (epsx:*:*)
      if (permParts.length === 3 && permParts[1] === '*' && permParts[2] === '*') {
        if (requiredParts[0] === permParts[0]) {
          return true;
        }
      }
      
      // Resource wildcard check (epsx:trading:*)
      if (permParts.length === 3 && permParts[2] === '*') {
        if (requiredParts[0] === permParts[0] && requiredParts[1] === permParts[1]) {
          return true;
        }
      }
    }
  }
  
  return false;
}

/**
 * Check if user can view specific ranking position with embedded timestamp validation
 * Automatically filters out expired permissions
 */
export function canViewRankingPosition(permissions: string[], position: number): boolean {
  const limit = extractRankingLimitFromPermissions(permissions);
  
  // Unlimited access
  if (limit === -1) {
    return true;
  }
  
  // Check if position is within limit
  return position <= limit;
}

// ============================================================================
// EMBEDDED TIMESTAMP UTILITIES
// ============================================================================

/**
 * Get ranking permissions with expiry information
 */
export function getRankingPermissionsWithExpiry(permissions: string[]): TimestampedPermission[] {
  return permissions
    .filter(perm => perm.startsWith('epsx:rankings:view:'))
    .map(createTimestampedPermission)
    .filter(tp => !tp.isExpired);
}

/**
 * Get the most permissive ranking limit that hasn't expired
 * Returns the highest limit among valid permissions
 */
export function getBestRankingLimit(permissions: string[]): number {
  const validPermissions = filterValidPermissions(permissions);
  let bestLimit = 5; // Default Bronze level
  
  for (const perm of validPermissions) {
    if (perm.startsWith('epsx:rankings:view:')) {
      const parts = perm.split(':');
      const limitPart = parts[3];
      
      if (limitPart === 'unlimited') {
        return -1; // Return immediately for unlimited
      }
      
      const parsed = parseInt(limitPart, 10);
      if (!isNaN(parsed) && (parsed > bestLimit || bestLimit === 5)) {
        bestLimit = parsed;
      }
    }
  }
  
  return bestLimit;
}

/**
 * Check if user has any ranking permissions that expire soon (within 24 hours)
 */
export function hasExpiringSoonRankingPermissions(permissions: string[]): boolean {
  const rankingPerms = getRankingPermissionsWithExpiry(permissions);
  const twentyFourHoursFromNow = Date.now() + (24 * 60 * 60 * 1000);
  
  return rankingPerms.some(tp => 
    tp.expiresAt && (tp.expiresAt * 1000) <= twentyFourHoursFromNow
  );
}

/**
 * Get next expiring ranking permission
 */
export function getNextExpiringRankingPermission(permissions: string[]): TimestampedPermission | null {
  const rankingPerms = getRankingPermissionsWithExpiry(permissions)
    .filter(tp => tp.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0));
  
  return rankingPerms[0] || null;
}

// ============================================================================
// TIER DERIVATION FOR UI COMPATIBILITY
// ============================================================================

/**
 * Derive tier display name from permissions for UI compatibility
 * This maintains the familiar tier names while using permission-based backend
 */
export function deriveTierFromPermissions(permissions: string[]): UserLevelType {
  const limit = extractRankingLimitFromPermissions(permissions);
  
  switch (limit) {
    case 5:
      return 'BRONZE';
    case 25:
      return 'SILVER';
    case 50:
      return 'GOLD';
    case 100:
      return 'PLATINUM';
    case -1:
      return 'VIP'; // Unlimited maps to VIP for display
    default:
      // Handle custom limits by mapping to appropriate tier ranges
      if (limit > 0 && limit <= 10) {
        return 'BRONZE';
      } else if (limit <= 30) {
        return 'SILVER';
      } else if (limit <= 75) {
        return 'GOLD';
      } else if (limit <= 150) {
        return 'PLATINUM';
      } else {
        return 'VIP';
      }
  }
}

/**
 * Derive tier from ranking limit (for backward compatibility)
 */
export function deriveTierFromRankingLimit(limit: number): UserLevelType {
  return deriveTierFromPermissions([`epsx:rankings:view:${limit === -1 ? 'unlimited' : limit}`]);
}

// ============================================================================
// PERMISSION-BASED PACKAGE COMPATIBILITY
// ============================================================================

/**
 * Get package-like information from permissions for UI compatibility
 */
export interface PermissionBasedPackage {
  rankingLimit: number;
  tier: UserLevelType;
  hasAdvancedTrading: boolean;
  hasPortfolioTools: boolean;
  hasPrioritySupport: boolean;
  hasResearchReports: boolean;
  hasCustomDashboards: boolean;
}

export function getPackageFromPermissions(permissions: string[]): PermissionBasedPackage {
  const rankingLimit = extractRankingLimitFromPermissions(permissions);
  const tier = deriveTierFromPermissions(permissions);
  
  return {
    rankingLimit,
    tier,
    hasAdvancedTrading: hasPermission(permissions, 'epsx:trading:advanced') || hasPermission(permissions, 'epsx:trading:premium'),
    hasPortfolioTools: hasPermission(permissions, 'epsx:portfolio:tools'),
    hasPrioritySupport: hasPermission(permissions, 'epsx:support:priority'),
    hasResearchReports: hasPermission(permissions, 'epsx:research:reports'),
    hasCustomDashboards: hasPermission(permissions, 'epsx:dashboards:custom'),
  };
}

// ============================================================================
// CONVERSION HELPERS
// ============================================================================

/**
 * Convert tier to permission set (for admin interface compatibility)
 */
export function convertTierToPermissions(tier: UserLevelType): string[] {
  switch (tier) {
    case 'BRONZE':
      return [
        'epsx:rankings:view:5',
        'epsx:trading:basic',
        'epsx:portfolio:view',
        'epsx:notifications:basic',
      ];
    
    case 'SILVER':
      return [
        'epsx:rankings:view:25',
        'epsx:trading:basic',
        'epsx:trading:advanced',
        'epsx:portfolio:view',
        'epsx:portfolio:history',
        'epsx:notifications:enhanced',
        'epsx:analytics:basic',
        'epsx:alerts:email',
      ];
    
    case 'GOLD':
      return [
        'epsx:rankings:view:50',
        'epsx:trading:basic',
        'epsx:trading:advanced',
        'epsx:trading:premium',
        'epsx:portfolio:view',
        'epsx:portfolio:history',
        'epsx:portfolio:tools',
        'epsx:notifications:enhanced',
        'epsx:analytics:basic',
        'epsx:analytics:advanced',
        'epsx:analytics:premium',
        'epsx:alerts:email',
        'epsx:support:priority',
      ];
    
    case 'PLATINUM':
      return [
        'epsx:rankings:view:100',
        'epsx:trading:basic',
        'epsx:trading:advanced',
        'epsx:trading:premium',
        'epsx:portfolio:view',
        'epsx:portfolio:history',
        'epsx:portfolio:tools',
        'epsx:notifications:enhanced',
        'epsx:analytics:basic',
        'epsx:analytics:advanced',
        'epsx:analytics:premium',
        'epsx:alerts:email',
        'epsx:support:priority',
        'epsx:research:reports',
        'epsx:dashboards:custom',
      ];
    
    case 'VIP':
    case 'DIAMOND':
    case 'API_PERSONAL':
    case 'API_COMPANY':  
    case 'API_PARTNER':
      return [
        'epsx:rankings:view:unlimited',
        'epsx:*:*',
        'epsx-pay:*:*',
        'epsx-token:*:*',
      ];
    
    default:
      return [
        'epsx:rankings:view:5',
        'epsx:trading:basic',
        'epsx:portfolio:view',
      ];
  }
}

/**
 * Check if permissions grant admin access
 */
export function hasAdminAccess(permissions: string[]): boolean {
  return hasPermission(permissions, 'admin:*:*');
}

/**
 * Get all ranking-related permissions from a permission set
 */
export function getRankingPermissions(permissions: string[]): string[] {
  return permissions.filter(perm => perm.startsWith('epsx:rankings:'));
}

/**
 * Validate if a permission string is properly formatted (with embedded timestamp support)
 */
export function isValidPermission(permission: string): boolean {
  const parts = permission.split(':');
  
  // Basic format validation (3 parts minimum, 5 maximum with timestamp)
  if (parts.length < 3 || parts.length > 5) {
    return false;
  }
  
  // Check for valid platform
  const validPlatforms = ['epsx', 'epsx-pay', 'epsx-token', 'admin'];
  if (!validPlatforms.includes(parts[0])) {
    return false;
  }
  
  // If there's a potential timestamp, validate it
  if (parts.length === 5 || (parts.length === 4 && parts[1] !== 'rankings')) {
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    if (isNaN(timestamp)) {
      return false; // Last part should be a valid timestamp
    }
  }
  
  // Special validation for ranking permissions
  if (parts[1] === 'rankings' && parts[2] === 'view') {
    if (parts.length === 4 || parts.length === 5) {
      const limit = parts[3];
      if (limit === 'unlimited') {
        return true;
      }
      const numLimit = parseInt(limit, 10);
      if (isNaN(numLimit) || numLimit < 0) {
        return false;
      }
      
      // If 5 parts, validate timestamp
      if (parts.length === 5) {
        const timestamp = parseInt(parts[4], 10);
        return !isNaN(timestamp);
      }
      
      return true;
    }
  }
  
  return true;
}

// ============================================================================
// ADVANCED EMBEDDED TIMESTAMP UTILITIES
// ============================================================================

/**
 * Get all permissions with expiry information for UI display
 */
export function getAllPermissionsWithExpiry(permissions: string[]): TimestampedPermission[] {
  return permissions.map(createTimestampedPermission);
}

/**
 * Check if user's effective permissions will change soon due to expiry
 */
export function willPermissionsChangeSoon(permissions: string[], hoursAhead: number = 24): boolean {
  const permissionsWithExpiry = getAllPermissionsWithExpiry(permissions);
  const checkTime = Date.now() + (hoursAhead * 60 * 60 * 1000);
  
  return permissionsWithExpiry.some(tp => 
    tp.expiresAt && (tp.expiresAt * 1000) <= checkTime
  );
}

/**
 * Get effective permissions at a specific time
 */
export function getEffectivePermissionsAtTime(permissions: string[], atTime: Date): string[] {
  const targetTimestamp = Math.floor(atTime.getTime() / 1000);
  
  return permissions.filter(perm => {
    const { timestamp } = parsePermissionWithTimestamp(perm);
    return !timestamp || timestamp > targetTimestamp;
  });
}

/**
 * Predict tier changes based on upcoming permission expiries
 */
export function predictTierChanges(permissions: string[], hoursAhead: number = 24): {
  currentTier: UserLevelType;
  futureTier: UserLevelType;
  willChange: boolean;
  changeTime?: Date;
} {
  const currentTier = deriveTierFromPermissions(permissions);
  const futureTime = new Date(Date.now() + (hoursAhead * 60 * 60 * 1000));
  const futurePermissions = getEffectivePermissionsAtTime(permissions, futureTime);
  const futureTier = deriveTierFromPermissions(futurePermissions);
  
  const willChange = currentTier !== futureTier;
  let changeTime: Date | undefined;
  
  if (willChange) {
    // Find the earliest expiry that would cause a tier change
    const nextExpiring = getNextExpiringRankingPermission(permissions);
    if (nextExpiring?.expiresAt) {
      changeTime = new Date(nextExpiring.expiresAt * 1000);
    }
  }
  
  return {
    currentTier,
    futureTier,
    willChange,
    changeTime
  };
}

/**
 * Get summary of permission health for dashboard display
 */
export function getPermissionHealthSummary(permissions: string[]): {
  total: number;
  active: number;
  expired: number;
  expiringSoon: number; // Within 24 hours
  permanent: number;
  nextExpiry?: TimestampedPermission;
  healthScore: 'excellent' | 'good' | 'warning' | 'critical';
} {
  const permissionsWithExpiry = getAllPermissionsWithExpiry(permissions);
  const now = Date.now();
  const twentyFourHoursFromNow = now + (24 * 60 * 60 * 1000);
  
  const active = permissionsWithExpiry.filter(tp => !tp.isExpired).length;
  const expired = permissionsWithExpiry.filter(tp => tp.isExpired).length;
  const expiringSoon = permissionsWithExpiry.filter(tp => 
    !tp.isExpired && 
    tp.expiresAt && 
    (tp.expiresAt * 1000) <= twentyFourHoursFromNow
  ).length;
  const permanent = permissionsWithExpiry.filter(tp => !tp.expiresAt).length;
  
  const nextExpiry = permissionsWithExpiry
    .filter(tp => !tp.isExpired && tp.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0];
  
  let healthScore: 'excellent' | 'good' | 'warning' | 'critical';
  
  if (expired > 0) {
    healthScore = 'critical';
  } else if (expiringSoon > 0) {
    healthScore = 'warning';
  } else if (permanent > active / 2) {
    healthScore = 'excellent';
  } else {
    healthScore = 'good';
  }
  
  return {
    total: permissions.length,
    active,
    expired,
    expiringSoon,
    permanent,
    nextExpiry,
    healthScore
  };
}