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
  
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'TRACK': return 'border-green-500/30 bg-green-50/50 dark:bg-green-900/10';
      case 'WATCH': return 'border-yellow-500/30 bg-yellow-50/50 dark:bg-yellow-900/10';
      case 'STOP': return 'border-red-500/30 bg-red-50/50 dark:bg-red-900/10';
      default: return 'border-green-500/30 bg-green-50/50 dark:bg-green-900/10';
    }
  };

  const getStatusIndicator = (status: string) => {
    switch (status) {
      case 'TRACK': return 'bg-green-500';
      case 'WATCH': return 'bg-yellow-500';
      case 'STOP': return 'bg-red-500';
      default: return 'bg-green-500';
    }
  };

  const daysUntil = cardData.next_quarter_estimate?.days_until_announcement || 185;
  const nextDate = cardData.next_quarter_estimate?.announcement_date || 'Feb 28, 2026';

  return (
    <div className={`rounded-lg border-2 bg-white p-4 shadow-sm transition-all hover:shadow-md dark:bg-slate-900 ${getStatusColor(cardData.active_status)}`}>
      {/* Header */}
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className={`h-2 w-2 rounded-full ${getStatusIndicator(cardData.active_status)}`}></div>
          <h3 className="text-lg font-bold text-slate-800 dark:text-slate-200">{cardData.symbol}</h3>
          <span className="text-sm text-slate-500 dark:text-slate-400">#{cardData.rank}</span>
        </div>
        <a
          href={`https://www.tradingview.com/symbols/NASDAQ-${cardData.symbol}/financials-earnings/?earnings-period=FQ&revenues-period=FQ`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
        >
          View →
        </a>
      </div>

      {/* Main Growth */}
      <div className="mb-3 text-center">
        <div className={`inline-flex items-center gap-1 rounded px-2 py-1 ${
          (latestQuarter?.eps_growth || 0) >= 0 
            ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300' 
            : 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-300'
        }`}>
          <span>{(latestQuarter?.eps_growth || 0) >= 0 ? '↗' : '↘'}</span>
          <span className="font-bold">{formatPercentage(latestQuarter?.eps_growth || 0)}</span>
        </div>
      </div>

      {/* Current Quarter */}
      <div className="mb-2 space-y-1 text-xs">
        <div className="font-medium text-slate-700 dark:text-slate-300">
          Current: {latestQuarter?.date || 'Jul 30, 2025'}
        </div>
        <div className="flex justify-between text-slate-600 dark:text-slate-400">
          <span>EPS Growth: {formatPercentage(latestQuarter?.eps_growth || 0)}</span>
          <span>EPS: {(latestQuarter?.eps || 0).toFixed(2)}</span>
        </div>
        <div className="flex justify-between text-slate-600 dark:text-slate-400">
          <span>Price: {formatPercentage(latestQuarter?.price_growth || 0)}</span>
          <span>{formatCurrency(latestQuarter?.price || 0)}</span>
        </div>
      </div>

      {/* Previous Quarter */}
      {previousQuarter && (
        <div className="mb-3 space-y-1 border-t border-slate-200 pt-2 text-xs dark:border-slate-700">
          <div className="font-medium text-slate-600 dark:text-slate-400">
            Previous: {previousQuarter.date}
          </div>
          <div className="flex justify-between text-slate-500 dark:text-slate-500">
            <span>EPS Growth: {formatPercentage(previousQuarter.eps_growth || 0)}</span>
            <span>EPS: {(previousQuarter.eps || 0).toFixed(2)}</span>
          </div>
          <div className="flex justify-between text-slate-500 dark:text-slate-500">
            <span>Price: {formatPercentage(previousQuarter.price_growth || 0)}</span>
            <span>{formatCurrency(previousQuarter.price || 0)}</span>
          </div>
        </div>
      )}

      {/* Footer Info */}
      <div className="flex justify-between text-xs text-slate-500 dark:text-slate-400">
        <span>P/E: {
          latestQuarter?.eps && latestQuarter.eps > 0 
            ? ((latestQuarter.price || 0) / latestQuarter.eps).toFixed(1)
            : 'N/A'
        }</span>
        <span>Next: {daysUntil}d</span>
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
            <h2 className="bg-gradient-to-r from-slate-700 to-slate-900 bg-clip-text text-2xl font-bold text-transparent dark:from-slate-200 dark:to-slate-400">
              Performance Analytics
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

        {/* Search Bar */}
        <div className="flex gap-4">
          <form action="/analytics" method="get" className="flex-1">
            {/* Preserve current non-search params */}
            {Object.entries(resolvedSearchParams).map(([key, value]) =>
              key !== 'search' && key !== 'page' ? (
                <input key={key} type="hidden" name={key} value={value} />
              ) : null
            )}
            <div className="relative max-w-md">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
              <Input
                type="text"
                name="search"
                placeholder="Search symbols (e.g., AAPL, MSFT)"
                defaultValue={resolvedSearchParams.search || ''}
                className="pl-9 pr-4"
              />
            </div>
          </form>
          
          {/* Quick Filter Chips */}
          <div className="flex items-center gap-2 flex-wrap">
            <form action="/analytics" method="get">
              {Object.entries(resolvedSearchParams).map(([key, value]) =>
                key !== 'sort_by' && key !== 'page' ? (
                  <input key={key} type="hidden" name={key} value={value} />
                ) : null
              )}
              <input type="hidden" name="sort_by" value="growth_factor" />
              <Button variant="outline" size="sm" type="submit" className="text-xs">
                Top Gainers
              </Button>
            </form>
            
            <form action="/analytics" method="get">
              {Object.entries(resolvedSearchParams).map(([key, value]) =>
                key !== 'min_growth' && key !== 'page' ? (
                  <input key={key} type="hidden" name={key} value={value} />
                ) : null
              )}
              <input type="hidden" name="min_growth" value="10" />
              <Button variant="outline" size="sm" type="submit" className="text-xs">
                High Growth
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

      {/* Compact Status Legend */}
      <Card className="border border-slate-200 bg-white/80 backdrop-blur-sm dark:border-slate-700 dark:bg-slate-800/80">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <h4 className="text-sm font-semibold text-slate-700 dark:text-slate-300">Status Guide:</h4>
            </div>
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-green-500"></div>
                <span className="text-xs font-medium text-slate-600 dark:text-slate-400">TRACK</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-yellow-500"></div>
                <span className="text-xs font-medium text-slate-600 dark:text-slate-400">WATCH</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-red-500"></div>
                <span className="text-xs font-medium text-slate-600 dark:text-slate-400">STOP</span>
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
