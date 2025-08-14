/**
 * User Overview Page - Server Component
 * Shows user's basic information, status, and quick actions
 */


import { getUnifiedUserData } from '@/lib/actions/users'
import { notFound } from 'next/navigation'
import { UserOverviewContent } from '@/components/users/UserOverviewContent'
import { auth } from '@/lib/auth'

interface UserOverviewPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserOverviewPage({ params }: UserOverviewPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  // Get current user session
  const session = await auth()
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