'use client';

import React from 'react';
import { 
  TrendingUp, 
  Users, 
  Crown, 
  BarChart3,
  Lock,
  Star
} from 'lucide-react';
import { formatLevelAsNumber } from '@/lib/level-utils';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@/components/ui';
import { 
  extractRankingLimitFromPermissions, 
  deriveTierFromPermissions,
  hasPermission 
} from '@/lib/permission-utils';

interface AnalyticsMetricsProps {
  userPermissions: string[];
  isExpired: boolean;
  // Legacy props for backward compatibility
  userLevel?: string; // @deprecated Use userPermissions instead
  maxRankings?: number; // @deprecated Use userPermissions instead  
}

export function AnalyticsMetrics({ 
  userPermissions,
  isExpired,
  userLevel, // Legacy backward compatibility
  maxRankings, // Legacy backward compatibility
}: AnalyticsMetricsProps) {
  // Extract permission-based data with backward compatibility
  const actualMaxRankings = maxRankings ?? extractRankingLimitFromPermissions(userPermissions);
  const actualUserLevel = userLevel ?? deriveTierFromPermissions(userPermissions);

  const getUpgradeInfo = (currentTier: string, currentLimit: number) => {
    const tierUpgrades: Record<string, { name: string; rankings: number }> = {
      BRONZE: { name: 'SILVER', rankings: 25 },
      SILVER: { name: 'GOLD', rankings: 50 },
      GOLD: { name: 'PLATINUM', rankings: 100 },
      PLATINUM: { name: 'VIP', rankings: -1 }, // Unlimited
    };
    
    // Handle custom limits
    if (currentLimit === 5) return tierUpgrades.BRONZE;
    if (currentLimit === 25) return tierUpgrades.SILVER;
    if (currentLimit === 50) return tierUpgrades.GOLD;
    if (currentLimit === 100) return tierUpgrades.PLATINUM;
    if (currentLimit === -1) return { name: 'MAX', rankings: -1 };
    
    // Fallback for custom limits
    if (currentLimit < 25) return tierUpgrades.BRONZE;
    if (currentLimit < 50) return tierUpgrades.SILVER;
    if (currentLimit < 100) return tierUpgrades.GOLD;
    return tierUpgrades.PLATINUM;
  };

  const upgradeInfo = getUpgradeInfo(actualUserLevel, actualMaxRankings);
  const accessPercentage = actualMaxRankings === -1 
    ? 100 
    : Math.min(Math.round((actualMaxRankings / 100) * 100), 100);

  return (
    <div className="grid gap-6 md:grid-cols-4">
      {/* Current Access */}
      <Card className="relative overflow-hidden">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Current Access
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <div className="text-2xl font-bold">
              {actualMaxRankings === -1 ? '∞' : actualMaxRankings}
            </div>
            <Badge variant={isExpired ? "destructive" : "default"} className="text-xs">
              {formatLevelAsNumber(actualUserLevel)}
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Top rankings available
          </p>
        </CardContent>
        <div className="absolute top-4 right-4">
          <TrendingUp className="h-4 w-4 text-muted-foreground" />
        </div>
      </Card>

      {/* Access Percentage */}
      <Card className="relative overflow-hidden">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Market Coverage
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <div className="text-2xl font-bold">{accessPercentage}%</div>
            <Badge variant="outline" className="text-xs">
              of top 100
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Premium stock access
          </p>
        </CardContent>
        <div className="absolute top-4 right-4">
          <BarChart3 className="h-4 w-4 text-muted-foreground" />
        </div>
      </Card>

      {/* Upgrade Potential */}
      <Card className="relative overflow-hidden">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Upgrade to {upgradeInfo.name}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <div className="text-2xl font-bold">
              {actualMaxRankings === -1 ? 
                '∞' : 
                upgradeInfo.rankings === -1 ? 
                  '∞' : 
                  `+${upgradeInfo.rankings - actualMaxRankings}`
              }
            </div>
            <Badge variant="secondary" className="text-xs">
              {actualMaxRankings === -1 ? 'unlimited' : 'more stocks'}
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            {actualMaxRankings === -1 ? 'Maximum access' : 'Additional rankings'}
          </p>
        </CardContent>
        <div className="absolute top-4 right-4">
          <Crown className="h-4 w-4 text-yellow-500" />
        </div>
      </Card>

      {/* Status Indicator */}
      <Card className="relative overflow-hidden">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Account Status
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <div className="text-2xl font-bold">
              {isExpired ? (
                <Lock className="h-6 w-6 text-red-500" />
              ) : (
                <Star className="h-6 w-6 text-green-500" />
              )}
            </div>
            <Badge variant={isExpired ? "destructive" : "default"} className="text-xs">
              {isExpired ? "Expired" : "Active"}
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Subscription status
          </p>
        </CardContent>
        <div className="absolute top-4 right-4">
          <Users className="h-4 w-4 text-muted-foreground" />
        </div>
      </Card>
    </div>
  );
}
