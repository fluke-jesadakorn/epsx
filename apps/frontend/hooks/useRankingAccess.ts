'use client';

import { useState, useEffect } from 'react';
import { useAuth } from '@/context/auth-context-improved';
import { status } from '@/services/pay';
import { getRankingLimitByLevel } from '@/app/constants/packages';
import { updateUserAccessCookies } from '@/middleware/userAccess';
import type { UserLevelType } from '@/app/constants/packages';
// import { useFeatureAccess } from './useFeatureAccess'; // For future subscription integration

interface RankingAccess {
  maxRankings: number;
  userLevel: UserLevelType;
  isExpired: boolean;
  canViewRanking: (index: number) => boolean;
  upgradeRequired: boolean;
}

export function useRankingAccess(): RankingAccess & { isLoading: boolean } {
  const { user } = useAuth();
  // const { getApiLimits } = useFeatureAccess(); // For future subscription integration
  const [userLevel, setUserLevel] = useState<UserLevelType>('BASIC');
  const [isExpired, setIsExpired] = useState(true);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchUserAccess = async () => {
      if (!user) {
        setUserLevel('BASIC');
        setIsExpired(true);
        setIsLoading(false);
        updateUserAccessCookies('BASIC', true);
        return;
      }

      try {
        // For now, use legacy payment system since Firebase User doesn't have subscription
        // TODO: Integrate subscription data from user profile/database
        const paymentStatus = await status();
        const level = (paymentStatus.level as UserLevelType) || 'BASIC';
        const expired = paymentStatus.expire
          ? new Date() > paymentStatus.expire
          : !paymentStatus.paid;

        setUserLevel(level);
        setIsExpired(expired);
        updateUserAccessCookies(level, expired);
      } catch (error) {
        console.error('Failed to fetch user access:', error);
        setUserLevel('BASIC');
        setIsExpired(true);
        updateUserAccessCookies('BASIC', true);
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
    ? getRankingLimitByLevel('BASIC')
    : rankingLimit;

  // Use legacy system until subscription integration is complete
  const finalMaxRankings = legacyMaxRankings;

  return {
    maxRankings: finalMaxRankings,
    userLevel,
    isExpired,
    isLoading,
    canViewRanking: (index: number) => index < finalMaxRankings,
    upgradeRequired: isExpired || userLevel === 'BASIC',
  };
}
