import LimitSelectorForm from './limit-selector-form';
import PaginationButton from './pagination-button';
import JumpToPageForm from './jump-to-page-form';

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

  const getVisiblePages = () => {
    if (totalPages <= 1) {return [1];}
    const delta = 2;
    const result: (number | string)[] = [];
    if (totalPages <= 7) {
      for (let i = 1; i <= totalPages; i++) { result.push(i); }
      return result;
    }
    result.push(1);
    const startPage = Math.max(2, page - delta);
    const endPage = Math.min(totalPages - 1, page + delta);
    if (startPage > 2) { result.push('...'); }
    for (let i = startPage; i <= endPage; i++) { result.push(i); }
    if (endPage < totalPages - 1) { result.push('...'); }
    result.push(totalPages);
    return result;
  };

  const visiblePages = getVisiblePages();
  const startItem = (page - 1) * limit + 1;
  const endItem = Math.min(page * limit, total);

  if (totalPages <= 1) {return null;}

  return (
    <div className="rounded-xl border border-gray-200 dark:border-border bg-white dark:bg-card backdrop-blur-sm p-4">
      {/* Results info */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3 mb-4">
        <span className="text-sm text-slate-400">
          Showing <span className="font-medium text-slate-200">{startItem}-{endItem}</span> of {total}
        </span>
        <LimitSelectorForm currentParams={currentParams} currentLimit={limit} />
      </div>

      {/* Page numbers */}
      <div className="flex items-center justify-center gap-1">
        <PaginationButton
          page={page - 1}
          currentParams={currentParams}
          disabled={!hasPrev}
          className="flex items-center gap-1 px-3 py-2 text-sm font-medium h-9 rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 text-slate-300 hover:bg-slate-700/60 hover:text-white disabled:opacity-30 disabled:pointer-events-none transition-colors"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m15 19-7-7 7-7" />
          </svg>
          <span className="hidden sm:block">Prev</span>
        </PaginationButton>

        <div className="flex items-center gap-1 mx-1">
          {visiblePages.map((pageNum) => {
            if (pageNum === '...') {
              return (
                <span key={`dots-${page}`} className="flex items-center justify-center w-9 h-9 text-slate-500 text-sm">
                  ...
                </span>
              );
            }
            const isCurrent = pageNum === page;
            return (
              <PaginationButton
                key={`page-${pageNum}`}
                page={pageNum as number}
                currentParams={currentParams}
                disabled={isCurrent}
                className={`w-9 h-9 text-sm font-medium flex items-center justify-center rounded-lg border transition-colors ${
                  isCurrent
                    ? 'bg-purple-600 text-white border-purple-500'
                    : 'border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 text-slate-300 hover:bg-slate-700/60 hover:text-white'
                }`}
              >
                {pageNum}
              </PaginationButton>
            );
          })}
        </div>

        <PaginationButton
          page={page + 1}
          currentParams={currentParams}
          disabled={!hasNext}
          className="flex items-center gap-1 px-3 py-2 text-sm font-medium h-9 rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 text-slate-300 hover:bg-slate-700/60 hover:text-white disabled:opacity-30 disabled:pointer-events-none transition-colors"
        >
          <span className="hidden sm:block">Next</span>
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m9 5 7 7-7 7" />
          </svg>
        </PaginationButton>
      </div>

      {/* Jump to page */}
      <div className="hidden lg:flex items-center justify-center gap-2 mt-3">
        <JumpToPageForm
          currentParams={currentParams}
          currentPage={page}
          totalPages={totalPages}
        />
      </div>
    </div>
  );
}
