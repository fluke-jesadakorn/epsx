import { Users, Mail, Plus, Search, Filter, AlertCircle, Loader2, Clock } from 'lucide-react'
import Link from 'next/link'
import { reloadPage } from '@/lib/actions/page-actions'
import { UserListFilters } from '@/components/users/UserListFilters'
import { ResponsiveUserDisplay } from '@/components/users/ResponsiveUserDisplay'
import { UserAnalyticsWrapper } from '@/components/users/UserAnalyticsWrapper'
import { CreateUserButton } from '@/components/users/CreateUserButton'
import { EditProfileButton } from '@/components/users/EditProfileButton'
import type { UnifiedUserData } from '@/lib/types/unified-user'

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

// Real data fetching from enhanced backend API
async function fetchUsersFromBackend(filters: {
  search: string;
  role: string;
  page: number;
  limit: number;
  sortBy: string;
  sortOrder: 'asc' | 'desc';
  status: string;
}) {
  try {
    const { getJWTFromCookies } = await import('@/lib/server/jwt');
    const token = await getJWTFromCookies();
    
    if (!token) {
      throw new Error('Authentication required');
    }
    
    const queryParams = new URLSearchParams({
      search: filters.search,
      role: filters.role,
      page: filters.page.toString(),
      limit: filters.limit.toString(),
      sortBy: filters.sortBy,
      sortOrder: filters.sortOrder,
      status: filters.status
    });

    const { env } = await import('@/config/env');
    const BACKEND_URL = env.BACKEND_URL;
    
    console.log(`Server-side fetch: ${BACKEND_URL}/api/v1/admin/users/search?${queryParams}`);
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/search?${queryParams}`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      next: { revalidate: 30 }, // Cache for 30 seconds
      signal: AbortSignal.timeout(15000), // 15 second timeout for server-side
    });

    if (!response.ok) {
      const errorText = await response.text().catch(() => 'Unknown error');
      console.error(`Backend API error: ${response.status} ${response.statusText} - ${errorText}`);
      
      throw new Error(`Backend API error: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    console.log(`Server-side fetched ${data.users?.length || 0} users successfully`);
    
    return {
      success: true,
      data: {
        users: data.users || [],
        total: data.total || 0,
        page: data.page || filters.page,
        totalPages: data.total_pages || Math.ceil((data.total || 0) / filters.limit),
        limit: data.limit || filters.limit
      }
    };
  } catch (error) {
    console.error('Server-side user fetch failed:', error);
    throw error;
  }
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

  // Fetch real data from enhanced backend API
  let result;
  let errorDetails = null;
  
  try {
    result = await fetchUsersFromBackend(filters);
  } catch (error) {
    console.error('Failed to fetch users from backend:', error);
    
    // Create a structured error response
    const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
    
    if (errorMessage.includes('Authentication')) {
      errorDetails = {
        type: 'AUTHENTICATION_ERROR',
        message: 'Please log in to view user data.',
        action: 'Redirecting to login...',
        statusCode: 401
      };
    } else if (errorMessage.includes('403') || errorMessage.includes('Forbidden')) {
      errorDetails = {
        type: 'PERMISSION_ERROR',
        message: 'You do not have permission to view user data.',
        action: 'Contact your administrator for access.',
        statusCode: 403
      };
    } else if (errorMessage.includes('timeout') || errorMessage.includes('AbortError')) {
      errorDetails = {
        type: 'TIMEOUT_ERROR',
        message: 'Request timed out. The server is taking too long to respond.',
        action: 'Please try again later.',
        statusCode: 408
      };
    } else if (errorMessage.includes('ECONNREFUSED') || errorMessage.includes('ENOTFOUND')) {
      errorDetails = {
        type: 'CONNECTION_ERROR',
        message: 'Unable to connect to the backend server.',
        action: 'Please check your connection and try again.',
        statusCode: 503
      };
    } else {
      errorDetails = {
        type: 'UNKNOWN_ERROR',
        message: 'An unexpected error occurred while loading user data.',
        action: 'Please try refreshing the page.',
        statusCode: 500
      };
    }
    
    result = {
      success: false,
      error: errorDetails
    };
  }


  if (!result.success) {
    const error = result.error || errorDetails;
    
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
            {error?.type !== 'AUTHENTICATION_ERROR' && error?.type !== 'PERMISSION_ERROR' && (
              <CreateUserButton />
            )}
          </div>

          <div className="flex items-center justify-center py-12">
            <div className="text-center max-w-md">
              {error?.type === 'AUTHENTICATION_ERROR' ? (
                <>
                  <AlertCircle className="h-12 w-12 text-yellow-500 mx-auto mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    Authentication Required
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-4">
                    {error.message}
                  </p>
                  <a 
                    href="/login"
                    className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors inline-block"
                  >
                    Go to Login
                  </a>
                </>
              ) : error?.type === 'PERMISSION_ERROR' ? (
                <>
                  <AlertCircle className="h-12 w-12 text-orange-500 mx-auto mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    Access Denied
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-4">
                    {error.message}
                  </p>
                  <p className="text-sm text-gray-500 dark:text-gray-500">
                    {error.action}
                  </p>
                </>
              ) : error?.type === 'CONNECTION_ERROR' ? (
                <>
                  <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    Connection Error
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-4">
                    {error.message}
                  </p>
                  <div className="space-y-2">
                    <form action={reloadPage}>
                      <button 
                        type="submit"
                        className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors mr-2"
                      >
                        Retry Connection
                      </button>
                    </form>
                    <p className="text-sm text-gray-500 dark:text-gray-500 mt-2">
                      {error.action}
                    </p>
                  </div>
                </>
              ) : error?.type === 'TIMEOUT_ERROR' ? (
                <>
                  <Clock className="h-12 w-12 text-yellow-500 mx-auto mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    Request Timeout
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-4">
                    {error.message}
                  </p>
                  <form action={reloadPage}>
                    <button 
                      type="submit"
                      className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
                    >
                      Try Again
                    </button>
                  </form>
                </>
              ) : (
                <>
                  <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    Failed to Load Users
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 mb-4">
                    {error?.message || 'An unexpected error occurred while loading user data.'}
                  </p>
                  <div className="space-y-2">
                    <form action={reloadPage}>
                      <button 
                        type="submit"
                        className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
                      >
                        Try Again
                      </button>
                    </form>
                    {error?.action && (
                      <p className="text-sm text-gray-500 dark:text-gray-500 mt-2">
                        {error.action}
                      </p>
                    )}
                  </div>
                </>
              )}
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

        <UserAnalyticsWrapper users={users} total={total} />

        <UserListFilters currentFilters={filters} />

        <ResponsiveUserDisplay
          users={users}
          total={total}
          page={page}
          totalPages={totalPages}
          limit={limit}
          startIndex={startIndex}
          endIndex={endIndex}
          filters={{
            search: filters.search,
            role: filters.role,
            status: filters.status
          }}
        />
      </div>
    </div>
  )
}