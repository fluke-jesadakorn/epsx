'use client';
import {
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader
} from '@/components/ui';
import type { QuarterData, StockFinancialData } from '@/types/financialChartData';
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

interface RankStyle {
  wrapper: string;
  badgeWrapper: string;
  badge: string;
  title: string;
  titleColor: string;
  dividerColor: string;
  button: string;
}

const BUTTON_CLASS = 'bg-gradient-to-r from-green-600 via-green-500 to-emerald-600 hover:from-green-700 hover:via-green-600 hover:to-emerald-700 shadow-lg shadow-green-500/20';

const getRankStyle = (rank: number): RankStyle => {
  if (rank === 1) {
    return {
      wrapper: 'bg-gradient-to-b from-amber-100 via-yellow-50 to-orange-50 dark:!bg-[#0f172a] dark:bg-none border-yellow-400 dark:border-yellow-500 shadow-[0_0_30px_rgba(251,191,36,0.3)] dark:shadow-[0_0_25px_rgba(234,179,8,0.3)]',
      badgeWrapper: 'bg-gradient-to-br from-yellow-400 to-amber-500 ring-4 ring-white dark:ring-slate-900',
      badge: '👑',
      title: '👑 CHAMPION',
      titleColor: 'text-amber-600 dark:text-amber-400',
      dividerColor: 'from-amber-500 to-yellow-500',
      button: BUTTON_CLASS,
    };
  }

  if (rank === 2) {
    return {
      wrapper: 'bg-gradient-to-b from-slate-100 via-gray-50 to-slate-100 dark:!bg-[#0f172a] dark:bg-none border-slate-300 dark:border-slate-500 shadow-[0_0_30px_rgba(148,163,184,0.3)] dark:shadow-[0_0_25px_rgba(148,163,184,0.2)]',
      badgeWrapper: 'bg-gradient-to-br from-slate-300 to-slate-400 ring-4 ring-white dark:ring-slate-900',
      badge: '🥈',
      title: '🥈 ELITE',
      titleColor: 'text-slate-500 dark:text-slate-400',
      dividerColor: 'from-slate-400 to-slate-500',
      button: BUTTON_CLASS,
    };
  }

  if (rank === 3) {
    return {
      wrapper: 'bg-gradient-to-b from-orange-100 via-amber-50 to-orange-50 dark:!bg-[#0f172a] dark:bg-none border-orange-400 dark:border-orange-500 shadow-[0_0_30px_rgba(251,146,60,0.3)] dark:shadow-[0_0_25px_rgba(249,115,22,0.3)]',
      badgeWrapper: 'bg-gradient-to-br from-orange-400 to-amber-500 ring-4 ring-white dark:ring-slate-900',
      badge: '🥉',
      title: '🥉 LEGEND',
      titleColor: 'text-orange-600 dark:text-orange-400',
      dividerColor: 'from-orange-500 to-amber-600',
      button: BUTTON_CLASS,
    };
  }

  return {
    wrapper: 'bg-gradient-to-b from-slate-50 to-gray-100 dark:!bg-[#0f172a] dark:bg-none border-slate-200 dark:border-slate-600',
    badgeWrapper: 'bg-slate-400 ring-4 ring-white dark:ring-slate-900',
    badge: '⭐',
    title: `⭐ RANK #${rank}`,
    titleColor: 'text-slate-600 dark:text-slate-400',
    dividerColor: 'from-slate-400 to-slate-500',
    button: BUTTON_CLASS,
  };
};

const calculateEpsGrowth = (latest: QuarterData | undefined, previous: QuarterData | undefined): number => {
  if (previous === undefined || latest === undefined) {
    return 0;
  }
  return ((latest.eps - previous.eps) / previous.eps) * 100;
};

interface RankingCardProps {
  stock: StockFinancialData;
  index: number;
}

