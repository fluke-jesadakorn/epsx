/**
 * Core State Management
 * Simplified state store with storage utilities and middleware
 */

import { useCallback, useEffect, useRef, useState } from 'react';

// ============================================================================
// Types
// ============================================================================

export interface StateConfig {
  persist?: {
    key: string;
    storage: 'localStorage' | 'sessionStorage';
    version?: number;
    migrate?: <T>(state: T, version: number) => T;
  };
  middleware?: StateMiddleware[];
  devtools?: boolean;
}

export type StateMiddleware = <T>(
  action: StateAction,
  prevState: T,
  nextState: T,
  store: string
) => void;

export interface StateAction<T = unknown> {
  type: string;
  payload?: T;
  meta?: {
    timestamp: number;
    source: string;
    optimistic?: boolean;
  };
}

export interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
  optimisticUpdates: T[];
}

export interface AppState {
  ui: {
    theme: 'light' | 'dark' | 'system';
    sidebar: {
      open: boolean;
      collapsed: boolean;
    };
    modals: Record<string, any>;
    toasts: any[];
    loading: Record<string, boolean>;
    errors: Record<string, string>;
  };
  user: AsyncState<any>;
  analytics: {
    data: {
      rankings: any[];
      metrics: any[];
    };
    filters: any;
    pagination: {
      page: number;
      per_page: number;
      total: number;
    };
  };
  notifications: AsyncState<any[]>;
  preferences: Record<string, any>;
}

// ============================================================================
// Storage Utilities
// ============================================================================

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
      console.warn('Failed to save to storage:', error instanceof Error ? error.message : error);
    }
  },
  
  remove: (key: string, storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') return;
    try {
      window[storageType].removeItem(key);
    } catch (error) {
      console.warn('Failed to remove from storage:', error instanceof Error ? error.message : error);
    }
  },

  clear: (storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') return;
    try {
      window[storageType].clear();
    } catch (error) {
      console.warn('Failed to clear storage:', error instanceof Error ? error.message : error);
    }
  }
};

// ============================================================================
// State Middleware
// ============================================================================

export const loggingMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  if (process.env.NODE_ENV !== 'development') return;

  // Filter out noisy actions
  const noisyActions = ['SET_LOADING', 'UPDATE_STOCK_PRICE', 'ADD_TOAST', 'REMOVE_TOAST'];
  if (noisyActions.includes(action.type)) return;

  const timestamp = new Date().toISOString();
  console.group(`%c${action.type}`, 'color: #2196F3; font-weight: bold;');
  console.log('Action:', action);
  console.log('Previous State:', prevState);
  console.log('Next State:', nextState);
  console.log('Timestamp:', timestamp);
  console.groupEnd();
};

export const persistenceMiddleware = (persistKey: string, storageType: 'localStorage' | 'sessionStorage' = 'localStorage'): StateMiddleware => {
  return (action, prevState, nextState, store) => {
    // Only persist certain action types
    const persistableActions = ['SET_THEME', 'UPDATE_PREFERENCES', 'SET_USER'];
    if (persistableActions.includes(action.type)) {
      storage.set(persistKey, nextState, storageType);
    }
  };
};

export const combineMiddleware = (...middlewares: StateMiddleware[]): StateMiddleware => {
  return (action, prevState, nextState, store) => {
    middlewares.forEach(middleware => {
      middleware(action, prevState, nextState, store);
    });
  };
};

// ============================================================================
// Core State Hook
// ============================================================================

export function useStateManager<T>(initialState: T, config: StateConfig = {}) {
  const [state, setState] = useState<T>(() => {
    // Try to load from persistence on initialization
    if (config.persist && typeof window !== 'undefined') {
      const persistedState = storage.get(config.persist.key, config.persist.storage);
      if (persistedState) {
        // Handle version migration if needed
        if (config.persist.migrate && config.persist.version) {
          return config.persist.migrate(persistedState, config.persist.version);
        }
        return persistedState;
      }
    }
    return initialState;
  });

  const middlewareRef = useRef(config.middleware || []);
  const storeNameRef = useRef(config.persist?.key || 'unnamed-store');

  const dispatch = useCallback((action: StateAction) => {
    setState(prevState => {
      const actionWithMeta: StateAction = {
        ...action,
        meta: {
          timestamp: Date.now(),
          source: 'dispatch',
          ...action.meta
        }
      };

      const nextState = stateReducer(prevState, actionWithMeta);

      // Apply middleware
      middlewareRef.current.forEach(middleware => {
        middleware(actionWithMeta, prevState, nextState, storeNameRef.current);
      });

      return nextState;
    });
  }, []);

  return { state, dispatch };
}

