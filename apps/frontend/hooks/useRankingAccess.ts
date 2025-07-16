'use client';

import { useState, useEffect } from 'react';
import { useAuth } from '@/context/auth-context';
import { status } from '@/services/pay';
import { getRankingLimitByLevel } from '@/app/constants/packages';
import { updateUserAccessCookies } from '@/middleware/userAccess';
import type { UserLevelType } from '@/app/constants/packages';

interface RankingAccess {
  maxRankings: number;
  userLevel: UserLevelType;
  isExpired: boolean;
  canViewRanking: (index: number) => boolean;
  upgradeRequired: boolean;
}

export function useRankingAccess(): RankingAccess & { isLoading: boolean } {
  const { user } = useAuth();
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
        const paymentStatus = await status();
        const level = paymentStatus.level as UserLevelType || 'BASIC';
        const expired = paymentStatus.expire 
          ? new Date() > paymentStatus.expire 
          : !paymentStatus.paid;

        setUserLevel(level);
        setIsExpired(expired);
        
        // Update cookies for API access validation
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

  const rankingLimit = getRankingLimitByLevel(userLevel);
  const maxRankings = isExpired ? getRankingLimitByLevel('BASIC') : rankingLimit;

  return {
    maxRankings,
    userLevel,
    isExpired,
    isLoading,
    canViewRanking: (index: number) => index < maxRankings,
    upgradeRequired: isExpired || userLevel === 'BASIC',
  };
}