const RankingCard = ({ stock, index }: RankingCardProps): JSX.Element => {
  const latestQuarter: QuarterData | undefined = stock.quarters[stock.quarters.length - 1];
  const previousQuarter: QuarterData | undefined = stock.quarters[stock.quarters.length - 2];
  const epsGrowth = calculateEpsGrowth(latestQuarter, previousQuarter);

  const displayRank: number = stock.rank ?? (index + 1);
  const rankStyle = getRankStyle(displayRank);

  return (
    <div
      key={stock.symbol}
      className={`rounded-2xl border-2 p-6 relative overflow-visible min-w-[280px] max-w-[320px] flex-shrink-0 transition-all duration-300 ${rankStyle.wrapper}`}
    >
      <div className={`absolute -top-6 -left-4 w-14 h-14 rounded-full flex items-center justify-center text-2xl shadow-lg z-10 ${rankStyle.badgeWrapper}`}>
        <span className="drop-shadow-md">{rankStyle.badge}</span>
      </div>

      <div className={`text-center mb-1 pt-0 text-xs font-bold tracking-[0.2em] uppercase ${rankStyle.titleColor}`}>
        {rankStyle.title} · RANK #{displayRank}
      </div>

      <h2 className="text-4xl font-black text-slate-800 dark:text-white text-center mb-6 tracking-tight">
        {stock.symbol}
      </h2>

      <div className="flex items-center justify-between mb-2">
        <span className="text-slate-400 dark:text-slate-500 text-[10px] font-bold uppercase tracking-widest">Next Action</span>
        <span className="text-green-500 font-bold text-sm">61d</span>
      </div>

      <div className="mb-6 relative">
        <div className="w-full bg-slate-200 dark:bg-slate-700 h-1.5 rounded-full overflow-hidden">
          <div className={`h-full bg-gradient-to-r ${rankStyle.dividerColor} rounded-full`} style={{ width: '75%' }} />
        </div>
      </div>

      <button className={`w-full text-white font-bold py-3.5 px-6 rounded-xl mb-6 flex items-center justify-center gap-2 transition-transform active:scale-95 ${rankStyle.button}`}>
        <span className="text-lg">📊</span>
        <span className="text-sm tracking-wide">VIEW DETAILS</span>
        <ArrowRight className="w-4 h-4" />
      </button>

      <div className="space-y-3">
        <div className="bg-slate-700/50 dark:bg-slate-800/80 backdrop-blur-sm rounded-xl p-4 shadow-sm border border-slate-600/50 dark:border-slate-700/50">
          <div className="flex items-center gap-2 mb-1 justify-center">
            <span className="text-base">📈</span>
            <span className="text-slate-200 dark:text-slate-400 font-medium text-xs uppercase tracking-wider">Growth</span>
          </div>
          <div className={`text-2xl font-black text-center ${epsGrowth >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
            {epsGrowth >= 0 ? '+' : ''}{epsGrowth !== 0 ? epsGrowth.toFixed(2) : '0.00'}%
          </div>
        </div>

        <div className="bg-slate-700/50 dark:bg-slate-800/60 backdrop-blur-sm rounded-xl p-4 shadow-sm border border-slate-600/50 dark:border-slate-700/50">
          <div className="flex items-center gap-2 mb-1 justify-center">
            <span className="text-base">💰</span>
            <span className="text-slate-200 dark:text-slate-400 font-medium text-xs uppercase tracking-wider">Price</span>
          </div>
          <div className="text-2xl font-black text-white dark:text-slate-200 text-center">
            ${stock.currentPrice !== null && stock.currentPrice !== undefined
              ? stock.currentPrice.toFixed(2)
              : (latestQuarter?.price !== null && latestQuarter?.price !== undefined ? latestQuarter.price.toFixed(2) : '0.00')}
          </div>
        </div>
      </div>
    </div>
  );
};

export function PublicRankingPreview({
  className,
  initialData,
}: PublicRankingPreviewProps) {
  const [data, setData] = useState<StockFinancialData[]>(initialData ?? []);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();

  useEffect(() => {
    async function fetchData(): Promise<void> {
      try {
        const response = await fetch('/api/public/rankings?type=preview&limit=6');
        if (response.ok) {
          const result: unknown = await response.json();
          if (Array.isArray(result)) {
            setData(result as StockFinancialData[]);
          }
        }
      } catch (_error) {
        // Error logged silently
      } finally {
        setIsLoading(false);
      }
    }

    void fetchData();
  }, []);

  const handleUpgrade = (): void => {
    router.push('/payment');
  };

  const handleViewMore = (): void => {
    router.push('/analytics');
  };

  if (isLoading) {
    return (
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <Card key={`loading-${String(i)}`} className="">
            <CardHeader>
              <div className="h-4 w-3/4 rounded bg-gray-200" />
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="h-6 rounded bg-gray-200" />
                <div className="h-4 w-1/2 rounded bg-gray-200" />
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
        {data.slice(0, 6).map((stock: StockFinancialData, index: number) => (
          <RankingCard key={stock.symbol} stock={stock} index={index} />
        ))}
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
