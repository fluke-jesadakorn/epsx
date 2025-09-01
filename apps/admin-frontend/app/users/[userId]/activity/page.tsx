/**
 * User Activity Page - Server Component
 * Comprehensive audit trail and activity history
 */

// Authentication is handled at the layout level by AdminAuthWrapper
import { AdminServerAPI } from '@/lib/server/admin-api'
import { notFound } from 'next/navigation'
import { UserActivityContent } from '@/components/users/UserActivityContent'
import { getServerSession } from '@/lib/auth/server-auth'

interface UserActivityPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserActivityPage({ params }: UserActivityPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Get current user session
  const session = await getServerSession()
  const currentUser = session?.user
  
  let userData
  try {
    userData = await AdminServerAPI.getUserData(userId)
  } catch (error) {
    console.error('Failed to fetch user data:', error)
    notFound()
  }

  return (
    <UserActivityContent 
      user={userData}
      currentUser={currentUser}
    />
  )
}