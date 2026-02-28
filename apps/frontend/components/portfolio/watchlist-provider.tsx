'use client';

import { addToWatchlistAction, removeFromWatchlistAction } from '@/app/actions/watchlist';
import { useRequireAuth } from '@/shared/hooks/use-require-auth';
import { createContext, useCallback, useContext, useMemo, useState, useTransition } from 'react';

interface WatchlistCtx {
  symbols: string[];
  isLoading: boolean;
  isWatchlisted: (symbol: string) => boolean;
  toggle: (symbol: string) => void;
}

const Ctx = createContext<WatchlistCtx>({
  symbols: [],
  isLoading: false,
  isWatchlisted: () => false,
  toggle: () => undefined,
});

export function useWatchlist() {
  return useContext(Ctx);
}

export function WatchlistProvider({
  initial,
  children,
}: {
  initial: string[];
  children: React.ReactNode;
}) {
  const [symbols, setSymbols] = useState<string[]>(initial);
  const [isPending, startTransition] = useTransition();
  const { requireAuth } = useRequireAuth();

  const symbolSet = useMemo(() => new Set(symbols), [symbols]);

  const isWatchlisted = useCallback(
    (s: string) => symbolSet.has(s.toUpperCase()),
    [symbolSet],
  );

  const doToggle = useCallback(async (upper: string, removing: boolean) => {
    const ok = await requireAuth();
    if (!ok) { return; }
    if (removing) {
      setSymbols((prev) => prev.filter((s) => s !== upper));
    } else {
      setSymbols((prev) => [...prev, upper]);
    }
    const updated = removing
      ? await removeFromWatchlistAction(upper)
      : await addToWatchlistAction(upper);
    if (updated !== null) { setSymbols(updated); }
  }, [requireAuth]);

  const toggle = useCallback(
    (symbol: string) => {
      const upper = symbol.toUpperCase();
      const removing = symbolSet.has(upper);
      startTransition(() => { void doToggle(upper, removing); });
    },
    [symbolSet, doToggle],
  );

  const value = useMemo(
    () => ({ symbols, isLoading: isPending, isWatchlisted, toggle }),
    [symbols, isPending, isWatchlisted, toggle],
  );

  return <Ctx value={value}>{children}</Ctx>;
}
