import AdminPaginationButton from './AdminPaginationButton'
import AdminLimitSelector from './AdminLimitSelector'

interface PaginationInfo {
  page: number
  limit: number
  total: number
  totalPages: number
  hasNext: boolean
  hasPrev: boolean
}

interface AdminEPSPaginationProps {
  pagination: PaginationInfo
  currentParams: string
}

export default function AdminEPSPagination({ pagination, currentParams }: AdminEPSPaginationProps) {

  // Generate page numbers to show
  const getVisiblePages = () => {
    const { page, totalPages } = pagination
    const delta = 2 // Show 2 pages on each side of current page
    const range = []
    const rangeWithDots = []

    for (
      let i = Math.max(2, page - delta);
      i <= Math.min(totalPages - 1, page + delta);
      i++
    ) {
      range.push(i)
    }

    if (page - delta > 2) {
      rangeWithDots.push(1, '...')
    } else {
      rangeWithDots.push(1)
    }

    rangeWithDots.push(...range)

    if (page + delta < totalPages - 1) {
      rangeWithDots.push('...', totalPages)
    } else if (totalPages > 1) {
      rangeWithDots.push(totalPages)
    }

    return rangeWithDots
  }

  const visiblePages = getVisiblePages()
  const startItem = (pagination.page - 1) * pagination.limit + 1
  const endItem = Math.min(pagination.page * pagination.limit, pagination.total)

  if (pagination.totalPages <= 1) {
    return null
  }

  return (
    <div className="bg-white/80 dark:bg-gray-900/80 backdrop-blur-md border border-orange-200/50 dark:border-orange-400/30 rounded-xl shadow-lg p-4">
      {/* Results info */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-4 mb-4">
        <div className="text-sm text-gray-700 dark:text-slate-200 font-medium">
          Showing {startItem}-{endItem} of {pagination.total} admin results
        </div>
        
        {/* Limit selector */}
        <AdminLimitSelector currentParams={currentParams} currentLimit={pagination.limit} />
      </div>

      {/* Pagination controls */}
      <div className="flex items-center justify-center gap-1">
        {/* Previous button */}
        <AdminPaginationButton
          page={pagination.page - 1}
          currentParams={currentParams}
          disabled={!pagination.hasPrev}
          className="flex items-center px-3 py-2 text-sm font-medium text-slate-600 dark:text-slate-300 bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-orange-200 dark:border-orange-400/30 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:text-orange-700 dark:hover:text-orange-300 hover:border-orange-300 dark:hover:border-orange-400 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px] min-w-[44px] transition-all duration-200"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m15 19-7-7 7-7" />
          </svg>
          <span className="hidden sm:block ml-1">Previous</span>
        </AdminPaginationButton>

        {/* Page numbers */}
        <div className="flex items-center gap-1">
          {visiblePages.map((pageNum, index) => {
            if (pageNum === '...') {
              return (
                <span key={`dots-${index}`} className="px-2 py-2 text-gray-400">
                  ...
                </span>
              )
            }

            const isCurrentPage = pageNum === pagination.page
            
            return (
              <AdminPaginationButton
                key={pageNum}
                page={pageNum as number}
                currentParams={currentParams}
                disabled={isCurrentPage}
                className={`min-w-[44px] min-h-[44px] px-3 py-2 text-sm font-medium rounded-lg transition-all duration-200 ${
                  isCurrentPage
                    ? 'bg-gradient-to-r from-orange-500 to-pink-500 text-white shadow-lg hover:from-orange-600 hover:to-pink-600'
                    : 'text-slate-700 dark:text-slate-300 bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-orange-200 dark:border-orange-400/30 hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:text-orange-700 dark:hover:text-orange-300 hover:border-orange-300 dark:hover:border-orange-400'
                }`}
              >
                {pageNum}
              </AdminPaginationButton>
            )
          })}
        </div>

        {/* Next button */}
        <AdminPaginationButton
          page={pagination.page + 1}
          currentParams={currentParams}
          disabled={!pagination.hasNext}
          className="flex items-center px-3 py-2 text-sm font-medium text-slate-600 dark:text-slate-300 bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-orange-200 dark:border-orange-400/30 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:text-orange-700 dark:hover:text-orange-300 hover:border-orange-300 dark:hover:border-orange-400 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px] min-w-[44px] transition-all duration-200"
        >
          <span className="hidden sm:block mr-1">Next</span>
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="m9 5 7 7-7 7" />
          </svg>
        </AdminPaginationButton>
      </div>
    </div>
  )
}