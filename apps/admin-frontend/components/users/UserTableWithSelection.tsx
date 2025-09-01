'use client'

import { useState, useCallback } from 'react'
import { Mail, Check, Square, Activity, CheckCircle2 } from 'lucide-react'
import Link from 'next/link'
import { EditProfileButton } from './EditProfileButton'
import { BulkOperationsInterface } from './BulkOperationsInterface'
import type { UnifiedUserData } from '@/lib/types/unified-user'

interface UserTableWithSelectionProps {
  users: UnifiedUserData[]
  total: number
  page: number
  totalPages: number
  limit: number
  startIndex: number
  endIndex: number
  filters: {
    search: string
    permissions: string
    status: string
  }
}

export function UserTableWithSelection({
  users,
  total,
  page,
  totalPages,
  limit,
  startIndex,
  endIndex,
  filters
}: UserTableWithSelectionProps) {
  const [selectedUserIds, setSelectedUserIds] = useState<Set<string>>(new Set())
  const [showBulkOps, setShowBulkOps] = useState(false)

  const handleUserSelect = useCallback((userId: string, selected: boolean) => {
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

  const handleSelectAll = useCallback(() => {
    if (selectedUserIds.size === users.length) {
      // Deselect all
      setSelectedUserIds(new Set())
    } else {
      // Select all current page users
      setSelectedUserIds(new Set(users.map(user => user.id)))
    }
  }, [users, selectedUserIds.size])

  const handleBulkOperationComplete = useCallback(() => {
    // Clear selections after bulk operation
    setSelectedUserIds(new Set())
    // Refresh the page to show updated data
    window.location.reload()
  }, [])

  const selectedUserIdsArray = Array.from(selectedUserIds)
  const allSelected = users.length > 0 && selectedUserIds.size === users.length
  const someSelected = selectedUserIds.size > 0 && selectedUserIds.size < users.length

  return (
    <div className="space-y-6">
      {/* Windows Phone Bulk Operations Interface */}
      {selectedUserIds.size > 0 && (
        <div className="bg-gradient-to-r from-yellow-50 via-orange-50 to-yellow-100 dark:from-gray-800 dark:via-orange-900/20 dark:to-gray-700 border-2 border-yellow-400/60 dark:border-orange-800/40 rounded-2xl p-6 shadow-lg hover:shadow-xl transition-all duration-300">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 bg-gradient-to-br from-yellow-400 to-orange-400 rounded-full flex items-center justify-center shadow-lg">
                <CheckCircle2 className="h-5 w-5 text-white" />
              </div>
              <span className="font-light text-lg text-foreground uppercase tracking-wider">
                {selectedUserIds.size} user{selectedUserIds.size !== 1 ? 's' : ''} selected
              </span>
            </div>
            <div className="flex items-center gap-3">
              <button
                onClick={() => setShowBulkOps(!showBulkOps)}
                className="bg-gradient-to-r from-yellow-400 to-orange-400 text-black px-6 py-3 rounded-xl hover:from-yellow-500 hover:to-orange-500 transition-all duration-300 font-medium hover:scale-105 hover:shadow-lg active:scale-95 min-h-[44px] uppercase tracking-wide text-sm"
              >
                {showBulkOps ? 'Hide' : 'Show'} Bulk Operations
              </button>
              <button
                onClick={() => setSelectedUserIds(new Set())}
                className="text-yellow-600 hover:text-yellow-800 dark:text-yellow-400 dark:hover:text-yellow-300 font-light uppercase tracking-wider text-sm min-h-[44px] px-4 hover:bg-yellow-100/50 dark:hover:bg-yellow-900/20 rounded-xl transition-all duration-300"
              >
                Clear Selection
              </button>
            </div>
          </div>
          
          {showBulkOps && (
            <BulkOperationsInterface
              selectedUserIds={selectedUserIdsArray}
              onOperationComplete={handleBulkOperationComplete}
            />
          )}
        </div>
      )}

      {/* Windows Phone User Table */}
      <div className="bg-gradient-to-br from-card via-card to-yellow-50/30 dark:from-gray-800 dark:via-gray-700 dark:to-orange-900/10 rounded-2xl overflow-hidden shadow-lg border border-yellow-200/40 dark:border-orange-800/30">
        <div className="px-6 py-4 border-b border-yellow-200/60 dark:border-orange-800/40 bg-yellow-50/50 dark:bg-orange-900/10">
          <div className="grid grid-cols-1 md:grid-cols-7 gap-4 text-sm font-light text-foreground uppercase tracking-wider">
            <div className="flex items-center gap-3">
              <button
                onClick={handleSelectAll}
                className="min-h-[44px] min-w-[44px] rounded-full hover:bg-yellow-100/50 dark:hover:bg-orange-900/30 transition-all duration-300 hover:scale-105 active:scale-95 flex items-center justify-center group"
                title={allSelected ? 'Deselect all' : 'Select all'}
              >
                {allSelected ? (
                  <div className="w-8 h-8 bg-gradient-to-br from-yellow-400 to-orange-400 border-2 border-yellow-500 rounded-full flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 group-hover:scale-110">
                    <Check className="h-5 w-5 text-white font-bold" />
                  </div>
                ) : someSelected ? (
                  <div className="w-8 h-8 bg-gradient-to-br from-yellow-400/80 to-orange-400/80 border-2 border-yellow-500 rounded-full flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 group-hover:scale-110">
                    <div className="w-4 h-1 bg-white rounded-full" />
                  </div>
                ) : (
                  <div className="w-8 h-8 border-2 border-yellow-300/60 dark:border-orange-700/60 rounded-full flex items-center justify-center hover:border-yellow-400 dark:hover:border-orange-600 transition-all duration-300 group-hover:scale-110">
                    <div className="w-6 h-6 rounded-full border border-current opacity-40" />
                  </div>
                )}
              </button>
              <span>Select</span>
            </div>
            <div className="md:col-span-2">User</div>
            <div>Permissions</div>
            <div>Subscription</div>
            <div>Status</div>
            <div>Actions</div>
          </div>
        </div>

        <div className="divide-y divide-yellow-200/40 dark:divide-orange-800/30">
          {users.map((user) => (
            <div key={user.id} className={`px-6 py-4 transition-all duration-300 hover:bg-gradient-to-r hover:from-yellow-50/50 hover:to-orange-50/30 dark:hover:from-orange-900/10 dark:hover:to-yellow-900/10 ${
              selectedUserIds.has(user.id) 
                ? 'bg-gradient-to-r from-yellow-100/80 to-orange-100/60 dark:from-orange-900/30 dark:to-yellow-900/20 ring-2 ring-yellow-400/50 dark:ring-orange-600/50 shadow-lg scale-[1.01] hover:scale-[1.02]'
                : 'hover:scale-[1.005]'
            }`}>
              <div className="grid grid-cols-1 md:grid-cols-7 gap-4 items-center">
                {/* Windows Phone Selection Indicator */}
                <div className="flex items-center">
                  <button
                    onClick={() => handleUserSelect(user.id, !selectedUserIds.has(user.id))}
                    className="min-h-[44px] min-w-[44px] rounded-full hover:bg-yellow-100/50 dark:hover:bg-orange-900/30 transition-all duration-300 hover:scale-105 active:scale-95 flex items-center justify-center group"
                    title={selectedUserIds.has(user.id) ? 'Deselect user' : 'Select user'}
                  >
                    {selectedUserIds.has(user.id) ? (
                      <div className="w-8 h-8 bg-gradient-to-br from-yellow-400 to-orange-400 border-2 border-yellow-500 rounded-full flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 group-hover:scale-110 animate-pulse">
                        <Check className="h-5 w-5 text-white font-bold" />
                      </div>
                    ) : (
                      <div className="w-8 h-8 border-2 border-yellow-300/60 dark:border-orange-700/60 rounded-full flex items-center justify-center hover:border-yellow-400 dark:hover:border-orange-600 transition-all duration-300 group-hover:scale-110">
                        <div className="w-6 h-6 rounded-full border border-current opacity-40" />
                      </div>
                    )}
                  </button>
                </div>

                {/* User Info */}
                <div className="md:col-span-2 flex items-center gap-3">
                  <div className="w-10 h-10 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 flex items-center justify-center text-white font-medium">
                    {user.email.charAt(0).toUpperCase()}
                  </div>
                  <div>
                    <div className="font-medium text-gray-900 dark:text-white">{user.email}</div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-1">
                      <Mail className="h-3 w-3" />
                      ID: {user.id.slice(0, 8)}...
                    </div>
                  </div>
                </div>

                {/* Permissions */}
                <div className="flex flex-wrap gap-1 text-xs">
                  {(() => {
                    const adminPerms = user.permissions?.filter(p => p.startsWith('admin:')).length || 0
                    const platformPerms = user.permissions?.filter(p => p.startsWith('epsx:')).length || 0
                    const totalPerms = user.permissions?.length || 0
                    
                    if (totalPerms === 0) {
                      return (
                        <span className="px-2 py-1 rounded-full bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400">
                          No Permissions
                        </span>
                      )
                    }
                    
                    return (
                      <div className="flex flex-col gap-1">
                        {adminPerms > 0 && (
                          <span className="px-2 py-1 rounded-full bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400">
                            Admin ({adminPerms})
                          </span>
                        )}
                        {platformPerms > 0 && (
                          <span className="px-2 py-1 rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400">
                            Platform ({platformPerms})
                          </span>
                        )}
                      </div>
                    )
                  })()}
                </div>

                {/* Subscription */}
                <div>
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                    user.billing?.tier === 'premium'
                      ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
                      : user.billing?.tier === 'basic'
                      ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                      : 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
                  }`}>
                    {user.billing?.tier || 'basic'}
                  </span>
                </div>

                {/* Status */}
                <div>
                  <span className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${user.status === 'active' ? 'bg-green-500' : 'bg-red-500'}`}></div>
                    <span className="text-sm text-gray-600 dark:text-gray-400 capitalize">
                      {user.status || 'inactive'}
                    </span>
                  </span>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2">
                  <Link 
                    href={`/users/${user.id}`}
                    className="text-blue-600 hover:text-blue-800 text-sm"
                  >
                    View
                  </Link>
                  <EditProfileButton 
                    userId={user.id}
                    className="!px-2 !py-1 !text-xs"
                  />
                  <Link
                    href={`/users/${user.id}/activity`}
                    className="px-3 py-2 text-xs bg-gradient-to-r from-yellow-400 to-orange-400 hover:from-yellow-500 hover:to-orange-500 text-black font-medium transition-all duration-300 flex items-center gap-1 rounded-xl hover:scale-105 hover:shadow-lg active:scale-95 min-h-[44px] uppercase tracking-wide"
                    title="View Activity Logs"
                  >
                    <Activity className="w-4 h-4" />
                    Activity
                  </Link>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
      
      {/* Pagination */}
      <div className="mt-6 flex justify-between items-center">
        <div className="text-sm text-gray-600 dark:text-gray-400">
          Showing {users.length > 0 ? startIndex : 0} to {endIndex} of {total} users
        </div>
        <div className="flex gap-2">
          {page > 1 && (
            <Link 
              href={`/users?page=${page - 1}&permissions=${filters.permissions}&search=${filters.search}&status=${filters.status}`}
              className="px-4 py-2 border-2 border-yellow-300/60 dark:border-orange-700/60 rounded-xl text-sm hover:bg-yellow-50 dark:hover:bg-orange-900/20 hover:border-yellow-400 dark:hover:border-orange-600 transition-all duration-300 hover:scale-105 active:scale-95 font-light uppercase tracking-wider min-h-[44px] flex items-center justify-center"
            >
              Previous
            </Link>
          )}
          
          {(() => {
            const maxVisiblePages = Math.min(5, totalPages)
            const startPage = Math.max(1, page - 2)
            const endPage = Math.min(totalPages, startPage + maxVisiblePages - 1)
            const pages = []
            
            for (let pageNum = startPage; pageNum <= endPage; pageNum++) {
              pages.push(
                <Link
                  key={pageNum}
                  href={`/users?page=${pageNum}&permissions=${filters.permissions}&search=${filters.search}&status=${filters.status}`}
                  className={`px-4 py-2 rounded-xl text-sm transition-all duration-300 hover:scale-105 active:scale-95 font-light min-h-[44px] flex items-center justify-center ${
                    pageNum === page
                      ? 'bg-gradient-to-r from-yellow-400 to-orange-400 text-black font-medium shadow-lg hover:shadow-xl hover:from-yellow-500 hover:to-orange-500'
                      : 'border-2 border-yellow-300/60 dark:border-orange-700/60 hover:bg-yellow-50 dark:hover:bg-orange-900/20 hover:border-yellow-400 dark:hover:border-orange-600'
                  }`}
                >
                  {pageNum}
                </Link>
              )
            }
            
            return pages
          })()}
          
          {page < totalPages && (
            <Link 
              href={`/users?page=${page + 1}&permissions=${filters.permissions}&search=${filters.search}&status=${filters.status}`}
              className="px-4 py-2 border-2 border-yellow-300/60 dark:border-orange-700/60 rounded-xl text-sm hover:bg-yellow-50 dark:hover:bg-orange-900/20 hover:border-yellow-400 dark:hover:border-orange-600 transition-all duration-300 hover:scale-105 active:scale-95 font-light uppercase tracking-wider min-h-[44px] flex items-center justify-center"
            >
              Next
            </Link>
          )}
        </div>
      </div>

    </div>
  )
}