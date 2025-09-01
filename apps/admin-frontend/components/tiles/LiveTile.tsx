'use client';

import { useState, useEffect, useRef } from 'react';
import Link from 'next/link';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  TileData, 
  TILE_SIZE_CLASSES, 
  TILE_COLOR_CLASSES,
  TileActionData 
} from './types';
import { 
  TrendingUp, 
  TrendingDown, 
  Minus, 
  Loader2, 
  AlertTriangle,
  Wifi,
  WifiOff,
  RotateCcw
} from 'lucide-react';
import { useSmartPolling } from '@/hooks/useSmartPolling';
import { cn } from '@/lib/utils';

interface LiveTileProps {
  tile: TileData;
  fetcher?: () => Promise<any>;
  onClick?: (tile: TileData, action?: TileActionData) => void;
  className?: string;
}

export function LiveTile({ tile, fetcher, onClick, className }: LiveTileProps) {
  const [isFlipping, setIsFlipping] = useState(false);
  const [showDetails, setShowDetails] = useState(false);
  const previousValueRef = useRef(tile.value);
  const flipTimeoutRef = useRef<NodeJS.Timeout>();
  
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
    tile.isRealTime && fetcher ? `tile-${tile.id}` : null,
    fetcher || (() => Promise.resolve(null)),
    {
      priority: tile.priority,
      customInterval: tile.refreshInterval,
      onSuccess: (newData) => {
        // Trigger flip animation if value changed significantly
        if (newData?.value !== undefined && newData.value !== previousValueRef.current) {
          setIsFlipping(true);
          previousValueRef.current = newData.value;
          
          flipTimeoutRef.current = setTimeout(() => setIsFlipping(false), 600);
        }
      },
      onError: (error) => {
        console.error(`Tile ${tile.id} polling error:`, error);
      }
    }
  );

  // Use fetched data if available, otherwise use provided tile data
  const currentData = data || tile;
  const isConnected = connectionStatus !== 'offline';
  const hasError = error || tile.hasError;
  const loading = isLoading || tile.isLoading;

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
    if (!tile.trend) return null;
    
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
    if (!tile.showProgress || !tile.metadata?.progress) return null;
    
    const progress = Math.min(Math.max(tile.metadata.progress, 0), 100);
    
    return (
      <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/20 rounded-b-xl overflow-hidden">
        <motion.div 
          className="h-full bg-white/40"
          initial={{ width: 0 }}
          animate={{ width: `${progress}%` }}
          transition={{ duration: 0.5 }}
        />
      </div>
    );
  };

  const colorClasses = TILE_COLOR_CLASSES[tile.color];
  const sizeClasses = TILE_SIZE_CLASSES[tile.size];

  // Base tile content
  const tileContent = (
    <motion.div
      className={cn(
        sizeClasses,
        colorClasses.background,
        colorClasses.text,
        colorClasses.hover,
        'relative rounded-xl shadow-lg overflow-hidden cursor-pointer',
        'transform transition-all duration-150 hover:scale-105 active:scale-95',
        'focus:outline-none focus:ring-2 focus:ring-white/50 focus:ring-offset-2 focus:ring-offset-gray-900',
        'min-h-[120px] touch-manipulation',
        hasError && 'ring-2 ring-red-400',
        className
      )}
      whileHover={{ y: -2 }}
      whileTap={{ scale: 0.98 }}
      layout
    >
      {/* Connection Status Indicator */}
      {tile.isRealTime && (
        <div className="absolute top-2 right-2">
          {!isConnected ? (
            <WifiOff className="h-3 w-3 text-white/60" />
          ) : pollingState.isPaused ? (
            <motion.div
              animate={{ scale: [1, 1.2, 1] }}
              transition={{ duration: 2, repeat: Infinity }}
            >
              <Wifi className="h-3 w-3 text-white/60" />
            </motion.div>
          ) : (
            <Wifi className="h-3 w-3 text-white/60" />
          )}
        </div>
      )}

      {/* Loading Overlay */}
      <AnimatePresence>
        {loading && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/20 flex items-center justify-center backdrop-blur-sm"
          >
            <Loader2 className="h-6 w-6 text-white animate-spin" />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Error Overlay */}
      <AnimatePresence>
        {hasError && !loading && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-red-900/80 flex flex-col items-center justify-center backdrop-blur-sm"
          >
            <AlertTriangle className="h-6 w-6 text-white mb-2" />
            <button
              onClick={handleRetry}
              className="flex items-center gap-1 text-xs text-white hover:text-gray-200 transition-colors"
            >
              <RotateCcw className="h-3 w-3" />
              Retry
            </button>
          </motion.div>
        )}
      </AnimatePresence>

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

        {/* Value with Flip Animation */}
        <div className="flex-1 flex items-center justify-center">
          <AnimatePresence mode="wait">
            <motion.div
              key={`${tile.id}-${currentData.value}`}
              initial={isFlipping ? { rotateY: -90, opacity: 0 } : false}
              animate={{ rotateY: 0, opacity: 1 }}
              exit={{ rotateY: 90, opacity: 0 }}
              transition={{ duration: 0.3 }}
              className="text-center"
              style={{ perspective: '1000px' }}
            >
              <div className="text-2xl font-bold">
                {typeof currentData.value === 'number' && currentData.value > 999 
                  ? (currentData.value / 1000).toFixed(1) + 'K'
                  : currentData.value}
              </div>
              {renderTrend()}
            </motion.div>
          </AnimatePresence>
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
              className="hover:opacity-100 transition-opacity"
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
        <motion.div
          className="absolute top-0 left-0 right-0 h-0.5 bg-white/30"
          initial={{ scaleX: 0 }}
          animate={{ scaleX: 1 }}
          transition={{
            duration: (pollingState.currentInterval || 60000) / 1000,
            repeat: Infinity,
            ease: 'linear'
          }}
        />
      )}
    </motion.div>
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
    icon: props.tile?.icon || (() => null),
    size: 'wide',
    color: 'primary',
    trend: props.trend,
    isRealTime: true,
    priority: 'important',
    refreshInterval: 60000,
    href: '/users',
    ...props.tile
  };

  return <LiveTile {...props} tile={tile} />;
}

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
    icon: props.tile?.icon || (() => null),
    size: 'small',
    color: props.alertCount > 0 ? colors[props.severity] : 'success',
    isRealTime: true,
    priority: 'critical',
    refreshInterval: 30000,
    href: '/security/alerts',
    ...props.tile
  };

  return <LiveTile {...props} tile={tile} />;
}

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
    icon: props.tile?.icon || (() => null),
    size: 'large',
    color: 'info',
    trend: props.trend,
    isRealTime: true,
    priority: 'normal',
    refreshInterval: 300000,
    href: '/analytics',
    ...props.tile
  };

  return <LiveTile {...props} tile={tile} />;
}