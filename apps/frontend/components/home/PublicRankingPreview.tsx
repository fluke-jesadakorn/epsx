'use client';
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui';
import type { StockFinancialData } from '@/types/financialChartData';
import {
  ArrowRight,
  Crown,
  Lock,
  TrendingDown,
  TrendingUp,
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
  const [data] = useState<StockFinancialData[]>(initialData || []);
  const [isLoading, setIsLoading] = useState(!initialData);
  const router = useRouter();

  useEffect(() => {
    // If no initial data provided, show empty state
    if (!initialData) {
      setIsLoading(false);
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
      {/* Preview Grid */}
      <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {(Array.isArray(data) ? data : []).slice(0, 6).map((stock, index) => {
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
              className="group relative transition-all duration-300 hover:shadow-lg"
            >
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2 text-lg">
                    <Badge variant="outline" className="text-xs">
                      #{stock.rank || 101 + index}
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
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">
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
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">
                      Current Value
                    </span>
                    <span className="font-medium">
                      $
                      {stock.currentPrice
                        ? stock.currentPrice.toFixed(2)
                        : latestQuarter?.price?.toFixed(2) || 'N/A'}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground text-sm">
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

              <div className="text-muted-foreground text-xs">
                Starting from $1/month • 30-day money-back guarantee
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
