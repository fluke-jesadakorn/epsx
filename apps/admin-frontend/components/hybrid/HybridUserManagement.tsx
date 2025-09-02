/**
 * Hybrid User Management Component - Admin Frontend Example
 * Demonstrates optimal serverless pattern with OIDC authentication
 * Server Component initial load + Client-side management + Server Actions for navigation
 */

import { Suspense } from 'react'
import { adminClientData, type AdminFilters } from '@/lib/admin-client-data'
import { adminServerData } from '@/lib/admin-server-data'

// ============================================================================
// Server Component - Initial User Data
// ============================================================================

interface UserManagementServerDataProps {
  initialFilters: AdminFilters
}

/**
 * Server Component for initial user data loading
 * Optimized for serverless with direct database access
 */
async function UserManagementServerData({ initialFilters }: UserManagementServerDataProps) {
  const userData = await adminServerData.getUsers(initialFilters)
  
  if (!userData?.users) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load initial user data</p>
      </div>
    )
  }
  
  return (
    <div className="space-y-4">
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h3 className="font-semibold text-blue-900">Server-loaded User Data</h3>
        <p className="text-blue-700">
          Found {userData.total} users, showing page {userData.page}
        </p>
        <p className="text-sm text-blue-500">Loaded server-side for SEO and performance</p>
      </div>
      
      <div className="overflow-x-auto">
        <table className="min-w-full bg-white border border-gray-200 rounded-lg">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">ID</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Email</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Name</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Role</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Permissions</th>
              <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {userData.users.slice(0, 5).map((user: any, index: number) => (
              <tr key={user.id} className={index % 2 === 0 ? 'bg-white' : 'bg-gray-50'}>
                <td className="px-4 py-3 text-sm font-medium text-gray-900">{user.id}</td>
                <td className="px-4 py-3 text-sm text-gray-700">{user.email}</td>
                <td className="px-4 py-3 text-sm text-gray-700">{user.name || 'N/A'}</td>
                <td className="px-4 py-3 text-sm text-gray-700">{user.role}</td>
                <td className="px-4 py-3 text-sm text-gray-700">
                  {user.permissions?.length || 0} permissions
                </td>
                <td className="px-4 py-3 text-sm">
                  <span className={`px-2 py-1 text-xs rounded-full ${
                    user.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                  }`}>
                    {user.is_active ? 'Active' : 'Inactive'}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

// ============================================================================
// Client Component - Dynamic User Filtering
// ============================================================================

interface UserManagementClientFiltersProps {
  initialFilters: AdminFilters
  onFiltersChange: (filters: AdminFilters) => void
}

/**
 * Client Component for dynamic user filtering and management
 * Uses SWR for real-time updates without server actions
 */
function UserManagementClientFilters({ initialFilters, onFiltersChange }: UserManagementClientFiltersProps) {
  const { data, error, isLoading } = adminClientData.useUsers(initialFilters)
  
  const handleFilterChange = (key: keyof AdminFilters, value: any) => {
    const newFilters = { ...initialFilters, [key]: value, page: 1 } // Reset to page 1
    onFiltersChange(newFilters)
  }
  
  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600">Failed to load filtered user data</p>
      </div>
    )
  }
  
  return (
    <div className="space-y-4">
      {/* Filter Controls */}
      <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
        <h3 className="font-semibold text-gray-900 mb-4">Dynamic User Filters</h3>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Role</label>
            <select 
              value={initialFilters.role || ''}
              onChange={(e) => handleFilterChange('role', e.target.value || undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="">All Roles</option>
              <option value="admin">Admin</option>
              <option value="user">User</option>
              <option value="guest">Guest</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Status</label>
            <select
              value={initialFilters.status || ''}
              onChange={(e) => handleFilterChange('status', e.target.value || undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value="">All Status</option>
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
              <option value="pending">Pending</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Search</label>
            <input
              type="text"
              value={initialFilters.search || ''}
              onChange={(e) => handleFilterChange('search', e.target.value || undefined)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="Search users..."
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Page Size</label>
            <select
              value={initialFilters.limit || 20}
              onChange={(e) => handleFilterChange('limit', parseInt(e.target.value))}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              <option value={10}>10 per page</option>
              <option value={20}>20 per page</option>
              <option value={50}>50 per page</option>
              <option value={100}>100 per page</option>
            </select>
          </div>
        </div>
      </div>
      
      {/* Filtered Results */}
      <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-semibold text-green-900">
            Client-filtered User Results
            {isLoading && <span className="ml-2 text-xs text-green-500">Updating...</span>}
          </h3>
          <span className="text-sm text-green-600">
            {data?.total || 0} total users
          </span>
        </div>
        
        {data?.users && (
          <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
            {data.users.slice(0, 6).map((user: any) => (
              <div key={user.id} className="p-3 bg-white border border-green-200 rounded-lg">
                <h4 className="font-medium text-green-900">{user.email}</h4>
                <p className="text-sm text-green-700">{user.name || 'No name'}</p>
                <div className="mt-2 text-xs space-y-1">
                  <p>Role: {user.role}</p>
                  <p>Permissions: {user.permissions?.length || 0}</p>
                  <p>Status: <span className="font-medium">
                    {user.is_active ? 'Active' : 'Inactive'}
                  </span></p>
                </div>
              </div>
            ))}
          </div>
        )}
        
        {!data && !isLoading && (
          <p className="text-green-600">No filtered results available</p>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// Server Action Components - Navigation Only
// ============================================================================

interface UserManagementPaginationProps {
  currentPage: number
  totalPages: number
  currentFilters: AdminFilters
}

/**
 * Pagination component using Server Actions for optimal serverless navigation
 */
function UserManagementPagination({ currentPage, totalPages, currentFilters }: UserManagementPaginationProps) {
  return (
    <div className="flex items-center justify-between p-4 bg-white border border-gray-200 rounded-lg">
      <div className="text-sm text-gray-600">
        Page {currentPage} of {totalPages}
      </div>
      
      <div className="flex gap-2">
        {currentPage > 1 && (
          <form action={adminServerData.navigateToUsers.bind(null, { ...currentFilters, page: currentPage - 1 })}>
            <button 
              type="submit"
              className="px-3 py-1 bg-blue-100 hover:bg-blue-200 border border-blue-300 rounded text-sm"
            >
              Previous
            </button>
          </form>
        )}
        
        {currentPage < totalPages && (
          <form action={adminServerData.navigateToUsers.bind(null, { ...currentFilters, page: currentPage + 1 })}>
            <button 
              type="submit"
              className="px-3 py-1 bg-blue-100 hover:bg-blue-200 border border-blue-300 rounded text-sm"
            >
              Next
            </button>
          </form>
        )}
      </div>
    </div>
  )
}

/**
 * Quick action buttons using Server Actions for navigation
 */
function UserManagementQuickActions({ currentFilters }: { currentFilters: AdminFilters }) {
  return (
    <div className="flex gap-2 p-4 bg-white border border-gray-200 rounded-lg">
      <form action={adminServerData.navigateToPage.bind(null, 'users', { ...currentFilters, view: 'create' })}>
        <button 
          type="submit"
          className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
        >
          Create User
        </button>
      </form>
      
      <form action={adminServerData.navigateToPage.bind(null, 'permissions', {})}>
        <button 
          type="submit"
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Manage Permissions
        </button>
      </form>
      
      <form action={adminServerData.refreshSection.bind(null, 'users')}>
        <button 
          type="submit"
          className="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
        >
          Refresh Data
        </button>
      </form>
    </div>
  )
}

// ============================================================================
// Main Hybrid User Management Component
// ============================================================================

interface HybridUserManagementProps {
  initialFilters: AdminFilters
  enableRealTime?: boolean
  showServerData?: boolean
  showClientFiltering?: boolean
}

/**
 * Complete Hybrid User Management Component
 * Demonstrates the full hybrid strategy for optimal serverless performance
 */
export function HybridUserManagement({ 
  initialFilters, 
  enableRealTime = true,
  showServerData = true,
  showClientFiltering = true
}: HybridUserManagementProps) {
  const { connectSSE } = adminClientData.useRealTime(enableRealTime)
  const { invalidateUsers } = adminClientData.useCache()
  
  // Connect to real-time updates
  if (enableRealTime && typeof window !== 'undefined') {
    connectSSE()
  }
  
  const handleFiltersChange = (newFilters: AdminFilters) => {
    // Update client-side filtering immediately
    invalidateUsers(newFilters)
  }
  
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Hybrid User Management</h1>
        <div className="flex gap-2">
          <button 
            onClick={() => invalidateUsers()}
            className="px-3 py-1 bg-gray-100 hover:bg-gray-200 rounded text-sm"
          >
            Clear Cache
          </button>
        </div>
      </div>
      
      {/* Strategy Explanation */}
      <div className="p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <h3 className="font-medium text-yellow-800 mb-2">Admin Hybrid Strategy Implementation:</h3>
        <div className="grid gap-2 md:grid-cols-3 text-sm text-yellow-700">
          <div>
            <strong>Server Components:</strong> Initial data load, SEO-optimized
          </div>
          <div>
            <strong>Client Components:</strong> Real-time filtering, SWR caching
          </div>
          <div>
            <strong>Server Actions:</strong> Navigation only (no fetch calls)
          </div>
        </div>
      </div>
      
      {/* Server-side Initial Data */}
      {showServerData && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Server-side Initial Load</h2>
          <Suspense fallback={
            <div className="p-8 bg-gray-50 border border-gray-200 rounded-lg">
              <div className="animate-pulse space-y-4">
                <div className="h-4 bg-gray-300 rounded w-1/2"></div>
                <div className="h-3 bg-gray-300 rounded w-3/4"></div>
                <div className="space-y-2">
                  {[1, 2, 3].map(i => (
                    <div key={i} className="h-8 bg-gray-300 rounded"></div>
                  ))}
                </div>
              </div>
            </div>
          }>
            <UserManagementServerData initialFilters={initialFilters} />
          </Suspense>
        </div>
      )}
      
      {/* Client-side Dynamic Filtering */}
      {showClientFiltering && (
        <div>
          <h2 className="text-lg font-semibold mb-4">Client-side Dynamic Filtering</h2>
          <UserManagementClientFilters 
            initialFilters={initialFilters}
            onFiltersChange={handleFiltersChange}
          />
        </div>
      )}
      
      {/* Server Action Navigation */}
      <div>
        <h2 className="text-lg font-semibold mb-4">Server Action Navigation & Quick Actions</h2>
        <div className="space-y-4">
          <UserManagementPagination 
            currentPage={initialFilters.page || 1}
            totalPages={5} // Would be calculated from data
            currentFilters={initialFilters}
          />
          <UserManagementQuickActions currentFilters={initialFilters} />
        </div>
      </div>
      
      {/* Performance Benefits Summary */}
      <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
        <h3 className="font-medium text-green-800 mb-2">Admin Performance Benefits Achieved:</h3>
        <ul className="text-sm text-green-700 space-y-1">
          <li>✅ Server-side initial render (SEO + Core Web Vitals)</li>
          <li>✅ Client-side dynamic filtering (Real-time UX)</li>
          <li>✅ OIDC Bearer authentication (Secure + Scalable)</li>
          <li>✅ No Server Actions with fetch() calls (Serverless optimal)</li>
          <li>✅ SWR caching with stale-while-revalidate (Performance)</li>
          <li>✅ Server Actions for navigation only (Stateless)</li>
          <li>✅ Real-time admin updates (Live data)</li>
          <li>✅ Optimistic UI updates (Instant feedback)</li>
          <li>✅ Direct database access in Server Components (Optimal)</li>
        </ul>
      </div>
    </div>
  )
}

export default HybridUserManagement