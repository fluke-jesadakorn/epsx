/**
 * User Permissions Page - Server Component
 * Consolidates IAM, roles, and permission profile management
 */


import { getUnifiedUserData } from '@/lib/actions/users'
import { notFound } from 'next/navigation'
import { UserPermissionsContent } from '@/components/users/UserPermissionsContent'
import { auth } from '@/lib/server-auth'

interface UserPermissionsPageProps {
  params: Promise<{ userId: string }>
}

export default async function UserPermissionsPage({ params }: UserPermissionsPageProps) {
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
    <UserPermissionsContent 
      user={userDataResult.data}
      currentUser={currentUser}
    />
  )
}