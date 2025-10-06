'use client';

import { ChevronLeft, ChevronRight, RotateCcw, Settings } from 'lucide-react';
import { useState, useEffect, useRef, useMemo } from 'react';

import { LiveTile } from './LiveTile';
import { TileData, TileGridConfig } from './types';

import { logger } from '@/lib/logger';
import { cn } from '@/lib/utils';

interface TileGridProps {
  tiles: TileData[];
  config?: Partial<TileGridConfig>;
  onTileClick?: (tile: TileData, action?: any) => void;
  onRefreshAll?: () => void;
  className?: string;
  title?: string;
  subtitle?: string;
  showControls?: boolean;
  horizontal?: boolean;
}

const DEFAULT_CONFIG: TileGridConfig = {
  columns: 4,
  gap: '1rem',
  minTileWidth: '120px',
  maxTileWidth: '200px'
};

/**
 *
 * @param root0
 * @param root0.tiles
 * @param root0.config
 * @param root0.onTileClick
 * @param root0.onRefreshAll
 * @param root0.className
 * @param root0.title
 * @param root0.subtitle
 * @param root0.showControls
 * @param root0.horizontal
 */
export function TileGrid({
  tiles,
  config = {},
  onTileClick,
  onRefreshAll,
  className,
  title,
  subtitle,
  showControls = true,
  horizontal = false
}: TileGridProps) {
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [scrollPosition, setScrollPosition] = useState(0);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);
  const gridRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const gridConfig = useMemo(() => ({ ...DEFAULT_CONFIG, ...config }), [config]);

  // Check scroll capabilities
  const checkScrollCapabilities = () => {
    if (!scrollContainerRef.current) {return;}

    const container = scrollContainerRef.current;
    const scrollLeft = container.scrollLeft;
    const scrollWidth = container.scrollWidth;
    const clientWidth = container.clientWidth;

    setCanScrollLeft(scrollLeft > 0);
    setCanScrollRight(scrollLeft < scrollWidth - clientWidth - 5); // 5px tolerance
  };

  useEffect(() => {
    checkScrollCapabilities();
    
    const container = scrollContainerRef.current;
    if (container) {
      container.addEventListener('scroll', checkScrollCapabilities);
      return () => container.removeEventListener('scroll', checkScrollCapabilities);
    }
  }, [tiles]);

  // Handle refresh all
  const handleRefreshAll = async () => {
    if (isRefreshing) {return;}
    
    setIsRefreshing(true);
    try {
      await onRefreshAll?.();
    } finally {
      setTimeout(() => setIsRefreshing(false), 1000);
    }
  };

  // Scroll functions
  const scrollLeft = () => {
    if (!scrollContainerRef.current) {return;}
    const container = scrollContainerRef.current;
    const scrollAmount = container.clientWidth * 0.8;
    container.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
  };

  const scrollRight = () => {
    if (!scrollContainerRef.current) {return;}
    const container = scrollContainerRef.current;
    const scrollAmount = container.clientWidth * 0.8;
    container.scrollBy({ left: scrollAmount, behavior: 'smooth' });
  };

  // Generate grid styles
  const gridStyles = horizontal ? {} : {
    display: 'grid',
    gridTemplateColumns: `repeat(auto-fit, minmax(${gridConfig.minTileWidth}, 1fr))`,
    gap: gridConfig.gap,
    maxWidth: '100%'
  };

  const containerClass = horizontal
    ? 'flex gap-4 overflow-x-auto pb-4 scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600'
    : 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4';

  return (
    <div className={cn('space-y-4', className)}>
      {/* Header */}
      {(title || subtitle || showControls) && (
        <div className="flex items-center justify-between">
          <div>
            {title && (
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
                {title}
              </h2>
            )}
            {subtitle && (
              <p className="text-gray-600 dark:text-gray-400 mt-1">
                {subtitle}
              </p>
            )}
          </div>
          
          {showControls && (
            <div className="flex items-center gap-2">
              {horizontal && (
                <>
                  <button
                    onClick={scrollLeft}
                    disabled={!canScrollLeft}
                    className="p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50 dark:hover:bg-gray-700 "
                    aria-label="Scroll left"
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </button>
                  <button
                    onClick={scrollRight}
                    disabled={!canScrollRight}
                    className="p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50 dark:hover:bg-gray-700 "
                    aria-label="Scroll right"
                  >
                    <ChevronRight className="h-4 w-4" />
                  </button>
                </>
              )}
              
              <button
                onClick={handleRefreshAll}
                disabled={isRefreshing}
                className="flex items-center gap-2 px-3 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 text-white  disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <RotateCcw className="h-4 w-4" />
                <span className="text-sm font-medium">
                  {isRefreshing ? 'Refreshing...' : 'Refresh All'}
                </span>
              </button>
            </div>
          )}
        </div>
      )}

      {/* Tiles Grid/Horizontal Scroll */}
      <div
        ref={scrollContainerRef}
        className={containerClass}
        style={gridStyles}
      >
        {tiles.map((tile, index) => (
          <div
            key={tile.id}
            className={horizontal ? 'flex-shrink-0' : ''}
            style={horizontal ? { minWidth: gridConfig.minTileWidth } : {}}
          >
            <LiveTile
              tile={tile}
              onClick={onTileClick}
              fetcher={tile.fetcher}
            />
          </div>
        ))}
      </div>

      {/* Empty State */}
      {tiles.length === 0 && (
        <div className="text-center py-12 text-gray-500 dark:text-gray-400">
          <div className="text-4xl mb-4">📊</div>
          <h3 className="text-lg font-medium mb-2">No tiles available</h3>
          <p className="text-sm">Tiles will appear here when data is available.</p>
        </div>
      )}

      {/* Performance indicator for development */}
      {process.env.NODE_ENV === 'development' && (
        <div className="text-xs text-gray-400 flex items-center gap-2 mt-4">
          <Settings className="h-3 w-3" />
          <span>{tiles.length} tiles • {horizontal ? 'Horizontal' : 'Grid'} layout</span>
        </div>
      )}
    </div>
  );
}

