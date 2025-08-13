/**
 * User Profile Layout - Server Component
 * Provides unified user data and tab navigation for all user-related pages
 */

import { ReactNode } from 'react'
import { notFound } from 'next/navigation'
import { requireAdminAuth } from '@/lib/auth/server-auth'
import { getUnifiedUserData } from '@/lib/actions/unified-user-actions'
import { UserProfileHeader } from '@/components/users/UserProfileHeader'
import { UserTabNavigation } from '@/components/users/UserTabNavigation'
import { UserDataProvider } from '@/components/users/UserDataProvider'

interface UserProfileLayoutProps {
  children: ReactNode
  params: Promise<{ userId: string }>
  searchParams?: Promise<{
    modal?: string
    userId?: string
  }>
}

export default async function UserProfileLayout({
  children,
  params,
  searchParams
}: UserProfileLayoutProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  const searchParamsData = searchParams ? await searchParams : {}
  
  // Server-side auth check
  const currentUser = await requireAdminAuth()
  
  // Fetch unified user data
  let userDataResult
  try {
    userDataResult = await getUnifiedUserData(userId)
  } catch (_error) {
    return (
      <div className="text-center text-red-600 py-8">
        <p>Failed to load user data</p>
        <p className="text-sm text-muted-foreground mt-2">
          {_error instanceof Error ? _error.message : 'Unknown error occurred'}
        </p>
      </div>
    )
  }
  
  if (!userDataResult.success || !userDataResult.data) {
    notFound()
  }
  
  const userData = userDataResult.data
  
  return (
    <div className="user-profile-layout space-y-6">
      {/* User Profile Header */}
      <UserProfileHeader 
        user={userData}
        currentUser={currentUser}
      />
      
      {/* Tab Navigation */}
      <UserTabNavigation userId={userId} />
      
      {/* Content with User Data Context */}
      <UserDataProvider userData={userData}>
        {children}
      </UserDataProvider>
      
    </div>
  )
}

// Generate metadata for SEO
export async function generateMetadata({ params }: { params: Promise<{ userId: string }> }) {
  try {
    const { userId } = await params
    const userDataResult = await getUnifiedUserData(userId)
    
    if (!userDataResult.success || !userDataResult.data) {
      return {
        title: 'User Not Found - EPSX Admin',
        description: 'The requested user profile could not be found.'
      }
    }
    
    const user = userDataResult.data
    const userName = user.displayName || user.email
    
    return {
      title: `${userName} - User Management - EPSX Admin`,
      description: `Manage ${userName}'s account, permissions, modules, and billing settings.`,
      openGraph: {
        title: `${userName} - User Profile`,
        description: `Administrative view of ${userName}'s account settings and permissions.`,
        type: 'profile'
      }
    }
  } catch (_error) {
    return {
      title: 'User Profile - EPSX Admin',
      description: 'User profile management interface'
    }
  }
}