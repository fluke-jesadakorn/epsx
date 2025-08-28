import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import {
  getAnalyticsData,
  type EPSQueryParams,
  type SymbolCardData,
} from '@/lib/analytics-server';
import { Filter, Search } from 'lucide-react';
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
          cardBg: 'bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20',
          text: 'text-green-700 dark:text-green-300'
        };
      case 'WATCH': 
        return { 
          bg: 'bg-gradient-to-br from-yellow-400 via-orange-400 to-amber-500',
          border: 'border-yellow-300/60',
          cardBg: 'bg-gradient-to-br from-yellow-50 via-orange-50 to-amber-50 dark:from-yellow-900/20 dark:via-orange-900/20 dark:to-amber-900/20',
          text: 'text-yellow-700 dark:text-yellow-300'
        };
      case 'STOP': 
        return { 
          bg: 'bg-gradient-to-br from-red-400 via-rose-400 to-pink-500',
          border: 'border-red-300/60',
          cardBg: 'bg-gradient-to-br from-red-50 via-rose-50 to-pink-50 dark:from-red-900/20 dark:via-rose-900/20 dark:to-pink-900/20',
          text: 'text-red-700 dark:text-red-300'
        };
      default: 
        return { 
          bg: 'bg-gradient-to-br from-emerald-400 via-green-400 to-teal-500',
          border: 'border-green-300/60',
          cardBg: 'bg-gradient-to-br from-green-50 via-emerald-50 to-teal-50 dark:from-green-900/20 dark:via-emerald-900/20 dark:to-teal-900/20',
          text: 'text-green-700 dark:text-green-300'
        };
    }
  };

  const statusConfig = getStatusConfig(cardData.active_status);
  const daysUntil = cardData.next_quarter_estimate?.days_until_announcement || 185;
  
  // Calculate progress (assuming 90 days max between quarters)
  const maxDays = 90;
  const progressPercentage = Math.max(0, Math.min(100, ((maxDays - daysUntil) / maxDays) * 100));

  return (
    <div className={`relative overflow-hidden rounded-2xl border-2 ${statusConfig.border} ${statusConfig.cardBg} p-4 shadow-lg backdrop-blur-sm transition-all duration-300 hover:scale-105 hover:shadow-xl`}>
      {/* Decorative gradient orb */}
      <div className="absolute -top-8 -right-8 h-16 w-16 rounded-full bg-white/20 blur-xl" />
      
      {/* Header with PancakeSwap-style badge */}
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className={`flex h-8 w-8 items-center justify-center rounded-full ${statusConfig.bg} shadow-lg`}>
            <span className="text-xs font-bold text-white">{cardData.rank}</span>
          </div>
          <h3 className="text-lg font-bold text-slate-800 dark:text-slate-100">{cardData.symbol}</h3>
        </div>
        <a
          href={`https://www.tradingview.com/symbols/NASDAQ-${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
          target="_blank"
          rel="noopener noreferrer"
          className={`rounded-full px-3 py-1 text-xs font-medium transition-all hover:scale-110 ${statusConfig.bg} text-white shadow-md`}
        >
          View 🔗
        </a>
      </div>

      {/* Status Badge */}
      <div className="mb-3 flex justify-center">
        <div className={`inline-flex items-center gap-2 rounded-full px-4 py-2 ${statusConfig.bg} shadow-lg`}>
          <div className="h-2 w-2 animate-pulse rounded-full bg-white/80" />
          <span className="text-sm font-bold text-white">{cardData.active_status}</span>
        </div>
      </div>

      {/* Main Growth - PancakeSwap style */}
      <div className="mb-3 text-center">
        <div className={`inline-flex items-center gap-2 rounded-xl px-4 py-2 shadow-lg ${
          (latestQuarter?.eps_growth || 0) >= 0 
            ? 'bg-gradient-to-r from-green-400 to-emerald-500' 
            : 'bg-gradient-to-r from-red-400 to-rose-500'
        }`}>
          <span className="text-lg">{(latestQuarter?.eps_growth || 0) >= 0 ? '🚀' : '📉'}</span>
          <span className="text-lg font-bold text-white">{formatPercentage(latestQuarter?.eps_growth || 0)}</span>
        </div>
      </div>

      {/* Progress Bar for Next Action */}
      <div className="mb-3">
        <div className="mb-1 flex items-center justify-between text-xs font-medium text-slate-700 dark:text-slate-300">
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

      {/* Current Quarter */}
      <div className="mb-2 rounded-lg bg-white/50 p-2 backdrop-blur-sm dark:bg-slate-800/50">
        <div className="mb-1 text-xs font-semibold text-slate-700 dark:text-slate-300">
          📊 Current: {latestQuarter?.date || 'Jul 30, 2025'}
        </div>
        <div className="grid grid-cols-2 gap-1 text-xs">
          <div className="text-slate-600 dark:text-slate-400">
            <span className="block">EPS Growth</span>
            <span className="font-semibold">{formatPercentage(latestQuarter?.eps_growth || 0)}</span>
          </div>
          <div className="text-slate-600 dark:text-slate-400">
            <span className="block">EPS Value</span>
            <span className="font-semibold">{(latestQuarter?.eps || 0).toFixed(2)}</span>
          </div>
          <div className="text-slate-600 dark:text-slate-400">
            <span className="block">Price</span>
            <span className="font-semibold">{formatPercentage(latestQuarter?.price_growth || 0)}</span>
          </div>
          <div className="text-slate-600 dark:text-slate-400">
            <span className="block">Value</span>
            <span className="font-semibold">{formatCurrency(latestQuarter?.price || 0)}</span>
          </div>
        </div>
      </div>

      {/* Previous Quarter */}
      {previousQuarter && (
        <div className="rounded-lg bg-white/30 p-2 backdrop-blur-sm dark:bg-slate-800/30">
          <div className="mb-1 text-xs font-semibold text-slate-600 dark:text-slate-400">
            📈 Previous: {previousQuarter.date}
          </div>
          <div className="grid grid-cols-2 gap-1 text-xs">
            <div className="text-slate-500 dark:text-slate-500">
              <span className="block">EPS Growth</span>
              <span className="font-medium">{formatPercentage(previousQuarter.eps_growth || 0)}</span>
            </div>
            <div className="text-slate-500 dark:text-slate-500">
              <span className="block">EPS Value</span>
              <span className="font-medium">{(previousQuarter.eps || 0).toFixed(2)}</span>
            </div>
            <div className="text-slate-500 dark:text-slate-500">
              <span className="block">Price</span>
              <span className="font-medium">{formatPercentage(previousQuarter.price_growth || 0)}</span>
            </div>
            <div className="text-slate-500 dark:text-slate-500">
              <span className="block">Value</span>
              <span className="font-medium">{formatCurrency(previousQuarter.price || 0)}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

async function CardGrid({ params }: { params: EPSQueryParams }) {
  const data = await getAnalyticsData(params);

  if (!data.success || !data.data || data.data.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="mb-4 text-gray-600 dark:text-gray-300">
          No data available
        </p>
      </div>
    );
  }

  return (
    <>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
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
        <div key={i} className="animate-pulse rounded-lg border bg-white p-4 dark:bg-slate-900">
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
            <Suspense fallback={<div className="text-slate-600">Loading sweet stats...</div>}>
              <StatsDisplay params={params} />
            </Suspense>
          </div>

          <div className="flex items-center gap-2">
            <form action="/analytics" method="get">
              {/* Preserve current params */}
              {Object.entries(resolvedSearchParams).map(([key, value]) =>
                key !== 'showFilters' ? (
                  <input key={key} type="hidden" name={key} value={value} />
                ) : null
              )}
              <input
                type="hidden"
                name="showFilters"
                value={showFilters ? 'false' : 'true'}
              />
              <Button
                type="submit"
                className="group flex items-center gap-2 rounded-full bg-gradient-to-r from-purple-500 to-pink-500 px-4 py-2 font-bold text-white shadow-lg transition-all hover:scale-105 hover:shadow-xl"
              >
                <Filter className="h-4 w-4 transition-transform group-hover:rotate-12" />
                Filters
              </Button>
            </form>
          </div>
        </div>

        {/* PancakeSwap-style Search Bar */}
        <div className="flex gap-4">
          <form action="/analytics" method="get" className="flex-1">
            {/* Preserve current non-search params */}
            {Object.entries(resolvedSearchParams).map(([key, value]) =>
              key !== 'search' && key !== 'page' ? (
                <input key={key} type="hidden" name={key} value={value} />
              ) : null
            )}
            <div className="relative max-w-md">
              <div className="absolute left-3 top-1/2 -translate-y-1/2">
                <Search className="h-4 w-4 text-pink-500" />
              </div>
              <Input
                type="text"
                name="search"
                placeholder="🔍 Search symbols (AAPL, MSFT...)"
                defaultValue={resolvedSearchParams.search || ''}
                className="border-2 border-pink-200/60 bg-white/80 pl-10 pr-4 backdrop-blur-sm transition-all focus:border-pink-400 focus:shadow-lg focus:shadow-pink-500/20 dark:border-pink-400/30 dark:bg-slate-800/80"
              />
            </div>
          </form>
          
          {/* PancakeSwap-style Quick Filter Chips */}
          <div className="flex items-center gap-2 flex-wrap">
            <form action="/analytics" method="get">
              {Object.entries(resolvedSearchParams).map(([key, value]) =>
                key !== 'sort_by' && key !== 'page' ? (
                  <input key={key} type="hidden" name={key} value={value} />
                ) : null
              )}
              <input type="hidden" name="sort_by" value="growth_factor" />
              <Button 
                type="submit" 
                className="group rounded-full bg-gradient-to-r from-green-400 to-emerald-500 px-4 py-2 text-xs font-bold text-white shadow-lg transition-all hover:scale-105 hover:shadow-xl"
              >
                🚀 Top Gainers
              </Button>
            </form>
            
            <form action="/analytics" method="get">
              {Object.entries(resolvedSearchParams).map(([key, value]) =>
                key !== 'min_growth' && key !== 'page' ? (
                  <input key={key} type="hidden" name={key} value={value} />
                ) : null
              )}
              <input type="hidden" name="min_growth" value="10" />
              <Button 
                type="submit" 
                className="group rounded-full bg-gradient-to-r from-blue-400 to-cyan-500 px-4 py-2 text-xs font-bold text-white shadow-lg transition-all hover:scale-105 hover:shadow-xl"
              >
                📈 High Growth
              </Button>
            </form>
          </div>
        </div>
      </div>

      {/* Filters */}
      {showFilters && (
        <Suspense fallback={<div>Loading filters...</div>}>
          <ServerFilters currentParams={params} />
        </Suspense>
      )}

      {/* PancakeSwap-inspired Status Legend */}
      <Card className="border-2 border-pink-200/60 bg-gradient-to-r from-pink-50/80 via-orange-50/60 to-yellow-50/80 backdrop-blur-sm shadow-xl shadow-pink-500/10 dark:border-pink-400/30 dark:bg-gradient-to-r dark:from-pink-900/20 dark:via-orange-900/20 dark:to-yellow-900/20">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <h4 className="bg-gradient-to-r from-pink-600 to-orange-600 bg-clip-text text-sm font-bold text-transparent dark:from-pink-400 dark:to-orange-400">🎯 Trading Signals:</h4>
            </div>
            <div className="flex items-center gap-4">
              <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-emerald-400 to-green-500 px-3 py-1 shadow-lg transition-all hover:scale-105">
                <div className="h-2 w-2 animate-pulse rounded-full bg-white/90"></div>
                <span className="text-xs font-bold text-white">TRACK</span>
              </div>
              <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-yellow-400 to-orange-500 px-3 py-1 shadow-lg transition-all hover:scale-105">
                <div className="h-2 w-2 animate-pulse rounded-full bg-white/90" style={{ animationDelay: '0.3s' }}></div>
                <span className="text-xs font-bold text-white">WATCH</span>
              </div>
              <div className="group flex cursor-pointer items-center gap-2 rounded-full bg-gradient-to-r from-red-400 to-rose-500 px-3 py-1 shadow-lg transition-all hover:scale-105">
                <div className="h-2 w-2 animate-pulse rounded-full bg-white/90" style={{ animationDelay: '0.6s' }}></div>
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

  return (
    <p className="text-gray-600 dark:text-gray-300">
      Showing {data.data?.length || 0} of {data.pagination?.total || 0}{' '}
      companies
      {data.processing_time_ms && (
        <span className="ml-2 text-sm text-gray-500">
          • Processed in {data.processing_time_ms}ms
        </span>
      )}
    </p>
  );
}
