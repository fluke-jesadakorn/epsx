"use client";

import { useCallback, useEffect, useRef, useState } from 'react';

export interface OptimisticUpdate<T> {
  id: string;
  type: 'create' | 'update' | 'delete';
  data: T;
  originalData?: T;
  timestamp: number;
  retryCount: number;
}

export interface UseOptimisticUpdatesOptions {
  maxRetries?: number;
  retryDelay?: number;
  rollbackDelay?: number;
}

/**
 *
 * @param options
 */
export function useOptimisticUpdates<T>(
  options: UseOptimisticUpdatesOptions = {}
) {
  const {
    maxRetries = 3,
    retryDelay = 1000,
    rollbackDelay = 5000
  } = options;

  const [pendingUpdates, setPendingUpdates] = useState<Map<string, OptimisticUpdate<T>>>(new Map());
  const timeoutRefs = useRef<Map<string, NodeJS.Timeout>>(new Map());
  const retryTimeoutRefs = useRef<Map<string, NodeJS.Timeout>>(new Map());

  const confirmUpdate = useCallback((id: string) => {
    setPendingUpdates(prev => {
      const newMap = new Map(prev);
      newMap.delete(id);
      return newMap;
    });

    // Clear timers
    const timeout = timeoutRefs.current.get(id);
    if (timeout) {
      clearTimeout(timeout);
      timeoutRefs.current.delete(id);
    }

    const retryTimeout = retryTimeoutRefs.current.get(id);
    if (retryTimeout) {
      clearTimeout(retryTimeout);
      retryTimeoutRefs.current.delete(id);
    }
  }, []);

  const rollbackUpdate = useCallback((id: string) => {
    const update = pendingUpdates.get(id);
    if (!update) { return null; }

    setPendingUpdates(prev => {
      const newMap = new Map(prev);
      newMap.delete(id);
      return newMap;
    });

    // Clear timers
    const timeout = timeoutRefs.current.get(id);
    if (timeout) {
      clearTimeout(timeout);
      timeoutRefs.current.delete(id);
    }

    const retryTimeout = retryTimeoutRefs.current.get(id);
    if (retryTimeout) {
      clearTimeout(retryTimeout);
      retryTimeoutRefs.current.delete(id);
    }

    return update;
  }, [pendingUpdates]);

  const addOptimisticUpdate = useCallback((
    id: string,
    type: OptimisticUpdate<T>['type'],
    data: T,
    originalData?: T
  ) => {
    const update: OptimisticUpdate<T> = {
      id,
      type,
      data,
      originalData,
      timestamp: Date.now(),
      retryCount: 0
    };

    setPendingUpdates(prev => new Map(prev.set(id, update)));

    // Set rollback timer if specified
    if (rollbackDelay > 0) {
      const timeoutId = setTimeout(() => {
        rollbackUpdate(id);
      }, rollbackDelay);

      timeoutRefs.current.set(id, timeoutId);
    }

    return update;
  }, [rollbackDelay, rollbackUpdate]);

  const retryUpdate = useCallback((id: string, retryFn: () => Promise<void>) => {
    const update = pendingUpdates.get(id);
    if (!update ?? update.retryCount >= maxRetries) {
      rollbackUpdate(id);
      return;
    }

    // Increment retry count
    const updatedUpdate = {
      ...update,
      retryCount: update.retryCount + 1
    };

    setPendingUpdates(prev => new Map(prev.set(id, updatedUpdate)));

    // Schedule retry
    const retryTimeout = setTimeout(() => {
      void (async () => {
        try {
          await retryFn();
          confirmUpdate(id);
        } catch (_error) {
          retryUpdate(id, retryFn);
        }
      })();
    }, retryDelay * Math.pow(2, update.retryCount)); // Exponential backoff

    retryTimeoutRefs.current.set(id, retryTimeout);
  }, [pendingUpdates, maxRetries, rollbackUpdate, confirmUpdate, retryDelay]);

  const isPending = useCallback((id: string) => {
    return pendingUpdates.has(id);
  }, [pendingUpdates]);

  const getPendingUpdate = useCallback((id: string) => {
    return pendingUpdates.get(id);
  }, [pendingUpdates]);

  const applyOptimisticUpdates = useCallback(<TItem extends { id: string | number }>(
    items: TItem[],
    updateMap: Map<string, OptimisticUpdate<TItem>>
  ): TItem[] => {
    const result = [...items];

    updateMap.forEach(update => {
      switch (update.type) {
        case 'create':
          result.push(update.data);
          break;

        case 'update': {
          const updateIndex = result.findIndex(item => String(item.id) === update.id);
          if (updateIndex !== -1) {
            result[updateIndex] = update.data;
          }
          break;
        }

        case 'delete': {
          const deleteIndex = result.findIndex(item => String(item.id) === update.id);
          if (deleteIndex !== -1) {
            result.splice(deleteIndex, 1);
          }
          break;
        }
      }
    });

    return result;
  }, []);

  const clearAllPendingUpdates = useCallback(() => {
    // Clear all timers
    timeoutRefs.current.forEach(timeout => clearTimeout(timeout));
    retryTimeoutRefs.current.forEach(timeout => clearTimeout(timeout));

    timeoutRefs.current.clear();
    retryTimeoutRefs.current.clear();
    setPendingUpdates(new Map());
  }, []);

  return {
    pendingUpdates,
    addOptimisticUpdate,
    confirmUpdate,
    rollbackUpdate,
    retryUpdate,
    isPending,
    getPendingUpdate,
    applyOptimisticUpdates,
    clearAllPendingUpdates
  };
}

