/**
 * Enhanced User List Component
 * Server-side filtering, sorting, and search
 */

import { Users } from 'lucide-react'
import { getUsersWithFilters } from '@/lib/actions/user-list-actions'
import { UserListFilters } from './UserListFilters'
import { UserCard } from './UserCard'
import { UserListPagination } from './UserListPagination'
import { CreateUserButton } from './CreateUserButton'
import { PancakeCard } from '@/components/ui/PancakeCard'

interface UserListProps {
  searchParams: {
    search?: string
    status?: string
    role?: string
    page?: string
    limit?: string
    sortBy?: string
    sortOrder?: string
    view?: string // Context hint for different management views
    modal?: string // Modal deep linking support
  }
}

export async function UserList({ searchParams }: UserListProps) {
  // Parse search parameters
  const filters = {
    search: searchParams.search || '',
    status: searchParams.status || 'all',
    role: searchParams.role || 'all',
    page: parseInt(searchParams.page || '1'),
    limit: parseInt(searchParams.limit || '20'),
    sortBy: searchParams.sortBy || 'createdAt',
    sortOrder: (searchParams.sortOrder || 'desc') as 'asc' | 'desc'
  }

  // Fetch users with server-side filtering
  const result = await getUsersWithFilters(filters)

  if (!result.success) {
    return (
      <div className="text-center text-error-600 py-8">
        <p>Unable to load users</p>
        {result.error && (
          <p className="text-sm text-muted-foreground mt-2">{result.error}</p>
        )}
      </div>
    )
  }

  const { users, total, page, totalPages } = result.data

  // Context-aware UI based on view parameter
  const getViewContext = () => {
    switch (searchParams.view) {
      case 'permissions':
        return {
          title: 'User Permissions Management',
          description: 'Manage user roles, permission profiles, and custom permissions',
          icon: 'shield'
        }
      case 'modules':
        return {
          title: 'User Module Access',
          description: 'Manage user access to modules and usage quotas',
          icon: 'package'
        }
      case 'billing':
        return {
          title: 'User Billing Management', 
          description: 'Manage user billing tiers and payment information',
          icon: 'creditcard'
        }
      case 'packages':
        return {
          title: 'Stock Ranking Packages',
          description: 'Manage user stock ranking package subscriptions',
          icon: 'trending'
        }
      default:
        return {
          title: 'User Management',
          description: 'Comprehensive user administration and oversight',
          icon: 'users'
        }
    }
  }

  const viewContext = getViewContext()

  return (
    <div className="space-y-6" data-testid="user-list-container">
      {/* Context Banner for Legacy Redirects */}
      {searchParams.view && (
        <PancakeCard variant="feature" className="bg-orange-50 border-orange-200 dark:bg-orange-950/50 dark:border-orange-800">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-xl flex items-center justify-center shadow-lg">
              <Users className="h-6 w-6 text-white" />
            </div>
            <div>
              <h3 className="font-bold text-lg bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent">
                {viewContext.title}
              </h3>
              <p className="text-sm text-orange-700 dark:text-orange-300 mb-1">{viewContext.description}</p>
              <p className="text-xs text-orange-600/80 dark:text-orange-400/80">
                This page has been consolidated into the unified user management interface. 
                Click on any user to access their {searchParams.view} settings.
              </p>
            </div>
          </div>
        </PancakeCard>
      )}

      {/* Search and Filters */}
      <PancakeCard variant="default">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent flex items-center gap-3">
            <div className="w-8 h-8 bg-gradient-to-br from-orange-400 to-yellow-400 rounded-lg flex items-center justify-center">
              <Users className="h-4 w-4 text-white" />
            </div>
            {viewContext.title} ({total.toLocaleString()})
          </h2>
          <CreateUserButton />
        </div>

        <UserListFilters currentFilters={filters} />
      </PancakeCard>

      {/* User Cards */}
      {users.length > 0 ? (
        <div className="space-y-4" data-testid="user-cards-container">
          {users.map((user) => (
            <UserCard key={user.id} user={user} />
          ))}
        </div>
      ) : (
        <PancakeCard variant="feature" className="p-12" data-testid="no-users-message">
          <div className="text-center">
            <div className="w-16 h-16 bg-gradient-to-br from-orange-100 to-yellow-100 dark:from-orange-900/50 dark:to-yellow-900/50 rounded-2xl mx-auto mb-4 flex items-center justify-center">
              <Users className="h-8 w-8 text-orange-400/60 dark:text-orange-500/60" />
            </div>
            <h3 className="text-xl font-bold bg-gradient-to-r from-orange-600 to-yellow-600 bg-clip-text text-transparent mb-2">
              No users found
            </h3>
            <p className="text-orange-600/80 dark:text-orange-400/80">
              {filters.search || filters.status !== 'all' || filters.role !== 'all' ? (
                'Try adjusting your search or filter criteria'
              ) : (
                'No users have been added yet'
              )}
            </p>
          </div>
        </PancakeCard>
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <UserListPagination
          currentPage={page}
          totalPages={totalPages}
          total={total}
          limit={filters.limit}
        />
      )}

      {/* Modal Manager for Deep Linking */}
    </div>
  )
}