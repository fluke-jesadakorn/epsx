'use client'

import { useState, useCallback, useEffect } from 'react'
import { LayoutGrid, LayoutList, Smartphone, Monitor } from 'lucide-react'
import { VirtualizedUserTable } from './VirtualizedUserTable'
import { UserCard } from './UserCard'
import { BulkOperationsInterface } from './BulkOperationsInterface'
import type { UnifiedUserData } from '@/lib/types/unified-user'

interface ResponsiveUserDisplayProps {
  users: UnifiedUserData[]
  total: number
  page: number
  totalPages: number
  limit: number
  startIndex: number
  endIndex: number
  filters: {
    search: string
    role: string
    status: string
  }
}

type ViewMode = 'table' | 'cards'

export function ResponsiveUserDisplay({
  users,
  total,
  page,
  totalPages,
  limit,
  startIndex,
  endIndex,
  filters
}: ResponsiveUserDisplayProps) {
  const [viewMode, setViewMode] = useState<ViewMode>('table')
  const [isMobile, setIsMobile] = useState(false)
  const [selectedUserIds, setSelectedUserIds] = useState<Set<string>>(new Set())
  const [showBulkOps, setShowBulkOps] = useState(false)

  // Use static users data (WebSocket removed for TradingView-only setup)
  const displayUsers = users

  // Detect screen size and set mobile mode
  useEffect(() => {
    const checkScreenSize = () => {
      const mobile = window.innerWidth < 768
      setIsMobile(mobile)
      
      // Auto-switch to cards on mobile for better UX
      if (mobile && viewMode === 'table') {
        setViewMode('cards')
      }
    }

    checkScreenSize()
    window.addEventListener('resize', checkScreenSize)
    return () => window.removeEventListener('resize', checkScreenSize)
  }, [viewMode])

  const handleViewModeChange = useCallback((mode: ViewMode) => {
    setViewMode(mode)
    // Clear selections when switching views
    setSelectedUserIds(new Set())
    setShowBulkOps(false)
  }, [])

  const handleBulkOperationComplete = useCallback(() => {
    setSelectedUserIds(new Set())
    setShowBulkOps(false)
    // Refresh the page to show updated data
    window.location.reload()
  }, [])

  const handleCardSelection = useCallback((userId: string, selected: boolean) => {
    setSelectedUserIds(prev => {
      const newSet = new Set(prev)
      if (selected) {
        newSet.add(userId)
      } else {
        newSet.delete(userId)
      }
      return newSet
    })
  }, [])

  const selectedUserIdsArray = Array.from(selectedUserIds)

  return (
    <div className="space-y-6">
      {/* Header with View Controls and Stats */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-4">
          {/* View Mode Toggle */}
          <div className="flex bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
            <button
              onClick={() => handleViewModeChange('table')}
              className={`flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                viewMode === 'table'
                  ? 'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 shadow-sm'
                  : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100'
              }`}
            >
              <LayoutList className="h-4 w-4" />
              Table
            </button>
            <button
              onClick={() => handleViewModeChange('cards')}
              className={`flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                viewMode === 'cards'
                  ? 'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 shadow-sm'
                  : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100'
              }`}
            >
              <LayoutGrid className="h-4 w-4" />
              Cards
            </button>
          </div>

          {/* Display Mode Indicator */}
          <div className="flex items-center gap-2">
            {isMobile ? (
              <Smartphone className="h-4 w-4 text-blue-500" />
            ) : (
              <Monitor className="h-4 w-4 text-blue-500" />
            )}
            <span className="text-sm font-medium">
              {startIndex + 1}-{Math.min(endIndex, total)} of {total.toLocaleString()} users
            </span>
          </div>

          {/* Mobile Optimized Badge */}
          {isMobile && (
            <span className="text-xs text-gray-500 bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
              Mobile optimized
            </span>
          )}
        </div>

        {/* Selection Info */}
        <div className="flex items-center gap-4">

          {/* Selection Info */}
          {selectedUserIds.size > 0 && (
            <div className="flex items-center gap-2 text-sm">
              <span className="text-blue-600 font-medium">
                {selectedUserIds.size} user{selectedUserIds.size !== 1 ? 's' : ''} selected
              </span>
              <button
                onClick={() => setShowBulkOps(!showBulkOps)}
                className="text-blue-600 hover:text-blue-800 underline"
              >
                {showBulkOps ? 'Hide' : 'Show'} Bulk Actions
              </button>
              <button
                onClick={() => setSelectedUserIds(new Set())}
                className="text-gray-600 hover:text-gray-800 underline"
              >
                Clear
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Bulk Operations Interface */}
      {selectedUserIds.size > 0 && showBulkOps && (
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
          <BulkOperationsInterface
            selectedUserIds={selectedUserIdsArray}
            onOperationComplete={handleBulkOperationComplete}
          />
        </div>
      )}

      {/* Content Display */}
      {viewMode === 'table' ? (
        <VirtualizedUserTable
          users={displayUsers}
          total={total}
          page={page}
          totalPages={totalPages}
          limit={limit}
          filters={filters}
          onSelectionChange={setSelectedUserIds}
          selectedUserIds={selectedUserIds}
        />
      ) : (
        <div className="space-y-4">
          {/* Card Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
            {displayUsers.map((user) => (
              <div key={user.id} className="relative">
                <UserCard user={user} />
                
                {/* Card Selection Overlay */}
                <div className="absolute top-3 left-3">
                  <input
                    type="checkbox"
                    checked={selectedUserIds.has(user.id)}
                    onChange={(e) => handleCardSelection(user.id, e.target.checked)}
                    className="h-4 w-4 text-blue-600 bg-white border-gray-300 rounded focus:ring-blue-500 focus:ring-2"
                  />
                </div>
              </div>
            ))}
          </div>

          {/* Card View Pagination */}
          <div className="flex flex-col items-center gap-4 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
            <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
              <span>
                Showing {startIndex + 1}-{Math.min(endIndex, total)} of {total.toLocaleString()} users
              </span>
              {selectedUserIds.size > 0 && (
                <span className="ml-2 text-blue-600">
                  • {selectedUserIds.size} selected
                </span>
              )}
            </div>
            
            <div className="flex flex-wrap justify-center gap-2">
              {page > 1 && (
                <a 
                  href={`/users?page=${page - 1}&role=${filters.role}&search=${filters.search}&status=${filters.status}`}
                  className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                >
                  Previous
                </a>
              )}
              
              <span className="px-3 py-2 text-sm font-medium text-gray-900 dark:text-gray-100">
                Page {page} of {totalPages}
              </span>
              
              {page < totalPages && (
                <a 
                  href={`/users?page=${page + 1}&role=${filters.role}&search=${filters.search}&status=${filters.status}`}
                  className="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                >
                  Next
                </a>
              )}
            </div>
          </div>

          {/* Mobile Performance Info */}
          {isMobile && (
            <div className="text-xs text-gray-500 text-center pt-4">
              Mobile-optimized card view • {displayUsers.length} users displayed
            </div>
          )}
        </div>
      )}

      {/* General Performance Info */}
      <div className="text-xs text-gray-400 text-center border-t pt-4">
        {viewMode === 'table' 
          ? `Virtual scrolling enabled • High performance for ${total.toLocaleString()} total users`
          : `Card view • Optimized for ${isMobile ? 'mobile' : 'desktop'} experience`
        }
      </div>
    </div>
  )
}