'use client';

import { Card, CardContent, CardHeader } from '@/components/ui';
import { useEffect, useState } from 'react';

import type { TableDataMetrics } from '@/types/stockFetchData';
import type { CSSProperties } from 'react';

interface Props {
  style?: CSSProperties;
  className?: string;
  initialData: TableDataMetrics[];
}

export default function ClientEpsCardSection({
  style,
  className,
  initialData,
}: Props) {
  const [data, setData] = useState<TableDataMetrics[]>(initialData);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function fetchData() {
      try {
        const response = await fetch('/api/public/rankings?type=cards&limit=3');
        if (response.ok) {
          const result = await response.json();
          setData(result);
        }
      } catch (error) {
        console.error('Failed to fetch top performing companies:', error);
      } finally {
        setIsLoading(false);
      }
    }

    fetchData();
  }, []);

  // Ensure data is always an array to prevent runtime errors
  const safeData = Array.isArray(data) ? data : [];
  const getMarketColor = (marketCode: string | undefined) => {
    switch (marketCode) {
      case 'TYO':
        return 'text-blue-500'; // Blue
      case 'BOM':
        return 'text-green-500'; // Green
      case 'OTC':
        return 'text-purple-600'; // Purple
      default:
        return 'text-gray-400'; // Grey
    }
  };

  return (
    <div
      className={`flex w-full flex-col gap-8 ${className || ''}`}
      style={style}
    >
      {/* Enhanced Section Header */}
      <div className="mb-6 space-y-4 text-center">
        <h2 className="pancake-gradient-text text-3xl font-bold sm:text-4xl">
          Top Performing Companies
        </h2>
        <p className="text-muted-foreground mx-auto max-w-2xl">
          Discover the data leaders with exceptional growth and performance
          metrics
        </p>
        <div className="pancake-gradient mx-auto h-1 w-24 rounded-full" />
      </div>

      {/* Analytics-style card grid */}
      <div className="flex flex-wrap items-stretch justify-center gap-3 px-2 sm:gap-6 sm:px-0">
        {isLoading ? (
          // Loading skeletons
          Array.from({ length: 3 }).map((_, index) => (
            <div
              key={index}
              className="relative w-full max-w-[350px] min-w-[240px] flex-shrink-0 overflow-visible rounded-3xl sm:min-w-[300px] bg-gradient-to-br from-gray-50 via-gray-100 to-gray-200 dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 animate-pulse"
            >
              <div className="p-8 pt-16">
                <div className="mb-6 text-center">
                  <div className="mb-3">
                    <div className="mb-1 h-3 w-32 bg-gray-300 dark:bg-gray-600 rounded mx-auto"></div>
                    <div className="mb-2 h-8 w-24 bg-gray-300 dark:bg-gray-600 rounded mx-auto"></div>
                    <div className="mx-auto h-px w-16 bg-gray-300 dark:bg-gray-600"></div>
                  </div>
                  <div className="flex justify-center">
                    <div className="h-12 w-32 bg-gray-300 dark:bg-gray-600 rounded-2xl"></div>
                  </div>
                </div>
              </div>
            </div>
          ))
        ) : (
          safeData
            .filter(
              item => item && typeof item === 'object' && item.symbol && item.name
            )
            .slice(0, 3)
            .map((item, index) => {
            const getRankBadge = (rank: number) => {
              if (rank === 1) return { badge: '👑', color: 'from-yellow-400 to-orange-500' };
              if (rank === 2) return { badge: '🥈', color: 'from-slate-400 to-slate-500' };
              if (rank === 3) return { badge: '🥉', color: 'from-orange-400 to-amber-500' };
              return { badge: '🔥', color: 'from-green-400 to-emerald-500' };
            };

            const rankInfo = getRankBadge(index + 1);

            const getUltraPremiumStyle = (rank: number) => {
              if (rank === 1) return {
                container: 'bg-gradient-to-br from-yellow-50 via-amber-50 to-orange-50 border-4 border-yellow-400/50 shadow-yellow-500/40',
                glow: 'shadow-2xl shadow-yellow-500/50',
              };
              if (rank === 2) return {
                container: 'bg-gradient-to-br from-slate-50 via-gray-50 to-zinc-50 border-4 border-slate-400/50 shadow-slate-500/40',
                glow: 'shadow-2xl shadow-slate-500/50',
              };
              if (rank === 3) return {
                container: 'bg-gradient-to-br from-orange-50 via-amber-50 to-yellow-50 border-4 border-orange-400/50 shadow-orange-500/40',
                glow: 'shadow-2xl shadow-orange-500/50',
              };
              return {
                container: 'bg-gradient-to-br from-white via-slate-50 to-gray-100 border-2 border-slate-300',
                glow: '',
              };
            };

            const ultraStyle = getUltraPremiumStyle(index + 1);

            return (
              <div
                key={item.symbol || index}
                className={`relative w-full max-w-[350px] min-w-[240px] flex-shrink-0 overflow-visible rounded-3xl sm:min-w-[300px] ${ultraStyle.container} dark:from-gray-800 dark:via-gray-700 dark:to-gray-900 ${ultraStyle.glow}`}
              >
                {/* Ultra-premium floating rank badge */}
                <div className={`absolute -top-4 -left-4 h-16 w-16 bg-gradient-to-br ${rankInfo.color} flex rotate-12 transform items-center justify-center rounded-full border-4 border-white text-2xl text-white shadow-2xl z-30`}>
                  {rankInfo.badge}
                </div>

                {/* Premium content wrapper */}
                <div className="p-8 pt-16">
                  {/* Ultra-premium header */}
                  <div className="mb-6 text-center">
                    <div className="mb-3">
                      <div className="mb-1 text-xs font-light tracking-[0.2em] text-slate-400 uppercase">
                        {index === 0 ? '👑 CHAMPION' : index === 1 ? '🥈 ELITE' : '🥉 LEGEND'} · RANK #{index + 1}
                      </div>
                      <h3 className="mb-2 bg-gradient-to-r from-slate-700 via-slate-900 to-slate-700 dark:from-slate-200 dark:via-white dark:to-slate-200 bg-clip-text text-2xl font-black tracking-tight text-transparent sm:text-3xl">
                        {item.symbol || 'N/A'}
                      </h3>
                      <div className="mx-auto h-px w-16 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                    </div>

                    <div className="flex justify-center">
                      <a
                        href={`https://www.tradingview.com/symbols/${item.symbol}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-2 rounded-2xl px-6 py-3 text-sm font-bold text-white shadow-xl bg-gradient-to-r from-green-600 via-green-500 to-emerald-600"
                      >
                        <span className="text-base sm:text-lg">📊</span>
                        <span className="tracking-wide text-sm sm:text-base">VIEW DETAILS</span>
                        <span className="text-xs opacity-75">→</span>
                      </a>
                    </div>
                  </div>

                  {/* Status section */}
                  <div className="mb-6">
                    <div className="mb-4 flex items-center justify-between">
                      <div className="relative inline-flex items-center gap-3 rounded-2xl border-2 px-4 py-2 text-sm font-bold backdrop-blur-md border-green-400/50 bg-green-500/20 text-green-900 shadow-green-400/20 shadow-lg">
                        <div className="h-3 w-3 rounded-full bg-green-500 shadow-lg shadow-green-500/50"></div>
                        <span className="tracking-wide">ACTIVE</span>
                      </div>
                      <div className="text-center">
                        <div className="mb-1 text-xs font-light tracking-wider text-slate-400 dark:text-slate-300 uppercase">Next Action</div>
                        <div className="text-lg font-bold text-green-700 dark:text-green-400">66d</div>
                      </div>
                    </div>
                  </div>

                  {/* Data showcase */}
                  <div className="flex flex-col gap-4">
                    <div className="rounded-3xl border-2 border-white/50 dark:border-gray-600/50 bg-gradient-to-br from-white/80 via-white/60 to-white/40 dark:from-gray-800/80 dark:via-gray-700/60 dark:to-gray-900/40 p-6 text-center shadow-xl backdrop-blur-md">
                      <div className="mb-4 flex items-center justify-center gap-3">
                        <span className="text-xl sm:text-2xl">📈</span>
                        <div className="text-center">
                          <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">Growth</div>
                        </div>
                      </div>
                      <div className="mb-2 text-center text-lg leading-tight font-black sm:text-2xl text-green-700 dark:text-green-400">
                        +{item.epsGrowth || '0%'}
                      </div>
                      <div className="h-px flex-shrink-0 bg-gradient-to-r from-transparent via-slate-300 dark:via-slate-500 to-transparent"></div>
                    </div>
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
