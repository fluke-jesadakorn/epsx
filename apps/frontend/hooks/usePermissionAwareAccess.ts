'use client';

import { useState, useEffect } from 'react';
import { useAuth } from '@/context/auth-context';
import { getRankingLimitByLevel } from '@/app/constants/packages';
import { usePermissionContext } from '@epsx/server-providers/client';
import type { UserLevelType } from '@/app/constants/packages';

interface PermissionAwareAccess {
  // Legacy interface compatibility
  maxRankings: number;
  userLevel: UserLevelType;
  isExpired: boolean;
  canViewRanking: (index: number) => boolean;
  upgradeRequired: boolean;
  
  // New permission-based interface
  permissions: {
    canRead: boolean;
    canAnalyze: boolean;
    canExport: boolean;
    canScreen: boolean;
    canManage: boolean;
  };
  
  // Access control functions
  canAccessResource: (resource: string, action: string) => boolean;
  canAccessRankings: (limit: number) => boolean;
  
  // Loading state
  isLoading: boolean;
}

/**
 * Permission-aware access hook that bridges legacy and new permission systems
 * This hook gradually transitions from the old role-based system to the new AWS IAM-inspired system
 */
export function usePermissionAwareAccess(): PermissionAwareAccess {
  const { user } = useAuth();
  const { paymentStatus, error } = usePermissionContext();
  const [userLevel, setUserLevel] = useState<UserLevelType>('BRONZE');
  const [isExpired, setIsExpired] = useState(true);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    if (!user) {
      setUserLevel('BRONZE');
      setIsExpired(true);
      setIsLoading(false);
      return;
    }

    if (error) {
      console.error('Permission context error:', error);
      setUserLevel('BRONZE');
      setIsExpired(true);
      setIsLoading(false);
      return;
    }

    if (paymentStatus) {
      const level = (paymentStatus?.userLevel as UserLevelType) || 'BRONZE';
      const expired = paymentStatus?.expireDate
        ? new Date() > paymentStatus.expireDate
        : !paymentStatus?.paid;

      setUserLevel(level);
      setIsExpired(expired);
      setIsLoading(false);
    }
  }, [user, paymentStatus, error]);

  // Calculate permissions based on current level and expiration
  const permissions = {
    canRead: true, // Everyone can read basic data
    canAnalyze: (userLevel !== 'BRONZE' && !isExpired) || userLevel === 'PLATINUM',
    canExport: ((userLevel === 'GOLD' || userLevel === 'PLATINUM') && !isExpired) || userLevel === 'PLATINUM',
    canScreen: (userLevel === 'PLATINUM' && !isExpired) || userLevel === 'PLATINUM',
    canManage: userLevel === 'PLATINUM' && !isExpired,
  };

  // Get max rankings based on level
  const maxRankings = isExpired 
    ? getRankingLimitByLevel('BRONZE')
    : getRankingLimitByLevel(userLevel);

  // Resource access control function
  const canAccessResource = (resource: string, action: string): boolean => {
    // Map legacy permissions to resource/action combinations
    const resourceActionMap: Record<string, Record<string, boolean>> = {
      'stock:rankings': {
        'read': permissions.canRead,
        'list': permissions.canRead,
        'analyze': permissions.canAnalyze,
        'export': permissions.canExport,
      },
      'stock:analytics': {
        'read': permissions.canRead,
        'analyze': permissions.canAnalyze,
        'export': permissions.canExport,
      },
      'stock:screener': {
        'read': permissions.canRead,
        'screen': permissions.canScreen,
      },
      'admin:users': {
        'manage': permissions.canManage,
      },
    };

    return resourceActionMap[resource]?.[action] || false;
  };

  // Rankings access control function
  const canAccessRankings = (limit: number): boolean => {
    return limit <= maxRankings;
  };

  return {
    // Legacy compatibility
    maxRankings,
    userLevel,
    isExpired,
    canViewRanking: (index: number) => index < maxRankings,
    upgradeRequired: isExpired || userLevel === 'BRONZE',
    
    // New permission-based interface
    permissions,
    canAccessResource,
    canAccessRankings,
    
    // Loading state
    isLoading,
  };
}

export default usePermissionAwareAccess;
