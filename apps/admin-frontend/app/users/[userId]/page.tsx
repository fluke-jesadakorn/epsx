/**
 * User Profile Main Page
 * Shows user profile directly using unified UserProfile component
 */

import { notFound } from 'next/navigation'
import { UserProfile } from '@/components/users/UserProfile'
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client'
import { UnifiedAuth } from '@/lib/auth/unified-auth'

interface UserProfilePageProps {
  params: Promise<{ userId: string }>
}

export default async function UserProfilePage({ params }: UserProfilePageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Get current user session
  const session = await UnifiedAuth.getSession()
  if (!session?.user) {
    notFound()
  }
  
  // Get user data
  const client = new UnifiedAdminClient()
  let userData
  try {
    userData = await client.getUser(userId)
  } catch (error) {
    console.error('Failed to fetch user data:', error)
    notFound()
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      {/* Background Decorations */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-32 right-24 w-32 h-32 bg-gradient-to-r from-yellow-400/15 to-orange-500/15 rounded-full blur-xl animate-pulse"></div>
        <div className="absolute bottom-40 left-20 w-24 h-24 bg-gradient-to-r from-pink-400/15 to-purple-500/15 rounded-full blur-lg"></div>
        <div className="absolute top-1/2 left-1/2 w-20 h-20 bg-gradient-to-r from-orange-400/10 to-yellow-500/10 rounded-full blur-lg animate-pulse" style={{animationDelay: '1.5s'}}></div>
      </div>

      <div className="relative z-10 max-w-6xl mx-auto">
        <UserProfile 
          userId={userId}
          user={userData}
          currentUser={session.user}
        />
      </div>
    </div>
  )
}