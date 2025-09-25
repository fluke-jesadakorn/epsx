/**
 * User Edit Profile Page
 * Server-side rendered edit form for better performance
 */

import { notFound } from 'next/navigation'
import { UserForms } from '@/components/users/UserForms'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'

interface Props {
  params: Promise<{ userId: string }>
}

export default async function UserEditPage({ params }: Props) {
  const { userId } = await params
  
  if (!userId) {
    notFound()
  }

  const session = await UnifiedAuth.getSession()
  if (!session?.user) {
    notFound()
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:users:manage')) {
    notFound()
  }

  const client = new UnifiedAdminClient()
  let user: any = null
  let availableRoles: any[] = []
  let availablePackageTiers: any[] = []
  let availablePermissions: any[] = []
  
  try {
    const userResponse = await client.getUser(userId)
    if (!userResponse.success || !userResponse.data) {
      console.error('Failed to fetch user data:', userResponse.error)
      notFound()
    }
    user = userResponse.data
    
    // Fetch additional data for dropdowns (with fallback handling)
    try {
      // Mock available roles - in production, fetch from API
      availableRoles = [
        { label: 'Admin', value: 'admin' },
        { label: 'User', value: 'user' },
        { label: 'Premium User', value: 'premium_user' }
      ]
      
      // Mock available tiers - in production, fetch from API
      availablePackageTiers = [
        { label: 'Basic', value: 'basic' },
        { label: 'Premium', value: 'premium' },
        { label: 'Pro', value: 'pro' },
        { label: 'Enterprise', value: 'enterprise' }
      ]
      
      // Mock available permissions - in production, fetch from API
      availablePermissions = [
        {
          id: 'admin:users:manage',
          name: 'Manage Users',
          description: 'Create, edit, and delete user accounts',
          category: 'User Management'
        },
        {
          id: 'admin:analytics:view',
          name: 'View Analytics',
          description: 'Access analytics and reporting features',
          category: 'Analytics'
        },
        {
          id: 'epsx:trading:access',
          name: 'Trading Access',
          description: 'Access to trading features and data',
          category: 'Trading'
        },
        {
          id: 'epsx:eps:premium',
          name: 'Premium EPS Data',
          description: 'Access to premium EPS rankings and data',
          category: 'EPS Analytics'
        }
      ]
    } catch (error) {
      console.warn('Failed to fetch dropdown data, using defaults:', error)
    }
  } catch (error) {
    console.error('Failed to fetch user data:', error)
    notFound()
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-lg animate-pulse animation-delay-1000"></div>
        <div className="absolute bottom-32 left-1/3 w-28 h-28 bg-gradient-to-r from-indigo-400/15 to-blue-500/15 rounded-full blur-xl animate-pulse animation-delay-2000"></div>
      </div>
      
      <div className="relative z-10 max-w-6xl mx-auto">
        <UserForms 
          mode="edit"
          editUser={user}
          currentUser={session.user as any}
          availableRoles={availableRoles}
          availablePackageTiers={availablePackageTiers}
          availablePermissions={availablePermissions}
        />
      </div>
    </div>
  )
}