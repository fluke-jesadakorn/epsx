'use server';

import { createApiClient, isApiError } from '@/lib/api-client';
import { getServerSession } from 'next-auth/react';
import { auth } from '@/lib/auth';

// ====================================
// Embedded Timestamp Permission Types
// ====================================

export interface TimestampedPermission {
  permission: string;
  basePermission: string;
  expiresAt?: number; // Unix timestamp
  isExpired: boolean;
  expiresIn?: string; // Human readable (e.g., "2 hours")
  timeRemaining?: number; // Milliseconds remaining
}

export interface PermissionExpiryInfo {
  valid: TimestampedPermission[];
  expired: TimestampedPermission[];
  expiringSoon: TimestampedPermission[];
  nextExpiry: number | null;
  hasExpiringPermissions: boolean;
}

export interface PermissionHealthSummary {
  totalPermissions: number;
  validCount: number;
  expiredCount: number;
  expiringSoonCount: number;
  healthScore: number; // 0-100
  status: 'excellent' | 'good' | 'warning' | 'critical';
}

export interface TierPrediction {
  currentTier: string;
  predictedTier: string;
  hoursAhead: number;
  confidence: number; // 0-100
  affectedPermissions: string[];
  tierChangeReason: string;
}

// ====================================
// Helper Functions
// ====================================

function parsePermissionWithTimestamp(permission: string): { basePermission: string; timestamp?: number } {
  const parts = permission.split(':');
  if (parts.length >= 4) {
    const lastPart = parts[parts.length - 1];
    const timestamp = parseInt(lastPart, 10);
    if (!isNaN(timestamp)) {
      const basePermission = parts.slice(0, -1).join(':');
      return { basePermission, timestamp };
    }
  }
  return { basePermission: permission };
}

function isPermissionExpired(permission: string): boolean {
  const { timestamp } = parsePermissionWithTimestamp(permission);
  if (!timestamp) return false;
  return Date.now() > timestamp * 1000;
}

function getTimeUntilExpiry(permission: string): number | null {
  const { timestamp } = parsePermissionWithTimestamp(permission);
  if (!timestamp) return null;
  const timeRemaining = timestamp * 1000 - Date.now();
  return timeRemaining > 0 ? timeRemaining : 0;
}

function formatTimeRemaining(milliseconds: number): string {
  if (milliseconds <= 0) return 'Expired';
  
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  
  if (days > 0) return `${days} day${days > 1 ? 's' : ''}`;
  if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''}`;
  if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''}`;
  return `${seconds} second${seconds > 1 ? 's' : ''}`;
}

// ====================================
// Server Actions
// ====================================

/**
 * Validate user's embedded timestamp permissions server-side
 */
