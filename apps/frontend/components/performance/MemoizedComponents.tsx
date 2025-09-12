'use client';

import React, { memo, useMemo, useCallback, forwardRef } from 'react';
import { Button, ButtonProps } from '@/components/ui/button';
import { Card, CardProps } from '@/components/ui/card';
import { Badge, BadgeProps } from '@/components/ui/badge';
import { uiLogger } from '@/lib/logger';

// Performance logging utility
const logRender = (componentName: string, props?: any) => {
  if (process.env.NODE_ENV === 'development') {
    uiLogger.debug(`Rendering ${componentName}`, { props });
  }
};

// Memoized Button with performance optimizations
export const MemoizedButton = memo<ButtonProps>(
  forwardRef<HTMLButtonElement, ButtonProps>(({ children, onClick, ...props }, ref) => {
    logRender('MemoizedButton', props);

    const handleClick = useCallback(
      (e: React.MouseEvent<HTMLButtonElement>) => {
        onClick?.(e);
      },
      [onClick]
    );

    return (
      <Button ref={ref} onClick={handleClick} {...props}>
        {children}
      </Button>
    );
  })
);

MemoizedButton.displayName = 'MemoizedButton';

// Memoized Card with deep comparison for complex props
export const MemoizedCard = memo<CardProps>(
  ({ children, className, ...props }) => {
    logRender('MemoizedCard', props);

    const cardClassName = useMemo(
      () => className,
      [className]
    );

    return (
      <Card className={cardClassName} {...props}>
        {children}
      </Card>
    );
  },
  (prevProps, nextProps) => {
    // Custom comparison for better memoization
    return (
      prevProps.className === nextProps.className &&
      JSON.stringify(prevProps.children) === JSON.stringify(nextProps.children) &&
      Object.keys(prevProps).every(key => prevProps[key] === nextProps[key])
    );
  }
);

MemoizedCard.displayName = 'MemoizedCard';

// Memoized Badge with variant-specific optimizations
export const MemoizedBadge = memo<BadgeProps>(
  ({ children, variant, ...props }) => {
    logRender('MemoizedBadge', { variant });

    const badgeContent = useMemo(
      () => children,
      [children]
    );

    return (
      <Badge variant={variant} {...props}>
        {badgeContent}
      </Badge>
    );
  }
);

MemoizedBadge.displayName = 'MemoizedBadge';

// Higher-order component for automatic memoization
export function withMemoization<T extends Record<string, any>>(
  Component: React.ComponentType<T>,
  displayName?: string,
  customComparator?: (prevProps: T, nextProps: T) => boolean
) {
  const MemoizedComponent = memo(Component, customComparator);
  MemoizedComponent.displayName = displayName || `Memoized${Component.displayName || Component.name}`;
  return MemoizedComponent;
}

// Performance-optimized list renderer
interface MemoizedListProps<T> {
  items: T[];
  renderItem: (item: T, index: number) => React.ReactNode;
  keyExtractor: (item: T, index: number) => string | number;
  className?: string;
  emptyMessage?: React.ReactNode;
}

export const MemoizedList = memo(<T extends any>(props: MemoizedListProps<T>) => {
  const { items, renderItem, keyExtractor, className, emptyMessage } = props;
  
  logRender('MemoizedList', { itemCount: items.length });

  const renderedItems = useMemo(
    () => items.map((item, index) => (
      <React.Fragment key={keyExtractor(item, index)}>
        {renderItem(item, index)}
      </React.Fragment>
    )),
    [items, renderItem, keyExtractor]
  );

  if (items.length === 0) {
    return emptyMessage ? <>{emptyMessage}</> : null;
  }

  return (
    <div className={className}>
      {renderedItems}
    </div>
  );
}) as <T>(props: MemoizedListProps<T>) => JSX.Element;

MemoizedList.displayName = 'MemoizedList';

// Memoized analytics card for stock rankings
interface StockCardData {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  rank: number;
}

interface MemoizedStockCardProps {
  data: StockCardData;
  onClick?: (symbol: string) => void;
  isSelected?: boolean;
  showRank?: boolean;
}

