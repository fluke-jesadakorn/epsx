'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Shield, Users, Settings, Activity } from 'lucide-react'

export const dynamic = 'force-dynamic'

// Simple loading component
function LoadingState() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-12">
          <div className="h-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl w-96 mx-auto mb-4 animate-pulse"></div>
          <div className="h-6 bg-gray-300 rounded-full w-64 mx-auto animate-pulse"></div>
        </div>
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 p-8">
          <div className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="h-20 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

// Enhanced authentication wrapper with better error handling
function AuthWrapper({ children }: { children: React.ReactNode }) {
  const [authState, setAuthState] = useState<'loading' | 'authenticated' | 'unauthenticated' | 'no-admin'>('loading')
  const [walletAddress, setWalletAddress] = useState<string | null>(null)
  const [authError, setAuthError] = useState<string | null>(null)

  useEffect(() => {
    async function checkAuth() {
      try {
        console.log('🔍 Checking authentication status...')
        
        // Get access token from localStorage (matches SharedOpenIDWeb3Client storage)
        const accessToken = localStorage.getItem('epsx-admin_access_token');
        
        if (!accessToken) {
          console.log('❌ No access token found in localStorage')
          setAuthError('No authentication token found')
          setAuthState('unauthenticated')
          return
        }
        
        console.log('🔍 Found access token, verifying with backend...')
        
        // Call backend session verification endpoint
        const response = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/v1/auth/session/verify`, {
          method: 'POST',
          headers: { 
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${accessToken}`
          },
          body: JSON.stringify({ admin_context: true }),
        })

        const data = await response.json()
        console.log('🔍 Backend auth response:', data)

        if (response.ok && data.success && data.authenticated) {
          setWalletAddress(data.wallet_address)
          setAuthState('authenticated')
          console.log('✅ Authentication successful:', data.wallet_address)
        } else if (response.status === 403 || (data.success === false && data.error?.includes('Admin'))) {
          setWalletAddress(data.wallet_address)
          setAuthError('Your wallet does not have admin permissions')
          setAuthState('no-admin')
        } else {
          setAuthError(data.error || 'Authentication failed')
          setAuthState('unauthenticated')
        }
      } catch (error) {
        console.error('❌ Auth check failed:', error)
        setAuthError('Failed to verify authentication')
        setAuthState('unauthenticated')
      }
    }

    checkAuth()
  }, [])

  if (authState === 'loading') {
    return <LoadingState />
  }

  if (authState === 'unauthenticated') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900">
        <div className="text-center max-w-md mx-auto">
          <div className="mb-8">
            <div className="w-16 h-16 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-full mx-auto mb-4 flex items-center justify-center">
              <Shield className="w-8 h-8 text-white" />
            </div>
            <h1 className="text-3xl font-bold mb-4">Admin Authentication Required</h1>
            <p className="text-gray-600 dark:text-gray-400 mb-2">
              Please connect your admin wallet to access the Web3 permissions panel.
            </p>
            {authError && (
              <p className="text-sm text-red-600 dark:text-red-400 mb-4">
                {authError}
              </p>
            )}
          </div>
          
          <div className="space-y-4">
            <Button 
              onClick={() => window.location.href = '/auth?return_url=' + encodeURIComponent('/permissions/web3')}
              className="w-full bg-gradient-to-r from-yellow-400 to-orange-500 text-white py-3 text-lg"
            >
              <Shield className="w-5 h-5 mr-2" />
              Connect Admin Wallet
            </Button>
            
            <div className="text-xs text-gray-500 dark:text-gray-400">
              <p>You need admin permissions to access this page.</p>
              <p>Contact your administrator if you believe this is an error.</p>
            </div>
          </div>
        </div>
      </div>
    )
  }

  if (authState === 'no-admin') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-purple-900">
        <div className="text-center max-w-md mx-auto">
          <div className="w-16 h-16 bg-red-500 rounded-full mx-auto mb-4 flex items-center justify-center">
            <Shield className="w-8 h-8 text-white" />
          </div>
          <h1 className="text-2xl font-bold mb-4">Access Denied</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            Your wallet ({walletAddress?.slice(0, 6)}...{walletAddress?.slice(-4)}) does not have admin permissions.
          </p>
          <div className="space-y-3">
            <Button 
              onClick={() => window.location.href = '/auth'}
              variant="outline"
              className="w-full"
            >
              Try Different Wallet
            </Button>
            <Button 
              onClick={() => window.location.href = '/dashboard'}
              className="w-full bg-gradient-to-r from-blue-400 to-blue-600 text-white"
            >
              Go to Dashboard
            </Button>
          </div>
        </div>
      </div>
    )
  }

  return <>{children}</>
}

