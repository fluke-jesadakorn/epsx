'use client'

import { useState, useCallback, useMemo } from 'react'
import { Mail, Check, Square, ExternalLink } from 'lucide-react'
import Link from 'next/link'
import { VirtualTable, type VirtualTableColumn } from '@/components/ui/virtual-scroll'
import { EditProfileButton } from './EditProfileButton'
import { BulkOperationsInterface } from './BulkOperationsInterface'
import type { UnifiedUserData } from '@/lib/types/unified-user'

interface VirtualizedUserTableProps {
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

// Extend UnifiedUserData to work with VirtualTable
type VirtualUserData = UnifiedUserData & { id: string }

export function VirtualizedUserTable({
  users,
  total,
  page,
  totalPages,
  limit,
  startIndex,
  endIndex,
  filters
}: VirtualizedUserTableProps) {
  const [selectedUserIds, setSelectedUserIds] = useState<Set<string>>(new Set())
  const [showBulkOps, setShowBulkOps] = useState(false)
  const [sortBy, setSortBy] = useState<string>('created_at')
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('desc')

  // Convert users to virtual table format
  const virtualUsers: VirtualUserData[] = useMemo(() => 
    users.map(user => ({ ...user, id: user.id })),
    [users]
  )

  // Define table columns with custom rendering
  const columns: VirtualTableColumn<VirtualUserData>[] = useMemo(() => [
    {
      key: 'email',
      header: 'User',
      width: '300px',
      sortable: true,
      render: (value: string, user: VirtualUserData) => (
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 flex items-center justify-center text-white font-medium text-sm">
            {user.email.charAt(0).toUpperCase()}
          </div>
          <div>
            <div className="font-medium text-gray-900 dark:text-white text-sm">{user.email}</div>
            <div className="text-xs text-gray-600 dark:text-gray-400 flex items-center gap-1">
              <Mail className="h-3 w-3" />
              ID: {user.id.slice(0, 8)}...
            </div>
          </div>
        </div>
      )
    },
    {
      key: 'roles',
      header: 'Role',
      width: '120px',
      sortable: true,
      render: (roles: any[], user: VirtualUserData) => {
        const primaryRole = roles?.[0]?.name || 'user'
        return (
          <span className={`px-2 py-1 rounded-full text-xs font-medium ${
            primaryRole === 'super_admin' 
              ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-400'
              : primaryRole === 'admin'
              ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400'
              : primaryRole === 'moderator'
              ? 'bg-indigo-100 text-indigo-800 dark:bg-indigo-900/20 dark:text-indigo-400'
              : 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
          }`}>
            {primaryRole}
          </span>
        )
      }
    },
    {
      key: 'billing',
      header: 'Subscription',
      width: '120px',
      sortable: true,
      render: (billing: any, user: VirtualUserData) => {
        const tier = billing?.tier || 'basic'
        return (
          <span className={`px-2 py-1 rounded-full text-xs font-medium ${
            tier === 'premium'
              ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
              : tier === 'enterprise'
              ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/20 dark:text-purple-400'
              : 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
          }`}>
            {tier}
          </span>
        )
      }
    },
    {
      key: 'status',
      header: 'Status',
      width: '100px',
      sortable: true,
      render: (status: string) => (
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${
            status === 'active' ? 'bg-green-500' : 
            status === 'pending' ? 'bg-yellow-500' :
            status === 'suspended' ? 'bg-orange-500' :
            'bg-red-500'
          }`}></div>
          <span className="text-sm text-gray-600 dark:text-gray-400 capitalize">
            {status || 'inactive'}
          </span>
        </div>
      )
    },
    {
      key: 'lastLogin',
      header: 'Last Login',
      width: '140px',
      sortable: true,
      render: (lastLogin: Date | null) => (
        <span className="text-sm text-gray-600 dark:text-gray-400">
          {lastLogin 
            ? new Date(lastLogin).toLocaleDateString('en-US', { 
                month: 'short', 
                day: 'numeric',
                year: lastLogin.getFullYear() !== new Date().getFullYear() ? 'numeric' : undefined
              })
            : 'Never'
          }
        </span>
      )
    },
    {
      key: 'usageMetrics',
      header: 'API Usage',
      width: '100px',
      render: (metrics: any) => (
        <div className="text-xs text-gray-600 dark:text-gray-400">
          <div>{metrics?.apiCallsThisMonth || 0}/mo</div>
          <div className="text-gray-500">{metrics?.sessionsThisMonth || 0} sessions</div>
        </div>
      )
    },
    {
      key: 'id',
      header: 'Actions',
      width: '120px',
      render: (_, user: VirtualUserData) => (
        <div className="flex items-center gap-2">
          <Link 
            href={`/users/${user.id}`}
            className="text-blue-600 hover:text-blue-800 text-sm flex items-center gap-1"
            onClick={(e) => e.stopPropagation()}
          >
            <ExternalLink className="h-3 w-3" />
            View
          </Link>
          <EditProfileButton 
            userId={user.id}
            className="!px-2 !py-1 !text-xs"
            onClick={(e) => e?.stopPropagation()}
          />
        </div>
      )
    }
  ], [])

  const handleSort = useCallback((column: string, direction: 'asc' | 'desc') => {
    setSortBy(column)
    setSortDirection(direction)
    // In a real implementation, this would trigger a server-side sort
    // For now, we'll just update the UI state
  }, [])

  const handleBulkOperationComplete = useCallback(() => {
    setSelectedUserIds(new Set())
    setShowBulkOps(false)
    // Refresh the page to show updated data
    window.location.reload()
  }, [])

  const selectedUserIdsArray = Array.from(selectedUserIds)

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

      {/* Virtualized Table */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden">
        <VirtualTable<VirtualUserData>
          items={virtualUsers}
          columns={columns}
          rowHeight={64} // Larger row height for user avatars and two lines of text
          containerHeight={600} // Fixed height for virtual scrolling
          selectedItems={selectedUserIds}
          onSelectionChange={setSelectedUserIds}
          sortBy={sortBy}
          sortDirection={sortDirection}
          onSort={handleSort}
          className="border-0"
          onRowClick={(user) => {
            // Optional: navigate to user detail on row click
            // router.push(`/users/${user.id}`)
          }}
        />
      </div>

      {/* Pagination Info and Controls */}
      <div className="flex justify-between items-center">
        <div className="text-sm text-gray-600 dark:text-gray-400">
          Showing {users.length > 0 ? startIndex : 0} to {endIndex} of {total} users
          {selectedUserIds.size > 0 && (
            <span className="ml-2 text-blue-600">
              • {selectedUserIds.size} selected
            </span>
          )}
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

      {/* Performance Info */}
      <div className="text-xs text-gray-500 text-center">
        Virtual scrolling enabled • Rendering {Math.min(10, users.length)} of {users.length} rows at a time
      </div>
    </div>
  )
}