export const MemoizedStockCard = memo<MemoizedStockCardProps>(
  ({ data, onClick, isSelected = false, showRank = true }) => {
    logRender('MemoizedStockCard', { symbol: data.symbol });

    const handleClick = useCallback(() => {
      onClick?.(data.symbol);
    }, [onClick, data.symbol]);

    const changeColor = useMemo(() => {
      return data.change >= 0 ? 'text-green-600' : 'text-red-600';
    }, [data.change]);

    const cardClassName = useMemo(() => {
      const baseClass = 'p-4 border rounded-lg cursor-pointer hover:shadow-md transition-shadow';
      return isSelected ? `${baseClass} border-blue-500 bg-blue-50` : `${baseClass} border-gray-200`;
    }, [isSelected]);

    return (
      <div className={cardClassName} onClick={handleClick}>
        <div className="flex justify-between items-start mb-2">
          <div>
            <h3 className="font-semibold text-lg">{data.symbol}</h3>
            <p className="text-sm text-gray-600 truncate">{data.name}</p>
          </div>
          {showRank && (
            <MemoizedBadge variant="secondary">
              #{data.rank}
            </MemoizedBadge>
          )}
        </div>
        
        <div className="flex justify-between items-end">
          <div>
            <p className="text-xl font-bold">${data.price.toFixed(2)}</p>
          </div>
          <div className={`text-right ${changeColor}`}>
            <p className="text-sm font-medium">
              {data.change >= 0 ? '+' : ''}${data.change.toFixed(2)}
            </p>
            <p className="text-xs">
              ({data.changePercent >= 0 ? '+' : ''}{data.changePercent.toFixed(2)}%)
            </p>
          </div>
        </div>
      </div>
    );
  },
  // Custom comparator to prevent unnecessary renders
  (prevProps, nextProps) => {
    const prevData = prevProps.data;
    const nextData = nextProps.data;
    
    return (
      prevData.symbol === nextData.symbol &&
      prevData.name === nextData.name &&
      prevData.price === nextData.price &&
      prevData.change === nextData.change &&
      prevData.changePercent === nextData.changePercent &&
      prevData.rank === nextData.rank &&
      prevProps.isSelected === nextProps.isSelected &&
      prevProps.showRank === nextProps.showRank &&
      prevProps.onClick === nextProps.onClick
    );
  }
);

MemoizedStockCard.displayName = 'MemoizedStockCard';

// Performance monitoring hook
export function useRenderCount(componentName: string) {
  const renderCount = React.useRef(0);
  
  React.useEffect(() => {
    renderCount.current += 1;
    if (process.env.NODE_ENV === 'development') {
      uiLogger.debug(`${componentName} render count: ${renderCount.current}`);
    }
  });

  return renderCount.current;
}

// Hook to detect expensive re-renders
export function useExpensiveOperation<T>(
  computeFn: () => T,
  dependencies: React.DependencyList,
  threshold = 10 // ms
) {
  return useMemo(() => {
    const start = performance.now();
    const result = computeFn();
    const duration = performance.now() - start;
    
    if (duration > threshold && process.env.NODE_ENV === 'development') {
      uiLogger.warn(`Expensive operation detected: ${duration.toFixed(2)}ms`, {
        dependencies,
        threshold
      });
    }
    
    return result;
  }, dependencies);
}

// Component wrapper for performance monitoring
export function withPerformanceMonitoring<T extends Record<string, any>>(
  Component: React.ComponentType<T>,
  componentName?: string
) {
  return memo(forwardRef<any, T>((props, ref) => {
    const name = componentName || Component.displayName || Component.name || 'Anonymous';
    const renderCount = useRenderCount(name);
    
    const start = useMemo(() => performance.now(), []);
    
    React.useEffect(() => {
      const duration = performance.now() - start;
      if (duration > 16 && process.env.NODE_ENV === 'development') {
        // More than one frame (16ms) to render
        uiLogger.warn(`Slow render detected in ${name}: ${duration.toFixed(2)}ms`);
      }
    });
    
    return <Component {...props} ref={ref} />;
  }));
}