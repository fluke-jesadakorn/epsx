/**
 * State Management Hooks
 * React hooks for state management, persistence, and optimization
 */

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

// ============================================================================
// Persistent State Hook
// ============================================================================

interface PersistentStateOptions<T> {
  key: string;
  defaultValue: T;
  storage?: 'localStorage' | 'sessionStorage';
  serializer?: {
    serialize: (value: T) => string;
    deserialize: (value: string) => T;
  };
}

export function usePersistentState<T>(options: PersistentStateOptions<T>) {
  const {
    key,
    defaultValue,
    storage: storageType = 'localStorage',
    serializer = {
      serialize: JSON.stringify,
      deserialize: JSON.parse
    }
  } = options;

  const [state, setState] = useState<T>(() => {
    if (typeof window === 'undefined') return defaultValue;

    try {
      const item = window[storageType].getItem(key);
      return item ? serializer.deserialize(item) : defaultValue;
    } catch (error) {
      console.warn(`Failed to load persisted state for key "${key}":`, error);
      return defaultValue;
    }
  });

  const setPersistedState = useCallback((newState: T | ((prevState: T) => T)) => {
    setState(prevState => {
      const nextState = typeof newState === 'function' ? (newState as Function)(prevState) : newState;

      try {
        window[storageType].setItem(key, serializer.serialize(nextState));
      } catch (error) {
        console.warn(`Failed to persist state for key "${key}":`, error);
      }

      return nextState;
    });
  }, [key, storageType, serializer]);

  return [state, setPersistedState] as const;
}

// ============================================================================
// Optimistic State Hook
// ============================================================================

interface OptimisticStateOptions<T> {
  initialState: T;
  key?: string;
  rollbackDelay?: number;
}

export function useOptimisticState<T>(options: OptimisticStateOptions<T>) {
  const { initialState, key = 'optimistic', rollbackDelay = 5000 } = options;
  const [state, setState] = useState<T>(initialState);
  const [isOptimistic, setIsOptimistic] = useState(false);
  const rollbackTimeoutRef = useRef<NodeJS.Timeout | undefined>(undefined);

  const setOptimistic = useCallback(async (
    optimisticState: T,
    asyncAction: () => Promise<T>
  ) => {
    // Clear any existing rollback
    if (rollbackTimeoutRef.current) {
      clearTimeout(rollbackTimeoutRef.current);
    }

    // Apply optimistic state immediately
    setState(optimisticState);
    setIsOptimistic(true);

    try {
      // Execute async action
      const result = await asyncAction();

      // Apply real result
      setState(result);
      setIsOptimistic(false);
    } catch (error) {
      // Rollback on error
      console.warn(`Optimistic update failed for ${key}, rolling back:`, error);

      rollbackTimeoutRef.current = setTimeout(() => {
        setState(initialState);
        setIsOptimistic(false);
      }, rollbackDelay);
    }
  }, [initialState, key, rollbackDelay]);

  useEffect(() => {
    return () => {
      if (rollbackTimeoutRef.current) {
        clearTimeout(rollbackTimeoutRef.current);
      }
    };
  }, []);

  return { state, isOptimistic, setOptimistic };
}

// ============================================================================
// Async Action Hook
// ============================================================================

interface AsyncActionState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
}

export function useAsyncAction<T>(
  asyncFn: () => Promise<T>,
  deps: React.DependencyList = []
) {
  const [state, setState] = useState<AsyncActionState<T>>({
    data: null,
    loading: false,
    error: null,
    lastUpdated: null
  });

  const execute = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      const data = await asyncFn();
      setState({
        data,
        loading: false,
        error: null,
        lastUpdated: Date.now()
      });
      return data;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setState(prev => ({
        ...prev,
        loading: false,
        error: errorMessage
      }));
      throw error;
    }
  }, deps);

  const reset = useCallback(() => {
    setState({
      data: null,
      loading: false,
      error: null,
      lastUpdated: null
    });
  }, []);

  return {
    ...state,
    execute,
    reset,
    isStale: (maxAge: number) => {
      return state.lastUpdated ? (Date.now() - state.lastUpdated) > maxAge : true;
    }
  };
}

// ============================================================================
// State Selector Hook
// ============================================================================

export function useStateSelector<T, R>(
  selector: (state: T) => R,
  state: T,
  equalityFn?: (a: R, b: R) => boolean
) {
  const selectedState = useMemo(() => selector(state), [state, selector]);
  const prevSelectedStateRef = useRef<R>(selectedState);

  const isEqual = equalityFn || ((a, b) => a === b);

  if (!isEqual(prevSelectedStateRef.current, selectedState)) {
    prevSelectedStateRef.current = selectedState;
  }

  return prevSelectedStateRef.current;
}

