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

const initialAnalyticsState = {
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
    lastHeartbeat: undefined,
  },
} as AnalyticsState;

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
        preferences: Partial<Record<string, any>>
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
        preferences: Partial<Record<string, any>>
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
  const payload = action.payload as any; // Cast to any for flexibility
  const userState = state.user as any; // Cast for AsyncState access
  const analyticsState = state.analytics as any; // Cast for AsyncState access
  const notificationState = state.notifications as any; // Cast for AsyncState access
  switch (action.type) {
    // UI Actions
    case 'SET_THEME':
      return {
        ...state,
        ui: { ...state.ui, theme: payload },
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
          sidebar: { ...state.ui.sidebar, collapsed: payload },
        },
      };

    case 'OPEN_MODAL':
      return {
        ...state,
        ui: {
          ...state.ui,
          modals: {
            ...state.ui.modals,
            [payload.key]: {
              open: true,
              data: payload.data,
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
            [payload]: { open: false },
          },
        },
      };

    case 'ADD_TOAST':
      const newToast = {
        ...payload,
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
          toasts: state.ui.toasts.filter(toast => toast.id !== payload),
        },
      };

    case 'SET_LOADING':
      return {
        ...state,
        ui: {
          ...state.ui,
          loading: payload.key
            ? {
                ...state.ui.loading,
                requests: {
                  ...state.ui.loading.requests,
                  [payload.key]: payload.loading,
                },
              }
            : {
                ...state.ui.loading,
                global: payload.loading,
              },
        },
      };

    case 'SET_RESPONSIVE':
      return {
        ...state,
        ui: {
          ...state.ui,
          responsive: { ...state.ui.responsive, ...payload },
        },
      };

    // User Actions
    case 'SET_USER_PROFILE':
      return {
        ...state,
        user: {
          ...state.user,
          data: userState.data
            ? {
                ...userState.data,
                profile: payload,
              }
            : null,
        },
      };

    case 'UPDATE_USER_PREFERENCES':
      return {
        ...state,
        user: {
          ...state.user,
          data: userState.data
            ? {
                ...userState.data,
                preferences: {
                  ...userState.data.preferences,
                  ...payload,
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
          data: analyticsState.data
            ? {
                ...analyticsState.data,
                rankings: payload,
              }
            : null,
        } as AnalyticsState,
      };

    case 'SET_FILTERS':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: analyticsState.data
            ? {
                ...analyticsState.data,
                filters: payload,
              }
            : null,
        } as AnalyticsState,
      };

    case 'ADD_BOOKMARK':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: analyticsState.data
            ? {
                ...analyticsState.data,
                bookmarks: [...analyticsState.data.bookmarks, payload],
              }
            : null,
        } as AnalyticsState,
      };

    case 'REMOVE_BOOKMARK':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: analyticsState.data
            ? {
                ...analyticsState.data,
                bookmarks: analyticsState.data.bookmarks.filter(
                  (symbol: any) => symbol !== payload
                ),
              }
            : null,
        } as AnalyticsState,
      };

    case 'ADD_RECENT_SEARCH':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          data: analyticsState.data
            ? {
                ...analyticsState.data,
                recentSearches: [
                  payload,
                  ...analyticsState.data.recentSearches.filter(
                    (search: any) => search !== payload
                  ).slice(0, 9)
                ],
              }
            : null,
        } as AnalyticsState,
      };

    case 'SET_ANALYTICS_REALTIME_STATUS':
      return {
        ...state,
        analytics: {
          ...state.analytics,
          realtime: {
            ...state.analytics.realtime,
            ...payload,
            lastHeartbeat: payload.connected ? Date.now() : null,
          },
        } as AnalyticsState,
      };

    // Cache Actions
    case 'SET_STOCK_DATA':
      return {
        ...state,
        cache: {
          ...state.cache,
          stockData: {
            ...state.cache.stockData,
            [payload.symbol]: {
              data: payload.data,
              timestamp: Date.now(),
              ttl: payload.ttl || 300000, // 5 minutes default
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
        dispatch({
          type: 'SET_THEME',
          payload: theme,
          meta: { timestamp: Date.now(), source: 'ui' },
        }),

      toggleSidebar: () =>
        dispatch({
          type: 'TOGGLE_SIDEBAR',
          meta: { timestamp: Date.now(), source: 'ui' },
        }),

      collapseSidebar: (collapsed: boolean) =>
        dispatch({
          type: 'COLLAPSE_SIDEBAR',
          payload: collapsed,
          meta: { timestamp: Date.now(), source: 'ui' },
        }),

      openModal: (key: string, data?: unknown) =>
        dispatch({
            type: 'OPEN_MODAL',
            payload: { key, data },
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      closeModal: (key: string) =>
        dispatch({
            type: 'CLOSE_MODAL',
            payload: key,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      addToast: (toast: Omit<UIState['toasts'][0], 'id' | 'timestamp'>) =>
        dispatch({
            type: 'ADD_TOAST',
            payload: toast,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      removeToast: (id: string) =>
        dispatch({
            type: 'REMOVE_TOAST',
            payload: id,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      setLoading: (key: string | null, loading: boolean) =>
        dispatch({
            type: 'SET_LOADING',
            payload: { key, loading },
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      setResponsive: (responsive: Partial<UIState['responsive']>) =>
        dispatch({
            type: 'SET_RESPONSIVE',
            payload: responsive,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),
    }),
    [dispatch]
  );

  // User Actions
  const userActions = useMemo(
    () => ({
      setProfile: (profile: UserProfile | null) =>
        dispatch({
            type: 'SET_USER_PROFILE',
            payload: profile,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      updatePreferences: (
        preferences: Record<string, any>
      ) =>
        dispatch({
            type: 'UPDATE_USER_PREFERENCES',
            payload: preferences,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      setSubscription: (subscription: UserSubscription | null) =>
        dispatch({
            type: 'SET_USER_SUBSCRIPTION',
            payload: subscription,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      updatePermissions: (permissions: string[]) =>
        dispatch({
            type: 'UPDATE_USER_PERMISSIONS',
            payload: permissions,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      setPackageTier: (tier: string) =>
        dispatch({
            type: 'SET_USER_PACKAGE_TIER',
            payload: tier,
            meta: { timestamp: Date.now(), source: 'user' },
          }),
    }),
    [dispatch]
  );

  // Analytics Actions
  const analyticsActions = useMemo(
    () => ({
      setRankings: (rankings: StockRanking[]) =>
        dispatch({
            type: 'SET_RANKINGS',
            payload: rankings,
            meta: { timestamp: Date.now(), source: 'analytics' },
          }),

      setFilters: (filters: AnalyticsFilters) =>
        dispatch({
            type: 'SET_FILTERS',
            payload: filters,
            meta: { timestamp: Date.now(), source: 'analytics' },
          }),

      addBookmark: (symbol: string) =>
        dispatch({
            type: 'ADD_BOOKMARK',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'analytics' },
          }),

      removeBookmark: (symbol: string) =>
        dispatch({
            type: 'REMOVE_BOOKMARK',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'analytics' },
          }),

      addRecentSearch: (symbol: string) =>
        dispatch({
            type: 'ADD_RECENT_SEARCH',
            payload: symbol,
            meta: { timestamp: Date.now(), source: 'analytics' },
          }),

      setRealtimeStatus: (status: {
        connected: boolean;
        subscriptions?: string[];
      }) =>
        dispatch({
            type: 'SET_ANALYTICS_REALTIME_STATUS',
            payload: status,
            meta: { timestamp: Date.now(), source: 'analytics' },
          }),
    }),
    [dispatch]
  );

  // Notification Actions
  const notificationActions = useMemo(
    () => ({
      setNotifications: (notifications: Notification[]) =>
        dispatch({
            type: 'SET_NOTIFICATIONS',
            payload: notifications,
            meta: { timestamp: Date.now(), source: 'notifications' },
          }),

      addNotification: (notification: Notification) =>
        dispatch({
            type: 'ADD_NOTIFICATION',
            payload: notification,
            meta: { timestamp: Date.now(), source: 'notifications' },
          }),

      markRead: (id: string) =>
        dispatch({
            type: 'MARK_NOTIFICATION_READ',
            payload: id,
            meta: { timestamp: Date.now(), source: 'notifications' },
          }),

      markAllRead: () =>
        dispatch({
            type: 'MARK_ALL_NOTIFICATIONS_READ',
            meta: { timestamp: Date.now(), source: 'notifications' },
          }),

      updatePreferences: (
        preferences: Record<string, any>
      ) =>
        dispatch({
            type: 'UPDATE_NOTIFICATION_PREFERENCES',
            payload: preferences,
            meta: { timestamp: Date.now(), source: 'notifications' },
          }),

      setRealtimeStatus: (status: { connected: boolean; lastSync?: number }) =>
        dispatch({
            type: 'SET_NOTIFICATION_REALTIME_STATUS',
            payload: status,
            meta: { timestamp: Date.now(), source: 'notifications' },
          }),
    }),
    [dispatch]
  );

  // Cache Actions
  const cacheActions = useMemo(
    () => ({
      setStockData: (symbol: string, data: StockItem, ttl?: number) =>
        dispatch({
            type: 'SET_STOCK_DATA',
            payload: { symbol, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          }),

      setRankingsData: (key: string, data: StockRanking[], ttl?: number) =>
        dispatch({
            type: 'SET_RANKINGS_DATA',
            payload: { key, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          }),

      setAnalyticsData: (key: string, data: unknown, ttl?: number) =>
        dispatch({
            type: 'SET_ANALYTICS_DATA',
            payload: { key, data, ttl },
            meta: { timestamp: Date.now(), source: 'cache' },
          }),

      clearExpired: () =>
        dispatch({
            type: 'CLEAR_EXPIRED_CACHE',
            meta: { timestamp: Date.now(), source: 'cache' },
          }),

      clearAll: () =>
        dispatch({
            type: 'CLEAR_ALL_CACHE',
            meta: { timestamp: Date.now(), source: 'cache' },
          }),
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
