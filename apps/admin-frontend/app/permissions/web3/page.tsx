import { Suspense } from 'react'
import { Web3PermissionManager } from '@/components/admin/Web3PermissionManager'
import { getWeb3AdminSession, createWeb3AdminUser } from '@/lib/web3-admin-session'
import { notFound, redirect } from 'next/navigation'

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

async function Web3PermissionsDataWrapper() {
  // Get Web3 admin session
  const session = await getWeb3AdminSession()
  
  if (!session?.isAuthenticated) {
    console.log('❌ Web3 Admin: No authenticated session, redirecting to login')
    redirect('/login?reason=no-session&return_url=/permissions/web3')
  }

  // Check if user has admin permissions
  if (!session.hasAdminAccess) {
    console.log('❌ Web3 Admin: User lacks admin permissions:', session.walletAddress)
    notFound()
  }

  // Create compatible user object
  const adminUser = createWeb3AdminUser(session)
  
  if (!adminUser) {
    console.error('❌ Web3 Admin: Failed to create admin user object')
    notFound()
  }

  console.log('✅ Web3 Admin: Session validated, rendering Web3PermissionManager for:', session.walletAddress)

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <div className="relative">
            <h1 className="text-6xl font-bold bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 bg-clip-text text-transparent mb-4 drop-shadow-lg">
              🌐 Web3 Permission Hub
            </h1>
            <div className="absolute -top-2 -right-2 text-2xl animate-pulse">⚡</div>
          </div>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-2xl mx-auto leading-relaxed">
            Advanced blockchain-based permission management with real NFT, token, and DAO governance verification
          </p>
          <div className="mt-4 flex items-center justify-center gap-4 text-sm text-gray-500 dark:text-gray-400">
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 bg-green-500 rounded-full"></span>
              Connected: {session.walletAddress?.slice(0, 6)}...{session.walletAddress?.slice(-4)}
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 bg-blue-500 rounded-full"></span>
              Level: {session.adminLevel}
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 bg-purple-500 rounded-full"></span>
              Permissions: {session.permissions?.length || 0}
            </span>
          </div>
        </div>

        {/* Web3 Permission Manager */}
        <Web3PermissionManager />
      </div>
    </div>
  )
}

export default function Web3AdminPermissionsPage() {
  return (
    <Suspense fallback={<Web3PermissionsHubSkeleton />}>
      <Web3PermissionsDataWrapper />
    </Suspense>
  )
}