/**
 * Group Analytics Page
 * Dedicated page for group-based permission analytics and insights
 */

'use client'

import { Suspense, useState, useEffect } from 'react'
import { GroupAnalyticsDashboard } from '@/components/groups/GroupAnalyticsDashboard'
import { GroupAssignmentHistory } from '@/components/groups/GroupAssignmentHistory'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { TrendingUp, History, BarChart3 } from 'lucide-react'

export const dynamic = 'force-dynamic'

function GroupAnalyticsHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-purple-400/20 to-blue-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-blue-400/20 to-green-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-green-400/15 to-purple-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-7xl mx-auto">
        {/* Page Header Skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-purple-400/30 to-blue-500/30 w-96 mx-auto mb-4 animate-pulse rounded-3xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300/50 to-gray-400/50 dark:from-gray-600/50 dark:to-gray-700/50 w-80 mx-auto animate-pulse rounded-2xl"></div>
        </div>
        
        {/* Stats Cards Skeleton */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="relative overflow-hidden rounded-3xl bg-gradient-to-r from-purple-400/20 via-blue-400/20 to-green-400/20 p-0.5 group">
              <div className="relative bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-6 h-32">
                <div className="absolute top-2 right-2 w-8 h-8 bg-gradient-to-br from-purple-300/30 to-blue-400/30 rounded-full blur-sm animate-pulse"></div>
                <div className="animate-pulse">
                  <div className="flex justify-between items-start mb-4">
                    <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded-xl w-16"></div>
                    <div className="w-10 h-10 bg-gradient-to-r from-purple-400/50 to-blue-500/50 rounded-2xl"></div>
                  </div>
                  <div className="h-8 bg-gray-300 dark:bg-gray-600 rounded-xl w-12 mb-2"></div>
                  <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded-xl w-24"></div>
                </div>
              </div>
            </div>
          ))}
        </div>
        
        {/* Tabs Skeleton */}
        <div className="mb-8">
          <div className="flex gap-8 border-b border-gray-200/50 dark:border-gray-700/50 pb-3">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-6 bg-gradient-to-r from-gray-300/50 to-gray-400/50 dark:from-gray-600/50 dark:to-gray-700/50 w-20 animate-pulse rounded-xl"></div>
            ))}
          </div>
        </div>
        
        {/* Content Area Skeleton */}
        <div className="bg-white/90 dark:bg-gray-900/90 backdrop-blur-xl rounded-3xl p-8">
          <div className="space-y-6">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-40 bg-gradient-to-r from-gray-200/50 to-gray-300/50 dark:from-gray-700/50 dark:to-gray-800/50 rounded-2xl animate-pulse"></div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

// Simple client-side authentication check component
function ClientAuthWrapper({ children }: { children: React.ReactNode }) {
  const [authState, setAuthState] = useState<'loading' | 'authenticated' | 'unauthenticated'>('loading')

  useEffect(() => {
    async function checkAuth() {
      try {
        // Simple client-side check - verify admin permissions
        const response = await fetch('/api/auth/web3/verify', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ admin_context: true }),
          credentials: 'include',
        })

        if (response.ok) {
          const data = await response.json()
          if (data.success && data.wallet_address) {
            console.log('✅ Analytics Auth: Authenticated as:', data.wallet_address)
            setAuthState('authenticated')
          } else {
            console.log('❌ Analytics Auth: Invalid response')
            setAuthState('unauthenticated')
          }
        } else {
          console.log('❌ Analytics Auth: Request failed')
          setAuthState('unauthenticated')
        }
      } catch (error) {
        console.error('❌ Analytics Auth: Error checking authentication:', error)
        setAuthState('unauthenticated')
      }
    }

    checkAuth()
  }, [])

  if (authState === 'loading') {
    return <GroupAnalyticsHubSkeleton />
  }

  if (authState === 'unauthenticated') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Authentication Required</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">Please connect your wallet to access analytics.</p>
          <button 
            onClick={() => window.location.href = '/login'}
            className="bg-gradient-to-r from-purple-400 to-blue-500 text-white px-6 py-3 rounded-lg font-medium"
          >
            Connect Wallet
          </button>
        </div>
      </div>
    )
  }

  return <>{children}</>
}

function GroupAnalyticsDataWrapper() {
  console.log('✅ Client: Rendering Group Analytics Dashboard')
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-purple-600 to-blue-600 bg-clip-text text-transparent mb-4">
            Group Analytics & Insights
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-300">
            Comprehensive analytics for your group-based permission system
          </p>
        </div>
        
        {/* Tabs for different analytics views */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <Tabs value="dashboard" onValueChange={() => {}} className="w-full">
            <div className="border-b border-gray-200/50 dark:border-gray-700/50 p-6">
              <TabsList className="grid w-full grid-cols-3 max-w-lg mx-auto">
                <TabsTrigger value="dashboard" className="flex items-center gap-2">
                  <TrendingUp className="h-4 w-4" />
                  Dashboard
                </TabsTrigger>
                <TabsTrigger value="history" className="flex items-center gap-2">
                  <History className="h-4 w-4" />
                  History
                </TabsTrigger>
                <TabsTrigger value="reports" className="flex items-center gap-2">
                  <BarChart3 className="h-4 w-4" />
                  Reports
                </TabsTrigger>
              </TabsList>
            </div>

            <TabsContent value="dashboard" className="p-6">
              <GroupAnalyticsDashboard />
            </TabsContent>

            <TabsContent value="history" className="p-6">
              <GroupAssignmentHistory />
            </TabsContent>

            <TabsContent value="reports" className="p-6">
              <GroupReportsGenerator />
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  )
}

// Placeholder components for reports

function GroupReportsGenerator() {
  return (
    <div className="space-y-6">
      <div className="text-center py-12">
        <BarChart3 className="h-16 w-16 text-gray-400 mx-auto mb-4" />
        <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
          Reports & Export
        </h3>
        <p className="text-gray-600 dark:text-gray-400">
          Generate and export comprehensive reports about group permissions and usage.
        </p>
      </div>
    </div>
  )
}

export default function GroupAnalyticsPage() {
  return (
    <ClientAuthWrapper>
      <Suspense fallback={<GroupAnalyticsHubSkeleton />}>
        <GroupAnalyticsDataWrapper />
      </Suspense>
    </ClientAuthWrapper>
  )
}