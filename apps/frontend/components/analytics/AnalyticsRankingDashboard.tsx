'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useRankingAccess } from '@/hooks/useRankingAccess';
import { fetchStockRankingDataForUser } from '@/app/actions/stockRanking';
import { fetchPublicRankingData } from '@/app/actions/publicRanking';
import RoleBasedFinancialTable from '@/components/shared/RoleBasedFinancialTable';
import { AnalyticsMetrics } from '@/components/analytics/AnalyticsMetrics';
import { UpgradePrompt } from '@/components/ui/upgrade-prompt';
import { 
  BarChart3, 
  TrendingUp, 
  Crown, 
  Users, 
  Lock,
  Eye,
  Star,
  Target
} from 'lucide-react';
import type { StockFinancialData } from '@/types/financialChartData';

export function AnalyticsRankingDashboard() {
  const { maxRankings, userLevel, isExpired, upgradeRequired, isLoading } = useRankingAccess();
  const [premiumData, setPremiumData] = useState<StockFinancialData[]>([]);
  const [publicData, setPublicData] = useState<StockFinancialData[]>([]);
  const [dataLoading, setDataLoading] = useState(true);
  const [activeTab, setActiveTab] = useState('premium');

  useEffect(() => {
    const loadAnalyticsData = async () => {
      try {
        setDataLoading(true);
        
        // Load user-specific premium data
        const userDataPromise = fetchStockRankingDataForUser(
          userLevel,
          isExpired,
          0,
          undefined,
          4
        );
        
        // Load public preview data
        const publicDataPromise = fetchPublicRankingData(100, 10);
        
        const [userData, publicData] = await Promise.all([
          userDataPromise,
          publicDataPromise
        ]);
        
        setPremiumData(userData);
        setPublicData(publicData);
      } catch (error) {
        console.error('Failed to load analytics data:', error);
      } finally {
        setDataLoading(false);
      }
    };

    if (!isLoading) {
      loadAnalyticsData();
    }
  }, [userLevel, isExpired, isLoading]);

  if (isLoading || dataLoading) {
    return (
      <div className="space-y-6">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/3 mb-4"></div>
          <div className="h-4 bg-gray-200 rounded w-2/3 mb-8"></div>
          <div className="grid gap-6 md:grid-cols-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-32 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  const getLevelInfo = () => {
    const levels = {
      BASIC: { color: 'bg-gray-500', name: 'Basic', maxRank: 5 },
      SILVER: { color: 'bg-gray-400', name: 'Silver', maxRank: 25 },
      GOLD: { color: 'bg-yellow-500', name: 'Gold', maxRank: 50 },
      PLATINUM: { color: 'bg-purple-500', name: 'Platinum', maxRank: 100 },
    };
    return levels[userLevel as keyof typeof levels] || levels.BASIC;
  };

  const levelInfo = getLevelInfo();

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <BarChart3 className="h-10 w-10 text-blue-600" />
          <h1 className="text-4xl font-bold">Analytics Dashboard</h1>
        </div>
        <p className="text-xl text-muted-foreground">
          Advanced stock ranking analytics based on your subscription
        </p>
        
        {/* User Level Badge */}
        <div className="flex justify-center">
          <Badge 
            className={`${levelInfo.color} text-white px-6 py-2 text-lg gap-2`}
          >
            <Crown className="h-5 w-5" />
            {levelInfo.name} Member
            {isExpired && <span className="text-xs">(Expired)</span>}
          </Badge>
        </div>
      </div>

      {/* Analytics Metrics */}
      <AnalyticsMetrics 
        userLevel={userLevel}
        maxRankings={maxRankings}
        isExpired={isExpired}
      />

      {/* Main Analytics Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="premium" className="gap-2">
            <Star className="h-4 w-4" />
            Your Rankings
          </TabsTrigger>
          <TabsTrigger value="overview" className="gap-2">
            <Eye className="h-4 w-4" />
            Overview
          </TabsTrigger>
          <TabsTrigger value="public" className="gap-2">
            <Users className="h-4 w-4" />
            Public Preview
          </TabsTrigger>
          <TabsTrigger value="upgrade" className="gap-2">
            <Target className="h-4 w-4" />
            Upgrade
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="space-y-6">
          <div className="grid gap-6 md:grid-cols-2">
            {/* Access Summary */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <TrendingUp className="h-5 w-5" />
                  Your Access Level
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex justify-between items-center">
                  <span>Rankings Available:</span>
                  <Badge variant="outline">{maxRankings} stocks</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span>Subscription Status:</span>
                  <Badge variant={isExpired ? "destructive" : "default"}>
                    {isExpired ? "Expired" : "Active"}
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span>Current Level:</span>
                  <Badge className={levelInfo.color}>{levelInfo.name}</Badge>
                </div>
              </CardContent>
            </Card>

            {/* Quick Stats */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <BarChart3 className="h-5 w-5" />
                  Quick Stats
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex justify-between items-center">
                  <span>Top Stocks Accessible:</span>
                  <span className="font-semibold">{premiumData.length}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Public Preview Available:</span>
                  <span className="font-semibold">{publicData.length}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span>Premium Features:</span>
                  <Badge variant={upgradeRequired ? "outline" : "default"}>
                    {upgradeRequired ? "Limited" : "Full Access"}
                  </Badge>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="premium" className="space-y-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-2xl font-bold">Your Premium Rankings</h3>
              <Badge className={levelInfo.color}>
                Top {maxRankings} Available
              </Badge>
            </div>
            
            {premiumData.length > 0 ? (
              <RoleBasedFinancialTable data={premiumData} />
            ) : (
              <Card>
                <CardContent className="p-12 text-center">
                  <Lock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                  <h3 className="text-lg font-semibold mb-2">No Premium Access</h3>
                  <p className="text-muted-foreground">
                    Upgrade your subscription to access premium rankings
                  </p>
                </CardContent>
              </Card>
            )}
          </div>
        </TabsContent>

        <TabsContent value="public" className="space-y-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-2xl font-bold">Public Preview Rankings</h3>
              <Badge variant="outline">Rankings #100-110</Badge>
            </div>
            <p className="text-muted-foreground">
              These rankings are available to everyone as a preview of our ranking system.
            </p>
            
            <RoleBasedFinancialTable 
              data={publicData} 
              isPublicPreview={true}
            />
          </div>
        </TabsContent>

        <TabsContent value="upgrade" className="space-y-6">
          <UpgradePrompt
            currentLevel={userLevel}
            lockedRankings={100 - maxRankings}
            className="max-w-4xl mx-auto"
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
