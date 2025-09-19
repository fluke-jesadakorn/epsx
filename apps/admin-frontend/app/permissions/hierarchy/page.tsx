import { Suspense } from 'react'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'
import HierarchyBuilder from '@/components/permissions/HierarchyBuilder'
import { RefreshCwIcon, TreePineIcon } from 'lucide-react'

export const dynamic = 'force-dynamic'

function HierarchySkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header skeleton */}
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center gap-3">
            <div className="h-6 w-6 bg-blue-400 rounded animate-pulse"></div>
            <div className="h-8 bg-gradient-to-r from-blue-400 to-indigo-500 rounded w-64 animate-pulse"></div>
          </div>
          <div className="flex items-center gap-3">
            <div className="h-9 bg-gray-200 rounded w-20 animate-pulse"></div>
            <div className="h-9 bg-blue-500 rounded w-32 animate-pulse"></div>
          </div>
        </div>
        
        {/* Statistics Cards */}
        <div className="grid grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="bg-white/80 backdrop-blur-sm rounded-2xl p-4 shadow-lg animate-pulse">
              <div className="flex items-center gap-2 mb-2">
                <div className="h-4 w-4 bg-blue-400 rounded"></div>
                <div className="h-4 bg-gray-200 rounded w-24"></div>
              </div>
              <div className="h-8 bg-blue-300 rounded w-16"></div>
            </div>
          ))}
        </div>
        
        {/* Main Content */}
        <div className="bg-white/90 backdrop-blur-sm rounded-3xl shadow-2xl p-8">
          <div className="flex items-center gap-2 mb-6">
            <TreePineIcon className="h-5 w-5 text-green-600" />
            <div className="h-6 bg-gray-200 rounded w-48 animate-pulse"></div>
          </div>
          
          <div className="space-y-4">
            {Array.from({ length: 8 }).map((_, i) => (
              <div key={i} className="flex items-center gap-2 p-2 rounded hover:bg-gray-50">
                <div className={`h-4 w-4 bg-blue-400 rounded ml-${i % 3 * 6}`}></div>
                <div className="h-4 bg-gray-200 rounded flex-1 max-w-xs"></div>
                <div className="h-6 bg-gray-100 rounded w-16"></div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

async function HierarchyDataWrapper() {
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:permissions:manage')) {
    notFound()
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        <HierarchyBuilder />
      </div>
    </div>
  )
}

export default function PermissionHierarchyPage() {
  return (
    <Suspense fallback={<HierarchySkeleton />}>
      <HierarchyDataWrapper />
    </Suspense>
  )
}