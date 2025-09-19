'use client';

import {
  analyticsMiddleware,
  createAsyncState,
  loggingMiddleware,
  useContextStore,
} from '@/lib/state';
import {
  AppState,
  CacheState,
  NotificationState,
  AnalyticsState,
  UIState,
  UserState,
  StateAction,
  UserProfile,
  UserSubscription,
  StockRanking,
  StockItem,
  Notification,
} from '@/lib/state/types';
import type { AnalyticsFilters } from '@/types/analytics';
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
        analyticsAlerts: true,
        priceAlerts: true,
      },
      analytics: {
        defaultView: 'table',
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

const initialAnalyticsState: AnalyticsState = {
  ...createAsyncState(),
  data: {
    rankings: [],
    filters: {},
    recentSearches: [],
    bookmarks: [],
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
      analyticsAlerts: true,
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
  analytics: initialAnalyticsState,
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
      openModal: (key: string, data?: unknown) => void;
      closeModal: (key: string) => void;
      addToast: (toast: Omit<UIState['toasts'][0], 'id' | 'timestamp'>) => void;
      removeToast: (id: string) => void;
      setLoading: (key: string | null, loading: boolean) => void;
      setResponsive: (responsive: Partial<UIState['responsive']>) => void;
    };
    user: {
      setProfile: (profile: UserProfile | null) => void;
      updatePreferences: (
        preferences: Partial<UserState['data']['preferences']>
      ) => void;
      setSubscription: (subscription: UserSubscription | null) => void;
      updatePermissions: (permissions: string[]) => void;
      setPackageTier: (tier: string) => void;
    };
    analytics: {
      setRankings: (rankings: StockRanking[]) => void;
      setFilters: (filters: AnalyticsFilters) => void;
      addBookmark: (symbol: string) => void;
      removeBookmark: (symbol: string) => void;
      addRecentSearch: (symbol: string) => void;
      setRealtimeStatus: (status: {
        connected: boolean;
        subscriptions?: string[];
      }) => void;
    };
    notifications: {
      setNotifications: (notifications: Notification[]) => void;
      addNotification: (notification: Notification) => void;
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
      setStockData: (symbol: string, data: StockItem, ttl?: number) => void;
      setRankingsData: (key: string, data: StockRanking[], ttl?: number) => void;
      setAnalyticsData: (key: string, data: unknown, ttl?: number) => void;
      clearExpired: () => void;
      clearAll: () => void;
    };
  };
}

const AppStateContext = createContext<AppStateContextType | undefined>(
  undefined
);

// Reducers
function appStateReducer(state: AppState, action: StateAction): AppState {
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

    // Analytics Actions
    case 'SET_RANKINGS':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: state.analytics.data
            ? {
                ...state.analytics.data,
                rankings: action.payload,
              }
            : null,
        },
      };

    case 'SET_FILTERS':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: state.analytics.data
            ? {
                ...state.analytics.data,
                filters: action.payload,
              }
            : null,
        },
      };

    case 'ADD_BOOKMARK':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: state.analytics.data
            ? {
                ...state.analytics.data,
                bookmarks: [...state.analytics.data.bookmarks, action.payload],
              }
            : null,
        },
      };

    case 'REMOVE_BOOKMARK':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: state.analytics.data
            ? {
                ...state.analytics.data,
                bookmarks: state.analytics.data.bookmarks.filter(
                  symbol => symbol !== action.payload
                ),
              }
            : null,
        },
      };

    case 'ADD_RECENT_SEARCH':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: state.analytics.data
            ? {
                ...state.analytics.data,
                recentSearches: [
                  action.payload,
                  ...state.analytics.data.recentSearches.filter(
                    search => search !== action.payload
                  ).slice(0, 9)
                ],
              }
            : null,
        },
      };

    case 'SET_ANALYTICS_REALTIME_STATUS':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          realtime: {
            ...state.analytics.realtime,
            ...action.payload,
            lastHeartbeat: action.payload.connected ? Date.now() : null,
          },
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

      openModal: (key: string, data?: unknown) =>
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
      setProfile: (profile: UserProfile | null) =>
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

      setSubscription: (subscription: UserSubscription | null) =>
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

  // Analytics Actions
  const analyticsActions = useMemo(
    () => ({
      setRankings: (rankings: StockRanking[]) =>
        dispatch(
          {
            type: 'SET_RANKINGS',
            payload: rankings,
            meta: { timestamp: Date.now(), source: 'analytics' },
          },
          appStateReducer
        ),

      setFilters: (filters: AnalyticsFilters) =>
        dispatch(
          {
            type: 'SET_FILTERS',
            payload: filters,
            meta: { timestamp: Date.now(), source: 'analytics' },
          },
          appStateReducer
        ),

      addBookmark: (symbol: string) =>
        dispatch(
          {
            type: 'ADD_BOOKMARK',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'analytics' },
          },
          appStateReducer
        ),

      removeBookmark: (symbol: string) =>
        dispatch(
          {
            type: 'REMOVE_BOOKMARK',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'analytics' },
          },
          appStateReducer
        ),

      addRecentSearch: (symbol: string) =>
        dispatch(
          {
            type: 'ADD_RECENT_SEARCH',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'analytics' },
          },
          appStateReducer
        ),

      setRealtimeStatus: (status: {
        connected: boolean;
        subscriptions?: string[];
      }) =>
        dispatch(
          {
            type: 'SET_ANALYTICS_REALTIME_STATUS',
            payload: status,
            meta: { timestamp: Date.now(), source: 'analytics' },
          },
          appStateReducer
        ),
    }),
    [dispatch]
  );

  // Notification Actions
  const notificationActions = useMemo(
    () => ({
      setNotifications: (notifications: Notification[]) =>
        dispatch(
          {
            type: 'SET_NOTIFICATIONS',
            payload: notifications,
            meta: { timestamp: Date.now(), source: 'notifications' },
          },
          appStateReducer
        ),

      addNotification: (notification: Notification) =>
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
      setStockData: (symbol: string, data: StockItem, ttl?: number) =>
        dispatch(
          {
            type: 'SET_STOCK_DATA',
            payload: { symbol, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),

      setRankingsData: (key: string, data: StockRanking[], ttl?: number) =>
        dispatch(
          {
            type: 'SET_RANKINGS_DATA',
            payload: { key, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          },
          appStateReducer
        ),

      setAnalyticsData: (key: string, data: unknown, ttl?: number) =>
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
      analytics: analyticsActions,
      notifications: notificationActions,
      cache: cacheActions,
    }),
    [uiActions, userActions, analyticsActions, notificationActions, cacheActions]
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

export function useAnalyticsState() {
  const { state } = useAppState();
  return state.analytics;
}

export function useNotificationState() {
  const { state } = useAppState();
  return state.notifications;
}

export function useCacheState() {
  const { state } = useAppState();
  return state.cache;
}
