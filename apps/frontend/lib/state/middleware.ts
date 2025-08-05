import { StateAction, StateMiddleware } from './types';

// Advanced logging middleware with filtering and formatting
export const advancedLoggingMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  if (process.env.NODE_ENV !== 'development') return;

  // Filter out noisy actions
  const noisyActions = ['SET_LOADING', 'UPDATE_STOCK_PRICE', 'ADD_TOAST', 'REMOVE_TOAST'];
  if (noisyActions.includes(action.type)) return;

  const timestamp = new Date().toISOString();
  const duration = action.meta?.timestamp ? Date.now() - action.meta.timestamp : 0;

  // State change debug logging removed for production
};

function getActionColor(actionType: string): string {
  if (actionType.startsWith('SET_')) return '#4CAF50';  // Green
  if (actionType.startsWith('UPDATE_')) return '#FF9800';  // Orange
  if (actionType.startsWith('ADD_')) return '#2196F3';  // Blue
  if (actionType.startsWith('REMOVE_')) return '#F44336';  // Red
  if (actionType.startsWith('TOGGLE_')) return '#9C27B0';  // Purple
  return '#757575';  // Gray
}

function getStateDiff(prev: any, next: any): Record<string, any> {
  const diff: Record<string, any> = {};
  
  function findDifferences(prevObj: any, nextObj: any, path = '') {
    if (prevObj === nextObj) return;
    
    if (typeof prevObj !== 'object' || typeof nextObj !== 'object' || 
        prevObj === null || nextObj === null) {
      diff[path || 'root'] = { from: prevObj, to: nextObj };
      return;
    }
    
    const allKeys = new Set([...Object.keys(prevObj), ...Object.keys(nextObj)]);
    
    for (const key of allKeys) {
      const currentPath = path ? `${path}.${key}` : key;
      const prevValue = prevObj[key];
      const nextValue = nextObj[key];
      
      if (prevValue !== nextValue) {
        if (typeof prevValue === 'object' && typeof nextValue === 'object' &&
            prevValue !== null && nextValue !== null) {
          findDifferences(prevValue, nextValue, currentPath);
        } else {
          diff[currentPath] = { from: prevValue, to: nextValue };
        }
      }
    }
  }
  
  findDifferences(prev, next);
  return diff;
}

// Analytics middleware for production
export const advancedAnalyticsMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  // Only track important user actions
  const trackableActions = [
    'ADD_TO_WATCHLIST', 'REMOVE_FROM_WATCHLIST', 'ADD_PRICE_ALERT',
    'SET_THEME', 'UPDATE_USER_PREFERENCES', 'MARK_NOTIFICATION_READ'
  ];
  
  if (!trackableActions.includes(action.type)) return;

  // Send to analytics service
  if (typeof window !== 'undefined') {
    // Google Analytics 4
    if (window.gtag) {
      window.gtag('event', 'state_change', {
        event_category: 'state_management',
        event_label: `${store}:${action.type}`,
        store_name: store,
        action_type: action.type,
        timestamp: action.meta?.timestamp || Date.now(),
        custom_parameter_1: action.payload ? JSON.stringify(action.payload).substring(0, 100) : undefined
      });
    }

    // Custom analytics service
    if (window.analytics) {
      window.analytics.track('State Change', {
        store,
        actionType: action.type,
        payload: action.payload,
        timestamp: action.meta?.timestamp || Date.now(),
        source: action.meta?.source || 'unknown',
        optimistic: action.meta?.optimistic || false
      });
    }

    // Send to backend for user behavior analysis
    fetch('/api/analytics/state-change', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({
        store,
        actionType: action.type,
        timestamp: action.meta?.timestamp || Date.now(),
        source: action.meta?.source,
        optimistic: action.meta?.optimistic,
        // Don't send sensitive payload data
        hasPayload: !!action.payload
      })
    }).catch(() => {
      // Silently fail - analytics shouldn't break the app
    });
  }
};