export async function validateUserEmbeddedPermissions(): Promise<{
  success: boolean;
  data?: PermissionExpiryInfo;
  error?: string;
}> {
  try {
    const session = await auth();
    
    if (!session?.user) {
      return {
        success: false,
        error: 'Not authenticated'
      };
    }

    const userPermissions = (session.user as any)?.permissions as string[] || [];
    
    const validPermissions: TimestampedPermission[] = [];
    const expiredPermissions: TimestampedPermission[] = [];
    const expiringSoonPermissions: TimestampedPermission[] = [];
    
    const currentTime = Date.now();
    const expiringSoonThreshold = 24 * 60 * 60 * 1000; // 24 hours
    
    let nextExpiry: number | null = null;
    
    userPermissions.forEach(permission => {
      const { basePermission, timestamp } = parsePermissionWithTimestamp(permission);
      
      if (!timestamp) {
        // Non-timestamped permission - always valid
        validPermissions.push({
          permission,
          basePermission,
          isExpired: false
        });
        return;
      }
      
      const expiresAt = timestamp;
      const expiresAtMs = timestamp * 1000;
      const timeRemaining = expiresAtMs - currentTime;
      const isExpired = timeRemaining <= 0;
      const isExpiringSoon = timeRemaining > 0 && timeRemaining <= expiringSoonThreshold;
      
      const timestampedPermission: TimestampedPermission = {
        permission,
        basePermission,
        expiresAt,
        isExpired,
        timeRemaining: Math.max(0, timeRemaining),
        expiresIn: formatTimeRemaining(timeRemaining)
      };
      
      if (isExpired) {
        expiredPermissions.push(timestampedPermission);
      } else if (isExpiringSoon) {
        expiringSoonPermissions.push(timestampedPermission);
        if (!nextExpiry || expiresAtMs < nextExpiry) {
          nextExpiry = expiresAtMs;
        }
      } else {
        validPermissions.push(timestampedPermission);
        if (!nextExpiry || expiresAtMs < nextExpiry) {
          nextExpiry = expiresAtMs;
        }
      }
    });
    
    const expiryInfo: PermissionExpiryInfo = {
      valid: validPermissions,
      expired: expiredPermissions,
      expiringSoon: expiringSoonPermissions,
      nextExpiry: nextExpiry ? Math.floor(nextExpiry / 1000) : null,
      hasExpiringPermissions: expiringSoonPermissions.length > 0 || expiredPermissions.length > 0
    };
    
    return {
      success: true,
      data: expiryInfo
    };
  } catch (error) {
    console.error('Error validating embedded permissions:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

/**
 * Get permission health summary server-side
 */
export async function getPermissionHealthSummary(): Promise<{
  success: boolean;
  data?: PermissionHealthSummary;
  error?: string;
}> {
  try {
    const validationResult = await validateUserEmbeddedPermissions();
    
    if (!validationResult.success || !validationResult.data) {
      return {
        success: false,
        error: validationResult.error || 'Failed to validate permissions'
      };
    }
    
    const { valid, expired, expiringSoon } = validationResult.data;
    const totalPermissions = valid.length + expired.length + expiringSoon.length;
    
    if (totalPermissions === 0) {
      return {
        success: true,
        data: {
          totalPermissions: 0,
          validCount: 0,
          expiredCount: 0,
          expiringSoonCount: 0,
          healthScore: 100,
          status: 'excellent'
        }
      };
    }
    
    const healthScore = Math.round((valid.length / totalPermissions) * 100);
    
    let status: 'excellent' | 'good' | 'warning' | 'critical';
    if (expired.length > 0) {
      status = 'critical';
    } else if (expiringSoon.length > totalPermissions * 0.5) {
      status = 'warning';
    } else if (expiringSoon.length > 0) {
      status = 'good';
    } else {
      status = 'excellent';
    }
    
    const healthSummary: PermissionHealthSummary = {
      totalPermissions,
      validCount: valid.length,
      expiredCount: expired.length,
      expiringSoonCount: expiringSoon.length,
      healthScore,
      status
    };
    
    return {
      success: true,
      data: healthSummary
    };
  } catch (error) {
    console.error('Error getting permission health summary:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

/**
 * Check if user has specific permission with embedded timestamp validation
 */
export async function checkUserPermissionWithTime(permission: string): Promise<{
  success: boolean;
  hasPermission?: boolean;
  isExpired?: boolean;
  timeRemaining?: number;
  expiresIn?: string;
  error?: string;
}> {
  try {
    const session = await auth();
    
    if (!session?.user) {
      return {
        success: false,
        error: 'Not authenticated'
      };
    }

    const userPermissions = (session.user as any)?.permissions as string[] || [];
    
    // Check for exact match or base permission match
    const matchingPermission = userPermissions.find(userPerm => {
      const { basePermission } = parsePermissionWithTimestamp(userPerm);
      return userPerm === permission || basePermission === permission;
    });
    
    if (!matchingPermission) {
      return {
        success: true,
        hasPermission: false
      };
    }
    
    const { timestamp } = parsePermissionWithTimestamp(matchingPermission);
    
    if (!timestamp) {
      // Non-timestamped permission - always valid
      return {
        success: true,
        hasPermission: true,
        isExpired: false
      };
    }
    
    const timeRemaining = getTimeUntilExpiry(matchingPermission) || 0;
    const isExpired = timeRemaining <= 0;
    
    return {
      success: true,
      hasPermission: !isExpired,
      isExpired,
      timeRemaining,
      expiresIn: formatTimeRemaining(timeRemaining)
    };
  } catch (error) {
    console.error('Error checking user permission with time:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

/**
 * Get time until next permission expiry
 */
export async function getTimeUntilNextExpiry(): Promise<{
  success: boolean;
  timeRemaining?: number;
  nextExpiryTimestamp?: number;
  expiresIn?: string;
  expiringPermission?: string;
  error?: string;
}> {
  try {
    const validationResult = await validateUserEmbeddedPermissions();
    
    if (!validationResult.success || !validationResult.data) {
      return {
        success: false,
        error: validationResult.error || 'Failed to validate permissions'
      };
    }
    
    const { valid, expiringSoon } = validationResult.data;
    const allActivePermissions = [...valid, ...expiringSoon];
    
    // Find the permission that expires soonest
    let nextExpiry: number | null = null;
    let expiringPermission: string | null = null;
    
    allActivePermissions.forEach(perm => {
      if (perm.expiresAt && perm.timeRemaining && perm.timeRemaining > 0) {
        const expiryTime = perm.expiresAt * 1000;
        if (!nextExpiry || expiryTime < nextExpiry) {
          nextExpiry = expiryTime;
          expiringPermission = perm.permission;
        }
      }
    });
    
    if (!nextExpiry || !expiringPermission) {
      return {
        success: true,
        timeRemaining: null,
        nextExpiryTimestamp: null
      };
    }
    
    const timeRemaining = nextExpiry - Date.now();
    
    return {
      success: true,
      timeRemaining: Math.max(0, timeRemaining),
      nextExpiryTimestamp: Math.floor(nextExpiry / 1000),
      expiresIn: formatTimeRemaining(timeRemaining),
      expiringPermission
    };
  } catch (error) {
    console.error('Error getting time until next expiry:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

/**
 * Predict tier changes based on expiring permissions
 */
export async function predictTierChanges(hoursAhead: number = 24): Promise<{
  success: boolean;
  data?: TierPrediction;
  error?: string;
}> {
  try {
    const session = await auth();
    
    if (!session?.user) {
      return {
        success: false,
        error: 'Not authenticated'
      };
    }

    const userPermissions = (session.user as any)?.permissions as string[] || [];
    
    // Get current ranking limit
    const getCurrentRankingLimit = (): number => {
      for (const perm of userPermissions) {
        if (perm.startsWith('epsx:rankings:view:')) {
          const { basePermission } = parsePermissionWithTimestamp(perm);
          const parts = basePermission.split(':');
          const limitPart = parts[3];
          if (limitPart === 'unlimited') return -1;
          const parsed = parseInt(limitPart, 10);
          if (!isNaN(parsed)) return parsed;
        }
      }
      return 5; // Default Bronze
    };
    
    const mapLimitToTier = (limit: number): string => {
      if (limit === -1) return 'Platinum';
      if (limit >= 100) return 'Gold';
      if (limit >= 25) return 'Silver';
      return 'Bronze';
    };
    
    const currentLimit = getCurrentRankingLimit();
    const currentTier = mapLimitToTier(currentLimit);
    
    // Simulate permissions after specified hours
    const futureTime = Date.now() + (hoursAhead * 60 * 60 * 1000);
    const futurePermissions = userPermissions.filter(permission => {
      const { timestamp } = parsePermissionWithTimestamp(permission);
      if (!timestamp) return true; // Non-timestamped permissions persist
      return timestamp * 1000 > futureTime; // Only keep non-expired permissions
    });
    
    // Get future ranking limit
    const getFutureRankingLimit = (): number => {
      for (const perm of futurePermissions) {
        if (perm.startsWith('epsx:rankings:view:')) {
          const { basePermission } = parsePermissionWithTimestamp(perm);
          const parts = basePermission.split(':');
          const limitPart = parts[3];
          if (limitPart === 'unlimited') return -1;
          const parsed = parseInt(limitPart, 10);
          if (!isNaN(parsed)) return parsed;
        }
      }
      return 5; // Default Bronze
    };
    
    const futureLimit = getFutureRankingLimit();
    const predictedTier = mapLimitToTier(futureLimit);
    
    // Calculate affected permissions
    const affectedPermissions = userPermissions.filter(permission => {
      const { timestamp } = parsePermissionWithTimestamp(permission);
      if (!timestamp) return false;
      const expiryTime = timestamp * 1000;
      return expiryTime > Date.now() && expiryTime <= futureTime;
    });
    
    // Calculate confidence based on how many permissions are changing
    const confidence = Math.max(20, 100 - (affectedPermissions.length * 10));
    
    let tierChangeReason = 'No tier change predicted';
    if (currentTier !== predictedTier) {
      if (affectedPermissions.length > 0) {
        tierChangeReason = `${affectedPermissions.length} permission${affectedPermissions.length > 1 ? 's' : ''} will expire`;
      } else {
        tierChangeReason = 'Tier change due to permission changes';
      }
    }
    
    const prediction: TierPrediction = {
      currentTier,
      predictedTier,
      hoursAhead,
      confidence,
      affectedPermissions: affectedPermissions.map(p => parsePermissionWithTimestamp(p).basePermission),
      tierChangeReason
    };
    
    return {
      success: true,
      data: prediction
    };
  } catch (error) {
    console.error('Error predicting tier changes:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}

/**
 * Get all permissions with embedded timestamp parsing
 */
export async function getAllPermissionsWithExpiry(): Promise<{
  success: boolean;
  data?: TimestampedPermission[];
  error?: string;
}> {
  try {
    const session = await auth();
    
    if (!session?.user) {
      return {
        success: false,
        error: 'Not authenticated'
      };
    }

    const userPermissions = (session.user as any)?.permissions as string[] || [];
    
    const permissionsWithExpiry = userPermissions.map(permission => {
      const { basePermission, timestamp } = parsePermissionWithTimestamp(permission);
      
      if (!timestamp) {
        return {
          permission,
          basePermission,
          isExpired: false
        };
      }
      
      const timeRemaining = getTimeUntilExpiry(permission) || 0;
      const isExpired = timeRemaining <= 0;
      
      return {
        permission,
        basePermission,
        expiresAt: timestamp,
        isExpired,
        timeRemaining,
        expiresIn: formatTimeRemaining(timeRemaining)
      };
    });
    
    return {
      success: true,
      data: permissionsWithExpiry
    };
  } catch (error) {
    console.error('Error getting all permissions with expiry:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred'
    };
  }
}