import React from 'react';
import { TrendingUp, TrendingDown } from 'lucide-react';
import { fetchPublicRankingData } from '@/app/actions/publicRanking';
import type { StockFinancialData } from '@/types/financialChartData';
import { PublicRankingPreviewClient } from './PublicRankingPreview.client';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@/components/ui';

interface PublicRankingPreviewServerProps {
  className?: string;
}

async function getPublicRankingData(): Promise<StockFinancialData[]> {
  try {
    return await fetchPublicRankingData(10, 10);
  } catch (error) {
    console.error('Failed to load public ranking data:', error);
    return [];
  }
}

export async function PublicRankingPreviewServer({ className }: PublicRankingPreviewServerProps) {
  const data = await getPublicRankingData();

  if (!data.length) {
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
      {/* Server-rendered preview grid */}
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

      {/* Client component for interactive features */}
      <PublicRankingPreviewClient />
    </div>
  );
}