/**
 * User Edit Profile Page
 * Server-side rendered edit form for better performance
 */

import { notFound, redirect } from 'next/navigation'
import { User, Mail, Shield, Phone, Globe, Clock } from 'lucide-react'
import { getUnifiedUserData } from '@/lib/actions/user-actions'
import { UserEditForm } from '@/components/users/UserEditForm'

interface Props {
  params: Promise<{ userId: string }>
}

export default async function UserEditPage({ params }: Props) {
  const { userId } = await params
  
  if (!userId) {
    redirect('/users')
  }

  const result = await getUnifiedUserData(userId)
  
  if (!result.success || !result.data) {
    notFound()
  }

  const user = result.data

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-2xl mx-auto px-4 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
              <span className="text-blue-600 dark:text-blue-400 font-medium text-lg">
                {user.displayName?.charAt(0)?.toUpperCase() || user.email?.charAt(0)?.toUpperCase()}
              </span>
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                Edit Profile
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                {user.displayName || 'No name'} • {user.email}
              </p>
            </div>
          </div>
          
          <nav className="flex items-center text-sm text-gray-500 dark:text-gray-400">
            <a href="/users" className="hover:text-blue-600 dark:hover:text-blue-400">
              Users
            </a>
            <span className="mx-2">/</span>
            <a href={`/users/${userId}`} className="hover:text-blue-600 dark:hover:text-blue-400">
              {user.displayName || user.email}
            </a>
            <span className="mx-2">/</span>
            <span className="text-gray-900 dark:text-gray-100">Edit</span>
          </nav>
        </div>

        {/* Form Card */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <UserEditForm user={user} />
        </div>
      </div>
    </div>
  )
}