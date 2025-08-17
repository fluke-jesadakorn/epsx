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
import { formatLevelAsNumber } from '@/utils/env';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@/components/ui';

interface AnalyticsMetricsProps {
  userLevel: string;
  maxRankings: number;
  isExpired: boolean;
}

export function AnalyticsMetrics({ 
  userLevel, 
  maxRankings, 
  isExpired 
}: AnalyticsMetricsProps) {
  const getUpgradeInfo = () => {
    const nextLevel = {
      BASIC: { name: 'SILVER', rankings: 25 },
      SILVER: { name: 'GOLD', rankings: 50 },
      GOLD: { name: 'PLATINUM', rankings: 100 },
      PLATINUM: { name: 'PLATINUM', rankings: 100 },
    };
    return nextLevel[userLevel as keyof typeof nextLevel] || nextLevel.BASIC;
  };

  const upgradeInfo = getUpgradeInfo();
  const accessPercentage = Math.round((maxRankings / 100) * 100);

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
            <div className="text-2xl font-bold">{maxRankings}</div>
            <Badge variant={isExpired ? "destructive" : "default"} className="text-xs">
              {formatLevelAsNumber(userLevel)}
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
            <div className="text-2xl font-bold">+{upgradeInfo.rankings - maxRankings}</div>
            <Badge variant="secondary" className="text-xs">
              more stocks
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Additional rankings
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