// Hook for background sync
export interface UseBackgroundSyncOptions {
  syncInterval?: number;
  enabled?: boolean;
  onSync?: () => Promise<void>;
  onError?: (error: Error) => void;
}

/**
 *
 * @param root0
 * @param root0.syncInterval
 * @param root0.enabled
 * @param root0.onSync
 * @param root0.onError
 */
export function useBackgroundSync({
  syncInterval = 30000,
  enabled = true,
  onSync,
  onError
}: UseBackgroundSyncOptions = {}) {
  const [isOnline, setIsOnline] = useState(false);
  const [lastSyncTime, setLastSyncTime] = useState<Date | null>(null);
  const [syncInProgress, setSyncInProgress] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const sync = useCallback(async () => {
    if (!onSync ?? syncInProgress ?? !isOnline) { return; }

    setSyncInProgress(true);
    try {
      await onSync();
      setLastSyncTime(new Date());
    } catch (_error) {
      onError?.(_error as Error);
    } finally {
      setSyncInProgress(false);
    }
  }, [onSync, syncInProgress, isOnline, onError]);

  // Setup online/offline detection
  useEffect(() => {
    // Check if running in browser
    if (typeof window === 'undefined') { return; }

    setIsOnline(navigator.onLine !== false);

    const handleOnline = () => {
      setIsOnline(true);
      // Trigger immediate sync when coming back online
      if (enabled) {
        void sync();
      }
    };

    const handleOffline = () => setIsOnline(false);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [enabled, sync]);

  // Setup interval sync
  useEffect(() => {
    if (enabled && isOnline) {
      intervalRef.current = setInterval(() => { void sync(); }, syncInterval);
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [enabled, isOnline, sync, syncInterval]);

  const forcSync = useCallback(() => {
    void sync();
  }, [sync]);

  return {
    isOnline,
    lastSyncTime,
    syncInProgress,
    forceSync: forcSync
  };
}

// Performance monitoring hook
export interface UsePerformanceMonitorOptions {
  enabled?: boolean;
  sampleRate?: number;
  onMetric?: (metric: PerformanceMetric) => void;
}

export interface PerformanceMetric {
  name: string;
  value: number;
  timestamp: number;
  category: 'navigation' | 'resource' | 'measure' | 'custom';
  metadata?: Record<string, unknown>;
}

/**
 *
 * @param root0
 * @param root0.enabled
 * @param root0.sampleRate
 * @param root0.onMetric
 */
export function usePerformanceMonitor({
  enabled = true,
  sampleRate = 1,
  onMetric
}: UsePerformanceMonitorOptions = {}) {
  const measurementRefs = useRef<Map<string, number>>(new Map());

  const startMeasurement = useCallback((name: string) => {
    if (!enabled ?? Math.random() > sampleRate) { return; }

    measurementRefs.current.set(name, performance.now());
  }, [enabled, sampleRate]);

  const endMeasurement = useCallback((name: string, metadata?: Record<string, unknown>) => {
    if (!enabled) { return; }

    const startTime = measurementRefs.current.get(name);
    if (startTime === undefined) { return; }

    const duration = performance.now() - startTime;
    measurementRefs.current.delete(name);

    const metric: PerformanceMetric = {
      name,
      value: duration,
      timestamp: Date.now(),
      category: 'measure',
      metadata
    };

    onMetric?.(metric);

    return metric;
  }, [enabled, onMetric]);

  const recordCustomMetric = useCallback((
    name: string,
    value: number,
    metadata?: Record<string, unknown>
  ) => {
    if (!enabled ?? Math.random() > sampleRate) { return; }

    const metric: PerformanceMetric = {
      name,
      value,
      timestamp: Date.now(),
      category: 'custom',
      metadata
    };

    onMetric?.(metric);

    return metric;
  }, [enabled, sampleRate, onMetric]);

  const getNavigationMetrics = useCallback(() => {
    if (!enabled) { return []; }

    const navEntries = performance.getEntriesByType('navigation') as PerformanceNavigationTiming[];
    return navEntries.map(entry => ({
      name: 'navigation',
      value: entry.loadEventEnd - entry.startTime,
      timestamp: Date.now(),
      category: 'navigation' as const,
      metadata: {
        domContentLoaded: entry.domContentLoadedEventEnd - entry.startTime,
        domInteractive: entry.domInteractive - entry.startTime,
        firstPaint: entry.responseEnd - entry.startTime
      }
    }));
  }, [enabled]);

  return {
    startMeasurement,
    endMeasurement,
    recordCustomMetric,
    getNavigationMetrics
  };
}