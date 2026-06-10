'use client';

import { StockDataCard } from '@/shared/components/cards/stock-data-card';
import type { UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';

interface GrowthLeadersProps {
  growthLeaders: any[];
  priceLeaders: any[];
}

export function GrowthLeadersSection({ growthLeaders, priceLeaders }: GrowthLeadersProps) {
  return (
    <div className="mb-8">
      <div className="relative overflow-hidden rounded-3xl border border-orange-200/50 bg-white/80 p-6 sm:p-8 shadow-2xl backdrop-blur-xl dark:border-orange-400/20 dark:bg-slate-800/80">
        <div className="absolute inset-0 bg-gradient-to-br from-orange-50/50 via-transparent to-yellow-50/50 dark:from-orange-900/10 dark:via-transparent dark:to-yellow-900/10" />
        <div className="absolute top-0 right-0 h-32 w-32 rounded-full bg-gradient-to-br from-orange-400/10 to-yellow-400/10 blur-2xl" />
        <div className="absolute bottom-0 left-0 h-40 w-40 rounded-full bg-gradient-to-br from-blue-400/10 to-cyan-400/10 blur-2xl" />

        <div className="relative z-10">
          <div className="mb-6 text-center sm:text-left">
            <h2 className="mb-3 text-xl font-bold sm:text-2xl">
              <span className="mr-2">🏆</span>
              <span className="animate-gradient-x bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-600 bg-clip-text text-transparent">
                Growth Performance Leaders
              </span>
            </h2>
            <p className="text-gray-600 dark:text-gray-300">
              Top performers in growth factor and price growth
            </p>
          </div>

          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
            <div className="rounded-2xl border border-green-200/50 bg-gradient-to-br from-green-50/80 to-emerald-50/80 p-5 backdrop-blur-sm transition-all duration-300 hover:scale-[1.02] dark:border-green-400/20 dark:from-green-900/20 dark:to-emerald-900/20">
              <h3 className="mb-4 flex items-center gap-3 text-sm font-bold text-green-700 dark:text-green-400">
                <div className="rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                  </svg>
                </div>
                Best Growth Factor
              </h3>
              <div className="space-y-3">
                {growthLeaders.slice(0, 3).map((leader, index) => (
                  <div key={leader.symbol} className="flex items-center justify-between rounded-xl bg-white/60 p-3 backdrop-blur-sm dark:bg-slate-800/60">
                    <div className="flex items-center gap-3">
                      <span className="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-green-500 to-emerald-500 text-xs font-bold text-white shadow-lg">
                        {index + 1}
                      </span>
                      <div>
                        <span className="font-semibold text-gray-900 dark:text-gray-100">{leader.symbol}</span>
                        <p className="text-xs text-gray-500 dark:text-gray-400">{leader.companyName}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <span className="rounded-lg bg-gradient-to-r from-green-500 to-emerald-500 px-3 py-1 text-sm font-bold text-white shadow-md">
                        +{(leader.epsGrowth || 0).toFixed(1)}%
                      </span>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        Score: {(leader.score || 0).toFixed(1)}
                      </p>
                    </div>
                  </div>
                ))}
                {growthLeaders.length === 0 && (
                  <div className="rounded-xl bg-white/60 p-4 text-center backdrop-blur-sm dark:bg-slate-800/60">
                    <p className="text-sm text-gray-500 dark:text-gray-400">No growth factor data available</p>
                  </div>
                )}
              </div>
            </div>

            <div className="rounded-2xl border border-blue-200/50 bg-gradient-to-br from-blue-50/80 to-cyan-50/80 p-5 backdrop-blur-sm transition-all duration-300 hover:scale-[1.02] dark:border-blue-400/20 dark:from-blue-900/20 dark:to-cyan-900/20">
              <h3 className="mb-4 flex items-center gap-3 text-sm font-bold text-blue-700 dark:text-blue-400">
                <div className="rounded-xl bg-gradient-to-r from-blue-500 to-cyan-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1" />
                  </svg>
                </div>
                Best Price Growth
              </h3>
              <div className="space-y-3">
                {priceLeaders.slice(0, 3).map((leader, index) => {
                  const priceGrowth = leader.momentum_1m ?? 0;

                  return (
                    <div key={leader.symbol} className="flex items-center justify-between rounded-xl bg-white/60 p-3 backdrop-blur-sm dark:bg-slate-800/60">
                      <div className="flex items-center gap-3">
                        <span className="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-r from-blue-500 to-cyan-500 text-xs font-bold text-white shadow-lg">
                          {index + 1}
                        </span>
                        <div>
                          <span className="font-semibold text-gray-900 dark:text-gray-100">{leader.symbol}</span>
                          <p className="text-xs text-gray-500 dark:text-gray-400">{leader.companyName}</p>
                        </div>
                      </div>
                      <div className="text-right">
                        <span className={`rounded-lg px-3 py-1 text-sm font-bold text-white shadow-md ${priceGrowth >= 0
                            ? 'bg-gradient-to-r from-blue-500 to-cyan-500'
                            : 'bg-gradient-to-r from-red-500 to-pink-500'
                          }`}>
                          {priceGrowth >= 0 ? '+' : ''}{priceGrowth.toFixed(1)}%
                        </span>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                          Vol: {(leader.volatility ?? 0).toFixed(1)}
                        </p>
                      </div>
                    </div>
                  );
                })}
                {priceLeaders.length === 0 && (
                  <div className="rounded-xl bg-white/60 p-4 text-center backdrop-blur-sm dark:bg-slate-800/60">
                    <p className="text-sm text-gray-500 dark:text-gray-400">No price growth data available</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface MetadataSectionProps {
  metadata: any;
}

export function MetadataSection({ metadata }: MetadataSectionProps) {
  return (
    <div className="mb-8">
      <div className="relative overflow-hidden rounded-3xl border border-purple-200/50 bg-white/80 p-6 shadow-2xl backdrop-blur-xl dark:border-purple-400/20 dark:bg-slate-800/80">
        <div className="absolute inset-0 bg-gradient-to-br from-purple-50/50 via-transparent to-blue-50/50 dark:from-purple-900/10 dark:via-transparent dark:to-blue-900/10" />

        <div className="relative z-10">
          <div className="mb-6 text-center sm:text-left">
            <h2 className="mb-3 text-xl font-bold sm:text-2xl">
              <span className="mr-2">🚀</span>
              <span className="animate-gradient-x bg-gradient-to-r from-purple-500 via-blue-500 to-purple-600 bg-clip-text text-transparent">
                Advanced Analytics Engine
              </span>
            </h2>
            <p className="text-gray-600 dark:text-gray-300">
              Powered by Diesel ORM with server-side rendering and intelligent caching
            </p>
          </div>

          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div className="rounded-2xl border border-green-200/50 bg-gradient-to-br from-green-50/80 to-emerald-50/80 p-4 backdrop-blur-sm dark:border-green-400/20 dark:from-green-900/20 dark:to-emerald-900/20">
              <div className="flex items-center gap-3 mb-2">
                <div className="rounded-xl bg-gradient-to-r from-green-500 to-emerald-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <span className="text-sm font-semibold text-green-700 dark:text-green-400">Processing Time</span>
              </div>
              <p className="text-lg font-bold text-green-800 dark:text-green-300">{metadata.query_time}ms</p>
            </div>

            <div className="rounded-2xl border border-blue-200/50 bg-gradient-to-br from-blue-50/80 to-cyan-50/80 p-4 backdrop-blur-sm dark:border-blue-400/20 dark:from-blue-900/20 dark:to-cyan-900/20">
              <div className="flex items-center gap-3 mb-2">
                <div className="rounded-xl bg-gradient-to-r from-blue-500 to-cyan-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
                  </svg>
                </div>
                <span className="text-sm font-semibold text-blue-700 dark:text-blue-400">Architecture</span>
              </div>
              <p className="text-sm font-bold text-blue-800 dark:text-blue-300">
                Server Components
              </p>
            </div>

            <div className="rounded-2xl border border-orange-200/50 bg-gradient-to-br from-orange-50/80 to-yellow-50/80 p-4 backdrop-blur-sm dark:border-orange-400/20 dark:from-orange-900/20 dark:to-yellow-900/20">
              <div className="flex items-center gap-3 mb-2">
                <div className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                  </svg>
                </div>
                <span className="text-sm font-semibold text-orange-700 dark:text-orange-400">Data Source</span>
              </div>
              <p className="text-sm font-bold text-orange-800 dark:text-orange-300">Analytics API</p>
            </div>

            <div className="rounded-2xl border border-purple-200/50 bg-gradient-to-br from-purple-50/80 to-pink-50/80 p-4 backdrop-blur-sm dark:border-purple-400/20 dark:from-purple-900/20 dark:to-pink-900/20">
              <div className="flex items-center gap-3 mb-2">
                <div className="rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 p-2">
                  <svg className="h-4 w-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                </div>
                <span className="text-sm font-semibold text-purple-700 dark:text-purple-400">Markets</span>
              </div>
              <p className="text-sm font-bold text-purple-800 dark:text-purple-300">
                Global Markets
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface LoadingStateProps {
  // No props needed
}

export function LoadingState(_props: LoadingStateProps) {
  return (
    <>
      <div className="mb-6 block sm:hidden">
        <div className="overflow-x-auto pb-4">
          <div className="flex gap-3">
            {Array.from({ length: 4 }).map((_, index) => (
              <div
                key={`skeleton-mobile-loading-${String(index)}`}
                className="w-72 flex-shrink-0 animate-pulse rounded-lg border border-gray-200 bg-white p-4"
              >
                <div className="mb-3 flex items-center gap-3">
                  <div className="h-8 w-8 rounded-full bg-gray-200" />
                  <div className="flex-1">
                    <div className="mb-1 h-4 rounded bg-gray-200" />
                    <div className="h-3 w-3/4 rounded bg-gray-200" />
                  </div>
                  <div className="h-6 w-16 rounded-full bg-gray-200" />
                </div>
                <div className="mb-3 grid grid-cols-2 gap-3">
                  <div className="rounded-lg bg-gray-100 p-3">
                    <div className="mb-1 h-3 rounded bg-gray-200" />
                    <div className="h-4 rounded bg-gray-200" />
                  </div>
                  <div className="rounded-lg bg-gray-100 p-3">
                    <div className="mb-1 h-3 rounded bg-gray-200" />
                    <div className="h-4 rounded bg-gray-200" />
                  </div>
                </div>
                <div className="mt-4 h-10 w-full rounded-lg bg-gray-200" />
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="mb-6 hidden sm:grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
        {Array.from({ length: 6 }).map((_, index) => (
          <div
            key={`skeleton-desktop-loading-${String(index)}`}
            className="animate-pulse rounded-lg border border-gray-200 bg-white p-4"
          >
            <div className="mb-3 flex items-center gap-3">
              <div className="h-8 w-8 rounded-full bg-gray-200" />
              <div className="flex-1">
                <div className="mb-1 h-4 rounded bg-gray-200" />
                <div className="h-3 w-3/4 rounded bg-gray-200" />
              </div>
              <div className="h-6 w-16 rounded-full bg-gray-200" />
            </div>
            <div className="mb-3 grid grid-cols-2 gap-3">
              <div className="rounded-lg bg-gray-100 p-3">
                <div className="mb-1 h-3 rounded bg-gray-200" />
                <div className="h-4 rounded bg-gray-200" />
              </div>
              <div className="rounded-lg bg-gray-100 p-3">
                <div className="mb-1 h-3 rounded bg-gray-200" />
                <div className="h-4 rounded bg-gray-200" />
              </div>
            </div>
            <div className="space-y-2">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={`item-${String(i)}`} className="flex justify-between">
                  <div className="h-3 w-1/3 rounded bg-gray-200" />
                  <div className="h-3 w-1/4 rounded bg-gray-200" />
                </div>
              ))}
            </div>
            <div className="mt-4 h-10 w-full rounded-lg bg-gray-200" />
          </div>
        ))}
      </div>
    </>
  );
}

interface RankingsListProps {
  data: UnifiedAnalyticsRankingsResponse;
}

export function RankingsList({ data }: RankingsListProps) {
  return (
    <>
      <div className="mb-6 block sm:hidden">
        <div className="overflow-x-auto pb-4">
          <div className="flex gap-3">
            {data.rankings.map((ranking, index) => (
              <div key={ranking.symbol} className="w-72 flex-shrink-0">
                <StockDataCard
                  symbol={ranking.symbol}
                  rank={ranking.rank || index + 1}
                  epsGrowth={ranking.epsGrowth || 0}
                  price={0}
                  currency="USD"
                />
              </div>
            ))}
          </div>
          <div className="mt-4 flex justify-center">
            <p className="text-xs text-gray-500">
              👈 Swipe to see more stocks →
            </p>
          </div>
        </div>
      </div>

      <div className="mb-6 hidden sm:grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
        {data.rankings.map(ranking => (
          <StockDataCard
            key={ranking.symbol}
            symbol={ranking.symbol}
            rank={ranking.rank || 0}
            epsGrowth={ranking.epsGrowth || 0}
            price={0}
            currency="USD"
          />
        ))}
      </div>
    </>
  );
}

interface EmptyStateProps {
  onReset: () => void;
}

export function EmptyState({ onReset }: EmptyStateProps) {
  return (
    <div className="rounded-2xl border border-gray-200/50 bg-white/80 p-8 text-center backdrop-blur-xl dark:border-gray-600/20 dark:bg-slate-800/80">
      <div className="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-gradient-to-br from-gray-100 to-gray-200 dark:from-gray-700 dark:to-gray-600">
        <svg
          className="h-10 w-10 text-gray-400 dark:text-gray-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
      </div>
      <h3 className="mb-3 bg-gradient-to-r from-gray-600 to-gray-800 bg-clip-text text-xl font-bold text-transparent dark:from-gray-300 dark:to-gray-100">
        No Results Found
      </h3>
      <p className="mb-6 max-w-md mx-auto text-gray-600 dark:text-gray-300">
        We couldn't find any companies matching your current filter criteria. Try adjusting your filters to discover more analytics data.
      </p>
      <button
        onClick={onReset}
        className="rounded-xl bg-gradient-to-r from-orange-500 to-yellow-500 px-6 py-3 font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
      >
        Clear All Filters
      </button>
    </div>
  );
}

interface ErrorStateProps {
  error: string;
  onRetry: () => void;
}

export function ErrorState({ error, onRetry }: ErrorStateProps) {
  return (
    <div className="mb-6 rounded-2xl border border-red-200/50 bg-gradient-to-br from-red-50/80 to-pink-50/80 p-6 backdrop-blur-sm dark:border-red-400/20 dark:from-red-900/20 dark:to-pink-900/20">
      <div className="flex items-center gap-4">
        <div className="rounded-xl bg-gradient-to-r from-red-500 to-pink-500 p-3">
          <svg
            className="h-6 w-6 text-white"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <div className="flex-1">
          <p className="font-bold text-red-700 dark:text-red-400">Unable to Load Data</p>
          <p className="mt-1 text-red-600 dark:text-red-300">{error}</p>
        </div>
        <button
          onClick={onRetry}
          className="rounded-xl bg-gradient-to-r from-red-500 to-pink-500 px-4 py-2 text-sm font-semibold text-white shadow-lg transition-all duration-300 hover:shadow-xl hover:scale-105"
        >
          Try Again
        </button>
      </div>
    </div>
  );
}
