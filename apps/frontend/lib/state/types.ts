// Core state management types
export interface StateConfig {
  persist?: {
    key: string;
    storage: 'localStorage' | 'sessionStorage';
    version?: number;
    migrate?: (state: any, version: number) => any;
  };
  middleware?: StateMiddleware[];
  devtools?: boolean;
}

export type StateMiddleware = (
  action: StateAction,
  prevState: any,
  nextState: any,
  store: string
) => void;

export interface StateAction {
  type: string;
  payload?: any;
  meta?: {
    timestamp: number;
    source: string;
    optimistic?: boolean;
  };
}

export interface AsyncState<T = any> {
  data: T | null;
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
}

export interface OptimisticUpdate<T = any> {
  id: string;
  action: StateAction;
  rollback: () => void;
  confirm: () => void;
  data: T;
  timestamp: number;
}

// Global App State
export interface AppState {
  ui: UIState;
  user: UserState;
  analytics: AnalyticsState;
  notifications: NotificationState;
  cache: CacheState;
}

// UI State
export interface UIState {
  theme: 'light' | 'dark' | 'system';
  sidebar: {
    open: boolean;
    collapsed: boolean;
  };
  modals: {
    [key: string]: {
      open: boolean;
      data?: any;
    };
  };
  toasts: Toast[];
  loading: {
    global: boolean;
    requests: Record<string, boolean>;
  };
  responsive: {
    isMobile: boolean;
    isTablet: boolean;
    breakpoint: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  };
}

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  description?: string;
  duration?: number;
  timestamp: number;
}

// User State (extends existing auth context)
export interface UserState extends AsyncState {
  data: {
    profile: UserProfile | null;
    preferences: UserPreferences;
    subscription: UserSubscription | null;
    permissions: string[];
    packageTier: string;
  } | null;
  optimisticUpdates: OptimisticUpdate[];
}

export interface UserProfile {
  id: string;
  email: string;
  displayName?: string;
  photoURL?: string;
  emailVerified: boolean;
  createdAt: string;
  lastSignInAt: string;
  role: string;
}

export interface UserPreferences {
  language: string;
  timezone: string;
  currency: string;
  notifications: {
    email: boolean;
    push: boolean;
    tradingAlerts: boolean;
    priceAlerts: boolean;
  };
  trading: {
    defaultView: 'grid' | 'list' | 'chart';
    riskTolerance: 'low' | 'medium' | 'high';
    autoRefresh: boolean;
    refreshInterval: number;
  };
}

export interface UserSubscription {
  tier: string;
  validUntil: string;
  isActive: boolean;
  features: string[];
}

// Analytics State
export interface AnalyticsState extends AsyncState {
  data: {
    rankings: StockRanking[];
    filters: any;
    recentSearches: string[];
    bookmarks: string[];
  } | null;
  optimisticUpdates: OptimisticUpdate[];
  realtime: {
    connected: boolean;
    subscriptions: string[];
    lastHeartbeat: number | null;
  };
}

export interface StockItem {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  volume: number;
  marketCap: number;
  lastUpdated: number;
}

export interface PortfolioItem extends StockItem {
  shares: number;
  avgCost: number;
  totalValue: number;
  unrealizedGain: number;
  unrealizedGainPercent: number;
}

export interface StockRanking {
  rank: number;
  symbol: string;
  name: string;
  epsGrowth: number;
  score: number;
  sector: string;
}

export interface PriceAlert {
  id: string;
  symbol: string;
  type: 'above' | 'below' | 'change';
  targetPrice?: number;
  changePercent?: number;
  isActive: boolean;
  createdAt: string;
}

// Notification State
export interface NotificationState extends AsyncState {
  data: {
    unreadCount: number;
    notifications: Notification[];
    preferences: NotificationPreferences;
  } | null;
  realtime: {
    connected: boolean;
    lastSync: number | null;
  };
}

