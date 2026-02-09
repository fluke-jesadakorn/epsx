import { useCallback, useRef, useState } from 'react';
import { useToasts } from '@/context/ui-context';

interface OptimisticUpdate<T> {
  id: string;
  optimisticState: T;
  rollback: () => void;
  timestamp: number;
}

interface OptimisticStateOptions {
  onError?: (error: Error, rollback: () => void) => void;
  errorMessage?: string;
  timeout?: number; // Auto-rollback timeout in ms
}

export function useOptimisticState<T>(
  initialState: T,
  options: OptimisticStateOptions = {}
) {
  const [state, setState] = useState<T>(initialState);
  const [isOptimistic, setIsOptimistic] = useState(false);
  const optimisticUpdates = useRef<Map<string, OptimisticUpdate<T>>>(new Map());
  const timeouts = useRef<Map<string, NodeJS.Timeout>>(new Map());
  const { error: showError } = useToasts();

  const { onError, errorMessage, timeout = 30000 } = options;

  const applyOptimisticUpdate = useCallback(<Args extends unknown[]>(
    updateFn: (current: T) => T,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    const updateId = Math.random().toString(36).slice(2, 11);
    const previousState = state;
    const optimisticState = updateFn(state);

    // Store rollback function
    const rollback = () => {
      setState(previousState);
      setIsOptimistic(false);
      optimisticUpdates.current.delete(updateId);

      // Clear timeout
      const timeoutId = timeouts.current.get(updateId);
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeouts.current.delete(updateId);
      }
    };

    // Store optimistic update
    optimisticUpdates.current.set(updateId, {
      id: updateId,
      optimisticState,
      rollback,
      timestamp: Date.now()
    });

    // Apply optimistic update
    setState(optimisticState);
    setIsOptimistic(true);

    // Set timeout for auto-rollback
    if (timeout > 0) {
      const timeoutId = setTimeout(() => {
        rollback();
        if (errorMessage) {
          showError(errorMessage, 'Request timed out');
        }
      }, timeout);
      timeouts.current.set(updateId, timeoutId);
    }

    // Execute async action
    const promise = asyncAction(...args)
      .then((result) => {
        // Confirm optimistic update
        const update = optimisticUpdates.current.get(updateId);
        if (update) {
          // Clear timeout
          const timeoutId = timeouts.current.get(updateId);
          if (timeoutId) {
            clearTimeout(timeoutId);
            timeouts.current.delete(updateId);
          }

          optimisticUpdates.current.delete(updateId);
          setIsOptimistic(optimisticUpdates.current.size > 0);
        }
        return result;
      })
      .catch((error) => {
        // Rollback optimistic update
        rollback();

        if (onError) {
          onError(error, rollback);
        } else if (errorMessage) {
          showError(errorMessage, error.message);
        }

        throw error;
      });

    return {
      promise,
      updateId,
      rollback
    };
  }, [state, onError, errorMessage, timeout, showError]);

  const rollbackUpdate = useCallback((updateId: string) => {
    const update = optimisticUpdates.current.get(updateId);
    if (update) {
      update.rollback();
    }
  }, []);

  const rollbackAll = useCallback(() => {
    // Rollback in reverse chronological order (newest first)
    const updates = Array.from(optimisticUpdates.current.values())
      .sort((a, b) => b.timestamp - a.timestamp);

    updates.forEach(update => update.rollback());
  }, []);

  const confirmUpdate = useCallback((updateId: string) => {
    const update = optimisticUpdates.current.get(updateId);
    if (update) {
      // Clear timeout
      const timeoutId = timeouts.current.get(updateId);
      if (timeoutId) {
        clearTimeout(timeoutId);
        timeouts.current.delete(updateId);
      }

      optimisticUpdates.current.delete(updateId);
      setIsOptimistic(optimisticUpdates.current.size > 0);
    }
  }, []);

  const getOptimisticUpdates = useCallback(() => {
    return Array.from(optimisticUpdates.current.values());
  }, []);

  return {
    state,
    setState,
    isOptimistic,
    applyOptimisticUpdate,
    rollbackUpdate,
    rollbackAll,
    confirmUpdate,
    getOptimisticUpdates,
    pendingCount: optimisticUpdates.current.size
  };
}

// Specialized hook for optimistic list operations
export function useOptimisticList<T extends { id: string | number }>(
  initialList: T[],
  options: OptimisticStateOptions = {}
) {
  const {
    state: list,
    setState: setList,
    isOptimistic,
    applyOptimisticUpdate,
    rollbackUpdate,
    rollbackAll,
    confirmUpdate,
    getOptimisticUpdates,
    pendingCount
  } = useOptimisticState(initialList, options);

  const add = useCallback(<Args extends unknown[]>(
    item: T,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      (currentList) => [...currentList, item],
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  const remove = useCallback(<Args extends unknown[]>(
    itemId: string | number,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      (currentList) => currentList.filter(item => item.id !== itemId),
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  const update = useCallback(<Args extends unknown[]>(
    itemId: string | number,
    updates: Partial<T>,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      (currentList) => currentList.map(item =>
        item.id === itemId ? { ...item, ...updates } : item
      ),
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  const move = useCallback(<Args extends unknown[]>(
    fromIndex: number,
    toIndex: number,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      (currentList) => {
        const newList = [...currentList];
        const [removed] = newList.splice(fromIndex, 1);
        newList.splice(toIndex, 0, removed);
        return newList;
      },
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  const replace = useCallback(<Args extends unknown[]>(
    newList: T[],
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      () => newList,
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  return {
    list,
    setList,
    isOptimistic,
    pendingCount,
    add,
    remove,
    update,
    move,
    replace,
    rollbackUpdate,
    rollbackAll,
    confirmUpdate,
    getOptimisticUpdates
  };
}

// Hook for optimistic form state
export function useOptimisticForm<T extends Record<string, unknown>>(
  initialValues: T,
  options: OptimisticStateOptions = {}
) {
  const {
    state: values,
    setState: setValues,
    isOptimistic,
    applyOptimisticUpdate,
    rollbackUpdate,
    rollbackAll,
    confirmUpdate,
    getOptimisticUpdates,
    pendingCount
  } = useOptimisticState(initialValues, options);

  const updateField = useCallback(<Args extends unknown[]>(
    field: keyof T,
    value: unknown,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      (currentValues) => ({ ...currentValues, [field]: value }),
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  const updateFields = useCallback(<Args extends unknown[]>(
    fields: Partial<T>,
    asyncAction: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    return applyOptimisticUpdate(
      (currentValues) => ({ ...currentValues, ...fields }),
      asyncAction,
      ...args
    );
  }, [applyOptimisticUpdate]);

  const reset = useCallback(<Args extends unknown[]>(
    asyncAction?: (...args: Args) => Promise<unknown>,
    ...args: Args
  ) => {
    if (asyncAction) {
      return applyOptimisticUpdate(
        () => initialValues,
        asyncAction,
        ...args
      );
    } else {
      setValues(initialValues);
      return Promise.resolve();
    }
  }, [applyOptimisticUpdate, setValues, initialValues]);

  return {
    values,
    setValues,
    isOptimistic,
    pendingCount,
    updateField,
    updateFields,
    reset,
    rollbackUpdate,
    rollbackAll,
    confirmUpdate,
    getOptimisticUpdates
  };
}
