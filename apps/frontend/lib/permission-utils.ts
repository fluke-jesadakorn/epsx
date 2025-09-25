/**
 * Permission utilities for frontend components
 * Provides helper functions for permission-based access control
 */

// Re-export from shared permissions utilities
export { 
  derivePackageTierFromPermissions as deriveTierFromPermissions 
} from '@/lib/shared';

// Type definitions
export type UserLevelType = 'FREE' | 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM' | 'ENTERPRISE';

/**
 * Extract ranking limit from permissions
 */
export function extractRankingLimitFromPermissions(permissions: string[]): number {
  for (const permission of permissions) {
    if (permission.startsWith('epsx:rankings:view:')) {
      const limitStr = permission.split(':')[3];
      if (limitStr === 'unlimited') return -1;
      const limit = parseInt(limitStr, 10);
      if (!isNaN(limit)) return limit;
    }
  }
  return 3; // Default free tier limit
}

/**
 * Check if user can view specific ranking position
 */
export function canViewRankingPosition(position: number, permissions: string[]): boolean {
  const limit = extractRankingLimitFromPermissions(permissions);
  if (limit === -1) return true; // Unlimited access
  return position <= limit;
}

/**
 * Get tier display name from permissions
 */
export function getTierFromPermissions(permissions: string[]): UserLevelType {
  const limit = extractRankingLimitFromPermissions(permissions);
  
  if (permissions.some(p => p.includes('admin:'))) return 'ENTERPRISE';
  if (limit === -1 || limit >= 100) return 'ENTERPRISE';
  if (limit >= 100) return 'PLATINUM';
  if (limit >= 50) return 'GOLD';
  if (limit >= 25) return 'SILVER';
  if (limit >= 5) return 'BRONZE';
  return 'FREE';
}

/**
 * Check if user has permission for a specific feature
 */
export function hasFeaturePermission(feature: string, permissions: string[]): boolean {
  switch (feature) {
    case 'analytics':
      return permissions.some(p => p.startsWith('epsx:analytics:') || p.includes('epsx:rankings:'));
    case 'admin':
      return permissions.some(p => p.startsWith('admin:'));
    case 'premium':
      return permissions.some(p => !p.includes(':view:3')); // Not free tier
    default:
      return true;
  }
}

/**
 * Filter valid permissions from array
 */
export function filterValidPermissions(permissions: string[]): string[] {
  return permissions.filter(perm => {
    // Valid permission format: platform:resource:action or platform:resource:action:timestamp
    const parts = perm.split(':');
    return parts.length >= 3 && parts[0] && parts[1] && parts[2];
  });
}

/**
 * Get package tier from permissions
 */
export function getPackageFromPermissions(permissions: string[]): string {
  return getTierFromPermissions(permissions);
}