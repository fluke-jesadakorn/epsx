/**
 * User List Pagination Component
 * Server-side pagination controls
 */

'use client'

import { useRouter, useSearchParams } from 'next/navigation'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { adminCardVariants, adminButtonVariants, cn } from '@/design-system'

interface UserListPaginationProps {
  currentPage: number
  totalPages: number
  total: number
  limit: number
}

export function UserListPagination({ currentPage, totalPages, total, limit }: UserListPaginationProps) {
  const router = useRouter()
  const searchParams = useSearchParams()

  const navigateToPage = (page: number) => {
    const params = new URLSearchParams(searchParams?.toString())
    if (page === 1) {
      params.delete('page')
    } else {
      params.set('page', page.toString())
    }
    
    const newURL = params.toString() ? `?${params.toString()}` : '/users'
    router.push(newURL)
  }

  const getPageNumbers = () => {
    const delta = 2 // Number of pages to show on each side of current page
    const pages: (number | string)[] = []
    
    // Always show first page
    pages.push(1)
    
    // Show pages around current page
    const start = Math.max(2, currentPage - delta)
    const end = Math.min(totalPages - 1, currentPage + delta)
    
    if (start > 2) {
      pages.push('...')
    }
    
    for (let i = start; i <= end; i++) {
      pages.push(i)
    }
    
    if (end < totalPages - 1) {
      pages.push('...')
    }
    
    // Always show last page (if different from first)
    if (totalPages > 1) {
      pages.push(totalPages)
    }
    
    return pages.filter((page, index, array) => 
      // Remove duplicate pages
      page !== array[index - 1]
    )
  }

  const startItem = (currentPage - 1) * limit + 1
  const endItem = Math.min(currentPage * limit, total)

  return (
    <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
      <div className="flex items-center justify-between">
        {/* Results info */}
        <div className="text-sm text-muted-foreground">
          Showing {startItem.toLocaleString()} to {endItem.toLocaleString()} of {total.toLocaleString()} users
        </div>

        {/* Pagination controls */}
        <div className="flex items-center gap-1">
          {/* Previous button */}
          <button
            onClick={() => navigateToPage(currentPage - 1)}
            disabled={currentPage === 1}
            className="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <ChevronLeft className="h-4 w-4" />
          </button>

          {/* Page numbers */}
          <div className="flex items-center gap-1">
            {getPageNumbers().map((page, index) => (
              <div key={`page-${page}-${index}`}>
                {page === '...' ? (
                  <span className="px-3 py-2 text-muted-foreground">...</span>
                ) : (
                  <button
                    onClick={() => navigateToPage(page as number)}
                    className={`px-3 py-2 text-sm rounded-lg transition-colors ${
                      page === currentPage
                        ? 'bg-blue-600 text-white'
                        : 'text-muted-foreground hover:text-foreground hover:bg-muted'
                    }`}
                  >
                    {page}
                  </button>
                )}
              </div>
            ))}
          </div>

          {/* Next button */}
          <button
            onClick={() => navigateToPage(currentPage + 1)}
            disabled={currentPage === totalPages}
            className="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>
      </div>
    </div>
  )
}