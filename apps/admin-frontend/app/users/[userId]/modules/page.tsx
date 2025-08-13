/**
 * User Modules Page - Server Component
 * Consolidates module assignment and quota management
 */

import { requireAdminAuth } from '@/lib/auth/server-auth'
import { getUnifiedUserData } from '@/lib/actions/unified-user-actions'
import { notFound } from 'next/navigation'
import { UserModulesContent } from '@/components/users/UserModulesContent'

interface UserModulesPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserModulesPage({ params }: UserModulesPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  const currentUser = await requireAdminAuth()
  
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