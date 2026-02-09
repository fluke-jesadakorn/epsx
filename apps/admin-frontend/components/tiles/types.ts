import type React from 'react';

// Windows Phone-inspired tile system types
export interface TileData {
  id: string;
  title: string;
  value: string | number;
  subtitle?: string;
  icon: React.ComponentType<{ className?: string }>; // Lucide icon component
  size: TileSize;
  color: TileColor;
  href?: string;
  refreshInterval?: number;
  showProgress?: boolean;
  trend?: TileTrend;
  metadata?: Record<string, unknown>;
  priority?: TilePriority;
  lastUpdated?: Date;
  isLoading?: boolean;
  hasError?: boolean;
  isRealTime?: boolean;
  fetcher?: () => Promise<unknown>;
}

export type TileSize = 'small' | 'square' | 'wide' | 'large';
export type TileColor = 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
export type TilePriority = 'critical' | 'important' | 'normal' | 'background';

export interface TileTrend {
  direction?: 'up' | 'down' | 'flat';
  value?: number | string;
  percentage?: boolean;
  period?: string; // e.g., "vs yesterday", "vs last month"
}

export interface TileGridConfig {
  columns: number;
  gap: string;
  minTileWidth: string;
  maxTileWidth: string;
}

export interface SmartPollingConfig {
  intervals: {
    critical: number;    // 30s - Security alerts, system errors
    important: number;   // 60s - User stats, active sessions  
    normal: number;      // 5min - Analytics, reports
    background: number;  // 15min - Historical data, trends
  };
  pauseWhenInactive: boolean;
  retryOnError: boolean;
  maxRetries: number;
}

export interface TileActionData {
  type: 'navigate' | 'modal' | 'action' | 'refresh';
  payload?: unknown;
  confirmRequired?: boolean;
  confirmMessage?: string;
}

export const DEFAULT_POLLING_CONFIG: SmartPollingConfig = {
  intervals: {
    critical: 30000,    // 30 seconds
    important: 60000,   // 1 minute
    normal: 300000,     // 5 minutes  
    background: 900000  // 15 minutes
  },
  pauseWhenInactive: true,
  retryOnError: true,
  maxRetries: 3
};

export const TILE_SIZE_CLASSES = {
  small: 'col-span-1 row-span-1 aspect-square',
  square: 'col-span-1 row-span-1 aspect-square',
  wide: 'col-span-2 row-span-1 aspect-[2/1]',
  large: 'col-span-2 row-span-2 aspect-square'
};

export const TILE_COLOR_CLASSES = {
  primary: {
    background: 'bg-gradient-to-br from-blue-500 to-blue-600 dark:from-blue-600 dark:to-blue-700',
    text: 'text-white',
    accent: 'bg-blue-400 dark:bg-blue-500',
    hover: 'hover:from-blue-600 hover:to-blue-700 dark:hover:from-blue-700 dark:hover:to-blue-800'
  },
  secondary: {
    background: 'bg-gradient-to-br from-gray-500 to-gray-600 dark:from-gray-600 dark:to-gray-700',
    text: 'text-white',
    accent: 'bg-gray-400 dark:bg-gray-500',
    hover: 'hover:from-gray-600 hover:to-gray-700 dark:hover:from-gray-700 dark:hover:to-gray-800'
  },
  success: {
    background: 'bg-gradient-to-br from-green-500 to-green-600 dark:from-green-600 dark:to-green-700',
    text: 'text-white',
    accent: 'bg-green-400 dark:bg-green-500',
    hover: 'hover:from-green-600 hover:to-green-700 dark:hover:from-green-700 dark:hover:to-green-800'
  },
  warning: {
    background: 'bg-gradient-to-br from-amber-500 to-amber-600 dark:from-amber-600 dark:to-amber-700',
    text: 'text-white',
    accent: 'bg-amber-400 dark:bg-amber-500',
    hover: 'hover:from-amber-600 hover:to-amber-700 dark:hover:from-amber-700 dark:hover:to-amber-800'
  },
  error: {
    background: 'bg-gradient-to-br from-red-500 to-red-600 dark:from-red-600 dark:to-red-700',
    text: 'text-white',
    accent: 'bg-red-400 dark:bg-red-500',
    hover: 'hover:from-red-600 hover:to-red-700 dark:hover:from-red-700 dark:hover:to-red-800'
  },
  info: {
    background: 'bg-gradient-to-br from-cyan-500 to-cyan-600 dark:from-cyan-600 dark:to-cyan-700',
    text: 'text-white',
    accent: 'bg-cyan-400 dark:bg-cyan-500',
    hover: 'hover:from-cyan-600 hover:to-cyan-700 dark:hover:from-cyan-700 dark:hover:to-cyan-800'
  }
};