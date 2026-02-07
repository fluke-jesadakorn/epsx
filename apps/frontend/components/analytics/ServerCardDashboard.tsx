import {
  getAnalyticsData,
  getPortfolioData,
  type EPSQueryParams,
} from '@/lib/unified-server-data';
import { Suspense } from 'react';
import { AnalyticsDashboardWrapper } from './AnalyticsDashboardWrapper';
import ServerFilters from './ServerFilters';

interface ServerCardDashboardProps {
  searchParams: {
    page?: string;
    limit?: string;
    country?: string;
    sector?: string;
    sort_by?: string;
    min_eps?: string;
    min_growth?: string;
    showFilters?: string;
    search?: string;
  };
  isPortfolio?: boolean;
}

function parseSearchParams(
  searchParams: ServerCardDashboardProps['searchParams']
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


async function CardGrid({ params, isPortfolio }: { params: EPSQueryParams; isPortfolio?: boolean }) {
  // On the first page, fetch extra items to account for Top 5 special section
  const adjustedParams = {
    ...params,
    limit: params.page === 1 ? params.limit + 5 : params.limit
  };

  const data = isPortfolio
    ? await getPortfolioData(adjustedParams)
    : await getAnalyticsData(adjustedParams);

  if (!data?.rankings || data.rankings.length === 0) {
    return (
      <div className="py-12 text-center">
        <p className="mb-4 text-gray-600 dark:text-white">No data available</p>
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
  isPortfolio,
}: ServerCardDashboardProps) {
  const resolvedSearchParams = await searchParams;
  const params = parseSearchParams(resolvedSearchParams);
  // Default to not showing filters permanently - always start with filters hidden
  const showFilters = resolvedSearchParams.showFilters === 'true';

  return (
    <div className="space-y-6">

      {/* Filters - Show conditionally based on showFilters parameter */}
      {showFilters && (
        <Suspense
          fallback={
            <div className="text-slate-600 dark:text-slate-200">
              Loading filters...
            </div>
          }
        >
          <ServerFilters currentParams={params} />
        </Suspense>
      )}

      {/* Cards grid */}
      <Suspense fallback={<LoadingGrid />}>
        <CardGrid params={params} isPortfolio={isPortfolio} />
      </Suspense>
    </div>
  );
}
