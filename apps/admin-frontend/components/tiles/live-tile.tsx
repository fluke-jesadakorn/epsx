'use client';

import {
  AlertTriangle,
  Loader2,
  Minus,
  RotateCcw,
  TrendingDown,
  TrendingUp,
  Wifi,
  WifiOff
} from 'lucide-react';
import Link from 'next/link';
import { useEffect, useRef, useState } from 'react';

import type {
  TileActionData,
  TileData
} from './types';
import {
  TILE_COLOR_CLASSES,
  TILE_SIZE_CLASSES
} from './types';

import { useSmartPolling } from '@/hooks/use-smart-polling';
import { logger } from '@/lib/logger';
import { cn } from '@/lib/utils';

interface LiveTileProps {
  tile: TileData;
  fetcher?: () => Promise<any>;
  onClick?: (tile: TileData, action?: TileActionData) => void;
  className?: string;
}

/**
 *
 * @param root0
 * @param root0.tile
 * @param root0.fetcher
 * @param root0.onClick
 * @param root0.className
 */
export function LiveTile({ tile, fetcher, onClick, className }: LiveTileProps) {
  const [isFlipping, setIsFlipping] = useState(false);
  const [showDetails, setShowDetails] = useState(false);
  const previousValueRef = useRef(tile.value);
  const flipTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Smart polling for real-time data if fetcher is provided
  const {
    data,
    error,
    isLoading,
    pollingState,
    retry,
    refresh,
    connectionStatus
  } = useSmartPolling(
    `tile-${tile.id}`,
    (tile.isRealTime && fetcher) ? fetcher : (() => Promise.resolve(null)),
    {
      priority: tile.priority,
      customInterval: tile.refreshInterval,
      onSuccess: (newData) => {
        // Trigger flip animation if value changed significantly
        const data = newData as Record<string, unknown> | null;
        const newValue = data?.value as string | number | undefined;
        if (newValue !== undefined && newValue !== previousValueRef.current) {
          setIsFlipping(true);
          previousValueRef.current = newValue;

          flipTimeoutRef.current = setTimeout(() => setIsFlipping(false), 600);
        }
      },
      onError: (error) => {
        logger.error(`Tile ${tile.id} polling error`, { tileId: tile.id, error });
      }
    }
  );

  // Use fetched data if available, otherwise use provided tile data
  const currentData = data ?? tile;
  const isConnected = connectionStatus !== 'offline';
  const hasError = error ?? tile.hasError;
  const loading = isLoading ?? tile.isLoading;

  useEffect(() => {
    return () => {
      if (flipTimeoutRef.current) {
        clearTimeout(flipTimeoutRef.current);
      }
    };
  }, []);

  const handleClick = () => {
    if (tile.href) {
      // Navigation will be handled by Link component
      return;
    }

    onClick?.(tile, {
      type: 'action',
      payload: currentData
    });
  };

  const handleRetry = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    retry();
  };

  const handleRefresh = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    refresh();
  };

  const renderTrend = () => {
    if (!tile.trend) { return null; }

    const { direction, value, percentage, period } = tile.trend;
    const TrendIcon = direction === 'up' ? TrendingUp : direction === 'down' ? TrendingDown : Minus;
    const trendColor = direction === 'up' ? 'text-green-300' : direction === 'down' ? 'text-red-300' : 'text-gray-300';

    return (
      <div className={`flex items-center gap-1 text-xs ${trendColor}`}>
        <TrendIcon className="h-3 w-3" />
        <span>{percentage ? `${value}%` : value} {period}</span>
      </div>
    );
  };

  const renderProgressBar = () => {
    if (!tile.showProgress ?? !tile.metadata?.progress) { return null; }

    const progress = Math.min(Math.max(tile.metadata.progress, 0), 100);

    return (
      <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/20 rounded-b-xl overflow-hidden">
        <div
          className="h-full bg-white/40"
          style={{ width: `${progress}%` }}
        />
      </div>
    );
  };

  const colorClasses = TILE_COLOR_CLASSES[tile.color];
  const sizeClasses = TILE_SIZE_CLASSES[tile.size];

  // Base tile content
  const tileContent = (
    <div
      className={cn(
        sizeClasses,
        colorClasses.background,
        colorClasses.text,
        colorClasses.hover,
        'relative rounded-xl shadow-lg overflow-hidden cursor-pointer',
        'focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-gray-900',
        'min-h-[120px] touch-manipulation',
        hasError && 'ring-2 ring-red-400',
        className
      )}
    >
      {/* Connection Status Indicator */}
      {tile.isRealTime && (
        <div className="absolute top-2 right-2">
          {!isConnected ? (
            <WifiOff className="h-3 w-3 text-white/60" />
          ) : pollingState.isPaused ? (
            <div>
              <Wifi className="h-3 w-3 text-white/60" />
            </div>
          ) : (
            <Wifi className="h-3 w-3 text-white/60" />
          )}
        </div>
      )}

      {/* Loading Overlay */}
      {loading && (
        <div className="absolute inset-0 bg-black/20 flex items-center justify-center backdrop-blur-sm">
          <Loader2 className="h-6 w-6 text-white" />
        </div>
      )}

      {/* Error Overlay */}
      {hasError && !loading && (
        <div className="absolute inset-0 bg-red-900/80 flex flex-col items-center justify-center backdrop-blur-sm">
          <AlertTriangle className="h-6 w-6 text-white mb-2" />
          <button
            onClick={handleRetry}
            className="flex items-center gap-1 text-xs text-white hover:text-gray-200"
          >
            <RotateCcw className="h-3 w-3" />
            Retry
          </button>
        </div>
      )}

      {/* Main Content */}
      <div className="p-4 h-full flex flex-col justify-between">
        {/* Header */}
        <div className="flex items-start justify-between">
          <div className="min-w-0 flex-1">
            <h3 className="font-medium text-sm truncate mb-1">{tile.title}</h3>
            {tile.subtitle && (
              <p className="text-xs opacity-75 truncate">{tile.subtitle}</p>
            )}
          </div>

          {/* Icon */}
          <div className={`p-2 rounded-lg ${colorClasses.accent} ml-2 flex-shrink-0`}>
            <tile.icon className="h-4 w-4" />
          </div>
        </div>

        {/* Value Display */}
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <div className="text-2xl font-bold">
              {typeof currentData.value === 'number' && currentData.value > 999
                ? `${(currentData.value / 1000).toFixed(1)  }K`
                : currentData.value}
            </div>
            {renderTrend()}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between text-xs opacity-75">
          {tile.lastUpdated && (
            <span>
              {new Date(tile.lastUpdated).toLocaleTimeString('en-US', {
                hour: '2-digit',
                minute: '2-digit'
              })}
            </span>
          )}

          {tile.isRealTime && fetcher && (
            <button
              onClick={handleRefresh}
              className="hover:opacity-100"
              title="Refresh now"
            >
              <RotateCcw className="h-3 w-3" />
            </button>
          )}
        </div>
      </div>

      {/* Progress Bar */}
      {renderProgressBar()}

      {/* Live Update Indicator */}
      {tile.isRealTime && !pollingState.isPaused && isConnected && (
        <div className="absolute top-0 left-0 right-0 h-0.5 bg-white/30" />
      )}
    </div>
  );

  // Wrap with Link if href is provided
  if (tile.href) {
    return (
      <Link href={tile.href} className="block">
        {tileContent}
      </Link>
    );
  }

  // Plain clickable div
  return (
    <div onClick={handleClick} className="block">
      {tileContent}
    </div>
  );
}

