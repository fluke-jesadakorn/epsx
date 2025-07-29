'use client';

import {
  analyticsMiddleware,
  createAsyncState,
  loggingMiddleware,
  useContextStore,
} from '@/lib/state/core';
import {
  AppState,
  CacheState,
  NotificationState,
  TradingState,
  UIState,
  UserState,
} from '@/lib/state/types';
import React, { createContext, useContext, useMemo } from 'react';

// Initial states
const initialUIState: UIState = {
  theme: 'system',
  sidebar: {
    open: true,
    collapsed: false,
  },
  modals: {},
  toasts: [],
  loading: {
    global: false,
    requests: {},
  },
  responsive: {
    isMobile: false,
    isTablet: false,
    breakpoint: 'lg',
  },
};

const initialUserState: UserState = {
  ...createAsyncState(),
  data: {
    profile: null,
    preferences: {
      language: 'en',
      timezone: 'UTC',
      currency: 'USD',
      notifications: {
        email: true,
        push: true,
        tradingAlerts: true,
        priceAlerts: true,
      },
      trading: {
        defaultView: 'grid',
        riskTolerance: 'medium',
        autoRefresh: true,
        refreshInterval: 30000,
      },
    },
    subscription: null,
    permissions: [],
    packageTier: 'FREE',
  },
  optimisticUpdates: [],
};

const initialTradingState: TradingState = {
  ...createAsyncState(),
  data: {
    watchlist: [],
    portfolio: [],
    rankings: [],
    alerts: [],
    recentSearches: [],
  },
  optimisticUpdates: [],
  realtime: {
    connected: false,
    subscriptions: [],
    lastHeartbeat: null,
  },
};

const initialNotificationState: NotificationState = {
  ...createAsyncState(),
  data: {
    unreadCount: 0,
    notifications: [],
    preferences: {
      email: true,
      push: true,
      inApp: true,
      tradingAlerts: true,
      systemUpdates: true,
    },
  },
  realtime: {
    connected: false,
    lastSync: null,
  },
};

const initialCacheState: CacheState = {
  stockData: {},
  rankings: {},
  analytics: {},
};

const initialAppState: AppState = {
  ui: initialUIState,
  user: initialUserState,
  trading: initialTradingState,
  notifications: initialNotificationState,
  cache: initialCacheState,
};

// Context
interface AppStateContextType {
  state: AppState;
  actions: {
    ui: {
      setTheme: (theme: UIState['theme']) => void;
      toggleSidebar: () => void;
      collapseSidebar: (collapsed: boolean) => void;
      openModal: (key: string, data?: any) => void;
      closeModal: (key: string) => void;
      addToast: (toast: Omit<UIState['toasts'][0], 'id' | 'timestamp'>) => void;
      removeToast: (id: string) => void;
      setLoading: (key: string | null, loading: boolean) => void;
      setResponsive: (responsive: Partial<UIState['responsive']>) => void;
    };
    user: {
      setProfile: (profile: any) => void;
      updatePreferences: (
        preferences: Partial<UserState['data']['preferences']>
      ) => void;
      setSubscription: (subscription: any) => void;
      updatePermissions: (permissions: string[]) => void;
      setPackageTier: (tier: string) => void;
    };
    trading: {
      setWatchlist: (watchlist: any[]) => void;
      addToWatchlist: (item: any) => void;
      removeFromWatchlist: (symbol: string) => void;
      updateStockPrice: (symbol: string, price: number, change: number) => void;
      setPortfolio: (portfolio: any[]) => void;
      setRankings: (rankings: any[]) => void;
      addPriceAlert: (alert: any) => void;
      removePriceAlert: (id: string) => void;
      addRecentSearch: (symbol: string) => void;
      setRealtimeStatus: (status: {
        connected: boolean;
        subscriptions?: string[];
      }) => void;
    };
    notifications: {
      setNotifications: (notifications: any[]) => void;
      addNotification: (notification: any) => void;
      markRead: (id: string) => void;
      markAllRead: () => void;
      updatePreferences: (
        preferences: Partial<NotificationState['data']['preferences']>
      ) => void;
      setRealtimeStatus: (status: {
        connected: boolean;
        lastSync?: number;
      }) => void;
    };
    cache: {
      setStockData: (symbol: string, data: any, ttl?: number) => void;
      setRankingsData: (key: string, data: any[], ttl?: number) => void;
      setAnalyticsData: (key: string, data: any, ttl?: number) => void;
      clearExpired: () => void;
      clearAll: () => void;
    };
  };
}

