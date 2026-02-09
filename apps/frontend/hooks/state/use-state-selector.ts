import { useAppState } from '@/context/app-state';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

type Theme = 'light' | 'dark' | 'system'

interface UiSlice {
  theme?: Theme
  toasts?: unknown[]
  modal?: unknown
  sidebar?: unknown
}

interface UserSlice {
  data?: {
    permissions?: unknown
    [key: string]: unknown
  } | null
  loading?: boolean
}

interface TradingSlice {
  data?: {
    watchlist?: unknown[]
    portfolio?: Array<{ value?: number }>
  }
  realtime?: unknown
}

interface NotificationsSlice {
  list?: Array<{ read?: boolean }>
}

interface AnalyticsSlice {
  data?: unknown
  rankings?: unknown[]
}

interface RootState {
  ui?: UiSlice
  user?: UserSlice
  trading?: TradingSlice
  notifications?: NotificationsSlice
  analytics?: AnalyticsSlice
}

type SelectorFunction<T, R> = (state: T) => R;
type EqualityFunction<R> = (prev: R, next: R) => boolean;

interface SelectorOptions<R> {
  equalityFn?: EqualityFunction<R>;
  debugName?: string;
}

const shallowEqual: EqualityFunction<unknown> = (prev, next) => {
  if (prev === next) {return true;}
  if (typeof prev !== 'object' || typeof next !== 'object') {return false;}
  if (prev === null || next === null) {return false;}

  const objA = prev as Record<string, unknown>;
  const objB = next as Record<string, unknown>;
  const keysA = Object.keys(objA);
  const keysB = Object.keys(objB);

  if (keysA.length !== keysB.length) {return false;}

  for (const key of keysA) {
    if (!keysB.includes(key) || objA[key] !== objB[key]) {
      return false;
    }
  }

  return true;
};

/**
 * Performance-optimized state selector hook
 * Only re-renders when the selected portion of state changes
 */
export function useStateSelector<R>(
  selector: SelectorFunction<RootState, R>,
  options: SelectorOptions<R> = {}
): R {
  const { state } = useAppState() as { state: RootState };
  const { equalityFn = shallowEqual, debugName } = options;

  const selectorRef = useRef(selector);
  const equalityRef = useRef(equalityFn);
  const [selectedState, setSelectedState] = useState(() => selector(state));

  // Update refs
  selectorRef.current = selector;
  equalityRef.current = equalityFn;

  useEffect(() => {
    const newSelectedState = selectorRef.current(state);

    if (!equalityRef.current(selectedState, newSelectedState)) {
      if (debugName && process.env.NODE_ENV === 'development') {
        // eslint-disable-next-line no-console
        console.log(`[Selector ${debugName}] State changed`, {
          prev: selectedState,
          next: newSelectedState
        });
      }
      setSelectedState(newSelectedState);
    }
  }, [state, selectedState, debugName]);

  return selectedState;
}

/**
 * Memoized state selector with dependency tracking
 */
export function useMemoizedSelector<R>(
  selector: SelectorFunction<RootState, R>,
  deps: unknown[] = [],
  options: SelectorOptions<R> = {}
): R {
  const memoizedSelector = useMemo(() => selector, deps);
  return useStateSelector(memoizedSelector, options);
}

/**
 * Multi-selector hook for selecting multiple pieces of state efficiently
 */
export function useMultiSelector<T extends Record<string, unknown>>(
  selectors: { [K in keyof T]: SelectorFunction<RootState, T[K]> },
  options: SelectorOptions<T> = {}
): T {
  const combinedSelector = useCallback((state: RootState) => {
    const result = {} as T;
    for (const [key, selector] of Object.entries(selectors)) {
      result[key as keyof T] = selector(state);
    }
    return result;
  }, [selectors]);

  return useStateSelector(combinedSelector, options);
}

/**
 * Specialized selectors for common use cases
 */

// UI State Selectors
export function useThemeSelector() {
  return useStateSelector((state) => state.ui?.theme, {
    debugName: 'theme'
  });
}

export function useToastsSelector() {
  return useStateSelector((state) => state.ui?.toasts ?? [], {
    debugName: 'toasts'
  });
}

export function useModalSelector() {
  return useStateSelector((state) => state.ui?.modal, {
    debugName: 'modal'
  });
}

export function useSidebarSelector() {
  return useStateSelector((state) => state.ui?.sidebar, {
    debugName: 'sidebar'
  });
}

