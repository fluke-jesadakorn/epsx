import { Suspense } from 'react'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'
import PolicyBuilder from '@/components/policies/PolicyBuilder'
import PolicyMonitor from '@/components/policies/PolicyMonitor'
import { ShieldIcon, ActivityIcon, BarChart3Icon } from 'lucide-react'

export const dynamic = 'force-dynamic'

function PoliciesSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 via-pink-50 to-red-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-3 sm:p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header skeleton */}
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 sm:mb-8 gap-4">
          <div className="flex items-center gap-2 sm:gap-3">
            <div className="h-5 w-5 sm:h-6 sm:w-6 bg-purple-400 rounded"></div>
            <div className="h-6 sm:h-8 bg-gradient-to-r from-purple-400 to-pink-500 rounded w-56 sm:w-72"></div>
          </div>
          <div className="flex items-center gap-2 sm:gap-3 w-full sm:w-auto">
            <div className="h-8 sm:h-9 bg-gray-200 rounded flex-1 sm:flex-none sm:w-24"></div>
            <div className="h-8 sm:h-9 bg-purple-500 rounded flex-1 sm:flex-none sm:w-28"></div>
          </div>
        </div>
        
        {/* Tab Navigation Skeleton */}
        <div className="flex space-x-1 bg-gray-100 p-1 rounded-lg mb-6 sm:mb-8 w-full sm:w-fit overflow-x-auto">
          <div className="h-9 sm:h-10 bg-purple-500 rounded w-28 sm:w-32 flex-shrink-0"></div>
          <div className="h-9 sm:h-10 bg-gray-200 rounded w-28 sm:w-32 flex-shrink-0"></div>
        </div>
        
        {/* Policy Builder Form Skeleton */}
        <div className="grid grid-cols-1 gap-6">
          {/* Configuration Card */}
          <div className="bg-white/90 backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-2xl p-4 sm:p-6 lg:p-8">
            <div className="h-5 sm:h-6 bg-gray-200 rounded w-36 sm:w-48 mb-4 sm:mb-6"></div>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4 mb-4 sm:mb-6">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="space-y-2">
                  <div className="h-3 sm:h-4 bg-gray-200 rounded w-20 sm:w-24"></div>
                  <div className="h-9 sm:h-10 bg-gray-100 rounded"></div>
                </div>
              ))}
            </div>
            <div className="h-20 sm:h-24 bg-gray-100 rounded"></div>
          </div>
          
          {/* Target Actions Card */}
          <div className="bg-white/90 backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-2xl p-4 sm:p-6 lg:p-8">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
              <div className="h-5 sm:h-6 bg-gray-200 rounded w-28 sm:w-32"></div>
              <div className="h-8 bg-purple-500 rounded w-full sm:w-24"></div>
            </div>
            <div className="space-y-2">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="flex items-center gap-2 sm:gap-3 p-2 sm:p-3 bg-gray-50 rounded-lg">
                  <div className="h-4 bg-gray-300 rounded flex-1"></div>
                  <div className="h-5 w-5 sm:h-6 sm:w-6 bg-red-300 rounded"></div>
                </div>
              ))}
            </div>
          </div>
          
          {/* Conditions Card */}
          <div className="bg-white/90 backdrop-blur-sm rounded-2xl sm:rounded-3xl shadow-2xl p-4 sm:p-6 lg:p-8">
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-4 gap-3">
              <div className="h-5 sm:h-6 bg-gray-200 rounded w-32 sm:w-40"></div>
              <div className="flex flex-col sm:flex-row items-start sm:items-center gap-2 sm:gap-3 w-full sm:w-auto">
                <div className="h-8 bg-gray-200 rounded w-full sm:w-48"></div>
                <div className="h-8 bg-purple-500 rounded w-full sm:w-28"></div>
              </div>
            </div>
            <div className="space-y-3 sm:space-y-4">
              {Array.from({ length: 2 }).map((_, i) => (
                <div key={i} className="p-3 sm:p-4 border border-gray-200 rounded-lg">
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
                    {Array.from({ length: 4 }).map((_, j) => (
                      <div key={j} className="space-y-1">
                        <div className="h-3 sm:h-4 bg-gray-200 rounded w-12 sm:w-16"></div>
                        <div className="h-9 sm:h-10 bg-gray-100 rounded"></div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

async function PoliciesDataWrapper() {
  const session = await UnifiedAuth.getSession()
  
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:policies:manage')) {
    notFound()
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
      <div className="max-w-7xl mx-auto space-y-6 sm:space-y-8">
        {/* Background Decorations */}
        <div className="fixed inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-yellow-400/20 to-orange-500/20 rounded-full blur-xl"></div>
          <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
          <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-xl"></div>
        </div>

        {/* Page Header */}
        <div className="text-center mb-8 sm:mb-12">
          <div className="relative inline-block">
            <h1 className="text-3xl sm:text-4xl lg:text-5xl font-bold bg-gradient-to-r from-yellow-600 via-orange-600 via-pink-600 to-purple-600 bg-clip-text text-transparent mb-4">
              🛡️ Dynamic Policies
            </h1>
            <div className="absolute -top-2 -right-2 w-4 h-4 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full"></div>
          </div>
          <p className="text-base sm:text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
            Build advanced permission policies with conditional logic and real-time monitoring
          </p>
        </div>

        {/* Tab Navigation */}
        <div className="relative overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5 mb-6 sm:mb-8">
          <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl sm:rounded-3xl">
            <div className="flex space-x-1 p-4">
              <button 
                className="px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl bg-gradient-to-r from-purple-600 to-blue-600 text-white whitespace-nowrap flex-shrink-0 min-h-[44px] shadow-lg"
                id="builder-tab"
              >
                <ShieldIcon className="h-4 w-4 inline mr-2" />
                Policy Builder
              </button>
              <button 
                className="px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 whitespace-nowrap flex-shrink-0 min-h-[44px]"
                id="monitor-tab"
              >
                <ActivityIcon className="h-4 w-4 inline mr-2" />
                Live Monitor
              </button>
            </div>
          </div>
        </div>
        
        {/* Policy Builder Section */}
        <div id="builder-section">
          <PolicyBuilder />
        </div>
        
        {/* Policy Monitor Section (Hidden by default) */}
        <div id="monitor-section" className="hidden">
          <PolicyMonitor />
        </div>
        
        {/* Client-side tab switching script */}
        <script dangerouslySetInnerHTML={{
          __html: `
            document.addEventListener('DOMContentLoaded', function() {
              const builderTab = document.getElementById('builder-tab');
              const monitorTab = document.getElementById('monitor-tab');
              const builderSection = document.getElementById('builder-section');
              const monitorSection = document.getElementById('monitor-section');
              
              builderTab?.addEventListener('click', function() {
                builderTab.className = 'px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl bg-gradient-to-r from-purple-600 to-blue-600 text-white whitespace-nowrap flex-shrink-0 min-h-[44px] shadow-lg';
                monitorTab.className = 'px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 whitespace-nowrap flex-shrink-0 min-h-[44px]';
                builderSection.classList.remove('hidden');
                monitorSection.classList.add('hidden');
              });
              
              monitorTab?.addEventListener('click', function() {
                monitorTab.className = 'px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl bg-gradient-to-r from-purple-600 to-blue-600 text-white whitespace-nowrap flex-shrink-0 min-h-[44px] shadow-lg';
                builderTab.className = 'px-4 sm:px-6 py-3 text-sm font-medium rounded-2xl text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 whitespace-nowrap flex-shrink-0 min-h-[44px]';
                monitorSection.classList.remove('hidden');
                builderSection.classList.add('hidden');
              });
            });
          `
        }} />
      </div>
    </div>
  )
}

export default function DynamicPoliciesPage() {
  return (
    <Suspense fallback={<PoliciesSkeleton />}>
      <PoliciesDataWrapper />
    </Suspense>
  )
}