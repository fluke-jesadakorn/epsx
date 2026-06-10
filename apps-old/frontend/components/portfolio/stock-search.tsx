'use client';

import { cn } from '@/lib/utils';
import { StockDataCard } from '@/shared/components';
import type { SymbolCardData } from '@/shared/types/analytics';
import { Heart, Search, X } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { useWatchlist } from './watchlist-provider';

interface StockSearchProps {
  rankings: SymbolCardData[];
}

export function StockSearch({ rankings }: StockSearchProps) {
  const [query, setQuery] = useState('');
  const [open, setOpen] = useState(false);
  const { isWatchlisted, toggle } = useWatchlist();

  const results = useMemo(() => {
    if (query.length < 1) {return [];}
    const q = query.toUpperCase();
    return rankings
      .filter(
        (r) =>
          r.symbol.includes(q) ||
          (r.company_name ?? r.name ?? '').toUpperCase().includes(q),
      )
      .slice(0, 12);
  }, [query, rankings]);

  const handleToggle = useCallback(
    (symbol: string) => {
      toggle(symbol);
    },
    [toggle],
  );

  return (
    <div className="space-y-4">
      {/* Search bar */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
        <input
          type="text"
          placeholder="Search stocks to add to watchlist..."
          value={query}
          onChange={(e) => { setQuery(e.target.value); setOpen(true); }}
          onFocus={() => setOpen(true)}
          className="w-full rounded-xl border border-gray-200 dark:border-slate-700 bg-gray-100 dark:bg-slate-800/50 py-3 pl-10 pr-10 text-sm text-foreground placeholder-slate-500 outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/30"
        />
        {query.length > 0 && (
          <button
            type="button"
            onClick={() => { setQuery(''); setOpen(false); }}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Results */}
      {open && results.length > 0 && (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {results.map((r) => {
            const latestQ = r.quarterly_performance[0];
            const watched = isWatchlisted(r.symbol);
            return (
              <div key={r.symbol} className="relative">
                <StockDataCard
                  symbol={r.symbol}
                  rank={r.rank}
                  epsGrowth={latestQ?.eps_growth ?? 0}
                  price={latestQ?.price ?? 0}
                  currency={r.currency}
                  daysUntilNextAction={r.next_quarter_estimate?.days_until_announcement ?? 0}
                  companyName={r.company_name ?? r.name}
                  variant={r.rank <= 5 ? 'premium' : 'standard'}
                  isWatchlisted={watched}
                  onWatchlistToggle={handleToggle}
                />
              </div>
            );
          })}
        </div>
      )}

      {open && query.length >= 1 && results.length === 0 && (
        <p className="py-4 text-center text-sm text-slate-500">
          No stocks found matching &quot;{query}&quot;
        </p>
      )}
    </div>
  );
}