// Performance monitoring middleware
export const performanceMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  if (process.env.NODE_ENV !== 'development') return;

  const startTime = action.meta?.timestamp || Date.now();
  const endTime = Date.now();
  const duration = endTime - startTime;

  // Warn about slow state changes
  if (duration > 100) {
    console.warn(`Slow state change detected: ${store}:${action.type}`, { duration, store, actionType: action.type });
  }

  // Calculate state size
  const stateSize = JSON.stringify(nextState).length;
  if (stateSize > 1000000) { // 1MB
    console.warn(`Large state detected: ${store}`, { store, sizeBytes: stateSize, sizeMB: (stateSize / 1024 / 1024).toFixed(2) });
  }

  // Track action frequency
  if (!window.__EPSX_ACTION_FREQUENCY__) {
    window.__EPSX_ACTION_FREQUENCY__ = new Map();
  }
  
  const actionKey = `${store}:${action.type}`;
  const frequency = window.__EPSX_ACTION_FREQUENCY__.get(actionKey) || 0;
  window.__EPSX_ACTION_FREQUENCY__.set(actionKey, frequency + 1);

  // Warn about frequent actions
  if (frequency > 100 && frequency % 50 === 0) {
    console.warn(`Frequent action detected: ${actionKey}`, { actionKey, frequency });
  }
};

// Error handling middleware
export const errorHandlingMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  try {
    // Validate state structure
    if (typeof nextState !== 'object' || nextState === null) {
      throw new Error(`Invalid state: ${store} state must be an object`);
    }

    // Check for circular references
    JSON.stringify(nextState);

    // Validate required properties based on store
    switch (store) {
      case 'ui':
        if (!nextState.theme || !nextState.sidebar || !nextState.loading) {
          console.warn('UI state missing required properties', { store, nextState });
        }
        break;
      case 'user':
        if (nextState.data && !nextState.data.preferences) {
          console.warn('User state missing preferences', { store, nextState });
        }
        break;
      case 'trading':
        if (nextState.data && (!Array.isArray(nextState.data.watchlist) || !Array.isArray(nextState.data.portfolio))) {
          console.warn('Trading state has invalid data structure', { store, nextState });
        }
        break;
    }
  } catch (error) {
    console.error(`State validation error in ${store}:${action.type}`, { 
      error: error instanceof Error ? error.message : error,
      store,
      actionType: action.type,
      actionPayload: action.payload
    });
    
    // Send error to monitoring service
    if (typeof window !== 'undefined' && window.Sentry) {
      window.Sentry.captureException(error, {
        tags: {
          store,
          actionType: action.type
        },
        extra: {
          action,
          prevState,
          nextState
        }
      });
    }
  }
};

// Persistence middleware for automatic state saving
export const persistenceMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  // Only persist certain stores
  const persistableStores = ['user', 'ui'];
  if (!persistableStores.includes(store)) return;

  // Debounce persistence to avoid excessive writes
  if (!window.__EPSX_PERSISTENCE_TIMEOUTS__) {
    window.__EPSX_PERSISTENCE_TIMEOUTS__ = new Map();
  }

  const timeoutId = window.__EPSX_PERSISTENCE_TIMEOUTS__.get(store);
  if (timeoutId) {
    clearTimeout(timeoutId);
  }

  window.__EPSX_PERSISTENCE_TIMEOUTS__.set(store, setTimeout(() => {
    try {
      localStorage.setItem(`epsx-${store}-state`, JSON.stringify({
        ...nextState,
        _version: 1,
        _timestamp: Date.now()
      }));
    } catch (error) {
      console.warn(`Failed to persist ${store} state`, { error: error instanceof Error ? error.message : error, store });
    }
  }, 1000));
};

