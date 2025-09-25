/**
 * User Creation Page - Unified Component
 * Uses consolidated UserForms component for user creation
 */

import { UserForms } from '@/components/users/UserForms'
import { UnifiedAuth } from '@/lib/auth/unified-auth'
import { notFound } from 'next/navigation'

interface Props {
  searchParams?: Promise<{
    error?: string
  }>
}

export default async function CreateUserPage({ searchParams }: Props) {
  // Get current user session
  const session = await UnifiedAuth.getSession()
  if (!session?.user) {
    notFound()
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <div className="relative z-10 max-w-6xl mx-auto">
        <UserForms
          mode="create"
          currentUser={session.user as any}
        />
      </div>
    </div>
  )
}