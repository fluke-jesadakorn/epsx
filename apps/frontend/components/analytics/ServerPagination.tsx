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

  // Calculate visible page numbers with cleaner logic
  const getVisiblePages = () => {
    if (totalPages <= 1) {return [1];}
    
    const delta = 2; // Show 2 pages on each side of current page
    const result: (number | string)[] = [];
    
    // For small page counts, show all pages
    if (totalPages <= 7) {
      for (let i = 1; i <= totalPages; i++) {
        result.push(i);
      }
      return result;
    }
    
    // Always show first page
    result.push(1);
    
    // Calculate the range around current page
    const startPage = Math.max(2, page - delta);
    const endPage = Math.min(totalPages - 1, page + delta);
    
    // Add dots if there's a gap after page 1
    if (startPage > 2) {
      result.push('...');
    }
    
    // Add pages around current page
    for (let i = startPage; i <= endPage; i++) {
      result.push(i);
    }
    
    // Add dots if there's a gap before last page
    if (endPage < totalPages - 1) {
      result.push('...');
    }
    
    // Always show last page
    result.push(totalPages);
    
    return result;
  };

  const visiblePages = getVisiblePages();
  const startItem = (page - 1) * limit + 1;
  const endItem = Math.min(page * limit, total);

  if (totalPages <= 1) {return null;}

  return (
    <div className="bg-white/80 dark:bg-gray-900/80 backdrop-blur-md border border-orange-200/50 dark:border-orange-400/30 rounded-xl shadow-lg p-4">
      {/* Results info */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-4">
        <div className="text-sm text-gray-700 dark:text-slate-200 font-medium">
          Showing {startItem}-{endItem} results
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
          className="flex items-center px-3 py-2 text-sm font-medium h-10 bg-white dark:bg-gray-800 text-slate-700 dark:text-slate-300 border border-orange-200 dark:border-orange-400/30 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:text-orange-700 dark:hover:text-orange-300 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m15 19-7-7 7-7" />
          </svg>
          <span className="hidden sm:block ml-1">Previous</span>
        </PaginationButton>

        {/* Page numbers */}
        <div className="flex items-center gap-1 mx-2">
          {visiblePages.map((pageNum, index) => {
            if (pageNum === '...') {
              return (
                <span 
                  key={`dots-${index}`} 
                  className="flex items-center justify-center w-10 h-10 text-gray-400 dark:text-gray-500 text-sm font-medium"
                >
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
                className={`w-10 h-10 text-sm font-medium flex items-center justify-center rounded-lg border ${
                  isCurrentPage
                    ? 'bg-orange-500 text-white border-orange-500'
                    : 'bg-white dark:bg-gray-800 text-slate-700 dark:text-slate-300 border-orange-200 dark:border-orange-400/30 hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:text-orange-700 dark:hover:text-orange-300'
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
          className="flex items-center px-3 py-2 text-sm font-medium h-10 bg-white dark:bg-gray-800 text-slate-700 dark:text-slate-300 border border-orange-200 dark:border-orange-400/30 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:text-orange-700 dark:hover:text-orange-300 disabled:opacity-50 disabled:cursor-not-allowed"
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