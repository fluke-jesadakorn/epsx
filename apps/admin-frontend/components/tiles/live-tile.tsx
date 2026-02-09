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
import type React from 'react';
import { useEffect, useRef, useState } from 'react';

import type {
  TileActionData,
  TileColor,
  TileData,
  TileSize,
  TileTrend
} from './types';
import {
  TILE_COLOR_CLASSES,
  TILE_SIZE_CLASSES
} from './types';

import { useSmartPolling } from '@/hooks/use-smart-polling';
import { logger } from '@/lib/logger';
import { cn } from '@/lib/utils';

interface TileDataResponse {
  value?: string | number;
  [key: string]: unknown;
}

interface LiveTileProps {
  tile: TileData;
  fetcher?: () => Promise<TileDataResponse | null>;
  onClick?: (tile: TileData, action?: TileActionData) => void;
  className?: string;
}

/**
 * Trend Display Sub-component
 */
function TileTrendDisplay({ trend }: { trend?: TileTrend }) {
  if (trend?.direction === undefined) { return null; }

  const { direction, value, percentage, period } = trend;
  const TrendIcon = direction === 'up' ? TrendingUp : direction === 'down' ? TrendingDown : Minus;
  const trendColor = direction === 'up' ? 'text-green-300' : direction === 'down' ? 'text-red-300' : 'text-gray-300';

  return (
    <div className={cn("flex items-center gap-1 text-xs", trendColor)}>
      <TrendIcon className="h-3 w-3" />
      <span>{percentage === true ? `${String(value)}%` : String(value)} {period}</span>
    </div>
  );
}

/**
 * Progress Bar Sub-component
 */
function TileProgressBar({ showProgress, metadata }: { showProgress?: boolean; metadata?: Record<string, unknown> }) {
  if (showProgress !== true || metadata?.progress === undefined) { return null; }

  const progress = Math.min(Math.max(Number(metadata.progress), 0), 100);

  return (
    <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/20 rounded-b-xl overflow-hidden">
      <div
        className="h-full bg-white/40"
        style={{ width: `${progress}%` }}
      />
    </div>
  );
}

/**
 * Status Indicators Sub-component
 */
function TileStatusIndicators({
  loading,
  hasError,
  isRealTime,
  isConnected,
  isPaused,
  onRetry
}: {
  loading: boolean;
  hasError: boolean;
  isRealTime: boolean;
  isConnected: boolean;
  isPaused: boolean;
  onRetry: (e: React.MouseEvent) => void;
}) {
  return (
    <>
      {isRealTime && (
        <div className="absolute top-2 right-2">
          {!isConnected ? (
            <WifiOff className="h-3 w-3 text-white/60" />
          ) : (
            <Wifi className={cn("h-3 w-3", isPaused ? "text-white/40" : "text-white/60")} />
          )}
        </div>
      )}

      {loading && (
        <div className="absolute inset-0 bg-black/20 flex items-center justify-center backdrop-blur-sm z-10">
          <Loader2 className="h-6 w-6 text-white animate-spin" />
        </div>
      )}

      {hasError && !loading && (
        <div className="absolute inset-0 bg-red-900/80 flex flex-col items-center justify-center backdrop-blur-sm z-10">
          <AlertTriangle className="h-6 w-6 text-white mb-2" />
          <button
            onClick={onRetry}
            className="flex items-center gap-1 text-xs text-white hover:text-gray-200"
          >
            <RotateCcw className="h-3 w-3" />
            Retry
          </button>
        </div>
      )}
    </>
  );
}

/**
 * Header Sub-component
 */
function TileHeader({ title, subtitle, icon: Icon, accentClass }: {
  title: string;
  subtitle?: string;
  icon: React.ComponentType<{ className?: string }>;
  accentClass: string;
}) {
  return (
    <div className="flex items-start justify-between">
      <div className="min-w-0 flex-1">
        <h3 className="font-medium text-sm truncate mb-1">{title}</h3>
        {typeof subtitle === 'string' && subtitle !== '' && (
          <p className="text-xs opacity-75 truncate">{subtitle}</p>
        )}
      </div>
      <div className={cn("p-2 rounded-lg ml-2 flex-shrink-0", accentClass)}>
        <Icon className="h-4 w-4" />
      </div>
    </div>
  );
}

