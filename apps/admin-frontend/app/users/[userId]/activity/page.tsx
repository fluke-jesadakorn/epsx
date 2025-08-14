/**
 * User Activity Page - Server Component
 * Comprehensive audit trail and activity history
 */

// Authentication is handled at the layout level by AdminAuthWrapper
import { getUnifiedUserData } from '@/lib/actions/user-actions'
import { notFound } from 'next/navigation'
import { UserActivityContent } from '@/components/users/UserActivityContent'
import { auth } from '@/lib/auth'

interface UserActivityPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserActivityPage({ params }: UserActivityPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Get current user session
  const session = await auth()
  const currentUser = session?.user
  
  const userDataResult = await getUnifiedUserData(userId)
  
  if (!userDataResult.success || !userDataResult.data) {
    notFound()
  }

  return (
    <UserActivityContent 
      user={userDataResult.data}
      currentUser={currentUser}
    />
  )
}