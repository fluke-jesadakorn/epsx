'use client'

import { useState, useCallback, useEffect } from 'react'
import { LayoutGrid, LayoutList, Smartphone, Monitor, Wifi, WifiOff, Clock } from 'lucide-react'
import { VirtualizedUserTable } from './VirtualizedUserTable'
import { UserCard } from './UserCard'
import { BulkOperationsInterface } from './BulkOperationsInterface'
import { useRealtimeUpdates } from '../../hooks/useRealtimeUpdates'
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

  // Real-time updates with optimistic UI
  const {
    isConnected,
    connectionStatus,
    lastUpdate,
    users: realtimeUsers,
    pendingUpdates,
    addOptimisticUpdate,
    confirmUpdate,
  } = useRealtimeUpdates(users, {
    events: ['user', 'notification'],
    autoConnect: true,
    onUserStatusUpdate: (update) => {
      console.log('User status updated:', update)
      // Optional: Show toast notification
    },
    onUserProfileUpdate: (update) => {
      console.log('User profile updated:', update)
    },
    onUserRoleUpdate: (update) => {
      console.log('User role updated:', update)
    },
    onConnect: () => {
      console.log('Real-time updates connected')
    },
    onDisconnect: () => {
      console.log('Real-time updates disconnected')
    },
    onError: (error) => {
      console.error('Real-time connection error:', error)
    }
  })

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

  // Use real-time users data with optimistic updates
  const displayUsers = realtimeUsers

  // Connection status indicator
  const getConnectionStatusIcon = () => {
    switch (connectionStatus) {
      case 'connected':
        return <Wifi className="h-4 w-4 text-green-500" />
      case 'connecting':
        return <Clock className="h-4 w-4 text-yellow-500 animate-pulse" />
      case 'disconnected':
      case 'error':
        return <WifiOff className="h-4 w-4 text-red-500" />
      default:
        return <WifiOff className="h-4 w-4 text-gray-400" />
    }
  }

  const getConnectionStatusText = () => {
    switch (connectionStatus) {
      case 'connected':
        return 'Real-time updates active'
      case 'connecting':
        return 'Connecting to real-time updates...'
      case 'disconnected':
        return 'Real-time updates disconnected'
      case 'error':
        return 'Real-time connection error'
      default:
        return 'Real-time updates unavailable'
    }
  }

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
      {/* View Toggle and Selection Info */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        {/* View Mode Toggle */}
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-1">
            {isMobile ? <Smartphone className="h-4 w-4" /> : <Monitor className="h-4 w-4" />}
            View:
          </span>
          
          <div className="bg-gray-100 dark:bg-gray-800 rounded-lg p-1 flex">
            <button
              onClick={() => handleViewModeChange('table')}
              className={`px-3 py-1 rounded-md text-sm transition-colors flex items-center gap-1 ${
                viewMode === 'table'
                  ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
                  : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
              }`}
              disabled={isMobile} // Disable table view on mobile for better UX
            >
              <LayoutList className="h-4 w-4" />
              Table
            </button>
            <button
              onClick={() => handleViewModeChange('cards')}
              className={`px-3 py-1 rounded-md text-sm transition-colors flex items-center gap-1 ${
                viewMode === 'cards'
                  ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
                  : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
              }`}
            >
              <LayoutGrid className="h-4 w-4" />
              Cards
            </button>
          </div>

          {isMobile && (
            <span className="text-xs text-gray-500 bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
              Mobile optimized
            </span>
          )}
        </div>

        {/* Real-time Status & Selection Info */}
        <div className="flex items-center gap-4">
          {/* Real-time Connection Status */}
          <div className="flex items-center gap-2 text-sm">
            {getConnectionStatusIcon()}
            <span className={`${
              connectionStatus === 'connected' ? 'text-green-600' :
              connectionStatus === 'connecting' ? 'text-yellow-600' :
              'text-gray-600'
            }`}>
              {getConnectionStatusText()}
            </span>
            {pendingUpdates.length > 0 && (
              <span className="bg-blue-100 text-blue-800 px-2 py-1 rounded-full text-xs">
                {pendingUpdates.length} pending
              </span>
            )}
            {lastUpdate && connectionStatus === 'connected' && (
              <span className="text-xs text-gray-500">
                Last update: {lastUpdate.toLocaleTimeString()}
              </span>
            )}
          </div>

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
          startIndex={startIndex}
          endIndex={endIndex}
          filters={filters}
        />
      ) : (
        <div className="space-y-4">
          {/* Cards View */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {displayUsers.map((user) => (
              <div key={user.id} className="relative">
                {/* Selection Checkbox for Cards */}
                <div className="absolute top-4 left-4 z-10">
                  <input
                    type="checkbox"
                    checked={selectedUserIds.has(user.id)}
                    onChange={(e) => handleCardSelection(user.id, e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500 focus:ring-offset-0"
                    onClick={(e) => e.stopPropagation()}
                  />
                </div>
                
                {/* User Card with Selection Styling */}
                <div className={`${
                  selectedUserIds.has(user.id) 
                    ? 'ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/10' 
                    : ''
                }`}>
                  <UserCard user={user} />
                </div>
              </div>
            ))}
          </div>

          {/* Cards View Pagination */}
          <div className="flex flex-col sm:flex-row justify-between items-center gap-4 pt-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">
              Showing {displayUsers.length > 0 ? startIndex : 0} to {endIndex} of {total} users
              {selectedUserIds.size > 0 && (
                <span className="ml-2 text-blue-600">
                  • {selectedUserIds.size} selected
                </span>
              )}
              {pendingUpdates.length > 0 && (
                <span className="ml-2 text-orange-600">
                  • {pendingUpdates.length} pending updates
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
              
              {(() => {
                const maxVisiblePages = Math.min(5, totalPages)
                const startPage = Math.max(1, page - 2)
                const endPage = Math.min(totalPages, startPage + maxVisiblePages - 1)
                const pages = []
                
                for (let pageNum = startPage; pageNum <= endPage; pageNum++) {
                  pages.push(
                    <a
                      key={pageNum}
                      href={`/users?page=${pageNum}&role=${filters.role}&search=${filters.search}&status=${filters.status}`}
                      className={`px-3 py-2 rounded-lg text-sm transition-colors ${
                        pageNum === page
                          ? 'bg-blue-600 text-white'
                          : 'border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
                      }`}
                    >
                      {pageNum}
                    </a>
                  )
                }
                
                return pages
              })()}
              
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
              {isConnected && ' • Real-time updates active'}
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