// Undo/Redo middleware
export const undoRedoMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  // Skip certain actions from undo history
  const skipActions = ['SET_LOADING', 'ADD_TOAST', 'REMOVE_TOAST', 'UPDATE_STOCK_PRICE'];
  if (skipActions.includes(action.type)) return;

  if (!window.__EPSX_UNDO_HISTORY__) {
    window.__EPSX_UNDO_HISTORY__ = new Map();
  }

  const history = window.__EPSX_UNDO_HISTORY__.get(store) || [];
  
  // Add current state to history
  history.push({
    state: prevState,
    action,
    timestamp: Date.now()
  });

  // Limit history size
  if (history.length > 50) {
    history.shift();
  }

  window.__EPSX_UNDO_HISTORY__.set(store, history);
};

// Development tools middleware
export const devToolsMiddleware: StateMiddleware = (action, prevState, nextState, store) => {
  if (typeof window === 'undefined' || process.env.NODE_ENV !== 'development') return;

  // Redux DevTools Extension support
  if (window.__REDUX_DEVTOOLS_EXTENSION__) {
    const devTools = window.__REDUX_DEVTOOLS_EXTENSION__.connect({
      name: `EPSX-${store}`,
      trace: true,
      traceLimit: 25
    });

    devTools.send(action, nextState);
  }

  // Custom dev tools
  if (!window.__EPSX_DEV_TOOLS__) {
    window.__EPSX_DEV_TOOLS__ = {
      stores: new Map(),
      actions: [],
      getStore: (storeName: string) => window.__EPSX_DEV_TOOLS__.stores.get(storeName),
      getActions: () => window.__EPSX_DEV_TOOLS__.actions,
      clearHistory: () => {
        window.__EPSX_DEV_TOOLS__.actions = [];
        window.__EPSX_DEV_TOOLS__.stores.clear();
      }
    };
  }

  window.__EPSX_DEV_TOOLS__.stores.set(store, nextState);
  window.__EPSX_DEV_TOOLS__.actions.push({
    store,
    action,
    timestamp: Date.now(),
    prevState,
    nextState
  });

  // Limit action history
  if (window.__EPSX_DEV_TOOLS__.actions.length > 1000) {
    window.__EPSX_DEV_TOOLS__.actions = window.__EPSX_DEV_TOOLS__.actions.slice(-500);
  }
};

// Combine multiple middlewares
export function combineMiddleware(...middlewares: StateMiddleware[]): StateMiddleware {
  return (action, prevState, nextState, store) => {
    middlewares.forEach(middleware => {
      try {
        middleware(action, prevState, nextState, store);
      } catch (error) {
        console.error('Middleware error', { error: error instanceof Error ? error.message : error });
      }
    });
  };
}

// Default middleware stack for development
export const developmentMiddleware = combineMiddleware(
  advancedLoggingMiddleware,
  performanceMiddleware,
  errorHandlingMiddleware,
  undoRedoMiddleware,
  devToolsMiddleware
);

// Default middleware stack for production
export const productionMiddleware = combineMiddleware(
  advancedAnalyticsMiddleware,
  errorHandlingMiddleware,
  persistenceMiddleware
);

// Conditional middleware based on environment
export const defaultMiddleware = process.env.NODE_ENV === 'development' 
  ? developmentMiddleware 
  : productionMiddleware;

// Type augmentation for window object
declare global {
  interface Window {
    __EPSX_ACTION_FREQUENCY__?: Map<string, number>;
    __EPSX_PERSISTENCE_TIMEOUTS__?: Map<string, NodeJS.Timeout>;
    __EPSX_UNDO_HISTORY__?: Map<string, any[]>;
    __EPSX_DEV_TOOLS__?: {
      stores: Map<string, any>;
      actions: any[];
      getStore: (storeName: string) => any;
      getActions: () => any[];
      clearHistory: () => void;
    };
    __REDUX_DEVTOOLS_EXTENSION__?: any;
    gtag?: (...args: any[]) => void;
    analytics?: {
      track: (event: string, properties: any) => void;
    };
    Sentry?: {
      captureException: (error: Error, options?: any) => void;
    };
  }
}