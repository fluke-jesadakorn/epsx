'use client';

import { memo } from 'react';
import type { PaginationProps } from '@/types/analytics';
import LimitSelector from './limit-selector';

const Pagination = memo<PaginationProps>(({ pagination, onPageChange, onLimitChange, isLoading }) => {
  const { page, totalPages, hasNext, hasPrev, total, limit } = pagination;

  // Calculate visible page numbers for mobile
  const getVisiblePages = () => {
    const delta = 1; // Show 1 page on each side of current page on mobile
    const range = [];
    const rangeWithDots = [];

    for (
      let i = Math.max(2, page - delta);
      i <= Math.min(totalPages - 1, page + delta);
      i++
    ) {
      range.push(i);
    }

    if (page - delta > 2) {
      rangeWithDots.push(1, '...');
    } else {
      rangeWithDots.push(1);
    }

    rangeWithDots.push(...range);

    if (page + delta < totalPages - 1) {
      rangeWithDots.push('...', totalPages);
    } else if (totalPages > 1) {
      rangeWithDots.push(totalPages);
    }

    return rangeWithDots;
  };

  const visiblePages = getVisiblePages();
  const startItem = (page - 1) * limit + 1;
  const endItem = Math.min(page * limit, total);

  if (totalPages <= 1) {return null;}

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4">
      {/* Results info and limit selector */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-4">
        <div className="text-sm text-gray-600">
          Showing {startItem}-{endItem} of {total} results
        </div>
        {onLimitChange && (
          <LimitSelector
            currentLimit={limit}
            onLimitChange={onLimitChange}
            isLoading={isLoading}
          />
        )}
      </div>

      {/* Pagination controls */}
      <div className="flex items-center justify-center gap-1">
        {/* Previous button */}
        <button
          onClick={() => onPageChange(page - 1)}
          disabled={!hasPrev || isLoading}
          className="flex items-center px-3 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px] min-w-[44px]"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m15 19-7-7 7-7" />
          </svg>
          <span className="hidden sm:block ml-1">Previous</span>
        </button>

        {/* Page numbers */}
        <div className="flex items-center gap-1">
          {visiblePages.map((pageNum, index) => {
            if (pageNum === '...') {
              return (
                <span key={`dots-${index}`} className="px-2 py-2 text-gray-400">
                  ...
                </span>
              );
            }

            const isCurrentPage = pageNum === page;
            return (
              <button
                key={pageNum}
                onClick={() => onPageChange(pageNum as number)}
                disabled={isLoading}
                className={`min-w-[44px] min-h-[44px] px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                  isCurrentPage
                    ? 'bg-orange-500 text-white'
                    : 'text-gray-700 bg-white border border-gray-300 hover:bg-gray-50'
                } disabled:opacity-50 disabled:cursor-not-allowed`}
              >
                {pageNum}
              </button>
            );
          })}
        </div>

        {/* Next button */}
        <button
          onClick={() => onPageChange(page + 1)}
          disabled={!hasNext || isLoading}
          className="flex items-center px-3 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px] min-w-[44px]"
        >
          <span className="hidden sm:block mr-1">Next</span>
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m9 5 7 7-7 7" />
          </svg>
        </button>
      </div>

      {/* Jump to page input - hidden on mobile to save space */}
      <div className="hidden lg:flex items-center justify-center gap-2 mt-4">
        <span className="text-sm text-gray-600">Go to page:</span>
        <input
          type="number"
          min={1}
          max={totalPages}
          className="w-16 px-2 py-1 text-sm border border-gray-300 rounded text-center"
          onKeyPress={(e) => {
            if (e.key === 'Enter') {
              const value = parseInt((e.target as HTMLInputElement).value);
              if (value >= 1 && value <= totalPages) {
                onPageChange(value);
                (e.target as HTMLInputElement).value = '';
              }
            }
          }}
          disabled={isLoading}
        />
      </div>
    </div>
  );
});

Pagination.displayName = 'pagination';

export default Pagination;