// Specialized tile variants for common use cases

/**
 *
 * @param props
 */
export function UserStatsTile(props: Omit<LiveTileProps, 'tile'> & {
  userCount: number;
  activeCount: number;
  trend?: any
}) {
  const tile: TileData = {
    id: 'user-stats',
    title: 'Users',
    value: props.userCount,
    subtitle: `${props.activeCount} active`,
    icon: (() => null),
    size: 'wide',
    color: 'primary',
    trend: props.trend,
    isRealTime: true,
    priority: 'important',
    refreshInterval: 60000,
    href: '/users'
  };

  return <LiveTile {...props} tile={tile} />;
}

/**
 *
 * @param props
 */
export function SecurityAlertsTile(props: Omit<LiveTileProps, 'tile'> & {
  alertCount: number;
  severity: 'low' | 'medium' | 'high' | 'critical'
}) {
  const colors = {
    low: 'success' as const,
    medium: 'warning' as const,
    high: 'error' as const,
    critical: 'error' as const
  };

  const tile: TileData = {
    id: 'security-alerts',
    title: 'Security',
    value: props.alertCount,
    subtitle: props.alertCount > 0 ? `${props.severity} alerts` : 'All clear',
    icon: (() => null),
    size: 'small',
    color: props.alertCount > 0 ? colors[props.severity] : 'success',
    isRealTime: true,
    priority: 'critical',
    refreshInterval: 30000,
    href: '/security/alerts',
  };

  return <LiveTile {...props} tile={tile} />;
}

/**
 *
 * @param props
 */
export function AnalyticsTile(props: Omit<LiveTileProps, 'tile'> & {
  metric: string;
  value: string | number;
  trend?: any;
  period?: string;
}) {
  const tile: TileData = {
    id: 'analytics',
    title: props.metric,
    value: props.value,
    subtitle: props.period,
    icon: (() => null),
    size: 'large',
    color: 'info',
    trend: props.trend,
    isRealTime: true,
    priority: 'normal',
    refreshInterval: 300000,
    href: '/analytics',
  };

  return <LiveTile {...props} tile={tile} />;
}