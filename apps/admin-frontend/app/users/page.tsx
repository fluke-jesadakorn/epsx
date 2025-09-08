import { Suspense } from 'react'
import UsersHub from '@/components/hubs/UsersHub'
import { UnifiedDataFetchers } from '@/lib/server/unified-data-fetchers'
import { ServerAuth } from '@/lib/server/auth-helpers'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

export interface UsersPageProps {
  searchParams?: Promise<{
    page?: string
    search?: string
    filter?: string
    limit?: string
  }>
}

function UsersHubSkeleton() {
  return (
    <div className="wp-pancake-page-bg p-6">
      
      <div className="relative z-10">
        <div className="mb-8">
          <div className="h-12 bg-gradient-to-r from-[#FFC107] to-[#FF8F00] w-64 mb-2 animate-pulse shadow-xl"></div>
          <div className="h-4 bg-gradient-to-r from-[#0078D4] to-[#106EBE] w-48 animate-pulse shadow-lg"></div>
        </div>
        
        {/* Windows Phone style stats tiles skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2 mb-6">
          {Array.from({ length: 4 }).map((_, i) => (
            <div 
              key={i} 
              className="h-32 bg-gradient-to-br from-[#FFC107] to-[#FF8F00] shadow-xl animate-pulse relative overflow-hidden"
            >
              <div className="absolute left-0 top-0 h-full w-1 bg-gradient-to-b from-yellow-400 to-orange-500"></div>
            </div>
          ))}
        </div>
        
        {/* Windows Phone pivot navigation skeleton */}
        <div className="mb-6">
          <div className="flex gap-6 border-b-2 border-[#FFC107]/30 pb-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-6 bg-gradient-to-r from-[#0078D4] to-[#106EBE] w-20 animate-pulse shadow-lg"></div>
            ))}
          </div>
        </div>
        
        {/* PancakeSwap controls skeleton */}
        <div className="flex flex-col sm:flex-row gap-4 mb-6">
          <div className="flex-1 h-14 bg-gradient-to-r from-gray-800 to-gray-700 animate-pulse shadow-xl border border-[#FFC107]/20"></div>
          <div className="flex gap-2">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="w-28 h-14 bg-gradient-to-r from-[#8764B8] to-[#744DA9] animate-pulse shadow-xl"></div>
            ))}
          </div>
        </div>
        
        {/* Windows Phone users list skeleton */}
        <div className="space-y-2">
          {Array.from({ length: 8 }).map((_, i) => (
            <div 
              key={i} 
              className="h-24 bg-gradient-to-r from-gray-800/50 to-gray-700/50 animate-pulse shadow-xl relative overflow-hidden border-l-4 border-[#FFC107]"
            >
              <div className="absolute top-0 right-0 h-6 w-6 bg-gradient-to-bl from-yellow-300/20 to-transparent"></div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

// Server component that fetches data and passes to client component
async function UsersDataWrapper({ searchParams }: { searchParams?: UsersPageProps['searchParams'] }) {
  // Check authentication first
  await ServerAuth.requirePermission('admin:users:manage')
  
  // Parse search parameters - await in Next.js 15
  const resolvedSearchParams = await searchParams
  const page = parseInt(resolvedSearchParams?.page || '1', 10)
  const limit = parseInt(resolvedSearchParams?.limit || '20', 10)
  const search = resolvedSearchParams?.search?.trim() || ''
  const filter = resolvedSearchParams?.filter || 'all'

  // Calculate offset for pagination
  const offset = (page - 1) * limit
  
  let usersData = { users: [], total: 0, page: 1, totalPages: 1 }
  let stats = {}

  try {
    // Use unified data fetchers for consistent data access
    if (search || filter !== 'all') {
      // Use IAM users with filters
      const filters = {
        ...(search && { search }),
        ...(filter === 'active' && { status: 'active' }),
        ...(filter === 'premium' && { role: 'premium' }),
        limit,
        offset
      }
      
      const users = await UnifiedDataFetchers.getIAMUsers(filters)
      usersData = {
        users,
        total: users.length,
        page,
        totalPages: Math.ceil(users.length / limit)
      }
    } else {
      // Use regular users endpoint
      usersData = await UnifiedDataFetchers.getUsers(offset, limit)
      usersData.page = page
      usersData.totalPages = Math.ceil(usersData.total / limit)
    }
    
    // Get user stats
    stats = await UnifiedDataFetchers.getUserStats()
    
  } catch (error) {
    console.error('Failed to fetch users data:', error)
  }

  // Apply client-side admin filter with embedded timestamp support
  if (filter === 'admins') {
    usersData.users = usersData.users.filter((user: any) => {
      if (!user.permissions) return false
      
      // Parse permissions with embedded timestamps and check for active admin permissions
      return user.permissions.some((permission: string) => {
        const parts = permission.split(':')
        
        // Handle embedded timestamp permissions
        if (parts.length >= 4 && /^\d+$/.test(parts[parts.length - 1])) {
          const timestamp = parseInt(parts[parts.length - 1])
          const permissionWithoutTimestamp = parts.slice(0, -1).join(':')
          const isExpired = new Date(timestamp * 1000) < new Date()
          return permissionWithoutTimestamp.startsWith('admin:') && !isExpired
        }
        
        // Handle permanent permissions (no timestamp)
        return permission.startsWith('admin:')
      })
    })
    
    usersData.total = usersData.users.length
    usersData.totalPages = Math.ceil(usersData.total / limit)
  }
  
  return (
    <UsersHub 
      initialData={{
        users: usersData.users,
        total: usersData.total,
        page: usersData.page,
        totalPages: usersData.totalPages,
        stats: stats
      }}
      searchParams={resolvedSearchParams}
    />
  )
}

export default function UsersPage(props: UsersPageProps) {
  return (
    <Suspense fallback={<UsersHubSkeleton />}>
      <UsersDataWrapper searchParams={props.searchParams} />
    </Suspense>
  )
}