const AppStateContext = createContext<AppStateContextType | undefined>(
  undefined
);

// Reducers
function appStateReducer(state: AppState, action: any): AppState {
  switch (action.type) {
    // UI Actions
    case 'SET_THEME':
      return {
        ...state,
        ui: { ...state.ui, theme: action.payload },
      };

    case 'TOGGLE_SIDEBAR':
      return {
        ...state,
        ui: {
          ...state.ui,
          sidebar: { ...state.ui.sidebar, open: !state.ui.sidebar.open },
        },
      };

    case 'COLLAPSE_SIDEBAR':
      return {
        ...state,
        ui: {
          ...state.ui,
          sidebar: { ...state.ui.sidebar, collapsed: action.payload },
        },
      };

    case 'OPEN_MODAL':
      return {
        ...state,
        ui: {
          ...state.ui,
          modals: {
            ...state.ui.modals,
            [action.payload.key]: {
              open: true,
              data: action.payload.data,
            },
          },
        },
      };

    case 'CLOSE_MODAL':
      return {
        ...state,
        ui: {
          ...state.ui,
          modals: {
            ...state.ui.modals,
            [action.payload]: { open: false },
          },
        },
      };

    case 'ADD_TOAST':
      const newToast = {
        ...action.payload,
        id: Math.random().toString(36).substr(2, 9),
        timestamp: Date.now(),
      };
      return {
        ...state,
        ui: {
          ...state.ui,
          toasts: [...state.ui.toasts, newToast],
        },
      };

    case 'REMOVE_TOAST':
      return {
        ...state,
        ui: {
          ...state.ui,
          toasts: state.ui.toasts.filter(toast => toast.id !== action.payload),
        },
      };

    case 'SET_LOADING':
      return {
        ...state,
        ui: {
          ...state.ui,
          loading: action.payload.key
            ? {
                ...state.ui.loading,
                requests: {
                  ...state.ui.loading.requests,
                  [action.payload.key]: action.payload.loading,
                },
              }
            : {
                ...state.ui.loading,
                global: action.payload.loading,
              },
        },
      };

    case 'SET_RESPONSIVE':
      return {
        ...state,
        ui: {
          ...state.ui,
          responsive: { ...state.ui.responsive, ...action.payload },
        },
      };

    // User Actions
    case 'SET_USER_PROFILE':
      return {
        ...state,
        user: {
          ...state.user,
          data: state.user.data
            ? {
                ...state.user.data,
                profile: action.payload,
              }
            : null,
        },
      };

    case 'UPDATE_USER_PREFERENCES':
      return {
        ...state,
        user: {
          ...state.user,
          data: state.user.data
            ? {
                ...state.user.data,
                preferences: {
                  ...state.user.data.preferences,
                  ...action.payload,
                },
              }
            : null,
        },
      };

    // Trading Actions
    case 'SET_WATCHLIST':
      return {
        ...state,
        trading: {
          ...state.trading,
          data: state.trading.data
            ? {
                ...state.trading.data,
                watchlist: action.payload,
              }
            : null,
        },
      };

    case 'ADD_TO_WATCHLIST':
      return {
        ...state,
        trading: {
          ...state.trading,
          data: state.trading.data
            ? {
                ...state.trading.data,
                watchlist: [...state.trading.data.watchlist, action.payload],
              }
            : null,
        },
      };

    case 'REMOVE_FROM_WATCHLIST':
      return {
        ...state,
        trading: {
          ...state.trading,
          data: state.trading.data
            ? {
                ...state.trading.data,
                watchlist: state.trading.data.watchlist.filter(
                  item => item.symbol !== action.payload
                ),
              }
            : null,
        },
      };

    // Cache Actions
    case 'SET_STOCK_DATA':
      return {
        ...state,
        cache: {
          ...state.cache,
          stockData: {
            ...state.cache.stockData,
            [action.payload.symbol]: {
              data: action.payload.data,
              timestamp: Date.now(),
              ttl: action.payload.ttl || 300000, // 5 minutes default
            },
          },
        },
      };

    case 'CLEAR_EXPIRED_CACHE':
      const now = Date.now();
      const stockData = Object.fromEntries(
        Object.entries(state.cache.stockData).filter(
          ([_, item]) => now - item.timestamp < item.ttl
        )
      );
      const rankings = Object.fromEntries(
        Object.entries(state.cache.rankings).filter(
          ([_, item]) => now - item.timestamp < item.ttl
        )
      );
      const analytics = Object.fromEntries(
        Object.entries(state.cache.analytics).filter(
          ([_, item]) => now - item.timestamp < item.ttl
        )
      );
      return {
        ...state,
        cache: { stockData, rankings, analytics },
      };

    default:
      return state;
  }
}