export interface Notification {
  id: string;
  type: 'trading' | 'system' | 'account' | 'price_alert';
  title: string;
  message: string;
  read: boolean;
  actionUrl?: string;
  createdAt: string;
  metadata?: Record<string, any>;
}

export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  inApp: boolean;
  tradingAlerts: boolean;
  systemUpdates: boolean;
}

// Cache State
export interface CacheState {
  stockData: Record<string, {
    data: StockItem;
    timestamp: number;
    ttl: number;
  }>;
  rankings: Record<string, {
    data: StockRanking[];
    timestamp: number;
    ttl: number;
  }>;
  analytics: Record<string, {
    data: any;
    timestamp: number;
    ttl: number;
  }>;
}

// Action types
export type UIAction = 
  | { type: 'SET_THEME'; payload: UIState['theme'] }
  | { type: 'TOGGLE_SIDEBAR' }
  | { type: 'COLLAPSE_SIDEBAR'; payload: boolean }
  | { type: 'OPEN_MODAL'; payload: { key: string; data?: any } }
  | { type: 'CLOSE_MODAL'; payload: string }
  | { type: 'ADD_TOAST'; payload: Omit<Toast, 'id' | 'timestamp'> }
  | { type: 'REMOVE_TOAST'; payload: string }
  | { type: 'SET_LOADING'; payload: { key?: string; loading: boolean } }
  | { type: 'SET_RESPONSIVE'; payload: Partial<UIState['responsive']> };

export type UserAction = 
  | { type: 'SET_PROFILE'; payload: UserProfile }
  | { type: 'UPDATE_PREFERENCES'; payload: Partial<UserPreferences> }
  | { type: 'SET_SUBSCRIPTION'; payload: UserSubscription }
  | { type: 'UPDATE_PERMISSIONS'; payload: string[] }
  | { type: 'SET_PACKAGE_TIER'; payload: string }
  | { type: 'START_OPTIMISTIC_UPDATE'; payload: OptimisticUpdate }
  | { type: 'CONFIRM_OPTIMISTIC_UPDATE'; payload: string }
  | { type: 'ROLLBACK_OPTIMISTIC_UPDATE'; payload: string };

export type TradingAction = 
  | { type: 'SET_WATCHLIST'; payload: StockItem[] }
  | { type: 'ADD_TO_WATCHLIST'; payload: StockItem }
  | { type: 'REMOVE_FROM_WATCHLIST'; payload: string }
  | { type: 'UPDATE_STOCK_PRICE'; payload: { symbol: string; price: number; change: number } }
  | { type: 'SET_PORTFOLIO'; payload: PortfolioItem[] }
  | { type: 'SET_RANKINGS'; payload: StockRanking[] }
  | { type: 'ADD_PRICE_ALERT'; payload: PriceAlert }
  | { type: 'REMOVE_PRICE_ALERT'; payload: string }
  | { type: 'ADD_RECENT_SEARCH'; payload: string }
  | { type: 'SET_REALTIME_STATUS'; payload: { connected: boolean; subscriptions?: string[] } };

export type NotificationAction = 
  | { type: 'SET_NOTIFICATIONS'; payload: Notification[] }
  | { type: 'ADD_NOTIFICATION'; payload: Notification }
  | { type: 'MARK_READ'; payload: string }
  | { type: 'MARK_ALL_READ' }
  | { type: 'UPDATE_PREFERENCES'; payload: Partial<NotificationPreferences> }
  | { type: 'SET_REALTIME_STATUS'; payload: { connected: boolean; lastSync?: number } };

export type CacheAction = 
  | { type: 'SET_STOCK_DATA'; payload: { symbol: string; data: StockItem; ttl?: number } }
  | { type: 'SET_RANKINGS_DATA'; payload: { key: string; data: StockRanking[]; ttl?: number } }
  | { type: 'SET_ANALYTICS_DATA'; payload: { key: string; data: any; ttl?: number } }
  | { type: 'CLEAR_EXPIRED' }
  | { type: 'CLEAR_ALL' };