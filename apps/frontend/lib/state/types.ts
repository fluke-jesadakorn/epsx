/**
 * State Management Types
 * Exported types for state management system
 */

// Import types from store for local usage
import type { AsyncState } from './store';

// Re-export types from store
export type {
  AsyncState, StateAction, StateConfig,
  StateMiddleware
} from './store';

// Additional types needed by app-state context
export interface UserProfile {
  id: string;
  email: string;
  name?: string;
  avatar?: string;
  permissions: Record<string, unknown>;
  tier?: string;
}

export interface UserSubscription {
  tier: string;
  status: 'active' | 'inactive' | 'cancelled';
  expiresAt?: string;
  features: string[];
}

export interface StockRanking {
  id: string;
  symbol: string;
  name: string;
  rank: number;
  price: number;
  change: number;
  changePercent: number;
}

export interface StockItem {
  id: string;
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  volume?: number;
  marketCap?: number;
}

export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: number;
  read?: boolean;
  actions?: {
    label: string;
    action: () => void;
  }[];
}

export interface Toast {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export interface OptimisticUpdate<T = unknown> {
  id: string;
  type: string;
  data: T;
  timestamp: number;
  retryCount?: number;
}

// State interfaces  
export interface UIState {
  theme: 'light' | 'dark' | 'system';
  sidebar: {
    open: boolean;
    collapsed: boolean;
  };
  modals: Record<string, unknown>;
  toasts: unknown[];
  loading: {
    global: boolean;
    requests: Record<string, boolean>;
  };
  errors?: Record<string, string>;
  responsive: {
    isMobile: boolean;
    isTablet: boolean;
    breakpoint: string;
  };
}

export interface UserData {
  profile: UserProfile | null;
  preferences: Record<string, unknown>;
  subscription: UserSubscription | null;
  permissions: string[];
  permissionGroup: string;
}

export interface UserState extends AsyncState<UserData> { }

export interface AnalyticsData {
  rankings: StockRanking[];
  filters: Record<string, unknown>;
  recentSearches: string[];
  bookmarks: StockItem[];
}

export interface AnalyticsState extends AsyncState<AnalyticsData> {
  realtime?: {
    connected: boolean;
    lastHeartbeat?: number;
    [key: string]: unknown;
  };
}

export interface NotificationState extends AsyncState<{
  unreadCount: number;
  notifications: Notification[];
  preferences: Record<string, unknown>;
}> {
  realtime?: {
    connected: boolean;
    lastSync?: number | null;
  };
}

export interface CacheState {
  [key: string]: {
    data: unknown;
    timestamp: number;
    ttl: number;
  } | {};
}

// Main app state interface that matches actual usage
export interface AppState {
  ui: UIState;
  user: UserState;
  analytics: AnalyticsState;
  notifications: NotificationState;
  cache: CacheState;
}