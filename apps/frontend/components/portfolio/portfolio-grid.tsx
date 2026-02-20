'use client';

import { StockDataCard } from '@/shared/components';
import type { SymbolCardData } from '@/shared/types/analytics';
import { Heart } from 'lucide-react';
import { useMemo } from 'react';
import { useWatchlist } from './watchlist-provider';

interface PortfolioGridProps {
  watchedStocks: SymbolCardData[];
  allRankings: SymbolCardData[];
}

export function PortfolioGrid({ watchedStocks: initialWatched, allRankings }: PortfolioGridProps) {
  const { symbols, isWatchlisted, toggle, isLoading } = useWatchlist();

  // Re-derive watched stocks from current watchlist state
  const watchedStocks = useMemo(() => {
    const set = new Set(symbols.map((s) => s.toUpperCase()));
    return allRankings.filter((r) => set.has(r.symbol.toUpperCase()));
  }, [symbols, allRankings]);

  if (watchedStocks.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16">
        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl bg-gray-100 dark:bg-slate-800/50">
          <Heart className="h-8 w-8 text-slate-400" />
        </div>
        <h3 className="mb-2 text-lg font-semibold text-white">No stocks in watchlist</h3>
        <p className="max-w-md text-center text-sm text-slate-400">
          Use the search bar above to find stocks and add them to your watchlist.
          Click the heart icon on any stock card to track it here.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-white">
          Your Watchlist
          <span className="ml-2 text-sm font-normal text-slate-400">
            ({watchedStocks.length} stock{watchedStocks.length !== 1 ? 's' : ''})
          </span>
        </h2>
        {isLoading && (
          <span className="text-xs text-slate-500">Updating...</span>
        )}
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5">
        {watchedStocks.map((card) => {
          const latestQ = card.quarterly_performance[0];
          return (
            <StockDataCard
              key={card.symbol}
              symbol={card.symbol}
              rank={card.rank}
              epsGrowth={latestQ?.eps_growth ?? 0}
              price={latestQ?.price ?? 0}
              currency={card.currency}
              daysUntilNextAction={card.next_quarter_estimate?.days_until_announcement ?? 0}
              companyName={card.company_name ?? card.name}
              variant={card.rank <= 5 ? 'premium' : 'standard'}
              isWatchlisted={true}
              onWatchlistToggle={toggle}
            />
          );
        })}
      </div>
    </div>
  );
}
