'use client'

import { Suspense, useState, useEffect } from 'react'
import { GroupManager } from '@/components/groups/GroupManager'
import { Web3AssignmentRulesManager } from '@/components/web3/Web3AssignmentRulesManager'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

export const dynamic = 'force-dynamic'

function Web3PermissionsHubSkeleton() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero section skeleton */}
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl w-96 mx-auto mb-4 shadow-xl"></div>
          <div className="h-6 bg-gradient-to-r from-gray-300 to-gray-400 rounded-full w-64 mx-auto"></div>
        </div>
        
        {/* Web3 Permission Manager skeleton */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="p-8">
            <div className="h-8 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-xl mb-6 w-1/3"></div>
            <div className="space-y-4">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-700 dark:to-gray-800 rounded-2xl">
                  <div className="flex items-center gap-4 flex-1">
                    <div className="h-10 w-10 bg-gradient-to-br from-yellow-400 to-orange-500 rounded-2xl"></div>
                    <div className="space-y-2 flex-1">
                      <div className="h-5 bg-gray-300 rounded-lg w-1/3"></div>
                      <div className="h-4 bg-gray-200 rounded-lg w-1/2"></div>
                    </div>
                  </div>
                  <div className="h-8 w-24 bg-gradient-to-r from-green-400 to-green-500 rounded-full"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

// Simple client-side authentication check component
function ClientAuthWrapper({ children }: { children: React.ReactNode }) {
  const [authState, setAuthState] = useState<'loading' | 'authenticated' | 'unauthenticated' | 'no-access'>('loading')
  const [walletAddress, setWalletAddress] = useState<string | null>(null)

  useEffect(() => {
    async function checkAuth() {
      try {
        // Simple client-side check - just verify we have wallet_address cookie
        const response = await fetch('/api/auth/web3/verify', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ admin_context: true }),
          credentials: 'include',
        })

        if (response.ok) {
          const data = await response.json()
          if (data.success && data.wallet_address) {
            console.log('✅ Client Auth: Authenticated as:', data.wallet_address)
            setWalletAddress(data.wallet_address)
            setAuthState('authenticated')
          } else {
            console.log('❌ Client Auth: Invalid response')
            setAuthState('unauthenticated')
          }
        } else {
          console.log('❌ Client Auth: Request failed')
          setAuthState('unauthenticated')
        }
      } catch (error) {
        console.error('❌ Client Auth: Error checking authentication:', error)
        setAuthState('unauthenticated')
      }
    }

    checkAuth()
  }, [])

  if (authState === 'loading') {
    return <Web3PermissionsHubSkeleton />
  }

  if (authState === 'unauthenticated') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Authentication Required</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">Please connect your wallet to access the admin panel.</p>
          <button 
            onClick={() => window.location.href = '/login'}
            className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white px-6 py-3 rounded-lg font-medium"
          >
            Connect Wallet
          </button>
        </div>
      </div>
    )
  }

  if (authState === 'no-access') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Access Denied</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">Your wallet does not have admin permissions.</p>
          <p className="text-sm text-gray-500">Wallet: {walletAddress}</p>
        </div>
      </div>
    )
  }

  return <>{children}</>
}

function Web3PermissionsDataWrapper() {
  console.log('✅ Client: Rendering Group-based Web3 Permission Manager')
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-transparent mb-4">
            Web3 Permission Management
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-300">
            Manage permission groups and blockchain-based auto-assignment rules
          </p>
        </div>
        
        {/* Tabs for different management areas */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <Tabs value="groups" onValueChange={() => {}} className="w-full">
            <div className="border-b border-gray-200/50 dark:border-gray-700/50 p-6">
              <TabsList className="grid w-full grid-cols-2 max-w-md mx-auto">
                <TabsTrigger value="groups">Permission Groups</TabsTrigger>
                <TabsTrigger value="web3rules">Web3 Auto-Assignment</TabsTrigger>
              </TabsList>
            </div>

            <TabsContent value="groups" className="p-6">
              <GroupManager />
            </TabsContent>

            <TabsContent value="web3rules" className="p-6">
              <Web3AssignmentRulesManager />
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  )
}

export default function Web3AdminPermissionsPage() {
  return (
    <ClientAuthWrapper>
      <Suspense fallback={<Web3PermissionsHubSkeleton />}>
        <Web3PermissionsDataWrapper />
      </Suspense>
    </ClientAuthWrapper>
  )
}