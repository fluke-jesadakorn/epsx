/**
 * User Overview Page - Server Component
 * Shows user's basic information, status, and quick actions
 */


import { getUnifiedUserData } from '@/lib/actions/users'
import { notFound } from 'next/navigation'
import { UserOverviewContent } from '@/components/users/UserOverviewContent'
import { getServerSession } from '@/lib/auth/server-auth'

interface UserOverviewPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserOverviewPage({ params }: UserOverviewPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Get current user session
  const session = await getServerSession()
  const currentUser = session?.user
  
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