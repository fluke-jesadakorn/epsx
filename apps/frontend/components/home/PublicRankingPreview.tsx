'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import {
  TrendingUp,
  TrendingDown,
  Lock,
  Crown,
  ArrowRight,
} from 'lucide-react';
import { fetchPublicRankingData } from '@/app/actions/publicRanking';
import type { StockFinancialData } from '@/types/financialChartData';
import { useRouter } from 'next/navigation';

interface PublicRankingPreviewProps {
  className?: string;
  initialData?: StockFinancialData[];
}

export function PublicRankingPreview({ className, initialData }: PublicRankingPreviewProps) {
  const [data, setData] = useState<StockFinancialData[]>(initialData || []);
  const [isLoading, setIsLoading] = useState(!initialData);
  const router = useRouter();

  useEffect(() => {
    // Only fetch if no initial data provided (fallback for client-side usage)
    if (!initialData) {
      const loadData = async () => {
        try {
          const publicData = await fetchPublicRankingData(10, 10);
          setData(publicData);
        } catch (error) {
          console.error('Failed to load public ranking data:', error);
        } finally {
          setIsLoading(false);
        }
      };
      loadData();
    }
  }, [initialData]);

  const handleUpgrade = () => {
    router.push('/payment');
  };

  const handleViewMore = () => {
    router.push('/analytics');
  };

  if (isLoading) {
    return (
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader>
              <div className="h-4 bg-gray-200 rounded w-3/4"></div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="h-6 bg-gray-200 rounded"></div>
                <div className="h-4 bg-gray-200 rounded w-1/2"></div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className={`space-y-8 ${className}`}>
      {/* Preview Grid */}
      <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {data.slice(0, 6).map((stock, index) => {
          const latestQuarter = stock.quarters[stock.quarters.length - 1];
          const previousQuarter = stock.quarters[stock.quarters.length - 2];
          const epsGrowth = previousQuarter
            ? ((latestQuarter?.eps - previousQuarter.eps) /
                previousQuarter.eps) *
              100
            : 0;

          return (
            <Card
              key={stock.symbol}
              className="relative group hover:shadow-lg transition-all duration-300"
            >
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg flex items-center gap-2">
                    <Badge variant="outline" className="text-xs">
                      #{100 + index}
                    </Badge>
                    {stock.symbol}
                  </CardTitle>
                  <div className="flex items-center gap-1">
                    {epsGrowth > 0 ? (
                      <TrendingUp className="h-4 w-4 text-green-500" />
                    ) : (
                      <TrendingDown className="h-4 w-4 text-red-500" />
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">
                      Index Growth
                    </span>
                    <span
                      className={`font-semibold ${
                        epsGrowth > 0 ? 'text-green-600' : 'text-red-600'
                      }`}
                    >
                      {epsGrowth ? `${epsGrowth.toFixed(2)}%` : 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">
                      Current Value
                    </span>
                    <span className="font-medium">
                      $
                      {stock.currentPrice
                        ? stock.currentPrice.toFixed(2)
                        : latestQuarter?.price?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">
                      Latest Index
                    </span>
                    <span className="text-sm font-medium">
                      ${latestQuarter?.eps?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Upgrade Section */}
      <div className="relative">
        <Card className="border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20">
          <CardContent className="p-8 text-center">
            <div className="space-y-6">
              <div className="flex justify-center">
                <div className="relative">
                  <Crown className="h-16 w-16 text-yellow-500" />
                  <Lock className="h-6 w-6 text-gray-600 absolute -top-1 -right-1 bg-white rounded-full p-1" />
                </div>
              </div>

              <div>
                <h3 className="text-2xl font-bold mb-3">
                  🚀 Access Top 100 Rankings
                </h3>
                <p className="text-lg text-muted-foreground mb-2">
                  You&apos;re seeing rankings #100-110. Unlock the top performers!
                </p>
                <div className="flex flex-wrap justify-center gap-2 text-sm">
                  <Badge variant="secondary">✨ Top 100 Entities</Badge>
                  <Badge variant="secondary">📊 Advanced Analytics</Badge>
                  <Badge variant="secondary">📈 Growth Insights</Badge>
                  <Badge variant="secondary">🎯 Performance Optimization</Badge>
                </div>
              </div>

              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button size="lg" onClick={handleUpgrade} className="gap-2">
                  <Crown className="h-5 w-5" />
                  Upgrade to Premium
                  <ArrowRight className="h-4 w-4" />
                </Button>
                <Button
                  size="lg"
                  variant="outline"
                  onClick={handleViewMore}
                  className="gap-2"
                >
                  View Analytics Demo
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </div>

              <div className="text-xs text-muted-foreground">
                Starting from $1/month • 30-day money-back guarantee
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
