 
import {
  getAnalyticsData,
  type EPSQueryParams,
} from '@/lib/unified-server-data';
import { Suspense } from 'react';
import { AnalyticsDashboardWrapper } from './analytics-dashboard-wrapper';
import ServerFilters from './server-filters';

interface ServerCardDashboardProps {
  searchParams: {
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    search?: string;
  };
}

function parseSearchParams(
  searchParams: ServerCardDashboardProps['searchParams']
): EPSQueryParams {
  return {
    page: parseInt(searchParams.page ?? '1', 10),
    limit: parseInt(searchParams.limit ?? '10', 10),
    country: searchParams.country ?? undefined,
    sector: searchParams.sector ?? undefined,
    sort_by: searchParams.sort_by ?? 'growth_factor',
    min_eps: searchParams.min_eps
      ? parseFloat(searchParams.min_eps)
      : undefined,
    min_growth: searchParams.min_growth
      ? parseFloat(searchParams.min_growth)
      : undefined,
    search: searchParams.search ?? undefined,
  };
}

async function CardGrid({ params }: { params: EPSQueryParams }) {
  // On the first page, fetch extra items to account for Top 5 special section
  const adjustedParams = {
    ...params,
    limit: params.page === 1 ? params.limit + 5 : params.limit
  };

  const data = await getAnalyticsData(adjustedParams);

  if (!data?.rankings || data.rankings.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="text-slate-400">No data available</p>
      </div>
    );
  }

  // Build current params string for pagination
  const currentParams = new URLSearchParams({
    page: String(params.page),
    limit: String(params.limit),
    ...(params.country && { country: params.country }),
    ...(params.sector && { sector: params.sector }),
    ...(params.sort_by && { sort_by: params.sort_by }),
    ...(params.min_eps !== undefined && { min_eps: String(params.min_eps) }),
    ...(params.min_growth !== undefined && { min_growth: String(params.min_growth) }),
    ...(params.search && { search: params.search }),
  }).toString();

  // Adjust pagination for display
  const adjustedPagination = data.pagination ? {
    ...data.pagination,
    limit: params.limit,
    totalPages: Math.ceil(data.pagination.total / params.limit)
  } : null;

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
          className="animate-pulse rounded-2xl border border-gray-200 dark:border-white/5 bg-white dark:bg-slate-900/60 p-6"
        >
          <div className="mb-4 h-4 w-20 rounded bg-gray-100 dark:bg-slate-800 mx-auto" />
          <div className="mb-2 h-10 w-24 rounded bg-gray-100 dark:bg-slate-800 mx-auto" />
          <div className="mb-6 h-3 w-16 rounded bg-gray-100 dark:bg-slate-800 mx-auto" />
          <div className="space-y-3">
            <div className="h-8 rounded-lg bg-gray-100 dark:bg-slate-800" />
            <div className="h-12 rounded-lg bg-gray-100 dark:bg-slate-800" />
          </div>
          <div className="mt-4 h-10 rounded-xl bg-gray-100 dark:bg-slate-800" />
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
          <div className="text-slate-600 dark:text-slate-200">
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