/**
 * Value Display Sub-component
 */
function TileValueDisplay({ value, trend }: { value: string | number; trend?: TileTrend }) {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center">
        <div className="text-2xl font-bold">
          {typeof value === 'number' && value > 999
            ? `${(value / 1000).toFixed(1)}K`
            : value}
        </div>
        <TileTrendDisplay trend={trend} />
      </div>
    </div>
  );
}

interface TileMainContentProps {
  tile: TileData;
  currentData: Partial<TileData>;
  isFlipping: boolean;
  hasError: boolean;
  className?: string;
  isPollingActive: boolean;
  isConnected: boolean;
  loading: boolean;
  fetcher?: unknown;
  handleRetry: (e: React.MouseEvent) => void;
  handleRefresh: (e: React.MouseEvent) => void;
}

/**
 * Main Content Sub-component to reduce LiveTile complexity
 */
function TileMainContent({
  tile,
  currentData,
  isFlipping,
  hasError,
  className,
  isPollingActive,
  isConnected,
  loading,
  fetcher,
  handleRetry,
  handleRefresh
}: TileMainContentProps) {
  const colorClasses = TILE_COLOR_CLASSES[tile.color];
  const sizeClasses = TILE_SIZE_CLASSES[tile.size];

  return (
    <div
      className={cn(
        sizeClasses,
        colorClasses.background,
        colorClasses.text,
        colorClasses.hover,
        'relative rounded-xl shadow-lg overflow-hidden cursor-pointer',
        'focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-gray-900',
        'min-h-[120px] touch-manipulation transition-transform duration-500',
        isFlipping && 'scale-95 opacity-80',
        hasError && 'ring-2 ring-red-400',
        className
      )}
    >
      <TileStatusIndicators
        loading={loading}
        hasError={hasError}
        isRealTime={tile.isRealTime === true}
        isConnected={isConnected}
        isPaused={!isPollingActive}
        onRetry={handleRetry}
      />

      <div className="p-4 h-full flex flex-col justify-between">
        <TileHeader
          title={tile.title}
          subtitle={tile.subtitle}
          icon={tile.icon}
          accentClass={colorClasses.accent}
        />

        <TileValueDisplay
          value={currentData.value ?? tile.value}
          trend={tile.trend}
        />

        <div className="flex items-center justify-between text-xs opacity-75">
          {tile.lastUpdated !== undefined && (
            <span>
              {new Date(tile.lastUpdated).toLocaleTimeString('en-US', {
                hour: '2-digit',
                minute: '2-digit'
              })}
            </span>
          )}

          {tile.isRealTime === true && fetcher !== undefined && (
            <button
              onClick={handleRefresh}
              className="hover:opacity-100 p-1"
              title="Refresh now"
            >
              <RotateCcw className="h-3 w-3" />
            </button>
          )}
        </div>
      </div>

      <TileProgressBar showProgress={tile.showProgress} metadata={tile.metadata} />
      {tile.isRealTime === true && isPollingActive && isConnected && (
        <div className="absolute top-0 left-0 right-0 h-0.5 bg-white/30" />
      )}
    </div>
  );
}

/**
 * Live Tile Component
 */