// ============================================================================
// Basic State Reducer
// ============================================================================

function stateReducer<T>(state: T, action: StateAction): T {
  switch (action.type) {
    case 'SET_STATE':
      // Ensure both state and payload are objects before spreading
      if (typeof state === 'object' && state !== null && 
          typeof action.payload === 'object' && action.payload !== null) {
        return { ...state, ...action.payload } as T;
      }
      return action.payload as T;
    
    case 'RESET_STATE':
      return action.payload as T;

    case 'UPDATE_NESTED':
      if (typeof action.payload === 'object' && action.payload && 'path' in action.payload && 'value' in action.payload) {
        const { path, value } = action.payload as { path: string; value: any };
        return updateNestedState(state, path, value);
      }
      return state;

    default:
      return state;
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

function updateNestedState<T>(state: T, path: string, value: any): T {
  const keys = path.split('.');
  const newState = { ...state } as any;
  
  let current = newState;
  for (let i = 0; i < keys.length - 1; i++) {
    const key = keys[i];
    current[key] = { ...current[key] };
    current = current[key];
  }
  
  current[keys[keys.length - 1]] = value;
  return newState;
}

// ============================================================================
// Server-Side State Utilities
// ============================================================================

export interface SSRStateOptions {
  includeAuth?: boolean;
  includeUserPreferences?: boolean;
  includeCache?: boolean;
  cacheKeys?: string[];
}

export async function getServerState(options: SSRStateOptions = {}): Promise<Partial<AppState>> {
  const {
    includeAuth = true,
    includeUserPreferences = true,
    includeCache = false,
    cacheKeys = []
  } = options;

  const serverState: Partial<AppState> = {
    ui: {
      theme: 'system',
      sidebar: {
        open: true,
        collapsed: false
      },
      modals: {},
      toasts: [],
      loading: {},
      errors: {}
    },
    user: {
      data: null,
      loading: false,
      error: null,
      lastUpdated: null,
      optimisticUpdates: []
    },
    analytics: {
      data: {
        rankings: [],
        metrics: []
      },
      filters: {},
      pagination: {
        page: 1,
        per_page: 50,
        total: 0
      }
    },
    notifications: {
      data: [],
      loading: false,
      error: null,
      lastUpdated: null,
      optimisticUpdates: []
    },
    preferences: {}
  };

  // Server-side state population would go here
  // This is a simplified version for the refactoring

  return serverState;
}

// ============================================================================
// State Validation
// ============================================================================

export const validateState = {
  ui: (state: any) => {
    return state && 
           typeof state.theme === 'string' &&
           typeof state.sidebar === 'object' &&
           Array.isArray(state.toasts);
  },
  
  user: (state: any) => {
    return state && 
           (state.data === null || typeof state.data === 'object') &&
           typeof state.loading === 'boolean' &&
           Array.isArray(state.optimisticUpdates);
  },
  
  analytics: (state: any) => {
    return state &&
           state.data &&
           Array.isArray(state.data.rankings) &&
           Array.isArray(state.data.metrics) &&
           typeof state.filters === 'object';
  }
};

// ============================================================================
// Action Creators
// ============================================================================

export const createActions = {
  ui: (dispatch: any) => ({
    setTheme: (theme: 'light' | 'dark' | 'system') => 
      dispatch({ type: 'SET_THEME', payload: theme }),
    addToast: (toast: any) => 
      dispatch({ type: 'ADD_TOAST', payload: toast }),
    openModal: (key: string, data?: any) => 
      dispatch({ type: 'OPEN_MODAL', payload: { key, data } })
  }),
  
  analytics: (dispatch: any) => ({
    addToAnalytics: (item: any) => 
      dispatch({ type: 'ADD_TO_ANALYTICS', payload: item }),
    updateMetrics: (id: string, data: any) => 
      dispatch({ type: 'UPDATE_METRICS', payload: { id, data } })
  }),
  
  notifications: (dispatch: any) => ({
    addNotification: (notification: any) => 
      dispatch({ type: 'ADD_NOTIFICATION', payload: notification }),
    markRead: (id: string) => 
      dispatch({ type: 'MARK_READ', payload: id })
  })
};