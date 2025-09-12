import {
  getAnalyticsData,
  type EPSQueryParams,
  type SymbolCardData,
} from '@/lib/analytics-server';
import { Suspense } from 'react';
import ServerFilters from './ServerFilters';
import ServerPagination from './ServerPagination';

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

  return (
    <div
      className={`relative w-full max-w-[320px] min-w-[240px] flex-shrink-0 overflow-hidden rounded-3xl border-2 bg-gradient-to-br from-white via-slate-50 to-gray-100 dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 shadow-2xl sm:min-w-[280px] ${
        cardData.active_status === 'TRACK'
          ? 'border-green-300 dark:border-green-600 shadow-green-500/20 hover:shadow-green-500/30'
          : cardData.active_status === 'STOP'
            ? 'border-red-300 dark:border-red-600 shadow-red-500/20 hover:shadow-red-500/30'
            : 'border-orange-300 dark:border-orange-600 shadow-orange-500/20 hover:shadow-orange-500/30'
      }`}
    >
      {/* Decorative corner accent */}
      <div
        className={`absolute top-0 right-0 h-16 w-16 bg-gradient-to-bl ${
          cardData.active_status === 'TRACK'
            ? 'from-green-400/20 to-transparent'
            : cardData.active_status === 'STOP'
              ? 'from-red-400/20 to-transparent'
              : 'from-orange-400/20 to-transparent'
        } rounded-bl-3xl`}
      ></div>

      <div className="p-6">
        {/* Premium header design */}
        <div className="mb-4">
          <div className="mb-2 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="text-center">
                <div className="text-xs font-medium tracking-wide text-slate-500 dark:text-slate-400 uppercase">
                  Rank #{cardData.rank}
                </div>
                <h3 className="text-lg font-bold text-slate-800 dark:text-slate-200 sm:text-xl">
                  {cardData.symbol}
                </h3>
              </div>
            </div>
            <div>
              <a
                href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
                target="_blank"
                rel="noopener noreferrer"
                className={`rounded-full px-4 py-2 text-xs font-semibold text-white shadow-lg ${
                  cardData.active_status === 'TRACK'
                    ? 'bg-gradient-to-r from-green-500 to-green-600'
                    : cardData.active_status === 'STOP'
                      ? 'bg-gradient-to-r from-red-500 to-red-600'
                      : 'bg-gradient-to-r from-orange-500 to-orange-600'
                }`}
              >
                📊 View
              </a>
            </div>
          </div>
        </div>

        {/* Premium status display */}
        <div className="mb-4">
          <div className="mb-3 flex items-center justify-between">
            <div
              className={`inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-semibold ${
                cardData.active_status === 'TRACK'
                  ? 'border border-green-200 bg-green-100 text-green-800'
                  : cardData.active_status === 'STOP'
                    ? 'border border-red-200 bg-red-100 text-red-800'
                    : 'border border-orange-200 bg-orange-100 text-orange-800'
              }`}
            >
              <div
                className={`h-2 w-2 rounded-full ${
                  cardData.active_status === 'TRACK'
                    ? 'bg-green-500'
                    : cardData.active_status === 'STOP'
                      ? 'bg-red-500'
                      : 'bg-orange-500'
                }`}
              ></div>
              {cardData.active_status === 'TRACK'
                ? 'ACTIVE'
                : cardData.active_status === 'STOP'
                  ? 'INACTIVE'
                  : cardData.active_status}
            </div>

            <div className="text-right">
              <div className="text-xs text-slate-500 dark:text-slate-400">Next Action</div>
              <div
                className={`text-sm font-bold ${
                  cardData.active_status === 'TRACK'
                    ? 'text-green-600 dark:text-green-400'
                    : cardData.active_status === 'STOP'
                      ? 'text-red-600 dark:text-red-400'
                      : 'text-orange-600 dark:text-orange-400'
                }`}
              >
                {cardData.next_quarter_estimate?.days_until_announcement || 0}{' '}
                days
              </div>
            </div>
          </div>

          <div className="h-1 rounded-full bg-slate-200 dark:bg-slate-600">
            <div
              className={`h-full rounded-full ${
                cardData.active_status === 'TRACK'
                  ? 'bg-gradient-to-r from-green-400 to-emerald-500'
                  : cardData.active_status === 'STOP'
                    ? 'bg-gradient-to-r from-red-400 to-pink-500'
                    : 'bg-gradient-to-r from-orange-400 to-yellow-500'
              }`}
              style={{
                width: `${Math.max(0, Math.min(100, ((90 - (cardData.next_quarter_estimate?.days_until_announcement || 0)) / 90) * 100))}%`,
              }}
            />
          </div>
        </div>

        {/* Premium data display */}
        <div className="flex flex-col gap-3">
          <div className="rounded-2xl border border-slate-200/50 dark:border-slate-600/50 bg-white/50 dark:bg-gray-800/50 p-4 text-center backdrop-blur-sm">
            <div className="mb-2 flex items-center justify-center gap-2">
              <span
                className={`text-base sm:text-lg`}
              >{(latestQuarter?.eps_growth || 0) >= 0 ? '📈' : '📉'}</span>
              <span className="text-sm font-medium text-slate-600 dark:text-slate-400">Growth</span>
            </div>
            <div
              className={`text-center text-base leading-tight font-bold sm:text-xl ${
                (latestQuarter?.eps_growth || 0) >= 0
                  ? 'text-green-600 dark:text-green-400'
                  : 'text-red-600 dark:text-red-400'
              }`}
            >
              {formatPercentage(latestQuarter?.eps_growth || 0)}
            </div>
          </div>

          <div className="rounded-2xl border border-slate-200/50 dark:border-slate-600/50 bg-white/50 dark:bg-gray-800/50 p-4 text-center backdrop-blur-sm">
            <div className="mb-2 flex items-center justify-center gap-2">
              <span className="text-base sm:text-lg">💰</span>
              <span className="text-sm font-medium text-slate-600 dark:text-slate-400">Price</span>
            </div>
            <div className="text-center text-base leading-tight font-bold text-slate-800 dark:text-slate-200 sm:text-xl">
              {formatCurrency(latestQuarter?.price || 0)}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

const Top5SpecialBox = ({ top5Data }: { top5Data: SymbolCardData[] }) => {
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

  const getLeaderInsights = (data: SymbolCardData[]) => {
    const avgGrowth =
      data.reduce(
        (sum, card) => sum + (card.quarterly_performance?.[0]?.eps_growth || 0),
        0
      ) / data.length;
    const activeCount = data.filter(
      card => card.active_status === 'TRACK'
    ).length;
    const totalValue = data.reduce(
      (sum, card) => sum + (card.quarterly_performance?.[0]?.price || 0),
      0
    );

    return { avgGrowth, activeCount, totalValue };
  };

  const insights = getLeaderInsights(top5Data);

  return (
    <div className="mb-8">
      {/* Cohesive cards grid using same styling as regular cards */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-4 sm:px-0 overflow-visible">
        {top5Data.map(cardData => {
          const quarters = cardData.quarterly_performance?.slice(0, 2) || [];
          const latestQuarter = quarters[0];

          const getRankBadge = (rank: number) => {
            if (rank === 1)
              return { badge: '👑', color: 'from-yellow-400 to-orange-500' };
            if (rank === 2)
              return { badge: '🥈', color: 'from-slate-400 to-slate-500' };
            if (rank === 3)
              return { badge: '🥉', color: 'from-orange-400 to-amber-500' };
            if (rank === 4)
              return { badge: '⭐', color: 'from-purple-400 to-pink-500' };
            if (rank === 5)
              return { badge: '💎', color: 'from-blue-400 to-cyan-500' };
            return { badge: '🔥', color: 'from-green-400 to-emerald-500' };
          };

          const rankInfo = getRankBadge(cardData.rank);

          const getUltraPremiumStyle = (rank: number) => {
            if (rank === 1)
              return {
                container:
                  'bg-gradient-to-br from-yellow-50 via-amber-50 to-orange-50 border-4 border-yellow-400/50 shadow-yellow-500/40 hover:shadow-yellow-500/60 hover:border-yellow-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-yellow-300/30 to-transparent',
                glow: 'shadow-2xl shadow-yellow-500/50 hover:shadow-yellow-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-yellow-400/20 before:via-transparent before:to-yellow-400/20 before:blur-xl',
              };
            if (rank === 2)
              return {
                container:
                  'bg-gradient-to-br from-slate-50 via-gray-50 to-zinc-50 border-4 border-slate-400/50 shadow-slate-500/40 hover:shadow-slate-500/60 hover:border-slate-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-slate-300/30 to-transparent',
                glow: 'shadow-2xl shadow-slate-500/50 hover:shadow-slate-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-slate-400/20 before:via-transparent before:to-slate-400/20 before:blur-xl',
              };
            if (rank === 3)
              return {
                container:
                  'bg-gradient-to-br from-orange-50 via-amber-50 to-yellow-50 border-4 border-orange-400/50 shadow-orange-500/40 hover:shadow-orange-500/60 hover:border-orange-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-orange-300/30 to-transparent',
                glow: 'shadow-2xl shadow-orange-500/50 hover:shadow-orange-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-orange-400/20 before:via-transparent before:to-orange-400/20 before:blur-xl',
              };
            if (rank === 4)
              return {
                container:
                  'bg-gradient-to-br from-purple-50 via-pink-50 to-fuchsia-50 border-4 border-purple-400/50 shadow-purple-500/40 hover:shadow-purple-500/60 hover:border-purple-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-purple-300/30 to-transparent',
                glow: 'shadow-2xl shadow-purple-500/50 hover:shadow-purple-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-purple-400/20 before:via-transparent before:to-purple-400/20 before:blur-xl',
              };
            if (rank === 5)
              return {
                container:
                  'bg-gradient-to-br from-cyan-50 via-blue-50 to-sky-50 border-4 border-cyan-400/50 shadow-cyan-500/40 hover:shadow-cyan-500/60 hover:border-cyan-400/70',
                shine:
                  'bg-gradient-to-r from-transparent via-cyan-300/30 to-transparent',
                glow: 'shadow-2xl shadow-cyan-500/50 hover:shadow-cyan-500/70',
                halo: 'before:absolute before:inset-0 before:rounded-3xl before:bg-gradient-to-r before:from-cyan-400/20 before:via-transparent before:to-cyan-400/20 before:blur-xl',
              };
            return {
              container:
                'bg-gradient-to-br from-white via-slate-50 to-gray-100 border-2 border-slate-300',
              shine: '',
              glow: '',
              halo: '',
            };
          };

          const ultraStyle = getUltraPremiumStyle(cardData.rank);

          return (
            <div
              key={cardData.symbol}
              className={`relative w-full max-w-[350px] min-w-[240px] flex-shrink-0 overflow-visible rounded-3xl sm:min-w-[300px] ${ultraStyle.container} dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 ${ultraStyle.glow} ${ultraStyle.halo}`}
            >
              {/* Animated shine effect */}
              <div
                className={`absolute inset-0 rounded-3xl ${ultraStyle.shine}`}
              ></div>

              {/* Ultra-premium floating rank badge */}
              <div
                className={`absolute -top-4 -left-4 h-16 w-16 bg-gradient-to-br ${rankInfo.color} flex rotate-12 transform items-center justify-center rounded-full border-4 border-white text-2xl text-white shadow-2xl z-30`}
              >
                {rankInfo.badge}
              </div>

              {/* Holographic corner effects */}
              <div
                className={`bg-gradient-radial absolute top-0 right-0 h-20 w-20 rounded-bl-3xl from-white/40 via-transparent to-transparent opacity-60`}
              ></div>
              <div
                className={`bg-gradient-radial absolute bottom-0 left-0 h-20 w-20 rounded-tr-3xl from-white/40 via-transparent to-transparent opacity-60`}
              ></div>

              {/* Premium content wrapper with extra padding */}

              <div className="p-8 pt-16">
                {/* Ultra-premium header with luxury typography */}
                <div className="mb-6 text-center">
                  <div className="mb-3">
                    <div className="mb-1 text-xs font-light tracking-[0.2em] text-slate-400 uppercase">
                      {cardData.rank === 1
                        ? '👑 CHAMPION'
                        : cardData.rank === 2
                          ? '🥈 ELITE'
                          : cardData.rank === 3
                            ? '🥉 LEGEND'
                            : cardData.rank === 4
                              ? '⭐ MASTER'
                              : '💎 DIAMOND'}{' '}
                      · RANK #{cardData.rank}
                    </div>
                    <h3 className="mb-2 bg-gradient-to-r from-slate-700 via-slate-900 to-slate-700 dark:from-slate-200 dark:via-white dark:to-slate-200 bg-clip-text text-2xl font-black tracking-tight text-transparent sm:text-3xl">
                      {cardData.symbol}
                    </h3>
                    <div className="mx-auto h-px w-16 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                  </div>

                  <div className="flex justify-center">
                    <a
                      href={`https://www.tradingview.com/symbols/${cardData.symbol}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className={`inline-flex items-center gap-2 rounded-2xl px-6 py-3 text-sm font-bold text-white shadow-xl ${
                        cardData.active_status === 'TRACK'
                          ? 'bg-gradient-to-r from-green-600 via-green-500 to-emerald-600'
                          : cardData.active_status === 'STOP'
                            ? 'bg-gradient-to-r from-red-600 via-red-500 to-rose-600'
                            : 'bg-gradient-to-r from-orange-600 via-orange-500 to-amber-600'
                      }`}
                    >
                      <span className="text-base sm:text-lg">
                        📊
                      </span>
                      <span className="tracking-wide text-sm sm:text-base">VIEW DETAILS</span>
                      <span className="text-xs opacity-75">→</span>
                    </a>
                  </div>
                </div>

                {/* Ultra-premium status section */}
                <div className="mb-6">
                  <div className="mb-4 flex items-center justify-between">
                    <div
                      className={`relative inline-flex items-center gap-3 rounded-2xl border-2 px-4 py-2 text-sm font-bold backdrop-blur-md ${
                        cardData.active_status === 'TRACK'
                          ? 'border-green-400/50 bg-green-500/20 text-green-900 shadow-green-400/20'
                          : cardData.active_status === 'STOP'
                            ? 'border-red-400/50 bg-red-500/20 text-red-900 shadow-red-400/20'
                            : 'border-orange-400/50 bg-orange-500/20 text-orange-900 shadow-orange-400/20'
                      } shadow-lg`}
                    >
                      <div
                        className={`h-3 w-3 rounded-full ${
                          cardData.active_status === 'TRACK'
                            ? 'bg-green-500 shadow-lg shadow-green-500/50'
                            : cardData.active_status === 'STOP'
                              ? 'bg-red-500 shadow-lg shadow-red-500/50'
                              : 'bg-orange-500 shadow-lg shadow-orange-500/50'
                        }`}
                      ></div>
                      <span className="tracking-wide">
                        {cardData.active_status === 'TRACK'
                          ? 'ACTIVE'
                          : cardData.active_status === 'STOP'
                            ? 'INACTIVE'
                            : cardData.active_status}
                      </span>
                    </div>

                    <div className="text-center">
                      <div className="mb-1 text-xs font-light tracking-wider text-slate-400 dark:text-slate-300 uppercase">
                        Next Action
                      </div>
                      <div
                        className={`text-lg font-bold ${
                          cardData.active_status === 'TRACK'
                            ? 'text-green-700 dark:text-green-400'
                            : cardData.active_status === 'STOP'
                              ? 'text-red-700 dark:text-red-400'
                              : 'text-orange-700 dark:text-orange-400'
                        }`}
                      >
                        {cardData.next_quarter_estimate
                          ?.days_until_announcement || 0}
                        d
                      </div>
                    </div>
                  </div>

                  {/* Luxury progress indicator */}
                  <div className="relative h-2 overflow-hidden rounded-full bg-gradient-to-r from-slate-200 via-slate-100 to-slate-200 dark:from-slate-600 dark:via-slate-500 dark:to-slate-600">
                    <div className="absolute inset-0 bg-gradient-to-r from-white/50 dark:from-slate-300/30 via-transparent to-white/50 dark:to-slate-300/30"></div>
                    <div
                      className={`relative h-full overflow-hidden rounded-full ${
                        cardData.active_status === 'TRACK'
                          ? 'bg-gradient-to-r from-green-500 via-emerald-400 to-green-600'
                          : cardData.active_status === 'STOP'
                            ? 'bg-gradient-to-r from-red-500 via-rose-400 to-red-600'
                            : 'bg-gradient-to-r from-orange-500 via-amber-400 to-orange-600'
                      }`}
                      style={{
                        width: `${Math.max(0, Math.min(100, ((90 - (cardData.next_quarter_estimate?.days_until_announcement || 0)) / 90) * 100))}%`,
                      }}
                    >
                      <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent"></div>
                    </div>
                  </div>
                </div>

                {/* Ultra-premium data showcase */}
                <div className="flex flex-col gap-4">
                  <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
                    <div className="mb-4 flex items-center justify-center gap-3">
                      <span
                        className={`text-xl sm:text-2xl`}
                      >{(latestQuarter?.eps_growth || 0) >= 0 ? '📈' : '📉'}</span>
                      <div className="text-center">
                        <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                          Growth
                        </div>
                      </div>
                    </div>
                    <div
                      className={`mb-2 text-center text-lg leading-tight font-black sm:text-2xl ${
                        (latestQuarter?.eps_growth || 0) >= 0
                          ? 'text-green-700 dark:text-green-400'
                          : 'text-red-700 dark:text-red-400'
                      }`}
                    >
                      {formatPercentage(latestQuarter?.eps_growth || 0)}
                    </div>
                    <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                  </div>

                  <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
                    <div className="mb-4 flex items-center justify-center gap-3">
                      <span className="text-xl sm:text-2xl">
                        💰
                      </span>
                      <div className="text-center">
                        <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                          Price
                        </div>
                      </div>
                    </div>
                    <div className="mb-2 text-center text-lg leading-tight font-black text-slate-800 dark:text-slate-200 sm:text-2xl">
                      {formatCurrency(latestQuarter?.price || 0)}
                    </div>
                    <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
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

      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-6 sm:px-0">
        {data.data
          .filter(cardData => cardData.rank > 5) // Skip ranks 1-5 to avoid duplication with Top 5 section
          .map(cardData =>
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


      {/* Cards grid */}
      <Suspense fallback={<LoadingGrid />}>
        <CardGrid params={params} />
      </Suspense>
    </div>
  );
}

