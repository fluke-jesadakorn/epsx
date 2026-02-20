'use client';

import { Input } from '@/components/ui/input';

interface PermissionItem {
  id: string;
  name: string;
  code: string;
}

interface PermissionListProps {
  title: string;
  items: PermissionItem[];
  count: number;
  searchValue: string;
  onSearchChange: (value: string) => void;
  listRef: React.RefObject<HTMLDivElement>;
  onDragOver: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent) => void;
  dropTarget: 'available' | 'selected' | null;
  touchSource: 'available' | 'selected' | null;
  source: 'available' | 'selected';
  draggingItem: string | null;
  onDragStart: (e: React.DragEvent, item: PermissionItem) => void;
  onDragEnd: (e: React.DragEvent) => void;
  onTouchStart: (e: React.TouchEvent, item: PermissionItem) => void;
  onItemClick: (item: PermissionItem) => void;
  emptyMessage: string;
  variant: 'available' | 'selected';
}

export function PermissionList({
  title,
  items,
  count,
  searchValue,
  onSearchChange,
  listRef,
  onDragOver,
  onDrop,
  dropTarget,
  touchSource,
  source,
  draggingItem,
  onDragStart,
  onDragEnd,
  onTouchStart,
  onItemClick,
  emptyMessage,
  variant
}: PermissionListProps) {
  const isAvailable = variant === 'available';
  const cn = (...classes: (string | boolean | undefined)[]) =>
    classes.filter(Boolean).join(' ');

  const shouldHighlightDrop =
    dropTarget === source &&
    touchSource !== null &&
    touchSource !== source;

  return (
    <div className="flex flex-col space-y-2">
      <div className="flex items-center justify-between px-1">
        <h3 className={cn(
          "text-xs uppercase font-bold tracking-widest flex items-center gap-2",
          isAvailable ? "text-gray-500" : "text-amber-600 dark:text-amber-500"
        )}>
          {title}{' '}
          <span className={cn(
            "px-2 py-0.5 rounded-full text-[10px]",
            isAvailable
              ? "bg-gray-100 dark:bg-gray-800"
              : "bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 border border-amber-200 dark:border-amber-800"
          )}>
            {count}
          </span>
        </h3>
      </div>

      <div className="relative group">
        <svg
          className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 group-focus-within:text-amber-500 transition-colors"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <Input
          value={searchValue}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={`Search ${isAvailable ? 'available' : 'authorized'}...`}
          className="pl-10 bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700 h-10 rounded-xl focus:border-amber-500/50"
        />
        {searchValue.length > 0 && (
          <button
            onClick={() => onSearchChange('')}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        )}
      </div>

      <div
        ref={listRef}
        onDragOver={onDragOver}
        onDrop={onDrop}
        className={cn(
          'flex flex-col gap-2 p-2 rounded-2xl border h-[250px] overflow-y-auto transition-all',
          isAvailable
            ? 'border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-card hover:border-gray-300 dark:hover:border-gray-600'
            : 'border-amber-200 dark:border-amber-800/30 bg-amber-50 dark:bg-amber-900/10 hover:border-amber-300 dark:hover:border-amber-700 ring-1 ring-amber-500/0 hover:ring-amber-500/5',
          shouldHighlightDrop && (isAvailable
            ? 'border-green-500/50 bg-green-500/5 ring-2 ring-green-500/20'
            : 'border-amber-500/50 bg-amber-500/10 ring-2 ring-amber-500/30')
        )}
        style={{ scrollbarWidth: 'thin' }}
      >
        {items.length === 0 ? (
          <div className={cn(
            "flex flex-col items-center justify-center h-full space-y-2",
            isAvailable ? "text-gray-400" : "text-amber-500/30"
          )}>
            <svg className={cn("w-8 h-8", isAvailable ? "opacity-30" : "")} fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
            <p className={cn("text-sm italic", !isAvailable && "text-amber-500/50")}>
              {searchValue.length > 0 ? 'No matches found' : emptyMessage}
            </p>
          </div>
        ) : (
          items.map((item) => (
            <div
              key={item.id}
              draggable
              onDragStart={(e) => onDragStart(e, item)}
              onDragEnd={onDragEnd}
              onTouchStart={(e) => onTouchStart(e, item)}
              onClick={() => onItemClick(item)}
              className={cn(
                'flex items-center justify-between p-3 rounded-xl border cursor-grab active:cursor-grabbing transition-all group/item select-none',
                isAvailable
                  ? 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800/50 hover:bg-amber-50 dark:hover:bg-amber-900/10 hover:border-amber-300 dark:hover:border-amber-700'
                  : 'border-amber-300 dark:border-amber-700/50 bg-amber-100 dark:bg-amber-900/20 hover:bg-amber-200 dark:hover:bg-amber-800/30 hover:border-red-300 dark:hover:border-red-700',
                draggingItem === item.id && (isAvailable
                  ? 'border-amber-500/50 bg-amber-500/5 opacity-60'
                  : 'border-red-500/50 bg-red-500/5 opacity-60')
              )}
              style={{ touchAction: 'manipulation' }}
            >
              <div className={cn("flex items-center gap-3", isAvailable ? "flex-1 min-w-0" : "min-w-0")}>
                <svg
                  className={cn(
                    "w-4 h-4 flex-shrink-0",
                    isAvailable
                      ? "text-gray-400 group-hover/item:text-amber-500/50"
                      : "text-amber-500/50 group-hover/item:text-amber-500"
                  )}
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                </svg>
                <div className="flex flex-col min-w-0">
                  <span className={cn(
                    "text-sm font-medium truncate",
                    isAvailable
                      ? "text-gray-700 dark:text-gray-300 group-hover/item:text-amber-700 dark:group-hover/item:text-amber-400"
                      : "text-amber-700 dark:text-amber-400 group-hover/item:text-gray-700 dark:group-hover/item:text-white"
                  )}>
                    {item.name}
                  </span>
                  <span className={cn(
                    "text-xs font-mono truncate",
                    isAvailable ? "text-gray-400" : "text-amber-600/60 dark:text-amber-500/60"
                  )}>{item.code}</span>
                </div>
              </div>
              <svg
                className={cn(
                  "w-4 h-4 transition-all flex-shrink-0",
                  isAvailable
                    ? "text-gray-400 group-hover/item:text-amber-500"
                    : "text-gray-400 group-hover/item:text-red-500"
                )}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d={isAvailable ? "M9 5l7 7-7 7" : "M15 19l-7-7 7-7"}
                />
              </svg>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
