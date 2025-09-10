/**
 * Bulk Operations Main Page - Unified Component
 * Consolidated bulk operations using unified components
 */

import { Shield, UserPlus, UserMinus, UserCheck, Zap, Activity, BarChart3, Settings } from 'lucide-react'
import { PermissionForms } from '@/components/permissions/PermissionForms'
import { UserForms } from '@/components/users/UserForms'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'

interface Props {
  searchParams?: Promise<{
    users?: string
    operation?: 'grant' | 'revoke' | 'roles' | 'validate' | 'export'
  }>
}

export default async function BulkOperationsPage({ searchParams }: Props) {
  const resolvedSearchParams = await searchParams
  const selectedUserIds = resolvedSearchParams?.users?.split(',').filter(Boolean) || []
  const operation = resolvedSearchParams?.operation || null
  
  if (selectedUserIds.length === 0) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="relative z-10 max-w-4xl mx-auto text-center py-16">
          <div className="w-16 h-16 bg-gradient-to-br from-red-400 to-pink-500 rounded-3xl flex items-center justify-center mx-auto mb-6">
            <UserMinus className="h-8 w-8 text-white" />
          </div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4">No Users Selected</h1>
          <p className="text-gray-600 dark:text-gray-300 mb-8">Please select users from the users page to perform bulk operations.</p>
          <a href="/users" className="inline-flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 rounded-2xl text-white font-semibold hover:shadow-lg transition-all duration-300">
            <UserPlus className="h-5 w-5" />
            Go to Users
          </a>
        </div>
      </div>
    )
  }

  // Get current user session
  const session = await UnifiedAuth.getSession()
  if (!session?.user) {
    notFound()
  }

  // Get user data for bulk operations
  const client = new UnifiedAdminClient()
  let users = []
  try {
    const userPromises = selectedUserIds.map(id => client.getUser(id))
    users = await Promise.all(userPromises)
  } catch (error) {
    console.error('Failed to fetch user data:', error)
  }

  // Show specific operation interface if selected
  if (operation) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
        <div className="relative z-10 max-w-6xl mx-auto">
          {operation === 'grant' && (
            <PermissionForms
              mode="bulkGrant"
              users={users}
              selectedUserIds={selectedUserIds}
              currentUser={session.user}
            />
          )}
          {operation === 'revoke' && (
            <PermissionForms
              mode="bulkRevoke"
              users={users}
              selectedUserIds={selectedUserIds}
              currentUser={session.user}
            />
          )}
          {operation === 'export' && (
            <UserForms
              mode="bulkExport"
              users={users}
              selectedUserIds={selectedUserIds}
              currentUser={session.user}
            />
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 right-20 w-40 h-40 bg-gradient-to-r from-yellow-400/15 to-orange-500/15 rounded-full blur-2xl animate-pulse"></div>
        <div className="absolute bottom-32 left-16 w-32 h-32 bg-gradient-to-r from-pink-400/15 to-purple-500/15 rounded-full blur-xl"></div>
        <div className="absolute top-1/2 left-1/3 w-24 h-24 bg-gradient-to-r from-orange-400/15 to-yellow-500/15 rounded-full blur-lg animate-pulse" style={{animationDelay: '2s'}}></div>
      </div>

      <div className="relative z-10 max-w-6xl mx-auto">
        {/* PancakeSwap Header */}
        <div className="mb-8">
          <div className="flex items-center gap-6 mb-6">
            <div className="w-16 h-16 bg-gradient-to-br from-yellow-400 via-orange-500 to-pink-500 rounded-3xl flex items-center justify-center shadow-xl relative overflow-hidden">
              <div className="absolute top-0 right-0 w-6 h-6 bg-gradient-to-bl from-white/40 to-transparent"></div>
              <div className="absolute bottom-1 left-1 w-3 h-3 bg-white/30 rounded-full"></div>
              <Shield className="h-8 w-8 text-white drop-shadow-sm" />
            </div>
            <div>
              <h1 className="text-4xl font-bold bg-gradient-to-r from-gray-900 via-gray-700 to-gray-900 dark:from-white dark:via-gray-100 dark:to-white bg-clip-text text-transparent">
                Choose Bulk Operation
              </h1>
              <p className="text-gray-600 dark:text-gray-300 mt-1 font-medium">
                Perform operation on {selectedUserIds.length} selected user{selectedUserIds.length !== 1 ? 's' : ''}
              </p>
            </div>
          </div>
          
          {/* Progress indicator */}
          <div className="flex items-center gap-4">
            <div className="flex-1 h-2 bg-white/40 dark:bg-gray-700/40 rounded-full overflow-hidden backdrop-blur-sm">
              <div className="h-full w-full bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 transition-all duration-1000 ease-out rounded-full" />
            </div>
            <div className="px-4 py-2 bg-white/60 dark:bg-gray-800/60 rounded-2xl border border-white/20 backdrop-blur-sm">
              <span className="text-sm font-semibold text-gray-700 dark:text-gray-200">Ready to Execute</span>
            </div>
          </div>
        </div>

        {/* PancakeSwap Operation Cards */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          {[
            {
              title: 'grant perms',
              subtitle: 'add permissions',
              description: 'Add new permissions to selected users. Existing permissions preserved.',
              icon: UserPlus,
              operation: 'grant',
              variant: 'user' as const,
              category: 'permissions'
            },
            {
              title: 'revoke perms',
              subtitle: 'remove access',
              description: 'Remove specific permissions from selected users safely.',
              icon: UserMinus,
              operation: 'revoke',
              variant: 'error' as const,
              category: 'permissions'
            },
            {
              title: 'export data',
              subtitle: 'download info',
              description: 'Export user data and permissions for analysis or backup.',
              icon: BarChart3,
              operation: 'export',
              variant: 'analytics' as const,
              category: 'data'
            },
            {
              title: 'validate access',
              subtitle: 'audit permissions',
              description: 'Check permission consistency and validity across users.',
              icon: UserCheck,
              operation: 'validate',
              variant: 'permission' as const,
              category: 'audit'
            }
          ].map((operationItem, index) => {
            const IconComponent = operationItem.icon
            
            const colors = {
              user: 'from-green-400 to-emerald-500',
              error: 'from-red-400 to-pink-500', 
              permission: 'from-blue-400 to-cyan-500',
              analytics: 'from-purple-400 to-indigo-500'
            }
            
            return (
              <a
                key={operationItem.title}
                href={`/users/bulk?users=${selectedUserIds.join(',')}&operation=${operationItem.operation}`}
                className="group relative bg-white/80 dark:bg-gray-800/80 backdrop-blur-xl rounded-3xl shadow-xl border border-white/20 dark:border-gray-700/50 overflow-hidden hover:shadow-2xl hover:scale-105 transition-all duration-300 min-h-[180px]"
              >
                {/* Gradient Background */}
                <div className={`absolute inset-0 bg-gradient-to-br ${colors[operationItem.variant]} opacity-10`}></div>
                
                {/* Decorative Elements */}
                <div className="absolute top-4 right-4 w-3 h-3 bg-yellow-400 rounded-full opacity-60"></div>
                <div className="absolute bottom-4 left-4 w-2 h-2 bg-orange-400 rounded-full opacity-40"></div>
                
                {/* Card Content */}
                <div className="relative z-10 p-6 h-full flex flex-col">
                  {/* Header with Icon */}
                  <div className="flex items-center justify-between mb-4">
                    <div className={`w-14 h-14 bg-gradient-to-br ${colors[operationItem.variant]} rounded-2xl flex items-center justify-center shadow-lg group-hover:scale-110 transition-all duration-300 relative overflow-hidden`}>
                      <div className="absolute top-0 right-0 w-4 h-4 bg-white/30 rounded-bl-2xl"></div>
                      <IconComponent className="h-7 w-7 text-white drop-shadow-sm" />
                    </div>
                    
                    <div className="h-6 w-6 text-gray-400 group-hover:text-yellow-500 group-hover:translate-x-1 transition-all duration-300">→</div>
                  </div>
                  
                  {/* Title and Description */}
                  <div className="flex-1 mb-4">
                    <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-1 capitalize">
                      {operationItem.title}
                    </h3>
                    <p className="text-base font-semibold text-gray-600 dark:text-gray-300 mb-3 capitalize">
                      {operationItem.subtitle}
                    </p>
                    <p className="text-sm text-gray-500 dark:text-gray-400 leading-relaxed">
                      {operationItem.description}
                    </p>
                  </div>
                  
                  {/* Bottom Stats */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 px-3 py-1 bg-gray-100 dark:bg-gray-700 rounded-xl">
                      <Activity className="h-4 w-4 text-gray-500" />
                      <span className="text-sm font-medium text-gray-700 dark:text-gray-200 capitalize">{operationItem.category}</span>
                    </div>
                    <div className="flex items-center gap-2 px-3 py-1 bg-yellow-100 dark:bg-yellow-900/30 rounded-xl">
                      <Zap className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
                      <span className="text-sm font-semibold text-yellow-700 dark:text-yellow-300">{selectedUserIds.length} users</span>
                    </div>
                  </div>
                </div>
            </a>
          )
        })}
      </div>


        {/* Safety Notice - PancakeSwap Warning Card */}
        <div className="relative bg-gradient-to-br from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20 backdrop-blur-xl rounded-3xl shadow-xl border-2 border-amber-200 dark:border-amber-700/50 overflow-hidden p-6">
          <div className="absolute inset-0 bg-gradient-to-br from-amber-400/5 to-orange-500/5"></div>
          <div className="absolute top-4 right-4 w-3 h-3 bg-red-500 rounded-full animate-pulse"></div>
          <div className="absolute bottom-4 left-4 w-2 h-2 bg-amber-400 rounded-full opacity-60"></div>
          
          <div className="relative z-10 flex items-start gap-6">
            <div className="w-16 h-16 bg-gradient-to-br from-amber-400 to-orange-500 rounded-3xl flex items-center justify-center shadow-xl relative overflow-hidden">
              <div className="absolute top-0 right-0 w-6 h-6 bg-white/30 rounded-bl-3xl"></div>
              <Shield className="h-8 w-8 text-white drop-shadow-sm" />
            </div>
            
            <div className="flex-1">
              <h4 className="text-2xl font-bold text-amber-800 dark:text-amber-200 mb-3 flex items-center gap-2">
                ⚠️ Safety Information
              </h4>
              <p className="text-base text-amber-700 dark:text-amber-300 leading-relaxed mb-4">
                Bulk operations affect multiple users simultaneously and <strong>cannot be undone easily</strong>. 
                Please review your selections carefully before proceeding. All operations are logged 
                for audit purposes.
              </p>
              
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-3 px-4 py-2 bg-amber-100 dark:bg-amber-900/40 rounded-2xl">
                  <div className="w-3 h-3 bg-amber-500 rounded-full animate-pulse"></div>
                  <span className="text-sm font-semibold text-amber-800 dark:text-amber-200">
                    {selectedUserIds.length} users will be affected
                  </span>
                </div>
                <div className="flex items-center gap-2 px-4 py-2 bg-green-100 dark:bg-green-900/40 rounded-2xl">
                  <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                  <span className="text-sm font-medium text-green-800 dark:text-green-200">
                    Audit logging enabled
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}