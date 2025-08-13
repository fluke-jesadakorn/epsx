/**
 * User Packages Page - Server Component
 * Consolidates stock ranking packages management
 */

import { requireAdminAuth } from '@/lib/auth/server-auth'
import { getUnifiedUserData } from '@/lib/actions/unified-user-actions'
import { notFound } from 'next/navigation'
import { UserPackagesContent } from '@/components/users/UserPackagesContent'

interface UserPackagesPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserPackagesPage({ params }: UserPackagesPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  const currentUser = await requireAdminAuth()
  
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