export function LiveTile({ tile, fetcher, onClick, className }: LiveTileProps) {
  const [isFlipping, setIsFlipping] = useState(false);
  const previousValueRef = useRef(tile.value);
  const flipTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const {
    data,
    error,
    isLoading,
    isPollingActive,
    retry,
    refresh,
    connectionStatus
  } = useSmartPolling<TileDataResponse | null>(
    `tile-${tile.id}`,
    (tile.isRealTime === true && fetcher !== undefined) ? fetcher : (async () => Promise.resolve(null)),
    {
      priority: tile.priority,
      customInterval: tile.refreshInterval,
    }
  );

  useEffect(() => {
    const newValue = data?.value;
    if (newValue !== undefined && newValue !== previousValueRef.current) {
      setIsFlipping(true);
      previousValueRef.current = newValue;
      if (flipTimeoutRef.current) { clearTimeout(flipTimeoutRef.current); }
      flipTimeoutRef.current = setTimeout(() => setIsFlipping(false), 600);
    }
    if (error !== null) {
      logger.error(`Tile ${tile.id} polling error`, { tileId: tile.id, error });
    }
  }, [data, error, tile.id]);

  useEffect(() => {
    return () => {
      if (flipTimeoutRef.current) { clearTimeout(flipTimeoutRef.current); }
    };
  }, []);

  const currentData = (data as Partial<TileData> | null) ?? tile;
  const isConnected = connectionStatus !== 'offline';
  const hasError = error !== null || tile.hasError === true;
  const loading = isLoading === true || (tile.isLoading === true);

  const handleClick = () => {
    if (typeof tile.href === 'string' && tile.href !== '') { return; }
    onClick?.(tile, { type: 'action', payload: currentData });
  };

  const handleRetry = (e: React.MouseEvent) => { e.preventDefault(); e.stopPropagation(); retry(); };
  const handleRefresh = (e: React.MouseEvent) => { e.preventDefault(); e.stopPropagation(); refresh(); };

  const tileContent = (
    <TileMainContent
      tile={tile}
      currentData={currentData}
      isFlipping={isFlipping}
      hasError={hasError}
      className={className}
      isPollingActive={isPollingActive}
      isConnected={isConnected}
      loading={loading}
      fetcher={fetcher}
      handleRetry={handleRetry}
      handleRefresh={handleRefresh}
    />
  );

  if (typeof tile.href === 'string' && tile.href !== '') {
    return <Link href={tile.href} className="block outline-none">{tileContent}</Link>;
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={handleClick}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { handleClick(); } }}
      className="block outline-none"
    >
      {tileContent}
    </div>
  );
}

/**
 * User Stats Tile Component
 */
export function UserStatsTile(props: Omit<LiveTileProps, 'tile'> & {
  userCount: number;
  activeCount: number;
  trend?: TileTrend;
}) {
  const tile: TileData = {
    id: 'user-stats',
    title: 'Users',
    value: props.userCount,
    subtitle: `${props.activeCount} active`,
    icon: Wifi,
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
 * Security Alerts Tile Component
 */
export function SecurityAlertsTile(props: Omit<LiveTileProps, 'tile'> & {
  alertCount: number;
  severity: 'low' | 'medium' | 'high' | 'critical';
}) {
  const colors: Record<string, TileColor> = {
    low: 'success',
    medium: 'warning',
    high: 'error',
    critical: 'error'
  };

  const tile: TileData = {
    id: 'security-alerts',
    title: 'Security',
    value: props.alertCount,
    subtitle: props.alertCount > 0 ? `${props.severity} alerts` : 'All clear',
    icon: AlertTriangle,
    size: 'square',
    color: props.alertCount > 0 ? colors[props.severity] ?? 'error' : 'success',
    isRealTime: true,
    priority: 'critical',
    refreshInterval: 30000,
    href: '/security/alerts',
  };

  return <LiveTile {...props} tile={tile} />;
}

/**
 * Analytics Tile Component
 */
export function AnalyticsTile(props: Omit<LiveTileProps, 'tile'> & {
  metric: string;
  value: string | number;
  trend?: TileTrend;
  period?: string;
  size?: TileSize;
}) {
  const tile: TileData = {
    id: `analytics-${props.metric}`,
    title: props.metric,
    value: props.value,
    subtitle: props.period,
    icon: Wifi,
    size: props.size ?? 'large',
    color: 'info',
    trend: props.trend,
    isRealTime: true,
    priority: 'normal',
    refreshInterval: 300000,
    href: '/analytics',
  };

  return <LiveTile {...props} tile={tile} />;
}