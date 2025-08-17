/**
 * User Modules Page - Server Component
 * Consolidates module assignment and quota management
 */


import { getUnifiedUserData } from '@/lib/actions/users'
import { notFound } from 'next/navigation'
import { UserModulesContent } from '@/components/users/UserModulesContent'
import { auth } from '@/lib/server-auth'

interface UserModulesPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserModulesPage({ params }: UserModulesPageProps) {
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
    <UserModulesContent 
      user={userDataResult.data}
      currentUser={currentUser}
    />
  )
}