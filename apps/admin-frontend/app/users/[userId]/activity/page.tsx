/**
 * User Activity Page - Server Component
 * Comprehensive audit trail and activity history
 */

import { requireAdminAuth } from '@/lib/auth/server-auth'
import { getUnifiedUserData } from '@/lib/actions/unified-user-actions'
import { notFound } from 'next/navigation'
import { UserActivityContent } from '@/components/users/UserActivityContent'

interface UserActivityPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserActivityPage({ params }: UserActivityPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  const currentUser = await requireAdminAuth()
  
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