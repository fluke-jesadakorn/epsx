import { Suspense } from 'react'
import UsersHub from '@/components/hubs/UsersHub'
import { ServerUserAPI } from '@/lib/api/admin-client'

// This page uses real backend data and should be dynamic
export const dynamic = 'force-dynamic'

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
async function UsersDataWrapper() {
  // Fetch data server-side
  const [usersResponse, statsResponse] = await Promise.allSettled([
    ServerUserAPI.getUsers(0, 50),
    ServerUserAPI.getUserStats()
  ])
  
  const usersData = usersResponse.status === 'fulfilled' ? usersResponse.value : { users: [], total: 0 }
  const stats = statsResponse.status === 'fulfilled' ? statsResponse.value : {}
  
  return (
    <UsersHub 
      initialData={{
        users: usersData.users,
        total: usersData.total,
        stats: stats
      }}
    />
  )
}

export default function UsersPage() {
  return (
    <Suspense fallback={<UsersHubSkeleton />}>
      <UsersDataWrapper />
    </Suspense>
  )
}