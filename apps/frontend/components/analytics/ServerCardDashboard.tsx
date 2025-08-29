import { Card, CardContent } from '@/components/ui/card';
import {
  getAnalyticsData,
  type EPSQueryParams,
  type SymbolCardData,
} from '@/lib/analytics-server';
import { Suspense } from 'react';
import ServerFilters from './ServerFilters';
import ServerPagination from './ServerPagination';

interface ServerCardDashboardProps {
  searchParams: Promise<{
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    showFilters?: string;
    search?: string;
  }>;
}

function parseSearchParams(
  searchParams: Awaited<ServerCardDashboardProps['searchParams']>
): EPSQueryParams {
  return {
    page: parseInt(searchParams.page || '1', 10),
    limit: parseInt(searchParams.limit || '10', 10),
    country: searchParams.country || undefined,
    sector: searchParams.sector || undefined,
    sort_by: searchParams.sort_by || 'growth_factor',
    min_eps: searchParams.min_eps
      ? parseFloat(searchParams.min_eps)
      : undefined,
    min_growth: searchParams.min_growth
      ? parseFloat(searchParams.min_growth)
      : undefined,
    search: searchParams.search || undefined,
  };
}

const formatCurrency = (value: number) => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
};

const formatPercentage = (value: number) => {
  return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

const SymbolCard = ({ cardData }: { cardData: SymbolCardData }) => {
  const quarters = cardData.quarterly_performance?.slice(0, 2) || [];
  const latestQuarter = quarters[0];
  const previousQuarter = quarters[1];

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'TRACK':
        return {
          bg: 'bg-gradient-to-br from-emerald-400 via-green-400 to-teal-500',
          border: 'border-green-300/60',
          cardBg:
            'bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20',
          text: 'text-green-700 dark:text-green-300',
        };
      case 'WATCH':
        return {
          bg: 'bg-gradient-to-br from-yellow-400 via-orange-400 to-amber-500',
          border: 'border-yellow-300/60',
          cardBg:
            'bg-gradient-to-br from-yellow-50 via-orange-50 to-amber-50 dark:from-yellow-900/20 dark:via-orange-900/20 dark:to-amber-900/20',
          text: 'text-yellow-700 dark:text-yellow-300',
        };
      case 'STOP':
        return {
          bg: 'bg-gradient-to-br from-red-400 via-rose-400 to-pink-500',
          border: 'border-red-300/60',
          cardBg:
            'bg-gradient-to-br from-red-50 via-rose-50 to-pink-50 dark:from-red-900/20 dark:via-rose-900/20 dark:to-pink-900/20',
          text: 'text-red-700 dark:text-red-300',
        };
      default:
        return {
          bg: 'bg-gradient-to-br from-emerald-400 via-green-400 to-teal-500',
          border: 'border-green-300/60',
          cardBg:
            'bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20',
          text: 'text-green-700 dark:text-green-300',
        };
    }
  };

  const statusConfig = getStatusConfig(cardData.active_status);
  const daysUntil =
    cardData.next_quarter_estimate?.days_until_announcement || 185;

  // Calculate progress (assuming 90 days max between quarters)
  const maxDays = 90;
  const progressPercentage = Math.max(
    0,
    Math.min(100, ((maxDays - daysUntil) / maxDays) * 100)
  );

  return (
    <div
      className={`relative overflow-hidden rounded-2xl border-2 ${statusConfig.border} ${statusConfig.cardBg} shadow-lg p-4 backdrop-blur-sm transition-all duration-300 hover:scale-105 hover:rotate-1 hover:shadow-xl`}
    >
      <div className="absolute -top-8 -right-8 h-16 w-16 rounded-full bg-white/20 blur-xl" />

      {/* Header with PancakeSwap-style badge */}
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div
            className={`flex h-8 w-8 items-center justify-center rounded-full ${statusConfig.bg} shadow-lg transition-all duration-300`}
          >
            <span className="text-xs font-bold text-white">
              {cardData.rank}
            </span>
          </div>
          <div className="flex flex-col">
            <h3 className="text-lg font-bold text-slate-800 dark:text-slate-100">
              {cardData.symbol}
            </h3>
          </div>
        </div>
        <div className="flex flex-col items-end gap-1">
          <a
            href={`https://www.tradingview.com/symbols/${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
            target="_blank"
            rel="noopener noreferrer"
            className={`rounded-full px-3 py-1 text-xs font-medium transition-all hover:scale-110 ${statusConfig.bg} text-white shadow-md`}
          >
            View 🔗
          </a>
        </div>
      </div>

      {/* Status Badge */}
      <div className="mb-3 flex justify-center">
        <div
          className={`inline-flex items-center gap-2 rounded-full px-4 py-2 ${statusConfig.bg} shadow-lg`}
        >
          <div className="h-2 w-2 animate-pulse rounded-full bg-white/80" />
          <span className="text-sm font-bold text-white">
            {cardData.active_status}
          </span>
        </div>
      </div>

      {/* Progress Bar for Next Action */}
      <div className="mb-3">
        <div className="mb-1 flex items-center justify-between text-xs font-medium text-slate-700 dark:text-white">
          <span>Next Action</span>
          <span>{daysUntil}d left</span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-white/60 shadow-inner">
          <div
            className={`h-full rounded-full transition-all duration-1000 ${statusConfig.bg} shadow-sm`}
            style={{ width: `${progressPercentage}%` }}
          />
        </div>
      </div>

      {/* Essential Data */}
      <div className="grid grid-cols-2 gap-3 text-xs">
        <div
          className={`rounded-xl text-center p-2 shadow-lg backdrop-blur-sm ${
            (latestQuarter?.eps_growth || 0) >= 0
              ? 'bg-gradient-to-br from-green-500 to-emerald-600'
              : 'bg-gradient-to-br from-red-500 to-rose-600'
          }`}
        >
          <div className="mb-1 font-bold text-white/90 text-xs">
            Growth
          </div>
          <div className="font-bold text-white text-sm">
            {formatPercentage(latestQuarter?.eps_growth || 0)}
          </div>
        </div>
        <div className="rounded-xl text-center p-2 shadow-lg backdrop-blur-sm bg-white/50 dark:bg-slate-700/60">
          <div className="text-slate-600 dark:text-slate-300 mb-1 font-bold text-xs">
            Price
          </div>
          <div className="font-bold text-sm text-slate-800 dark:text-white">
            {formatCurrency(latestQuarter?.price || 0)}
          </div>
        </div>
      </div>
    </div>
  );
};

const Top5SpecialBox = ({ top5Data }: { top5Data: SymbolCardData[] }) => {
  const getTopRankStyle = (rank: number) => {
    if (rank === 1)
      return {
        crown: '👑',
        glow: 'shadow-2xl shadow-yellow-500/80 hover:shadow-3xl hover:shadow-yellow-400/90',
        border: 'border-4 border-yellow-400 hover:border-yellow-300',
        bg: 'bg-gradient-to-br from-yellow-200 via-amber-100 to-orange-200 dark:from-yellow-800 dark:via-amber-700 dark:to-orange-800',
        sparkle: '✨',
        special: '🏆 CHAMPION',
      };
    if (rank === 2)
      return {
        crown: '🥈',
        glow: 'shadow-2xl shadow-slate-500/80 hover:shadow-3xl hover:shadow-slate-400/90',
        border: 'border-4 border-slate-400 hover:border-slate-300',
        bg: 'bg-gradient-to-br from-slate-200 via-gray-100 to-zinc-200 dark:from-slate-800 dark:via-gray-700 dark:to-zinc-800',
        sparkle: '🌟',
        special: '🥈 ELITE',
      };
    if (rank === 3)
      return {
        crown: '🥉',
        glow: 'shadow-2xl shadow-orange-500/80 hover:shadow-3xl hover:shadow-orange-400/90',
        border: 'border-4 border-orange-400 hover:border-orange-300',
        bg: 'bg-gradient-to-br from-orange-200 via-amber-100 to-yellow-200 dark:from-orange-800 dark:via-amber-700 dark:to-yellow-800',
        sparkle: '💫',
        special: '🥉 LEGEND',
      };
    if (rank === 4)
      return {
        crown: '⭐',
        glow: 'shadow-2xl shadow-purple-500/80 hover:shadow-3xl hover:shadow-purple-400/90',
        border: 'border-4 border-purple-400 hover:border-purple-300',
        bg: 'bg-gradient-to-br from-purple-200 via-pink-100 to-fuchsia-200 dark:from-purple-800 dark:via-pink-700 dark:to-fuchsia-800',
        sparkle: '🌟',
        special: '⭐ MASTER',
      };
    if (rank === 5)
      return {
        crown: '💎',
        glow: 'shadow-2xl shadow-cyan-500/80 hover:shadow-3xl hover:shadow-cyan-400/90',
        border: 'border-4 border-cyan-400 hover:border-cyan-300',
        bg: 'bg-gradient-to-br from-cyan-200 via-blue-100 to-sky-200 dark:from-cyan-800 dark:via-blue-700 dark:to-sky-800',
        sparkle: '✨',
        special: '💎 DIAMOND',
      };
    return null;
  };

  const formatPercentage = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
  };

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  };

  const getStatusConfig = (status: string) => {
    switch (status) {
      case 'TRACK':
        return {
          bg: 'bg-gradient-to-br from-emerald-400 via-green-400 to-teal-500',
          border: 'border-green-300/60',
          cardBg:
            'bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20',
          text: 'text-green-700 dark:text-green-300',
        };
      case 'WATCH':
        return {
          bg: 'bg-gradient-to-br from-yellow-400 via-orange-400 to-amber-500',
          border: 'border-yellow-300/60',
          cardBg:
            'bg-gradient-to-br from-yellow-50 via-orange-50 to-amber-50 dark:from-yellow-900/20 dark:via-orange-900/20 dark:to-amber-900/20',
          text: 'text-yellow-700 dark:text-yellow-300',
        };
      case 'STOP':
        return {
          bg: 'bg-gradient-to-br from-red-400 via-rose-400 to-pink-500',
          border: 'border-red-300/60',
          cardBg:
            'bg-gradient-to-br from-red-50 via-rose-50 to-pink-50 dark:from-red-900/20 dark:via-rose-900/20 dark:to-pink-900/20',
          text: 'text-red-700 dark:text-red-300',
        };
      default:
        return {
          bg: 'bg-gradient-to-br from-emerald-400 via-green-400 to-teal-500',
          border: 'border-green-300/60',
          cardBg:
            'bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20',
          text: 'text-green-700 dark:text-green-300',
        };
    }
  };

  return (
    <div className="mb-8">
      {/* Special Header */}
      <div className="mb-6 text-center">
        <div className="relative inline-block">
          {/* Floating sparkles */}
          <div className="absolute -top-6 -left-6 animate-ping text-2xl">
            ✨
          </div>
          <div
            className="absolute -top-4 -right-6 animate-pulse text-xl"
            style={{ animationDelay: '0.5s' }}
          >
            🌟
          </div>
          <div
            className="absolute -bottom-4 -left-4 animate-bounce text-lg"
            style={{ animationDelay: '1s' }}
          >
            💫
          </div>
          <div
            className="absolute -right-4 -bottom-6 animate-spin text-2xl"
            style={{ animationDuration: '3s' }}
          >
            ⭐
          </div>

          <div className="inline-flex items-center gap-3 rounded-full bg-gradient-to-r from-purple-500 via-pink-500 to-red-500 px-8 py-4 shadow-2xl ring-4 shadow-purple-500/40 ring-purple-200 ring-offset-4 ring-offset-white">
            <span className="animate-bounce text-3xl">👑</span>
            <h2 className="text-2xl font-bold tracking-wide text-white">
              TOP 5 ULTIMATE LEGENDS
            </h2>
            <span
              className="animate-bounce text-3xl"
              style={{ animationDelay: '0.5s' }}
            >
              👑
            </span>
          </div>
        </div>
        <p className="mt-4 bg-gradient-to-r from-purple-600 via-pink-600 to-red-600 bg-clip-text text-lg font-semibold text-transparent">
          🏆 The Ultimate Elite Circle - Where Champions Reign Supreme 🏆
        </p>
      </div>

      {/* Special Box Container */}
      <div className="rounded-3xl bg-gradient-to-br from-purple-900 via-pink-900 to-red-900 p-4 shadow-2xl ring-4 ring-purple-300/50 ring-offset-4 ring-offset-white">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
          {top5Data.map((cardData) => {
            const quarters = cardData.quarterly_performance?.slice(0, 2) || [];
            const latestQuarter = quarters[0];
            const statusConfig = getStatusConfig(cardData.active_status);
            
            return (
              <div
                key={cardData.symbol}
                className={`relative overflow-hidden rounded-xl border-2 ${statusConfig.border} ${statusConfig.cardBg} shadow-lg p-3 backdrop-blur-sm transition-all duration-300 hover:scale-105 hover:rotate-1 hover:shadow-xl`}
              >
                <div className="absolute -top-8 -right-8 h-16 w-16 rounded-full bg-white/20 blur-xl" />

                {/* Header */}
                <div className="mb-2 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className={`flex h-8 w-8 items-center justify-center rounded-full ${statusConfig.bg} shadow-lg transition-all duration-300`}>
                      <span className="text-xs font-bold text-white">
                        {cardData.rank}
                      </span>
                    </div>
                    <div className="flex flex-col">
                      <h3 className="text-lg font-bold text-slate-800 dark:text-slate-100">
                        {cardData.symbol}
                      </h3>
                    </div>
                  </div>
                  <div className="flex flex-col items-end gap-1">
                    <a
                      href={`https://www.tradingview.com/symbols/${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className={`rounded-full px-3 py-1 text-xs font-medium transition-all hover:scale-110 ${statusConfig.bg} text-white shadow-md`}
                    >
                      View 🔗
                    </a>
                  </div>
                </div>

                {/* Status Badge */}
                <div className="mb-2 flex justify-center">
                  <div className={`inline-flex items-center gap-2 rounded-full px-4 py-2 ${statusConfig.bg} shadow-lg`}>
                    <div className="h-2 w-2 animate-pulse rounded-full bg-white/80" />
                    <span className="text-sm font-bold text-white">
                      {cardData.active_status}
                    </span>
                  </div>
                </div>

                {/* Progress Bar */}
                <div className="mb-2">
                  <div className="mb-1 flex items-center justify-between text-xs font-medium text-slate-700 dark:text-white">
                    <span>Next Action</span>
                    <span>{cardData.next_quarter_estimate?.days_until_announcement || 185}d left</span>
                  </div>
                  <div className="h-2 overflow-hidden rounded-full bg-white/60 shadow-inner">
                    <div
                      className={`h-full rounded-full transition-all duration-1000 ${statusConfig.bg} shadow-sm`}
                      style={{ width: `${Math.max(0, Math.min(100, ((90 - (cardData.next_quarter_estimate?.days_until_announcement || 185)) / 90) * 100))}%` }}
                    />
                  </div>
                </div>

                {/* Essential Data */}
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div
                    className={`rounded-lg text-center p-1.5 shadow-md backdrop-blur-sm ${
                      (latestQuarter?.eps_growth || 0) >= 0
                        ? 'bg-gradient-to-br from-green-500 to-emerald-600'
                        : 'bg-gradient-to-br from-red-500 to-rose-600'
                    }`}
                  >
                    <div className="mb-0.5 font-bold text-white/90 text-xs">
                      Growth
                    </div>
                    <div className="font-bold text-white text-xs">
                      {formatPercentage(latestQuarter?.eps_growth || 0)}
                    </div>
                  </div>
                  <div className="rounded-lg text-center p-1.5 shadow-md backdrop-blur-sm bg-white/50 dark:bg-slate-700/60">
                    <div className="text-slate-600 dark:text-slate-300 mb-0.5 font-bold text-xs">
                      Price
                    </div>
                    <div className="font-bold text-xs text-slate-800 dark:text-white">
                      {formatCurrency(latestQuarter?.price || 0)}
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

async function CardGrid({ params }: { params: EPSQueryParams }) {
  const data = await getAnalyticsData(params);

  if (!data.success || !data.data || data.data.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="mb-4 text-gray-600 dark:text-white">No data available</p>
      </div>
    );
  }

  const isFirstPage = params.page === 1;
  const hasTopRanks = data.data.some(card => card.rank <= 5);
  const top5Data = data.data.filter(card => card.rank <= 5);

  return (
    <>
      {/* Show special Top 5 box only on first page with top 5 ranks */}
      {isFirstPage && hasTopRanks && top5Data.length > 0 && (
        <Top5SpecialBox top5Data={top5Data} />
      )}

      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
        {data.data.map(cardData =>
          cardData && cardData.symbol ? (
            <SymbolCard key={cardData.symbol} cardData={cardData} />
          ) : null
        )}
      </div>

      {data.pagination && data.pagination.totalPages > 1 && (
        <div className="mt-8">
          <ServerPagination
            pagination={data.pagination}
            currentParams={new URLSearchParams({
              page: String(params.page),
              limit: String(params.limit),
              ...(params.country && { country: params.country }),
              ...(params.sector && { sector: params.sector }),
              ...(params.sort_by && { sort_by: params.sort_by }),
              ...(params.min_eps !== undefined && {
                min_eps: String(params.min_eps),
              }),
              ...(params.min_growth !== undefined && {
                min_growth: String(params.min_growth),
              }),
              ...(params.search && { search: params.search }),
            }).toString()}
          />
        </div>
      )}
    </>
  );
}

function LoadingGrid() {
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
      {Array.from({ length: 12 }).map((_, i) => (
        <div
          key={i}
          className="animate-pulse rounded-lg border bg-white p-4 dark:bg-slate-900"
        >
          <div className="mb-3 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="h-2 w-2 rounded-full bg-gray-300" />
              <div className="h-5 w-12 rounded bg-gray-300" />
              <div className="h-4 w-8 rounded bg-gray-200" />
            </div>
            <div className="h-4 w-12 rounded bg-gray-200" />
          </div>
          <div className="mb-3 flex justify-center">
            <div className="h-6 w-16 rounded bg-gray-300" />
          </div>
          <div className="space-y-2">
            {Array.from({ length: 3 }).map((_, j) => (
              <div key={j} className="flex justify-between">
                <div className="h-3 w-1/3 rounded bg-gray-200" />
                <div className="h-3 w-1/4 rounded bg-gray-200" />
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export default async function ServerCardDashboard({
  searchParams,
}: ServerCardDashboardProps) {
  const resolvedSearchParams = await searchParams;
  const params = parseSearchParams(resolvedSearchParams);
  // Default to not showing filters permanently - always start with filters hidden
  const showFilters = resolvedSearchParams.showFilters === 'true';

  return (
    <div className="space-y-6">
      {/* Header with search and filters */}
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="bg-gradient-to-r from-pink-600 via-orange-600 to-yellow-600 bg-clip-text text-2xl font-bold text-transparent dark:from-pink-400 dark:via-orange-400 dark:to-yellow-400">
              🍰 Performance Analytics
            </h2>
            <Suspense
              fallback={
                <div className="text-slate-600 dark:text-slate-200">
                  Loading sweet stats...
                </div>
              }
            >
              <StatsDisplay params={params} />
            </Suspense>
          </div>
        </div>
      </div>

      {/* Filters - Show by default but not permanently adjustable */}
      <Suspense
        fallback={
          <div className="text-slate-600 dark:text-slate-200">
            Loading filters...
          </div>
        }
      >
        <ServerFilters currentParams={params} />
      </Suspense>

      {/* PancakeSwap-inspired Status Legend */}
      <Card className="border-2 border-pink-200/60 bg-gradient-to-r from-pink-50/80 via-orange-50/60 to-yellow-50/80 shadow-xl shadow-pink-500/10 backdrop-blur-sm dark:border-pink-400/30 dark:bg-gradient-to-r dark:from-pink-900/20 dark:via-orange-900/20 dark:to-yellow-900/20">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <h4 className="bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text text-sm font-bold text-transparent dark:from-pink-400 dark:to-orange-400">
                🎯 Legend :
              </h4>
            </div>
            <div className="flex items-center gap-4">
              <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-emerald-400 to-green-500 px-3 py-1 shadow-lg transition-all hover:scale-105">
                <div className="h-2 w-2 animate-pulse rounded-full bg-white/90"></div>
                <span className="text-xs font-bold text-white">TRACK</span>
              </div>
              <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 px-3 py-1 shadow-lg transition-all hover:scale-105">
                <div
                  className="h-2 w-2 animate-pulse rounded-full bg-white/90"
                  style={{ animationDelay: '0.3s' }}
                ></div>
                <span className="text-xs font-bold text-white">WATCH</span>
              </div>
              <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-red-400 to-rose-500 px-3 py-1 shadow-lg transition-all hover:scale-105">
                <div
                  className="h-2 w-2 animate-pulse rounded-full bg-white/90"
                  style={{ animationDelay: '0.6s' }}
                ></div>
                <span className="text-xs font-bold text-white">STOP</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Cards grid */}
      <Suspense fallback={<LoadingGrid />}>
        <CardGrid params={params} />
      </Suspense>
    </div>
  );
}

async function StatsDisplay({ params }: { params: EPSQueryParams }) {
  const data = await getAnalyticsData(params);
  const isFirstPage = params.page === 1;
  const hasTopRanks = data.data && data.data.some(card => card.rank <= 5);

  return (
    <p className="text-gray-600 dark:text-slate-200">
      {isFirstPage && hasTopRanks
        ? `👑 Showing ${data.data?.length || 0} ultimate legends from ${data.pagination?.total || 0} companies`
        : `Showing ${data.data?.length || 0} of ${data.pagination?.total || 0} companies`}
      {data.processing_time_ms && (
        <span className="ml-2 text-sm text-gray-500 dark:text-slate-300">
          • Lightning fast {data.processing_time_ms}ms
        </span>
      )}
    </p>
  );
}