// Main page component
function Web3PermissionsPage() {
  const [activeTab, setActiveTab] = useState('groups')

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
        
        {/* Navigation Tabs */}
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-3xl shadow-2xl border border-white/30 overflow-hidden">
          <div className="border-b border-gray-200/50 dark:border-gray-700/50 p-6">
            <div className="flex space-x-4 justify-center">
              <Button
                variant={activeTab === 'groups' ? 'default' : 'outline'}
                onClick={() => setActiveTab('groups')}
                className="min-w-32"
              >
                <Users className="w-4 h-4 mr-2" />
                Groups
              </Button>
              <Button
                variant={activeTab === 'rules' ? 'default' : 'outline'}
                onClick={() => setActiveTab('rules')}
                className="min-w-32"
              >
                <Shield className="w-4 h-4 mr-2" />
                Web3 Rules
              </Button>
              <Button
                variant={activeTab === 'analytics' ? 'default' : 'outline'}
                onClick={() => setActiveTab('analytics')}
                className="min-w-32"
              >
                <Activity className="w-4 h-4 mr-2" />
                Analytics
              </Button>
            </div>
          </div>

          <div className="p-6">
            {activeTab === 'groups' && (
              <div className="space-y-6">
                <div className="flex justify-between items-center">
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Permission Groups</h2>
                  <Button className="bg-gradient-to-r from-yellow-400 to-orange-500 text-white">
                    <Users className="w-4 h-4 mr-2" />
                    Create Group
                  </Button>
                </div>
                
                <Alert>
                  <Shield className="h-4 w-4" />
                  <AlertDescription>
                    Permission groups allow you to organize users and assign bulk permissions efficiently.
                  </AlertDescription>
                </Alert>

                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {[
                    { name: 'Admin Group', users: 5, permissions: 15, status: 'Active' },
                    { name: 'Premium Users', users: 142, permissions: 8, status: 'Active' },
                    { name: 'Beta Testers', users: 23, permissions: 12, status: 'Active' },
                  ].map((group, index) => (
                    <Card key={index} className="hover:shadow-lg">
                      <CardHeader className="pb-3">
                        <div className="flex justify-between items-center">
                          <CardTitle className="text-lg">{group.name}</CardTitle>
                          <Badge variant="default" className="bg-green-100 text-green-800">
                            {group.status}
                          </Badge>
                        </div>
                      </CardHeader>
                      <CardContent>
                        <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
                          <div className="flex justify-between">
                            <span>Users:</span>
                            <span className="font-medium">{group.users}</span>
                          </div>
                          <div className="flex justify-between">
                            <span>Permissions:</span>
                            <span className="font-medium">{group.permissions}</span>
                          </div>
                        </div>
                        <div className="mt-4 flex space-x-2">
                          <Button size="sm" variant="outline">
                            <Settings className="w-3 h-3 mr-1" />
                            Edit
                          </Button>
                          <Button size="sm" variant="outline">
                            <Users className="w-3 h-3 mr-1" />
                            Members
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </div>
            )}

            {activeTab === 'rules' && (
              <div className="space-y-6">
                <div className="flex justify-between items-center">
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Web3 Assignment Rules</h2>
                  <Button className="bg-gradient-to-r from-blue-400 to-purple-500 text-white">
                    <Shield className="w-4 h-4 mr-2" />
                    Create Rule
                  </Button>
                </div>
                
                <Alert>
                  <Shield className="h-4 w-4" />
                  <AlertDescription>
                    Web3 rules automatically assign users to groups based on their blockchain holdings or activity.
                  </AlertDescription>
                </Alert>

                <div className="space-y-4">
                  {[
                    { 
                      name: 'NFT Holder Rule', 
                      type: 'NFT Collection', 
                      chain: 'BSC',
                      condition: 'Owns PancakeSquad NFT',
                      group: 'Premium Users',
                      status: 'Active',
                      processed: 342
                    },
                    { 
                      name: 'Token Holder Rule', 
                      type: 'Token Balance', 
                      chain: 'BSC',
                      condition: 'CAKE > 1000',
                      group: 'VIP Users',
                      status: 'Active',
                      processed: 156
                    },
                  ].map((rule, index) => (
                    <Card key={index} className="hover:shadow-lg">
                      <CardHeader className="pb-3">
                        <div className="flex justify-between items-center">
                          <CardTitle className="text-lg">{rule.name}</CardTitle>
                          <div className="flex space-x-2">
                            <Badge variant="outline">{rule.chain}</Badge>
                            <Badge variant="default" className="bg-green-100 text-green-800">
                              {rule.status}
                            </Badge>
                          </div>
                        </div>
                      </CardHeader>
                      <CardContent>
                        <div className="grid grid-cols-2 gap-4 text-sm">
                          <div>
                            <span className="text-gray-500">Type:</span>
                            <div className="font-medium">{rule.type}</div>
                          </div>
                          <div>
                            <span className="text-gray-500">Target Group:</span>
                            <div className="font-medium">{rule.group}</div>
                          </div>
                          <div className="col-span-2">
                            <span className="text-gray-500">Condition:</span>
                            <div className="font-medium">{rule.condition}</div>
                          </div>
                          <div>
                            <span className="text-gray-500">Processed:</span>
                            <div className="font-medium">{rule.processed} wallets</div>
                          </div>
                        </div>
                        <div className="mt-4 flex space-x-2">
                          <Button size="sm" variant="outline">
                            <Settings className="w-3 h-3 mr-1" />
                            Edit
                          </Button>
                          <Button size="sm" variant="outline">
                            <Activity className="w-3 h-3 mr-1" />
                            Analytics
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </div>
            )}

            {activeTab === 'analytics' && (
              <div className="space-y-6">
                <div className="flex justify-between items-center">
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Permission Analytics</h2>
                </div>
                
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                  {[
                    { label: 'Total Groups', value: '8', change: '+2' },
                    { label: 'Active Rules', value: '15', change: '+3' },
                    { label: 'Users Managed', value: '1,247', change: '+89' },
                    { label: 'Auto-Assignments', value: '342', change: '+45' },
                  ].map((stat, index) => (
                    <Card key={index}>
                      <CardContent className="p-6">
                        <div className="text-2xl font-bold">{stat.value}</div>
                        <p className="text-sm text-gray-600 dark:text-gray-400">{stat.label}</p>
                        <div className="text-xs text-green-600 mt-1">{stat.change} this week</div>
                      </CardContent>
                    </Card>
                  ))}
                </div>

                <Alert>
                  <Activity className="h-4 w-4" />
                  <AlertDescription>
                    Analytics show real-time performance of your permission system and Web3 rule processing.
                  </AlertDescription>
                </Alert>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default function Web3AdminPermissionsPage() {
  return (
    <AuthWrapper>
      <Web3PermissionsPage />
    </AuthWrapper>
  )
}