'use client';
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader
} from '@/components/ui';
import type { StockFinancialData } from '@/types/financialChartData';
import {
  ArrowRight,
  Crown,
  Lock
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

interface PublicRankingPreviewProps {
  className?: string;
  initialData?: StockFinancialData[];
}

export function PublicRankingPreview({
  className,
  initialData,
}: PublicRankingPreviewProps) {
  const [data, setData] = useState<StockFinancialData[]>(initialData || []);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();

  useEffect(() => {
    async function fetchData() {
      try {
        const response = await fetch('/api/public/rankings?type=preview&limit=6');
        if (response.ok) {
          const result = await response.json();
          setData(result);
        }
      } catch (error) {
        console.error('Failed to fetch public rankings:', error);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, []);

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
          <Card key={i} className="">
            <CardHeader>
              <div className="h-4 w-3/4 rounded bg-gray-200"></div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="h-6 rounded bg-gray-200"></div>
                <div className="h-4 w-1/2 rounded bg-gray-200"></div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className={`space-y-8 ${className}`}>
      {/* Analytics-style ranking cards grid */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-4 sm:px-0">
        {(Array.isArray(data) ? data : []).slice(0, 6).map((stock, index) => {
          const latestQuarter = stock.quarters[stock.quarters.length - 1];
          const previousQuarter = stock.quarters[stock.quarters.length - 2];
          const epsGrowth = previousQuarter
            ? ((latestQuarter?.eps - previousQuarter.eps) /
                previousQuarter.eps) *
              100
            : 0;

          const getBorderColor = (rank: number) => {
            if (rank <= 5) return 'border-green-400';
            if (rank <= 10) return 'border-blue-400';
            return 'border-slate-600';
          };

          const displayRank = stock.rank || (index + 6);

          return (
            <div
              key={stock.symbol}
              className={`rounded-3xl bg-slate-800 dark:bg-slate-900 p-6 shadow-xl border-2 ${getBorderColor(displayRank)} relative overflow-hidden min-w-[280px] max-w-[320px] flex-shrink-0`}
            >
              {/* Rank badge */}
              <div className="flex items-center justify-between mb-6">
                <div className="text-slate-400 text-sm font-medium">
                  RANK #{displayRank}
                </div>
                <div className="bg-green-500 px-4 py-2 rounded-full">
                  <div className="flex items-center gap-2">
                    <span className="text-white font-bold text-sm">📊 View</span>
                  </div>
                </div>
              </div>
              
              {/* Symbol */}
              <h2 className="text-3xl font-bold text-white mb-6">{stock.symbol}</h2>
              
              {/* Status and Next Action */}
              <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-2 bg-green-500/20 px-3 py-2 rounded-full">
                  <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                  <span className="text-green-400 font-semibold text-sm">ACTIVE</span>
                </div>
                <div className="text-right">
                  <div className="text-slate-400 text-xs mb-1">Next Action</div>
                  <div className="text-green-400 font-bold">66 days</div>
                </div>
              </div>
              
              {/* Progress bar */}
              <div className="mb-6">
                <div className="w-full bg-slate-700 dark:bg-slate-600 h-2 rounded-full">
                  <div className="h-full bg-green-500 rounded-full" style={{ width: '75%' }}></div>
                </div>
              </div>
              
              {/* Growth section */}
              <div className="bg-slate-900/50 dark:bg-slate-800/50 rounded-2xl p-4 mb-4">
                <div className="flex items-center gap-2 mb-3">
                  <span className="text-xl">{epsGrowth >= 0 ? '📈' : '📉'}</span>
                  <span className="text-slate-400 font-medium">Growth</span>
                </div>
                <div className={`text-2xl font-bold ${epsGrowth >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                  {epsGrowth >= 0 ? '+' : ''}{epsGrowth ? epsGrowth.toFixed(2) : '0.00'}%
                </div>
              </div>
              
              {/* Price section */}
              <div className="bg-slate-900/50 dark:bg-slate-800/50 rounded-2xl p-4">
                <div className="flex items-center gap-2 mb-3">
                  <span className="text-xl">💰</span>
                  <span className="text-slate-400 font-medium">Price</span>
                </div>
                <div className="text-2xl font-bold text-white">
                  $
                  {stock.currentPrice
                    ? stock.currentPrice.toFixed(2)
                    : latestQuarter?.price?.toFixed(2) || '0.00'}
                </div>
              </div>
            </div>
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
                  <Lock className="absolute -top-1 -right-1 h-6 w-6 rounded-full bg-white p-1 text-gray-600" />
                </div>
              </div>

              <div>
                <h3 className="mb-3 text-2xl font-bold">
                  🚀 Access Top 100 Rankings
                </h3>
                <p className="text-muted-foreground mb-2 text-lg">
                  You&apos;re seeing rankings #101-105. Unlock the top
                  performers!
                </p>
                <div className="flex flex-wrap justify-center gap-2 text-sm">
                  <Badge variant="secondary">✨ Top 100 Entities</Badge>
                  <Badge variant="secondary">📊 Advanced Analytics</Badge>
                  <Badge variant="secondary">📈 Growth Insights</Badge>
                  <Badge variant="secondary">🎯 Performance Optimization</Badge>
                </div>
              </div>

              <div className="flex flex-col justify-center gap-4 sm:flex-row">
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

            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
