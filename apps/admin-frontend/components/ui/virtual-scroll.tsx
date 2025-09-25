"use client";

import React, { useState, useEffect, useRef, useCallback } from 'react';

interface VirtualScrollItem {
  id: string | number;
  height?: number;
}

interface VirtualScrollProps<T extends VirtualScrollItem> {
  items: T[];
  itemHeight: number;
  containerHeight: number;
  renderItem: (item: T, index: number) => React.ReactNode;
  className?: string;
  overscan?: number;
  onScroll?: (scrollTop: number) => void;
  loading?: boolean;
  loadingComponent?: React.ReactNode;
  emptyComponent?: React.ReactNode;
}

export function VirtualScroll<T extends VirtualScrollItem>({
  items,
  itemHeight,
  containerHeight,
  renderItem,
  className = '',
  overscan = 5,
  onScroll,
  loading = false,
  loadingComponent,
  emptyComponent
}: VirtualScrollProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  const totalHeight = items.length * itemHeight;
  const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  const endIndex = Math.min(
    items.length - 1,
    Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan
  );

  const visibleItems = items.slice(startIndex, endIndex + 1);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const newScrollTop = e.currentTarget.scrollTop;
    setScrollTop(newScrollTop);
    onScroll?.(newScrollTop);
  }, [onScroll]);

  const scrollToItem = useCallback((index: number, align: 'start' | 'center' | 'end' = 'start') => {
    if (!containerRef.current) return;

    let scrollTop: number;
    switch (align) {
      case 'start':
        scrollTop = index * itemHeight;
        break;
      case 'center':
        scrollTop = index * itemHeight - containerHeight / 2 + itemHeight / 2;
        break;
      case 'end':
        scrollTop = index * itemHeight - containerHeight + itemHeight;
        break;
    }

    containerRef.current.scrollTop = Math.max(0, Math.min(scrollTop, totalHeight - containerHeight));
  }, [itemHeight, containerHeight, totalHeight]);

  const scrollToTop = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = 0;
    }
  }, []);

  const scrollToBottom = useCallback(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = totalHeight - containerHeight;
    }
  }, [totalHeight, containerHeight]);

  if (loading && loadingComponent) {
    return (
      <div 
        className={`relative overflow-hidden ${className}`}
        style={{ height: containerHeight }}
      >
        {loadingComponent}
      </div>
    );
  }

  if (items.length === 0 && emptyComponent) {
    return (
      <div 
        className={`relative overflow-hidden ${className}`}
        style={{ height: containerHeight }}
      >
        {emptyComponent}
      </div>
    );
  }

  return (
    <div 
      ref={containerRef}
      className={`relative overflow-auto ${className}`}
      style={{ height: containerHeight }}
      onScroll={handleScroll}
    >
      <div style={{ height: totalHeight }}>
        <div 
          style={{ 
            transform: `translateY(${startIndex * itemHeight}px)`,
            position: 'relative'
          }}
        >
          {visibleItems.map((item, index) => (
            <div 
              key={item.id}
              style={{ height: item.height || itemHeight }}
            >
              {renderItem(item, startIndex + index)}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// Hook for virtual scroll control
export function useVirtualScroll<T extends VirtualScrollItem>(items: T[]) {
  const [scrollController, setScrollController] = useState<{
    scrollToItem: (index: number, align?: 'start' | 'center' | 'end') => void;
    scrollToTop: () => void;
    scrollToBottom: () => void;
  } | null>(null);

  const registerController = useCallback((controller: typeof scrollController) => {
    setScrollController(controller);
  }, []);

  return { scrollController, registerController };
}

// Enhanced virtual table component
interface VirtualTableColumn<T> {
  key: keyof T | string;
  header: string;
  width?: number | string;
  render?: (value: any, item: T, index: number) => React.ReactNode;
  sortable?: boolean;
}

interface VirtualTableProps<T extends VirtualScrollItem> {
  items: T[];
  columns: VirtualTableColumn<T>[];
  rowHeight?: number;
  containerHeight: number;
  className?: string;
  onRowClick?: (item: T, index: number) => void;
  sortBy?: keyof T | string;
  sortDirection?: 'asc' | 'desc';
  onSort?: (column: keyof T | string, direction: 'asc' | 'desc') => void;
  loading?: boolean;
  selectedItems?: Set<string | number>;
  onSelectionChange?: (selected: Set<string | number>) => void;
}

export function VirtualTable<T extends VirtualScrollItem>({
  items,
  columns,
  rowHeight = 48,
  containerHeight,
  className = '',
  onRowClick,
  sortBy,
  sortDirection,
  onSort,
  loading = false,
  selectedItems = new Set(),
  onSelectionChange
}: VirtualTableProps<T>) {
  const handleSort = (column: VirtualTableColumn<T>) => {
    if (!column.sortable || !onSort) return;
    
    const key = column.key as keyof T | string;
    const newDirection = sortBy === key && sortDirection === 'asc' ? 'desc' : 'asc';
    onSort(key, newDirection);
  };

  const handleSelectAll = (checked: boolean) => {
    if (!onSelectionChange) return;
    
    if (checked) {
      onSelectionChange(new Set(items.map(item => item.id)));
    } else {
      onSelectionChange(new Set());
    }
  };

  const handleSelectItem = (itemId: string | number, checked: boolean) => {
    if (!onSelectionChange) return;
    
    const newSelected = new Set(selectedItems);
    if (checked) {
      newSelected.add(itemId);
    } else {
      newSelected.delete(itemId);
    }
    onSelectionChange(newSelected);
  };

  const renderRow = (item: T, index: number) => (
    <div 
      className={`flex items-center border-b border-gray-200 hover:bg-gray-50 cursor-pointer ${
        selectedItems.has(item.id) ? 'bg-blue-50' : ''
      }`}
      onClick={() => onRowClick?.(item, index)}
    >
      {onSelectionChange && (
        <div className="w-12 flex justify-center">
          <input
            type="checkbox"
            checked={selectedItems.has(item.id)}
            onChange={(e) => {
              e.stopPropagation();
              handleSelectItem(item.id, e.target.checked);
            }}
            className="rounded border-gray-300"
          />
        </div>
      )}
      {columns.map((column, colIndex) => {
        const value = typeof column.key === 'string' ? 
          (item as any)[column.key] : 
          (item as any)[column.key as keyof T];
        
        return (
          <div
            key={colIndex}
            className="px-4 py-3 flex-shrink-0"
            style={{ 
              width: column.width || `${100 / columns.length}%`,
              minWidth: typeof column.width === 'number' ? `${column.width}px` : undefined
            }}
          >
            {column.render ? column.render(value, item, index) : String(value || '')}
          </div>
        );
      })}
    </div>
  );

  return (
    <div className={`border border-gray-200 rounded-lg overflow-hidden ${className}`}>
      {/* Header */}
      <div className="bg-gray-50 border-b border-gray-200 flex items-center sticky top-0 z-10">
        {onSelectionChange && (
          <div className="w-12 flex justify-center">
            <input
              type="checkbox"
              checked={selectedItems.size === items.length && items.length > 0}
              ref={(input) => {
                if (input) {
                  input.indeterminate = selectedItems.size > 0 && selectedItems.size < items.length;
                }
              }}
              onChange={(e) => handleSelectAll(e.target.checked)}
              className="rounded border-gray-300"
            />
          </div>
        )}
        {columns.map((column, index) => (
          <div
            key={index}
            className={`px-4 py-3 font-medium text-sm text-gray-700 flex-shrink-0 ${
              column.sortable ? 'cursor-pointer hover:bg-gray-100' : ''
            }`}
            style={{ 
              width: column.width || `${100 / columns.length}%`,
              minWidth: typeof column.width === 'number' ? `${column.width}px` : undefined
            }}
            onClick={() => handleSort(column)}
          >
            <div className="flex items-center gap-1">
              {column.header}
              {column.sortable && sortBy === column.key && (
                <span className="text-blue-500">
                  {sortDirection === 'asc' ? '↑' : '↓'}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Virtual scrolling content */}
      <VirtualScroll
        items={items}
        itemHeight={rowHeight}
        containerHeight={containerHeight - 49} // Subtract header height
        renderItem={renderRow}
        loading={loading}
        loadingComponent={
          <div className="flex items-center justify-center h-full">
            <div className="text-blue-600 font-medium">Loading...</div>
          </div>
        }
        emptyComponent={
          <div className="flex items-center justify-center h-full text-gray-500">
            <div className="text-center">
              <p className="text-lg font-medium">No items found</p>
              <p className="text-sm">Try adjusting your filters</p>
            </div>
          </div>
        }
      />
    </div>
  );
}