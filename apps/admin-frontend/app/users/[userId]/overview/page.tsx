/**
 * User Overview Page - Server Component
 * Shows user's basic information, status, and quick actions
 */

import { requireAdminAuth } from '@/lib/auth/server-auth'
import { getUnifiedUserData } from '@/lib/actions/unified-user-actions'
import { notFound } from 'next/navigation'
import { UserOverviewContent } from '@/components/users/UserOverviewContent'

interface UserOverviewPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserOverviewPage({ params }: UserOverviewPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Server-side auth check
  const currentUser = await requireAdminAuth()
  
  // Get user data (this will be cached from layout)
  const userDataResult = await getUnifiedUserData(userId)
  
  if (!userDataResult.success || !userDataResult.data) {
    notFound()
  }

  return (
    <UserOverviewContent 
      user={userDataResult.data}
      currentUser={currentUser}
    />
  )
}