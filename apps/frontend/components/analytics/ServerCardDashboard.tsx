import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Download, Filter, RefreshCw } from 'lucide-react';
import { getAnalyticsData, type EPSQueryParams, type SymbolCardData } from '@/lib/analytics-server';
import ServerPagination from './ServerPagination';
import ServerFilters from './ServerFilters';
import { Suspense } from 'react';

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
  }>;
}

function parseSearchParams(searchParams: Awaited<ServerCardDashboardProps['searchParams']>): EPSQueryParams {
  return {
    page: parseInt(searchParams.page || '1', 10),
    limit: parseInt(searchParams.limit || '10', 10),
    country: searchParams.country || undefined,
    sector: searchParams.sector || undefined,
    sort_by: searchParams.sort_by || 'growth_factor',
    min_eps: searchParams.min_eps ? parseFloat(searchParams.min_eps) : undefined,
    min_growth: searchParams.min_growth ? parseFloat(searchParams.min_growth) : undefined,
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

  const getActionInfo = (status: string) => {
    switch (status) {
      case 'TRACK':
        return { action: 'KEEP', emoji: '🟢' };
      case 'WATCH':
        return { action: 'WATCH', emoji: '🟡' };
      case 'STOP':
        return { action: 'PAUSE', emoji: '🔴' };
      default:
        return { action: 'KEEP', emoji: '🟢' };
    }
  };

  const actionInfo = getActionInfo(cardData.active_status);
  const daysUntil = cardData.next_quarter_estimate?.days_until_announcement || 185;
  const nextDate = cardData.next_quarter_estimate?.announcement_date || 'Feb 28, 2026';

  const currentEPSDate = new Date(latestQuarter?.date || 'Jul 30, 2025');
  const nextEPSDate = new Date(nextDate);
  const totalDays = Math.ceil(
    (nextEPSDate.getTime() - currentEPSDate.getTime()) / (1000 * 60 * 60 * 24)
  );
  const daysPassed = Math.max(0, totalDays - daysUntil);
  const progressPercentage =
    totalDays > 0
      ? Math.max(0, Math.min(100, (daysPassed / totalDays) * 100))
      : 0;

  return (
    <div className="mx-auto w-full max-w-sm touch-manipulation overflow-hidden rounded-3xl border-2 border-transparent bg-white shadow-2xl shadow-pink-500/20 transition-all duration-300 hover:border-pink-200 dark:bg-slate-900 dark:shadow-cyan-500/20 dark:hover:border-cyan-400/50">
      {/* Header */}
      <div className="bg-gradient-to-r from-pink-400 via-purple-500 to-indigo-600 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-xl font-bold text-white drop-shadow-sm">
              {cardData.symbol}
            </span>
            <span className="text-lg text-pink-100">#{cardData.rank}</span>
          </div>
          <a
            href={`https://www.tradingview.com/symbols/NASDAQ-${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xl text-white transition-transform hover:scale-125 hover:text-yellow-300"
          >
            🔗
          </a>
        </div>
      </div>

      {/* Main Action */}
      <div className="bg-gradient-to-br from-pink-50 to-purple-50 py-8 text-center dark:from-slate-800 dark:to-slate-700">
        <div className="inline-flex items-center gap-3 rounded-full border border-pink-200/50 bg-white px-6 py-3 shadow-lg dark:border-cyan-400/30 dark:bg-slate-800">
          <span className="text-2xl">{actionInfo.emoji}</span>
          <span className="bg-gradient-to-r from-pink-600 to-purple-600 bg-clip-text text-2xl font-bold text-transparent dark:from-cyan-400 dark:to-blue-400">
            {actionInfo.action}
          </span>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="bg-gradient-to-br from-pink-50 to-purple-50 px-6 pb-6 dark:from-slate-800 dark:to-slate-700">
        <div className="mb-3 text-center">
          <span className="text-sm font-semibold text-purple-700 dark:text-cyan-300">
            Action Phase
          </span>
        </div>
        
        <div className="mb-2 flex justify-between text-xs font-medium text-purple-600 dark:text-cyan-400">
          <span>{latestQuarter?.date || 'Jul 30, 2025'}</span>
          <span>
            {nextDate.includes(',') ? nextDate.split(',')[0] : nextDate}
          </span>
        </div>

        <div className="flex items-center gap-4">
          <div className="h-6 flex-1 rounded-full bg-pink-100 shadow-inner dark:bg-slate-600">
            <div
              className="h-6 rounded-full bg-gradient-to-r from-pink-400 via-purple-500 to-indigo-500 shadow-sm transition-all duration-700"
              style={{ width: `${progressPercentage}%` }}
            />
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600 dark:text-cyan-400">
              {daysUntil}
            </div>
            <div className="text-xs font-medium text-purple-500 dark:text-cyan-300">
              days
            </div>
          </div>
        </div>
      </div>

      {/* Main Growth */}
      <div className="bg-white py-6 text-center dark:bg-slate-900">
        <div className="inline-flex items-center gap-2 rounded-2xl bg-gradient-to-r from-green-400 to-emerald-500 px-4 py-2 shadow-lg">
          <span className="text-2xl">↗️</span>
          <span className="text-2xl font-bold text-white drop-shadow-sm">
            {formatPercentage(latestQuarter?.eps_growth || 0)}
          </span>
        </div>
      </div>

      {/* Separator */}
      <div className="bg-white px-6 dark:bg-slate-900">
        <div className="border-t-2 border-dashed border-pink-200 dark:border-cyan-400/30" />
      </div>

      {/* Content */}
      <div className="space-y-4 bg-white px-6 pt-6 pb-6 dark:bg-slate-900">
        {quarters.length >= 2 && (
          <>
            {/* Previous Quarter */}
            <div className="rounded-2xl border border-purple-200/50 bg-gradient-to-r from-purple-50 to-pink-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
              <div className="mb-2 text-sm font-semibold text-purple-600 dark:text-cyan-400">
                {previousQuarter?.date || 'Apr 30, 2025'}
              </div>
              <div className="space-y-1 text-sm text-purple-700 dark:text-cyan-200">
                <div>
                  • Growth:{' '}
                  {formatPercentage(previousQuarter?.eps_growth || 0)} | EPS:{' '}
                  {(previousQuarter?.eps || 0).toFixed(2)}
                </div>
                <div>
                  • Price:{' '}
                  {formatPercentage(previousQuarter?.price_growth || 0)} |{' '}
                  {formatCurrency(previousQuarter?.price || 0)}
                </div>
              </div>
            </div>

            {/* Latest Quarter */}
            <div className="rounded-2xl border border-green-200/50 bg-gradient-to-r from-green-50 to-emerald-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
              <div className="mb-2 text-sm font-semibold text-green-600 dark:text-cyan-400">
                {latestQuarter?.date || 'Jul 30, 2025'}
              </div>
              <div className="space-y-1 text-sm text-green-700 dark:text-cyan-200">
                <div>
                  • Growth: {formatPercentage(latestQuarter?.eps_growth || 0)}{' '}
                  | EPS: {(latestQuarter?.eps || 0).toFixed(2)}
                </div>
                <div>
                  • Price:{' '}
                  {formatPercentage(latestQuarter?.price_growth || 0)} |{' '}
                  {formatCurrency(latestQuarter?.price || 0)}
                </div>
              </div>
            </div>
          </>
        )}

        {/* Next Checkpoint */}
        <div className="rounded-2xl border border-amber-200/50 bg-gradient-to-r from-amber-50 to-orange-50 p-4 dark:border-cyan-400/20 dark:from-slate-800 dark:to-slate-700">
          <div className="text-sm font-semibold text-amber-600 dark:text-cyan-400">
            Next: {nextDate.includes(',') ? nextDate.split(',')[0] : nextDate}
          </div>
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
        <p className="mb-4 text-gray-600 dark:text-gray-300">
          No data available
        </p>
      </div>
    );
  }

  return (
    <>
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
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
              ...(params.min_eps !== undefined && { min_eps: String(params.min_eps) }),
              ...(params.min_growth !== undefined && { min_growth: String(params.min_growth) }),
            }).toString()}
          />
        </div>
      )}
    </>
  );
}

function LoadingGrid() {
  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {Array.from({ length: 8 }).map((_, i) => (
        <Card key={i} className="animate-pulse">
          <CardHeader>
            <div className="mb-2 h-6 rounded bg-gray-200" />
            <div className="h-4 w-3/4 rounded bg-gray-200" />
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {Array.from({ length: 3 }).map((_, j) => (
                <div key={j} className="flex justify-between">
                  <div className="h-3 w-1/3 rounded bg-gray-200" />
                  <div className="h-3 w-1/4 rounded bg-gray-200" />
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

export default async function ServerCardDashboard({ searchParams }: ServerCardDashboardProps) {
  const resolvedSearchParams = await searchParams;
  const params = parseSearchParams(resolvedSearchParams);
  const showFilters = resolvedSearchParams.showFilters === 'true';

  return (
    <div className="space-y-6">
      {/* Header with filters */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-2xl font-bold text-transparent">
            📋 Performance Watch
          </h2>
          <Suspense fallback={<div>Loading stats...</div>}>
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
              variant="outline"
              size="sm"
              type="submit"
              className="flex items-center gap-2"
            >
              <Filter className="h-4 w-4" />
              Filters
            </Button>
          </form>
        </div>
      </div>

      {/* Filters */}
      {showFilters && (
        <Suspense fallback={<div>Loading filters...</div>}>
          <ServerFilters currentParams={params} />
        </Suspense>
      )}

      {/* Legend */}
      <Card className="border border-orange-200/50 bg-white/90 backdrop-blur-sm dark:border-orange-400/20 dark:bg-slate-800/90">
        <CardHeader className="pb-4">
          <CardTitle className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            📖 Status Legend
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            <div className="space-y-3">
              <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                Status Indicators
              </h4>
              <div className="space-y-2">
                <div className="flex items-center gap-3">
                  <Badge className="bg-green-500 text-white">
                    <span className="mr-1">🟢</span>
                    TRACK
                  </Badge>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Strong performance, actively tracking
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <Badge className="bg-yellow-500 text-white">
                    <span className="mr-1">🟡</span>
                    WATCH
                  </Badge>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Watch closely, mixed signals
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <Badge className="bg-red-500 text-white">
                    <span className="mr-1">🔴</span>
                    STOP
                  </Badge>
                  <span className="text-sm text-gray-600 dark:text-gray-400">
                    Weak performance, avoid investment
                  </span>
                </div>
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
      Showing {data.data?.length || 0} of {data.pagination?.total || 0} companies
      {data.processing_time_ms && (
        <span className="ml-2 text-sm text-gray-500">
          • Processed in {data.processing_time_ms}ms
        </span>
      )}
    </p>
  );
}