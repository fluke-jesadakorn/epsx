import { Button } from '@/components/ui/button';
import { navigateToPage } from '@/lib/server-actions';
import LimitSelectorForm from './LimitSelectorForm';
import PaginationButton from './PaginationButton';
import JumpToPageForm from './JumpToPageForm';

interface ServerPaginationProps {
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  currentParams: string;
}

export default function ServerPagination({ 
  pagination, 
  currentParams 
}: ServerPaginationProps) {
  const { page, totalPages, hasNext, hasPrev, total, limit } = pagination;

  // Calculate visible page numbers
  const getVisiblePages = () => {
    const delta = 2; // Show 2 pages on each side of current page
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

  if (totalPages <= 1) return null;

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4">
      {/* Results info */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-4">
        <div className="text-sm text-gray-600">
          Showing {startItem}-{endItem} of {total} results
        </div>
        
        {/* Limit selector form */}
        <LimitSelectorForm currentParams={currentParams} currentLimit={limit} />
      </div>

      {/* Pagination controls */}
      <div className="flex items-center justify-center gap-1">
        {/* Previous button */}
        <PaginationButton
          page={page - 1}
          currentParams={currentParams}
          disabled={!hasPrev}
          className="flex items-center px-3 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px] min-w-[44px]"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m15 19-7-7 7-7" />
          </svg>
          <span className="hidden sm:block ml-1">Previous</span>
        </PaginationButton>

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
              <PaginationButton
                key={pageNum}
                page={pageNum as number}
                currentParams={currentParams}
                disabled={isCurrentPage}
                variant={isCurrentPage ? 'default' : 'outline'}
                className={`min-w-[44px] min-h-[44px] px-3 py-2 text-sm font-medium rounded-lg ${
                  isCurrentPage
                    ? 'bg-orange-500 text-white'
                    : 'text-gray-700 bg-white border border-gray-300 hover:bg-gray-50'
                }`}
              >
                {pageNum}
              </PaginationButton>
            );
          })}
        </div>

        {/* Next button */}
        <PaginationButton
          page={page + 1}
          currentParams={currentParams}
          disabled={!hasNext}
          className="flex items-center px-3 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px] min-w-[44px]"
        >
          <span className="hidden sm:block mr-1">Next</span>
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m9 5 7 7-7 7" />
          </svg>
        </PaginationButton>
      </div>

      {/* Jump to page input - hidden on mobile */}
      <div className="hidden lg:flex items-center justify-center gap-2 mt-4">
        <JumpToPageForm 
          currentParams={currentParams}
          currentPage={page}
          totalPages={totalPages}
        />
      </div>
    </div>
  );
}