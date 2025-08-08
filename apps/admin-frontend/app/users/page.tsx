import { Users, Mail, Plus, Search, Filter, AlertCircle, Loader2 } from 'lucide-react'
import Link from 'next/link'
import { getUsersList } from '@/lib/actions/unified-user-actions'
import { reloadPage } from '@/lib/actions/page-actions'
import UserListClient from './UserListClient'
import { CreateUserButton } from '@/components/users/CreateUserButton'
import { EditProfileButton } from '@/components/users/EditProfileButton'

interface PageProps {
  searchParams?: Promise<{
    search?: string
    role?: string
    page?: string
    status?: string
    modal?: string
    userId?: string
  }>
}

export default async function UsersPage({ searchParams }: PageProps) {
  // Await searchParams properly
  const params = searchParams ? await searchParams : {}
  
  const filters = {
    search: params.search || '',
    role: params.role || 'all',
    page: parseInt(params.page || '1', 10),
    limit: 20,
    sortBy: 'created_at',
    sortOrder: 'desc' as const,
    status: params.status || 'all'
  }

  const result = await getUsersList(filters)


  if (!result.success) {
    return (
      <div className="space-y-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-4">
              <div className="p-3 rounded-full bg-gradient-to-r from-blue-500 to-purple-500">
                <Users className="h-8 w-8 text-white" />
              </div>
              <div>
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                  User Management
                </h1>
                <p className="text-gray-600 dark:text-gray-400">
                  Manage user accounts and permissions
                </p>
              </div>
            </div>
            <CreateUserButton />
          </div>

          <div className="flex items-center justify-center py-12">
            <div className="text-center">
              <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                Failed to Load Users
              </h3>
              <p className="text-gray-600 dark:text-gray-400 mb-4">
                {result.error?.message || 'Unable to connect to the server. Please try again later.'}
              </p>
              <form action={reloadPage}>
                <button 
                  type="submit"
                  className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
                >
                  Try Again
                </button>
              </form>
            </div>
          </div>
        </div>
      </div>
    )
  }

  const { users, total, page, totalPages, limit } = result.data || { users: [], total: 0, page: 1, totalPages: 1, limit: 20 }
  const startIndex = (page - 1) * limit + 1
  const endIndex = Math.min(page * limit, total)

  return (
    <div className="space-y-6">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-4">
            <div className="p-3 rounded-full bg-gradient-to-r from-blue-500 to-purple-500">
              <Users className="h-8 w-8 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                User Management
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                Manage user accounts and permissions
              </p>
            </div>
          </div>
          <CreateUserButton />
        </div>

        <UserListClient currentFilters={filters} />

        <div className="bg-gray-50 dark:bg-gray-700/50 rounded-lg overflow-hidden">
          <div className="px-6 py-3 border-b border-gray-200 dark:border-gray-600">
            <div className="grid grid-cols-1 md:grid-cols-6 gap-4 text-sm font-medium text-gray-700 dark:text-gray-300">
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
                <div className="grid grid-cols-1 md:grid-cols-6 gap-4 items-center">
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
                  <div>
                    <span className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${user.status === 'active' ? 'bg-green-500' : 'bg-red-500'}`}></div>
                      <span className="text-sm text-gray-600 dark:text-gray-400 capitalize">
                        {user.status || 'inactive'}
                      </span>
                    </span>
                  </div>
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
        
        <div className="mt-6 flex justify-between items-center">
          <div className="text-sm text-gray-600 dark:text-gray-400">
            Showing {users.length > 0 ? startIndex : 0} to {endIndex} of {total} users
          </div>
          <div className="flex gap-2">
            {page > 1 && (
              <Link 
                href={`/users?page=${page - 1}&role=${filters.role}&search=${filters.search}`}
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
                    href={`/users?page=${pageNum}&role=${filters.role}&search=${filters.search}`}
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
                href={`/users?page=${page + 1}&role=${filters.role}&search=${filters.search}`}
                className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded text-sm hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                Next
              </Link>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}