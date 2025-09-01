/**
 * User Overview Page - Server Component
 * Shows user's basic information, status, and quick actions
 */


import { AdminServerAPI } from '@/lib/server/admin-api'
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
  
  // Get user data directly from AdminServerAPI
  let userData
  try {
    userData = await AdminServerAPI.getUserData(userId)
  } catch (error) {
    console.error('Failed to fetch user data:', error)
    notFound()
  }

  return (
    <UserOverviewContent 
      user={userData}
      currentUser={currentUser}
    />
  )
}