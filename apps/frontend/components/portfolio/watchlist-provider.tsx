'use client';

import {
  addToWatchlistAction,
  removeFromWatchlistAction,
} from '@/app/actions/watchlist';
import { useRequireAuth } from '@/shared/hooks/use-require-auth';
import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  useTransition,
} from 'react';

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

function normalizeSymbol(symbol: string) {
  return symbol.trim().toUpperCase();
}

function normalizeSymbols(symbols: string[]) {
  return Array.from(new Set(symbols.map(normalizeSymbol).filter(Boolean)));
}

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
  const [symbols, setSymbols] = useState<string[]>(() =>
    normalizeSymbols(initial)
  );
  const [isPending, startTransition] = useTransition();
  const { requireAuth } = useRequireAuth();

  const symbolSet = useMemo(
    () => new Set(normalizeSymbols(symbols)),
    [symbols]
  );

  const isWatchlisted = useCallback(
    (s: string) => symbolSet.has(normalizeSymbol(s)),
    [symbolSet]
  );

  const doToggle = useCallback(
    async (upper: string, removing: boolean) => {
      const ok = await requireAuth();
      if (!ok) {
        return;
      }
      if (removing) {
        setSymbols(prev => normalizeSymbols(prev).filter(s => s !== upper));
      } else {
        setSymbols(prev => normalizeSymbols([...prev, upper]));
      }
      const updated = removing
        ? await removeFromWatchlistAction(upper)
        : await addToWatchlistAction(upper);
      if (updated !== null) {
        setSymbols(normalizeSymbols(updated));
        return;
      }
      setSymbols(prev =>
        removing
          ? normalizeSymbols([...prev, upper])
          : normalizeSymbols(prev).filter(s => s !== upper)
      );
    },
    [requireAuth]
  );

  const toggle = useCallback(
    (symbol: string) => {
      const upper = normalizeSymbol(symbol);
      if (!upper) {
        return;
      }
      const removing = symbolSet.has(upper);
      startTransition(() => {
        void doToggle(upper, removing);
      });
    },
    [symbolSet, doToggle]
  );

  const value = useMemo(
    () => ({ symbols, isLoading: isPending, isWatchlisted, toggle }),
    [symbols, isPending, isWatchlisted, toggle]
  );

  return <Ctx value={value}>{children}</Ctx>;
}
