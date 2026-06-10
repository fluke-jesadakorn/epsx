
import {
  getAnalyticsData,
  type EPSQueryParams,
} from '@/lib/server/data';
import { Suspense } from 'react';
import { AnalyticsDashboardWrapper } from './analytics-dashboard-wrapper';
import ServerFilters from './server-filters';

interface ServerCardDashboardProps {
  searchParams: Promise<{
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    search?: string;
  }>;
}

function parseSearchParams(
  sp: Awaited<ServerCardDashboardProps['searchParams']>
): EPSQueryParams {
  return {
    page: parseInt(sp.page ?? '1', 10),
    limit: parseInt(sp.limit ?? '10', 10),
    country: sp.country,
    sector: sp.sector,
    sort_by: sp.sort_by ?? 'growth_factor',
    min_eps: sp.min_eps !== undefined ? parseFloat(sp.min_eps) : undefined,
    min_growth: sp.min_growth !== undefined ? parseFloat(sp.min_growth) : undefined,
    search: sp.search,
  };
}

function buildCurrentParams(params: EPSQueryParams): string {
  const entries: Record<string, string> = {
    page: String(params.page),
    limit: String(params.limit),
  };
  if (params.country !== undefined && params.country !== '') { entries.country = params.country; }
  if (params.sector !== undefined && params.sector !== '') { entries.sector = params.sector; }
  if (params.sort_by !== '') { entries.sort_by = params.sort_by; }
  if (params.min_eps !== undefined) { entries.min_eps = String(params.min_eps); }
  if (params.min_growth !== undefined) { entries.min_growth = String(params.min_growth); }
  if (params.search !== undefined && params.search !== '') { entries.search = params.search; }
  return new URLSearchParams(entries).toString();
}

async function CardGrid({ params }: { params: EPSQueryParams }) {
  const adjustedParams = {
    ...params,
    limit: params.page === 1 ? params.limit + 5 : params.limit
  };

  const data = await getAnalyticsData(adjustedParams);

  if (data.rankings.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="text-slate-400">No data available</p>
      </div>
    );
  }

  const currentParams = buildCurrentParams(params);

  const adjustedPagination = {
    ...data.pagination,
    limit: params.limit,
    totalPages: Math.ceil(data.pagination.total / params.limit)
  };

  return (
    <AnalyticsDashboardWrapper
      rankings={data.rankings}
      pagination={adjustedPagination}
      currentParams={currentParams}
    />
  );
}

function LoadingGrid() {
  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5">
      {Array.from({ length: 10 }).map((_, i) => (
        <div
          key={`loading-${String(i)}`}
          className="animate-pulse rounded-2xl border border-border/20 bg-card p-6"
        >
          <div className="mb-4 h-4 w-20 rounded bg-gray-100 dark:bg-card mx-auto" />
          <div className="mb-2 h-10 w-24 rounded bg-gray-100 dark:bg-card mx-auto" />
          <div className="mb-6 h-3 w-16 rounded bg-gray-100 dark:bg-card mx-auto" />
          <div className="space-y-3">
            <div className="h-8 rounded-lg bg-gray-100 dark:bg-card" />
            <div className="h-12 rounded-lg bg-gray-100 dark:bg-card" />
          </div>
          <div className="mt-4 h-10 rounded-xl bg-gray-100 dark:bg-card" />
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

  return (
    <div className="space-y-6">

      {/* Filters */}
      <Suspense
        fallback={
          <div className="text-slate-600 dark:text-foreground">
            Loading filters...
          </div>
        }
      >
        <ServerFilters currentParams={params} />
      </Suspense>

      {/* Cards grid */}
      <Suspense fallback={<LoadingGrid />}>
        <CardGrid params={params} />
      </Suspense>
    </div>
  );
}