// Provider Props
interface AppStateProviderProps {
  children: React.ReactNode;
  initialState?: Partial<AppState>;
}

export function AppStateProvider({
  children,
  initialState,
}: AppStateProviderProps) {
  const { state, dispatch } = useContextStore(
    { ...initialAppState, ...initialState },
    {
      persist: {
        key: 'epsx-app-state',
        storage: 'localStorage',
        version: 1,
      },
      middleware: [loggingMiddleware, analyticsMiddleware],
      devtools: true,
    }
  );

  // UI Actions
  const uiActions = useMemo(
    () => ({
      setTheme: (theme: UIState['theme']) =>
        dispatch(
          {
            type: 'SET_THEME',
            payload: theme,
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      toggleSidebar: () =>
        dispatch(
          {
            type: 'TOGGLE_SIDEBAR',
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      collapseSidebar: (collapsed: boolean) =>
        dispatch(
          {
            type: 'COLLAPSE_SIDEBAR',
            payload: collapsed,
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      openModal: (key: string, data?: any) =>
        dispatch(
          {
            type: 'OPEN_MODAL',
            payload: { key, data },
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      closeModal: (key: string) =>
        dispatch(
          {
            type: 'CLOSE_MODAL',
            payload: key,
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      addToast: (toast: Omit<UIState['toasts'][0], 'id' | 'timestamp'>) =>
        dispatch(
          {
            type: 'ADD_TOAST',
            payload: toast,
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      removeToast: (id: string) =>
        dispatch(
          {
            type: 'REMOVE_TOAST',
            payload: id,
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      setLoading: (key: string | null, loading: boolean) =>
        dispatch(
          {
            type: 'SET_LOADING',
            payload: { key, loading },
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),

      setResponsive: (responsive: Partial<UIState['responsive']>) =>
        dispatch(
          {
            type: 'SET_RESPONSIVE',
            payload: responsive,
            meta: { timestamp: Date.now(), source: 'ui' },
          },
          appStateReducer
        ),
    }),
    [dispatch]
  );

  // User Actions
  const userActions = useMemo(
    () => ({
      setProfile: (profile: any) =>
        dispatch(
          {
            type: 'SET_USER_PROFILE',
            payload: profile,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          appStateReducer
        ),

      updatePreferences: (
        preferences: Partial<UserState['data']['preferences']>
      ) =>
        dispatch(
          {
            type: 'UPDATE_USER_PREFERENCES',
            payload: preferences,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          appStateReducer
        ),

      setSubscription: (subscription: any) =>
        dispatch(
          {
            type: 'SET_USER_SUBSCRIPTION',
            payload: subscription,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          appStateReducer
        ),

      updatePermissions: (permissions: string[]) =>
        dispatch(
          {
            type: 'UPDATE_USER_PERMISSIONS',
            payload: permissions,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          appStateReducer
        ),

      setPackageTier: (tier: string) =>
        dispatch(
          {
            type: 'SET_USER_PACKAGE_TIER',
            payload: tier,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          appStateReducer
        ),
    }),
    [dispatch]
  );

  // Trading Actions
  const tradingActions = useMemo(
    () => ({
      setWatchlist: (watchlist: any[]) =>
        dispatch(
          {
            type: 'SET_WATCHLIST',
            payload: watchlist,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      addToWatchlist: (item: any) =>
        dispatch(
          {
            type: 'ADD_TO_WATCHLIST',
            payload: item,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      removeFromWatchlist: (symbol: string) =>
        dispatch(
          {
            type: 'REMOVE_FROM_WATCHLIST',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      updateStockPrice: (symbol: string, price: number, change: number) =>
        dispatch(
          {
            type: 'UPDATE_STOCK_PRICE',
            payload: { symbol, price, change },
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      setPortfolio: (portfolio: any[]) =>
        dispatch(
          {
            type: 'SET_PORTFOLIO',
            payload: portfolio,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      setRankings: (rankings: any[]) =>
        dispatch(
          {
            type: 'SET_RANKINGS',
            payload: rankings,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      addPriceAlert: (alert: any) =>
        dispatch(
          {
            type: 'ADD_PRICE_ALERT',
            payload: alert,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      removePriceAlert: (id: string) =>
        dispatch(
          {
            type: 'REMOVE_PRICE_ALERT',
            payload: id,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      addRecentSearch: (symbol: string) =>
        dispatch(
          {
            type: 'ADD_RECENT_SEARCH',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),

      setRealtimeStatus: (status: {
        connected: boolean;
        subscriptions?: string[];
      }) =>
        dispatch(
          {
            type: 'SET_TRADING_REALTIME_STATUS',
            payload: status,
            meta: { timestamp: Date.now(), source: 'trading' },
          },
          appStateReducer
        ),
    }),
    [dispatch]
  );

  // Notification Actions
  const notificationActions = useMemo(
    () => ({
      setNotifications: (notifications: any[]) =>
        dispatch(
          {
            type: 'SET_NOTIFICATIONS',
            payload: notifications,
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),

      addNotification: (notification: any) =>
        dispatch(
          {
            type: 'ADD_NOTIFICATION',
            payload: notification,
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),

      markRead: (id: string) =>
        dispatch(
          {
            type: 'MARK_NOTIFICATION_READ',
            payload: id,
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),

      markAllRead: () =>
        dispatch(
          {
            type: 'MARK_ALL_NOTIFICATIONS_READ',
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),

      updatePreferences: (
        preferences: Partial<NotificationState['data']['preferences']>
      ) =>
        dispatch(
          {
            type: 'UPDATE_NOTIFICATION_PREFERENCES',
            payload: preferences,
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),

      setRealtimeStatus: (status: { connected: boolean; lastSync?: number }) =>
        dispatch(
          {
            type: 'SET_NOTIFICATION_REALTIME_STATUS',
            payload: status,
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),
    }),
    [dispatch]
  );

  // Cache Actions
  const cacheActions = useMemo(
    () => ({
      setStockData: (symbol: string, data: any, ttl?: number) =>
        dispatch(
          {
            type: 'SET_STOCK_DATA',
            payload: { symbol, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),

      setRankingsData: (key: string, data: any[], ttl?: number) =>
        dispatch(
          {
            type: 'SET_RANKINGS_DATA',
            payload: { key, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),

      setAnalyticsData: (key: string, data: any, ttl?: number) =>
        dispatch(
          {
            type: 'SET_ANALYTICS_DATA',
            payload: { key, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),

      clearExpired: () =>
        dispatch(
          {
            type: 'CLEAR_EXPIRED_CACHE',
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),

      clearAll: () =>
        dispatch(
          {
            type: 'CLEAR_ALL_CACHE',
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),
    }),
    [dispatch]
  );

  const actions = useMemo(
    () => ({
      ui: uiActions,
      user: userActions,
      trading: tradingActions,
      notifications: notificationActions,
      cache: cacheActions,
    }),
    [uiActions, userActions, tradingActions, notificationActions, cacheActions]
  );

  const contextValue = useMemo(
    () => ({
      state,
      actions,
    }),
    [state, actions]
  );

  return (
    <AppStateContext.Provider value={contextValue}>
      {children}
    </AppStateContext.Provider>
  );
}

// Custom hook to use app state
export function useAppState() {
  const context = useContext(AppStateContext);
  if (context === undefined) {
    throw new Error('useAppState must be used within an AppStateProvider');
  }
  return context;
}

// Selector hooks for performance
export function useUIState() {
  const { state } = useAppState();
  return state.ui;
}

export function useUserState() {
  const { state } = useAppState();
  return state.user;
}

export function useTradingState() {
  const { state } = useAppState();
  return state.trading;
}

export function useNotificationState() {
  const { state } = useAppState();
  return state.notifications;
}

export function useCacheState() {
  const { state } = useAppState();
  return state.cache;
}
