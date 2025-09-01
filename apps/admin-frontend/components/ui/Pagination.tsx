'use client'

import React from 'react'
import { ChevronLeft, ChevronRight, MoreHorizontal } from 'lucide-react'

export interface PaginationProps {
  currentPage: number
  totalPages: number
  onPageChange: (page: number) => void
  showPages?: number
  disabled?: boolean
}

/**
 * PancakeSwap x Windows Phone Pagination Component
 * Modern pagination with tile-style buttons and mobile optimization
 */
export default function Pagination({
  currentPage,
  totalPages,
  onPageChange,
  showPages = 5,
  disabled = false
}: PaginationProps) {
  if (totalPages <= 1) return null

  const getVisiblePages = () => {
    const pages: (number | 'ellipsis')[] = []
    const half = Math.floor(showPages / 2)
    
    let start = Math.max(1, currentPage - half)
    let end = Math.min(totalPages, currentPage + half)
    
    // Adjust if we're near the beginning or end
    if (currentPage <= half) {
      end = Math.min(totalPages, showPages)
    }
    if (currentPage > totalPages - half) {
      start = Math.max(1, totalPages - showPages + 1)
    }
    
    // Add first page and ellipsis
    if (start > 1) {
      pages.push(1)
      if (start > 2) {
        pages.push('ellipsis')
      }
    }
    
    // Add visible pages
    for (let i = start; i <= end; i++) {
      pages.push(i)
    }
    
    // Add ellipsis and last page
    if (end < totalPages) {
      if (end < totalPages - 1) {
        pages.push('ellipsis')
      }
      pages.push(totalPages)
    }
    
    return pages
  }

  const visiblePages = getVisiblePages()
  const hasPrevious = currentPage > 1
  const hasNext = currentPage < totalPages

  return (
    <div className="flex items-center justify-between bg-gray-50 dark:bg-gray-900 border-t border-gray-200 dark:border-gray-700 px-4 py-4 sm:px-6">
      {/* Mobile view */}
      <div className="flex flex-1 justify-between sm:hidden">
        <button
          onClick={() => onPageChange(currentPage - 1)}
          disabled={!hasPrevious || disabled}
          className="relative inline-flex items-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-white dark:disabled:hover:bg-gray-800 transition-all"
        >
          Previous
        </button>
        <button
          onClick={() => onPageChange(currentPage + 1)}
          disabled={!hasNext || disabled}
          className="relative ml-3 inline-flex items-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-white dark:disabled:hover:bg-gray-800 transition-all"
        >
          Next
        </button>
      </div>
      
      {/* Desktop view */}
      <div className="hidden sm:flex sm:flex-1 sm:items-center sm:justify-between">
        <div>
          <p className="text-sm text-gray-700 dark:text-gray-300 font-light">
            Page <span className="font-medium text-yellow-600 dark:text-yellow-400">{currentPage}</span> of{' '}
            <span className="font-medium">{totalPages}</span>
          </p>
        </div>
        
        <div>
          <nav className="isolate inline-flex -space-x-px shadow-sm" aria-label="Pagination">
            {/* Previous button */}
            <button
              onClick={() => onPageChange(currentPage - 1)}
              disabled={!hasPrevious || disabled}
              className="relative inline-flex items-center px-2 py-2 text-gray-400 dark:text-gray-500 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 focus:z-20 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-white dark:disabled:hover:bg-gray-800 transition-all"
              aria-label="Go to previous page"
            >
              <ChevronLeft className="h-5 w-5" aria-hidden="true" />
            </button>
            
            {/* Page numbers */}
            {visiblePages.map((page, index) => {
              if (page === 'ellipsis') {
                return (
                  <span
                    key={`ellipsis-${index}`}
                    className="relative inline-flex items-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600"
                  >
                    <MoreHorizontal className="h-5 w-5" aria-hidden="true" />
                  </span>
                )
              }
              
              const isCurrentPage = page === currentPage
              
              return (
                <button
                  key={page}
                  onClick={() => onPageChange(page)}
                  disabled={disabled}
                  className={`relative inline-flex items-center px-4 py-2 text-sm font-medium transition-all ${
                    isCurrentPage
                      ? 'z-10 bg-gradient-to-r from-yellow-400 to-orange-500 border-yellow-400 text-black font-medium'
                      : 'text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                  } border focus:z-20 disabled:opacity-50 disabled:cursor-not-allowed`}
                  aria-current={isCurrentPage ? 'page' : undefined}
                  aria-label={`Go to page ${page}`}
                >
                  {page}
                </button>
              )
            })}
            
            {/* Next button */}
            <button
              onClick={() => onPageChange(currentPage + 1)}
              disabled={!hasNext || disabled}
              className="relative inline-flex items-center px-2 py-2 text-gray-400 dark:text-gray-500 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 focus:z-20 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-white dark:disabled:hover:bg-gray-800 transition-all"
              aria-label="Go to next page"
            >
              <ChevronRight className="h-5 w-5" aria-hidden="true" />
            </button>
          </nav>
        </div>
      </div>
    </div>
  )
}

/**
 * Simple pagination info component for showing current results
 */
export function PaginationInfo({ 
  currentPage, 
  totalPages, 
  totalItems, 
  itemsPerPage,
  itemName = 'items' 
}: {
  currentPage: number
  totalPages: number
  totalItems: number
  itemsPerPage: number
  itemName?: string
}) {
  const start = (currentPage - 1) * itemsPerPage + 1
  const end = Math.min(currentPage * itemsPerPage, totalItems)
  
  return (
    <div className="text-sm text-gray-600 dark:text-gray-400 font-light">
      Showing <span className="font-medium text-gray-900 dark:text-white">{start}</span> to{' '}
      <span className="font-medium text-gray-900 dark:text-white">{end}</span> of{' '}
      <span className="font-medium text-gray-900 dark:text-white">{totalItems.toLocaleString()}</span> {itemName}
    </div>
  )
}