// Specialized grid layouts

/**
 *
 * @param props
 */
export function DashboardTileGrid(props: Omit<TileGridProps, 'title' | 'subtitle'>) {
  return (
    <TileGrid
      {...props}
      title="Dashboard Overview"
      subtitle="Real-time system metrics and analytics"
      className="mb-8"
    />
  );
}

/**
 *
 * @param props
 */
export function HorizontalTileScroller(props: Omit<TileGridProps, 'horizontal'>) {
  return (
    <TileGrid
      {...props}
      horizontal={true}
      config={{
        minTileWidth: '160px',
        maxTileWidth: '200px',
        gap: '1rem'
      }}
    />
  );
}

/**
 *
 * @param props
 */
export function CompactTileGrid(props: TileGridProps) {
  return (
    <TileGrid
      {...props}
      config={{
        columns: 6,
        gap: '0.75rem',
        minTileWidth: '100px',
        maxTileWidth: '140px',
        ...props.config
      }}
    />
  );
}

// Utility functions

async function fetchTileData(tileId: string): Promise<any> {
  // This would be replaced with actual API calls based on tile type
  try {
    const response = await fetch(`/api/v1/admin/tiles/${tileId}`);
    if (!response.ok) {throw new Error(`HTTP ${response.status}`);}
    return response.json();
  } catch (_error) {
    logger.error(`Failed to fetch data for tile ${tileId}`, { tileId, _error });
    throw _error;
  }
}

// Tile data transformers for common API responses

/**
 *
 * @param apiResponse
 */
export function transformUserStats(apiResponse: any): TileData {
  return {
    id: 'user-stats',
    title: 'Users',
    value: apiResponse.total_users || 0,
    subtitle: `${apiResponse.active_users || 0} active`,
    icon: () => null, // Will be provided by the consuming component
    size: 'wide',
    color: 'primary',
    trend: apiResponse.user_growth ? {
      direction: apiResponse.user_growth > 0 ? 'up' : 'down',
      value: Math.abs(apiResponse.user_growth),
      percentage: true,
      period: 'vs last month'
    } : undefined,
    isRealTime: true,
    priority: 'important',
    lastUpdated: new Date(),
    href: '/users'
  };
}

/**
 *
 * @param apiResponse
 */
export function transformAnalytics(apiResponse: any): TileData {
  return {
    id: 'analytics',
    title: 'Performance',
    value: `${apiResponse.performance_score || 0}%`,
    subtitle: 'System performance',
    icon: () => null,
    size: 'large',
    color: 'info',
    trend: apiResponse.performance_trend ? {
      direction: apiResponse.performance_trend > 0 ? 'up' : 'down',
      value: Math.abs(apiResponse.performance_trend),
      percentage: true,
      period: 'vs yesterday'
    } : undefined,
    showProgress: true,
    metadata: {
      progress: apiResponse.performance_score || 0
    },
    isRealTime: true,
    priority: 'normal',
    lastUpdated: new Date(),
    href: '/analytics'
  };
}