// User State Selectors
export function useUserSelector() {
  return useStateSelector((state) => state.user?.data, {
    debugName: 'user'
  });
}

export function useUserLoadingSelector() {
  return useStateSelector((state) => state.user?.loading ?? false, {
    equalityFn: (prev, next) => prev === next,
    debugName: 'userLoading'
  });
}

export function useUserPermissionsSelector() {
  return useStateSelector((state) => {
    const perms = state.user?.data?.permissions
    return Array.isArray(perms) ? perms : []
  }, {
    debugName: 'userPermissions'
  });
}

// Trading State Selectors
export function useWatchlistSelector() {
  return useStateSelector((state) => state.trading?.data?.watchlist ?? [], {
    debugName: 'watchlist'
  });
}

export function usePortfolioSelector() {
  return useStateSelector((state) => state.trading?.data?.portfolio ?? [], {
    debugName: 'portfolio'
  });
}

export function usePortfolioTotalSelector() {
  return useStateSelector((state) => {
    const portfolio = state.trading?.data?.portfolio ?? [];
    return portfolio.reduce((total, item) => total + (item.value ?? 0), 0);
  }, {
    equalityFn: (prev, next) => prev === next,
    debugName: 'portfolioTotal'
  });
}

export function useRealtimeDataSelector() {
  return useStateSelector((state) => state.trading?.realtime, {
    debugName: 'realtimeData'
  });
}

// Notification Selectors
export function useNotificationsSelector() {
  return useStateSelector((state) => state.notifications?.list ?? [], {
    debugName: 'notifications'
  });
}

export function useUnreadNotificationsSelector() {
  return useStateSelector((state) => {
    const notifications = state.notifications?.list ?? [];
    return notifications.filter((n) => !n.read);
  }, {
    debugName: 'unreadNotifications'
  });
}

export function useNotificationCountSelector() {
  return useStateSelector((state) => {
    const notifications = state.notifications?.list ?? [];
    return notifications.filter((n) => !n.read).length;
  }, {
    equalityFn: (prev, next) => prev === next,
    debugName: 'notificationCount'
  });
}

// Analytics Selectors
export function useAnalyticsDataSelector() {
  return useStateSelector((state) => state.analytics?.data, {
    debugName: 'analyticsData'
  });
}

export function useRankingsSelector() {
  return useStateSelector((state) => state.analytics?.rankings ?? [], {
    debugName: 'rankings'
  });
}

// Performance monitoring hook
export function useSelectorPerformance(_name: string) {
  const renderCount = useRef(0);
  const lastRender = useRef(Date.now());

  useEffect(() => {
    renderCount.current++;
    const now = Date.now();
    const _timeSinceLastRender = now - lastRender.current;
    lastRender.current = now;

    if (process.env.NODE_ENV === 'development') {
      // Performance logging could be added here
    }
  });

  return {
    renderCount: renderCount.current,
    lastRenderTime: lastRender.current
  };
}

// Utility function to create custom selectors
export function createSelector<T, R>(
  selector: SelectorFunction<T, R>,
  options?: SelectorOptions<R>
) {
  return () => useStateSelector(selector as unknown as SelectorFunction<RootState, R>, options);
}

// Batched selector updates
export function useBatchedSelectors<T extends Record<string, unknown>>(
  selectors: { [K in keyof T]: () => T[K] }
): T {
  const results = {} as T;

  for (const [key, selector] of Object.entries(selectors)) {
    results[key as keyof T] = selector();
  }

  return results;
}

// Responsive breakpoint hook
export function useResponsive() {
  const [breakpoint, setBreakpoint] = useState('lg');

  useEffect(() => {
    const updateBreakpoint = () => {
      const width = window.innerWidth;
      if (width < 640) {setBreakpoint('sm');}
      else if (width < 768) {setBreakpoint('md');}
      else if (width < 1024) {setBreakpoint('lg');}
      else if (width < 1280) {setBreakpoint('xl');}
      else {setBreakpoint('2xl');}
    };

    updateBreakpoint();
    window.addEventListener('resize', updateBreakpoint);

    return () => window.removeEventListener('resize', updateBreakpoint);
  }, []);

  return {
    isMobile: breakpoint === 'sm',
    isTablet: breakpoint === 'md',
    isDesktop: ['lg', 'xl', '2xl'].includes(breakpoint),
    breakpoint
  };
}