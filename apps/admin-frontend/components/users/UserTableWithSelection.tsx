'use client'

import { useState, useCallback } from 'react'
import { Mail, Check, Square } from 'lucide-react'
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
    role: string
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
      {/* Bulk Operations Interface */}
      {selectedUserIds.size > 0 && (
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <Check className="h-5 w-5 text-blue-600" />
              <span className="font-medium text-blue-900 dark:text-blue-100">
                {selectedUserIds.size} user{selectedUserIds.size !== 1 ? 's' : ''} selected
              </span>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowBulkOps(!showBulkOps)}
                className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors text-sm"
              >
                {showBulkOps ? 'Hide' : 'Show'} Bulk Operations
              </button>
              <button
                onClick={() => setSelectedUserIds(new Set())}
                className="text-blue-600 hover:text-blue-800 text-sm"
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

      {/* User Table */}
      <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg overflow-hidden">
        <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-600">
          <div className="grid grid-cols-1 md:grid-cols-7 gap-4 text-sm font-medium text-gray-700 dark:text-gray-300">
            <div className="flex items-center gap-2">
              <button
                onClick={handleSelectAll}
                className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                title={allSelected ? 'Deselect all' : 'Select all'}
              >
                {allSelected ? (
                  <div className="w-4 h-4 bg-blue-600 border border-blue-600 rounded flex items-center justify-center">
                    <Check className="h-3 w-3 text-white" />
                  </div>
                ) : someSelected ? (
                  <div className="w-4 h-4 bg-blue-600 border border-blue-600 rounded flex items-center justify-center">
                    <div className="w-2 h-0.5 bg-white" />
                  </div>
                ) : (
                  <Square className="h-4 w-4 text-gray-400" />
                )}
              </button>
              <span>Select</span>
            </div>
            <div className="md:col-span-2">User</div>
            <div>Role</div>
            <div>Subscription</div>
            <div>Status</div>
            <div>Actions</div>
          </div>
        </div>

        <div className="divide-y divide-gray-200 dark:divide-gray-600">
          {users.map((user) => (
            <div key={user.id} className="px-6 py-4 hover:bg-white dark:hover:bg-gray-700 transition-colors">
              <div className="grid grid-cols-1 md:grid-cols-7 gap-4 items-center">
                {/* Selection Checkbox */}
                <div className="flex items-center">
                  <button
                    onClick={() => handleUserSelect(user.id, !selectedUserIds.has(user.id))}
                    className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                    title={selectedUserIds.has(user.id) ? 'Deselect user' : 'Select user'}
                  >
                    {selectedUserIds.has(user.id) ? (
                      <div className="w-4 h-4 bg-blue-600 border border-blue-600 rounded flex items-center justify-center">
                        <Check className="h-3 w-3 text-white" />
                      </div>
                    ) : (
                      <Square className="h-4 w-4 text-gray-400" />
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

                {/* Role */}
                <div>
                  <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                    user.roles?.some(r => r.name === 'super_admin') 
                      ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-400'
                      : user.roles?.some(r => r.name === 'admin')
                      ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400'
                      : user.roles?.some(r => r.name === 'moderator')
                      ? 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900/20 dark:text-indigo-400'
                      : 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
                  }`}>
                    {user.roles?.[0]?.name || 'user'}
                  </span>
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
              href={`/users?page=${page - 1}&role=${filters.role}&search=${filters.search}&status=${filters.status}`}
              className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded text-sm hover:bg-gray-50 dark:hover:bg-gray-700"
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
                  href={`/users?page=${pageNum}&role=${filters.role}&search=${filters.search}&status=${filters.status}`}
                  className={`px-3 py-1 rounded text-sm ${
                    pageNum === page
                      ? 'bg-blue-600 text-white'
                      : 'border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'
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
              href={`/users?page=${page + 1}&role=${filters.role}&search=${filters.search}&status=${filters.status}`}
              className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded text-sm hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              Next
            </Link>
          )}
        </div>
      </div>
    </div>
  )
}