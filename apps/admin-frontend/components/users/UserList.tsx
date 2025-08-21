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
import { adminCardVariants, cn } from '@/design-system'

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
        <div className={cn(adminCardVariants({ variant: 'pancake' }), 'bg-info-50 border-info-200')}>
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-info-600 rounded-lg flex items-center justify-center">
              <Users className="h-4 w-4 text-white" />
            </div>
            <div>
              <h3 className="font-medium text-info-900">{viewContext.title}</h3>
              <p className="text-sm text-info-700">{viewContext.description}</p>
              <p className="text-xs text-info-600 mt-1">
                This page has been consolidated into the unified user management interface. 
                Click on any user to access their {searchParams.view} settings.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Search and Filters */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <Users className="h-5 w-5" />
            {viewContext.title} ({total.toLocaleString()})
          </h2>
          <CreateUserButton />
        </div>

        <UserListFilters currentFilters={filters} />
      </div>

      {/* User Cards */}
      {users.length > 0 ? (
        <div className="space-y-4" data-testid="user-cards-container">
          {users.map((user) => (
            <UserCard key={user.id} user={user} />
          ))}
        </div>
      ) : (
        <div className={cn(adminCardVariants({ variant: 'pancake' }), 'p-12')} data-testid="no-users-message">
          <div className="text-center text-muted-foreground">
            <Users className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <h3 className="text-lg font-medium mb-2">No users found</h3>
            <p className="text-sm">
              {filters.search || filters.status !== 'all' || filters.role !== 'all' ? (
                'Try adjusting your search or filter criteria'
              ) : (
                'No users have been added yet'
              )}
            </p>
          </div>
        </div>
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