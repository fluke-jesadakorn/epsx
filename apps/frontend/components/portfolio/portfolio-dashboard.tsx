import { getWatchlistAction } from '@/app/actions/watchlist';
import { getAnalyticsData } from '@/lib/server/data';
import type { SymbolCardData } from '@/shared/types/analytics';
import { Suspense } from 'react';
import { WatchlistProvider } from './watchlist-provider';
import { PortfolioGrid } from './portfolio-grid';
import { StockSearch } from './stock-search';

async function WatchlistContent() {
  const [watchlist, allData] = await Promise.all([
    getWatchlistAction(),
    getAnalyticsData({ page: 1, limit: 100, sort_by: 'eps_growth' }),
  ]);

  const allRankings = allData.rankings;
  const watchlistSet = new Set(watchlist.map((s) => s.toUpperCase()));
  const watchedStocks = allRankings.filter((r) => watchlistSet.has(r.symbol.toUpperCase()));

  return (
    <WatchlistProvider initial={watchlist}>
      <div className="space-y-8">
        {/* Search to add stocks */}
        <StockSearch rankings={allRankings} />

        {/* Watchlisted cards */}
        <PortfolioGrid watchedStocks={watchedStocks} allRankings={allRankings} />
      </div>
    </WatchlistProvider>
  );
}

function LoadingSkeleton() {
  return (
    <div className="space-y-6">
      <div className="h-12 w-full animate-pulse rounded-xl bg-gray-100 dark:bg-slate-800/50" />
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <div
            key={`skel-${String(i)}`}
            className="h-[350px] animate-pulse rounded-2xl border border-gray-200 dark:border-border bg-white dark:bg-card"
          />
        ))}
      </div>
    </div>
  );
}

export default async function PortfolioDashboard() {
  return (
    <Suspense fallback={<LoadingSkeleton />}>
      <WatchlistContent />
    </Suspense>
  );
}
