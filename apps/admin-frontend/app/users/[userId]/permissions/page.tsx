/**
 * User Permissions Page - Server Component
 * Consolidates IAM, roles, and permission profile management
 */

import { requireAdminAuth } from '@/lib/auth/server-auth-enhanced'
import { getUnifiedUserData } from '@/lib/actions/unified-user-actions'
import { notFound } from 'next/navigation'
import { UserPermissionsContent } from '@/components/users/UserPermissionsContent'

interface UserPermissionsPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserPermissionsPage({ params }: UserPermissionsPageProps) {
  // Await params properly for Next.js 15
  const { userId } = await params
  
  const currentUser = await requireAdminAuth()
  
  const userDataResult = await getUnifiedUserData(userId)
  
  if (!userDataResult.success || !userDataResult.data) {
    notFound()
  }

  return (
    <UserPermissionsContent 
      user={userDataResult.data}
      currentUser={currentUser}
    />
  )
}