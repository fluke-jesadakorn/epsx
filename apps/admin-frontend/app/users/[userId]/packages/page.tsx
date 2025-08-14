/**
 * User Packages Page - Server Component
 * Consolidates stock ranking packages management
 */


import { getUnifiedUserData } from '@/lib/actions/user-actions'
import { notFound } from 'next/navigation'
import { UserPackagesContent } from '@/components/users/UserPackagesContent'
import { auth } from '@/lib/auth'

interface UserPackagesPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserPackagesPage({ params }: UserPackagesPageProps) {
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
    <UserPackagesContent 
      user={userDataResult.data}
      currentUser={currentUser}
    />
  )
}