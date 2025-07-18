import { NextRequest } from 'next/server';
import { getPackageByLevel } from '@/app/constants/packages';
import type { UserLevelType } from '@/app/constants/packages';

export function validateRankingAccess(
  request: NextRequest,
  userLevel: UserLevelType = 'BRONZE',
  isExpired: boolean = true
) {
  const url = new URL(request.url);
  const requestedLimit = parseInt(url.searchParams.get('limit') || '10');
  
  const currentPackage = getPackageByLevel(userLevel);
  const maxAllowed = isExpired ? 5 : (currentPackage?.rankingLimit || 5);
  
  // Limit the request to user's maximum allowed
  if (requestedLimit > maxAllowed) {
    url.searchParams.set('limit', maxAllowed.toString());
    return {
      modifiedUrl: url.toString(),
      wasLimited: true,
      maxAllowed,
      userLevel,
      isExpired
    };
  }
  
  return {
    modifiedUrl: null,
    wasLimited: false,
    maxAllowed,
    userLevel,
    isExpired
  };
}