// ============================================================================
// Optimistic List Hook
// ============================================================================

export function useOptimisticList<T extends { id: string }>(initialList: T[]) {
  const [list, setList] = useState<T[]>(initialList);
  const [optimisticItems, setOptimisticItems] = useState<Set<string>>(new Set());

  const add = useCallback(async (item: T, asyncFn: () => Promise<void>) => {
    // Add optimistically
    setList(prev => [...prev, item]);
    setOptimisticItems(prev => new Set(prev).add(item.id));

    try {
      await asyncFn();
      // Remove from optimistic set on success
      setOptimisticItems(prev => {
        const next = new Set(prev);
        next.delete(item.id);
        return next;
      });
    } catch (error) {
      // Remove on failure
      setList(prev => prev.filter(i => i.id !== item.id));
      setOptimisticItems(prev => {
        const next = new Set(prev);
        next.delete(item.id);
        return next;
      });
      throw error;
    }
  }, []);

  const remove = useCallback(async (id: string, asyncFn: () => Promise<void>) => {
    const item = list.find(i => i.id === id);
    if (!item) return;

    // Remove optimistically
    setList(prev => prev.filter(i => i.id !== id));
    setOptimisticItems(prev => new Set(prev).add(id));

    try {
      await asyncFn();
      // Remove from optimistic set on success
      setOptimisticItems(prev => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
    } catch (error) {
      // Restore on failure
      setList(prev => [...prev, item]);
      setOptimisticItems(prev => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
      throw error;
    }
  }, [list]);

  const update = useCallback(async (id: string, updates: Partial<T>, asyncFn: () => Promise<void>) => {
    const originalItem = list.find(i => i.id === id);
    if (!originalItem) return;

    // Update optimistically
    setList(prev => prev.map(item =>
      item.id === id ? { ...item, ...updates } : item
    ));
    setOptimisticItems(prev => new Set(prev).add(id));

    try {
      await asyncFn();
      // Remove from optimistic set on success
      setOptimisticItems(prev => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
    } catch (error) {
      // Restore original on failure
      setList(prev => prev.map(item =>
        item.id === id ? originalItem : item
      ));
      setOptimisticItems(prev => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
      throw error;
    }
  }, [list]);

  return {
    list,
    add,
    remove,
    update,
    isOptimistic: (id: string) => optimisticItems.has(id),
    optimisticCount: optimisticItems.size
  };
}

// ============================================================================
// User Preferences Hook
// ============================================================================

export function useUserPreferences<T = Record<string, unknown>>(defaultPreferences?: T) {
  return usePersistentState({
    key: 'user-preferences',
    defaultValue: defaultPreferences || {} as T,
    storage: 'localStorage'
  });
}

// ============================================================================
// Theme Hook
// ============================================================================

export function useTheme() {
  const [theme, setTheme] = usePersistentState({
    key: 'app-theme',
    defaultValue: 'system' as 'light' | 'dark' | 'system',
    storage: 'localStorage'
  });

  const [systemTheme, setSystemTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    setSystemTheme(mediaQuery.matches ? 'dark' : 'light');

    const handler = (e: MediaQueryListEvent) => {
      setSystemTheme(e.matches ? 'dark' : 'light');
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  const resolvedTheme = theme === 'system' ? systemTheme : theme;

  return {
    theme,
    resolvedTheme,
    setTheme,
    isSystem: theme === 'system',
    isDark: resolvedTheme === 'dark'
  };
}

// ============================================================================
// Local Storage Hook
// ============================================================================

export function useLocalStorage<T>(key: string, defaultValue: T) {
  return usePersistentState({
    key,
    defaultValue,
    storage: 'localStorage'
  });
}

// ============================================================================
// Session Storage Hook
// ============================================================================

export function useSessionStorage<T>(key: string, defaultValue: T) {
  return usePersistentState({
    key,
    defaultValue,
    storage: 'sessionStorage'
  });
}

// ============================================================================
// Debounced State Hook
// ============================================================================

export function useDebouncedState<T>(initialValue: T, delay: number = 500) {
  const [value, setValue] = useState<T>(initialValue);
  const [debouncedValue, setDebouncedValue] = useState<T>(initialValue);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return [value, debouncedValue, setValue] as const;
}