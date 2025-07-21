import { NextRequest } from 'next/server';

interface RankingAccessResult {
  userLevel: string;
  maxAllowed: number;
  wasLimited: boolean;
  modifiedUrl?: string;
}

export function validateRankingAccess(
  request: NextRequest, 
  userLevel: string, 
  isExpired: boolean
): RankingAccessResult {
  // Basic implementation - can be expanded later
  const maxAllowed = userLevel === 'GOLD' ? 100 : userLevel === 'SILVER' ? 50 : 10;
  const effectiveLevel = isExpired ? 'BASIC' : userLevel;
  
  return {
    userLevel: effectiveLevel,
    maxAllowed,
    wasLimited: false,
    modifiedUrl: request.url
  };
}
