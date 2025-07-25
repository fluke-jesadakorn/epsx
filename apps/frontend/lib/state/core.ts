import { useCallback, useEffect, useRef, useState } from 'react';
import { StateConfig, StateAction, StateMiddleware, AsyncState } from './types';
import { logger } from '../logger';

// Storage utilities
export const storage = {
  get: (key: string, storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') return null;
    try {
      const item = window[storageType].getItem(key);
      return item ? JSON.parse(item) : null;
    } catch {
      return null;
    }
  },
  
  set: (key: string, value: any, storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') return;
    try {
      window[storageType].setItem(key, JSON.stringify(value));
    } catch (error) {
      logger.warn('Failed to save to storage', { error: error instanceof Error ? error.message : error, key, storageType });
    }
  },
  
  remove: (key: string, storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') return;
    try {
      window[storageType].removeItem(key);
    } catch (error) {
      logger.warn('Failed to remove from storage', { error: error instanceof Error ? error.message : error, key, storageType });
    }
  }
};

// Create async state helper
export function createAsyncState<T = any>(initialData: T | null = null): AsyncState<T> {
  return {
    data: initialData,
    loading: false,
    error: null,
    lastUpdated: null
  };
}

// State middleware for logging
export const loggingMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  if (process.env.NODE_ENV === 'development') {
    logger.debug(`State change: ${store}:${action.type}`, {
      store,
      actionType: action.type,
      action,
      hasStateChange: JSON.stringify(prevState) !== JSON.stringify(nextState)
    });
  }
};

// State middleware for analytics
export const analyticsMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  // In production, send to analytics service
  if (typeof window !== 'undefined' && window.gtag) {
    window.gtag('event', 'state_change', {
      store,
      action_type: action.type,
      timestamp: action.meta?.timestamp
    });
  }
};

// Debounce utility for performance
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

// Custom hook for creating a context store with persistence and middleware
export function useContextStore<T>(
  initialState: T,
  config: StateConfig = {}
) {
  const { persist, middleware = [], devtools = process.env.NODE_ENV === 'development' } = config;
  
  // Load persisted state on mount
  const [state, setState] = useState<T>(() => {
    if (persist && typeof window !== 'undefined') {
      const savedState = storage.get(persist.key, persist.storage);
      if (savedState) {
        // Handle state migration
        if (persist.version && persist.migrate && savedState._version !== persist.version) {
          const migratedState = persist.migrate(savedState, savedState._version || 0);
          return { ...migratedState, _version: persist.version };
        }
        return savedState;
      }
    }
    return persist?.version ? { ...initialState, _version: persist.version } : initialState;
  });

  // Save to storage when state changes
  const debouncedState = useDebounce(state, 500);
  useEffect(() => {
    if (persist && typeof window !== 'undefined') {
      storage.set(persist.key, debouncedState, persist.storage);
    }
  }, [debouncedState, persist]);

  // Dispatch function with middleware support
  const dispatch = useCallback((action: StateAction, reducer: (state: T, action: StateAction) => T) => {
    setState(prevState => {
      const nextState = reducer(prevState, action);
      
      // Run middleware
      const storeName = persist?.key || 'store';
      middleware.forEach(mw => mw(action, prevState, nextState, storeName));
      
      // DevTools support
      if (devtools && typeof window !== 'undefined' && (window as any).__REDUX_DEVTOOLS_EXTENSION__) {
        const devTools = (window as any).__REDUX_DEVTOOLS_EXTENSION__.connect({
          name: storeName
        });
        devTools.send(action, nextState);
      }
      
      return nextState;
    });
  }, [middleware, devtools, persist?.key]);

  return { state, dispatch };
}

// Optimistic updates helper
export function useOptimisticUpdates<T>() {
  const optimisticUpdates = useRef<Map<string, () => void>>(new Map());

  const startOptimisticUpdate = useCallback((
    id: string, 
    optimisticUpdate: () => void, 
    rollback: () => void
  ) => {
    optimisticUpdate();
    optimisticUpdates.current.set(id, rollback);
  }, []);

  const confirmOptimisticUpdate = useCallback((id: string) => {
    optimisticUpdates.current.delete(id);
  }, []);

  const rollbackOptimisticUpdate = useCallback((id: string) => {
    const rollback = optimisticUpdates.current.get(id);
    if (rollback) {
      rollback();
      optimisticUpdates.current.delete(id);
    }
  }, []);

  const rollbackAll = useCallback(() => {
    optimisticUpdates.current.forEach(rollback => rollback());
    optimisticUpdates.current.clear();
  }, []);

  return {
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate,
    rollbackAll
  };
}

// Async action helper
export async function withAsyncState<T, R>(
  setState: (updater: (prev: AsyncState<T>) => AsyncState<T>) => void,
  asyncFn: () => Promise<R>,
  successMapper?: (result: R) => T
): Promise<R> {
  setState(prev => ({ ...prev, loading: true, error: null }));
  
  try {
    const result = await asyncFn();
    setState(prev => ({
      ...prev,
      loading: false,
      data: successMapper ? successMapper(result) : result as any,
      lastUpdated: Date.now()
    }));
    return result;
  } catch (error) {
    setState(prev => ({
      ...prev,
      loading: false,
      error: error instanceof Error ? error.message : 'An error occurred'
    }));
    throw error;
  }
}

// SSR-safe hook for window/client-only operations
export function useClientOnly<T>(clientValue: T, serverValue: T): T {
  const [isClient, setIsClient] = useState(false);
  
  useEffect(() => {
    setIsClient(true);
  }, []);
  
  return isClient ? clientValue : serverValue;
}

// Performance monitoring for state changes
export function useStatePerformance(stateName: string) {
  const renderCount = useRef(0);
  const lastRenderTime = useRef(Date.now());
  
  useEffect(() => {
    renderCount.current++;
    const now = Date.now();
    const timeSinceLastRender = now - lastRenderTime.current;
    
    if (process.env.NODE_ENV === 'development' && timeSinceLastRender < 16) {
      logger.warn(`${stateName} is rendering frequently`, { 
        stateName, 
        timeSinceLastRender, 
        totalRenders: renderCount.current 
      });
    }
    
    lastRenderTime.current = now;
  });
}