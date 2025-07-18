'use client';

import { useState, useEffect } from 'react';
import { useAuth } from '@/context/auth-context-improved';
import { status } from '@/services/pay';
import { getRankingLimitByLevel, getNumericLevelByLevel } from '@/app/constants/packages';
import { updateUserAccessCookies } from '@/middleware/userAccess';
import type { UserLevelType } from '@/app/constants/packages';
// TODO: Enable when permission service is fully integrated
// import { useStockAnalyticsPermissions } from '@epsx/auth/permission-service';

interface RankingAccess {
  maxRankings: number;
  userLevel: UserLevelType;
  isExpired: boolean;
  canViewRanking: (index: number) => boolean;
  upgradeRequired: boolean;
  // New: Preserve level number for display even when expired
  displayLevel: UserLevelType;
  displayNumericLevel: number;
  // New: Permission service integration
  canAnalyze: boolean;
  canExport: boolean;
  canScreen: boolean;
}

export function useRankingAccess(): RankingAccess & { isLoading: boolean } {
  const { user } = useAuth();
  // const { getApiLimits } = useFeatureAccess(); // For future subscription integration
  const [userLevel, setUserLevel] = useState<UserLevelType>('BRONZE');
  const [isExpired, setIsExpired] = useState(true);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchUserAccess = async () => {
      if (!user) {
        setUserLevel('BRONZE');
        setIsExpired(true);
        setIsLoading(false);
        updateUserAccessCookies('BRONZE', true);
        return;
      }

      try {
        // For now, use legacy payment system since Firebase User doesn't have subscription
        // TODO: Integrate subscription data from user profile/database
        const paymentStatus = await status();
        const level = (paymentStatus.level as UserLevelType) || 'BRONZE';
        const expired = paymentStatus.expire
          ? new Date() > paymentStatus.expire
          : !paymentStatus.paid;

        setUserLevel(level);
        setIsExpired(expired);
        updateUserAccessCookies(level, expired);
      } catch (error) {
        console.error('Failed to fetch user access:', error);
        setUserLevel('BRONZE');
        setIsExpired(true);
        updateUserAccessCookies('BRONZE', true);
      } finally {
        setIsLoading(false);
      }
    };

    fetchUserAccess();
  }, [user]);

  // Get limits from new payment system for future use
  // const limits = getApiLimits();
  // const newSystemMaxRankings = limits.maxRankings;

  // Legacy system limits (currently used)
  const rankingLimit = getRankingLimitByLevel(userLevel);
  const legacyMaxRankings = isExpired
    ? getRankingLimitByLevel('BRONZE')  // Expired users get Bronze access
    : rankingLimit;

  // Use legacy system until subscription integration is complete
  const finalMaxRankings = legacyMaxRankings;

  return {
    maxRankings: finalMaxRankings,
    userLevel,
    isExpired,
    isLoading,
    canViewRanking: (index: number) => index < finalMaxRankings,
    upgradeRequired: isExpired || userLevel === 'BRONZE',
    // New: Preserve level number for display even when expired
    displayLevel: userLevel,
    displayNumericLevel: getNumericLevelByLevel(userLevel),
    // New: Permission service integration (temporary implementation)
    canAnalyze: userLevel !== 'BRONZE' && !isExpired,
    canExport: (userLevel === 'GOLD' || userLevel === 'PLATINUM') && !isExpired,
    canScreen: userLevel === 'PLATINUM' && !isExpired,
  };
}
